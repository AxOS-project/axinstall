// To be reviewed

use crate::args::{self};
use crate::args::PartitionMode;
use crate::internal::exec::*;
use crate::internal::*;
use std::path::{Path, PathBuf};
use log;

pub fn fmt_mount(mountpoint: &str, filesystem: &str, blockdevice: &str) {
    match filesystem {
        "ext4" => exec_eval(
            exec("mkfs.ext4", vec![String::from(blockdevice)]),
            format!("Formatting {blockdevice} as ext4").as_str(),
        ),
        "fat32" => exec_eval(
            exec("mkfs.fat", vec![String::from("-F32"), String::from(blockdevice)]),
            format!("Formatting {blockdevice} as fat32").as_str(),
        ),
        "btrfs" => exec_eval(
            exec("mkfs.btrfs", vec![String::from("-f"), String::from(blockdevice)]),
            format!("Formatting {blockdevice} as btrfs").as_str(),
        ),
        "xfs" => exec_eval(
            exec("mkfs.xfs", vec![String::from(blockdevice)]),
            format!("Formatting {blockdevice} as xfs").as_str(),
        ),
        "noformat" | "don't format" => {
            log::debug!("Not formatting {}", blockdevice);
        }
        _ => crash(
            format!("Unknown filesystem {filesystem}, used in partition {blockdevice}"),
            1,
        ),
    }

    exec_eval(
        exec("mkdir", vec![String::from("-p"), String::from(mountpoint)]),
        format!("Creating mountpoint {mountpoint} for {blockdevice}").as_str(),
    );
    mount(blockdevice, mountpoint, "");
}

fn split_partition(blockdevice: &str) -> (String, String) {
    let num_start = blockdevice.trim_end_matches(|c: char| c.is_ascii_digit()).len();
    let num = blockdevice[num_start..].to_string();

    let disk = if blockdevice.contains("nvme") || blockdevice.contains("mmcblk") {
        // /dev/nvme0n1p2 -> /dev/nvme0n1
        blockdevice[..num_start].trim_end_matches('p').to_string()
    } else {
        // /dev/sda2 -> /dev/sda
        blockdevice[..num_start].to_string()
    };

    (disk, num)
}

pub fn partition(
    device: PathBuf,
    mode: PartitionMode,
    efi: bool,
    partitions: &mut Vec<args::Partition>,
) {
    println!("{:?}", mode);
    match mode {
        PartitionMode::Auto => {
            if !device.exists() {
                crash(format!("The device {device:?} doesn't exist"), 1);
            }
            log::debug!("automatically partitioning {device:?}");
            partition_with_efi(&device);
            
            if device.to_string_lossy().contains("nvme")
                || device.to_string_lossy().contains("mmcblk")
            {
                part_nvme(&device, efi);
            } else {
                part_disk(&device, efi);
            }
        }
        PartitionMode::Manual => {
            log::debug!("Manual partitioning");
            partitions.sort_by(|a, b| a.mountpoint.len().cmp(&b.mountpoint.len()));
            for partition in partitions {
                fmt_mount(
                    &partition.mountpoint,
                    &partition.filesystem,
                    &partition.blockdevice,
                );
                let (disk, part_num) = split_partition(&partition.blockdevice);
                if &partition.mountpoint == "/boot/efi" {
                    exec_eval(
                        exec(
                            "parted",
                            vec![
                                String::from("-s"),
                                String::from(&disk),
                                String::from("set"),
                                String::from(&part_num),
                                String::from("esp"),
                                String::from("on"),
                            ],
                        ),
                        "set EFI partition as ESP",
                    );
                }
            }
        }
    }
}


fn partition_with_efi(device: &Path) {
    let device = device.to_string_lossy().to_string();
    exec_eval(
        exec(
            "parted",
            vec![
                String::from("-s"),
                String::from(&device),
                String::from("mklabel"),
                String::from("gpt"),
            ],
        ),
        format!("create gpt label on {}", &device).as_str(),
    );
    exec_eval(
        exec(
            "parted",
            vec![
                String::from("-s"),
                String::from(&device),
                String::from("mkpart"),
                String::from("fat32"),
                String::from("0"),
                String::from("300"),
            ],
        ),
        "create EFI partition",
    );
    exec_eval(
        exec(
            "parted",
            vec![
                String::from("-s"),
                String::from(&device),
                String::from("set"),
                String::from("1"),
                String::from("esp"),
                String::from("on"),
            ],
        ),
        "set EFI partition as ESP",
    );
    exec_eval(
        exec(
            "parted",
            vec![
                String::from("-s"),
                device,
                String::from("mkpart"),
                String::from("primary"),
                String::from("ext4"),
                String::from("512MIB"),
                String::from("100%"),
            ],
        ),
        "create ext4 root partition",
    );
}

fn part_nvme(device: &Path, efi: bool) {
    let device = device.to_string_lossy().to_string();
    if efi {
        exec_eval(
            exec(
                "mkfs.fat",
                vec![String::from("-F32"), format!("{}p1", device)],
            ),
            format!("format {}p1 as fat32", device).as_str(),
        );
        exec_eval(
            exec(
                "mkfs.ext4",
                vec![format!("{}p2", device)],
            ),
            format!("format {}p2 as ext4", device).as_str(),
        );
        mount(format!("{}p2", device).as_str(), "/mnt", "");
        files_eval(files::create_directory("/mnt/boot"), "create /mnt/boot");
        files_eval(
            files::create_directory("/mnt/boot/efi"),
            "create /mnt/boot/efi",
        );
        mount(format!("{}p1", device).as_str(), "/mnt/boot/efi", "");
        exec_eval(
        exec(
            "parted",
            vec![
                String::from("-s"),
                String::from(&device),
                String::from("set"),
                String::from("1"),
                String::from("esp"),
                String::from("on"),
            ],
        ),
        "set EFI partition as ESP",
    );
    } else if !efi {
        exec_eval(
            exec("mkfs.ext4", vec![format!("{}p1", device)]),
            format!("format {}p1 as ext4", device).as_str(),
        );
        exec_eval(
            exec(
                "mkfs.ext4",
                vec![format!("{}p2", device)],
            ),
            format!("format {}p2 as ext4", device).as_str(),
        );
        mount(format!("{}p2", device).as_str(), "/mnt/", "");
        files_eval(files::create_directory("/mnt/boot"), "create /mnt/boot");
        mount(format!("{}p1", device).as_str(), "/mnt/boot", "");
    } else {
        crash("NVMe devices must be partitioned with EFI", 1);
    }
}

fn part_disk(device: &Path, efi: bool) {
    let device = device.to_string_lossy().to_string();
    if efi {
        exec_eval(
            exec(
                "mkfs.fat",
                vec![String::from("-F32"), format!("{}1", device)],
            ),
            format!("format {}1 as fat32", device).as_str(),
        );
        exec_eval(
            exec("mkfs.ext4", vec![format!("{}2", device)]),
            format!("format {}2 as ext4", device).as_str(),
        );
        mount(format!("{}2", device).as_str(), "/mnt", "");
        files_eval(files::create_directory("/mnt/boot"), "create /mnt/boot");
        files_eval(
            files::create_directory("/mnt/boot/efi"),
            "create /mnt/boot/efi",
        );
        mount(format!("{}1", device).as_str(), "/mnt/boot/efi", "");
        exec_eval(
            exec(
                "parted",
                vec![
                    String::from("-s"),
                    String::from(&device),
                    String::from("set"),
                    String::from("1"),
                    String::from("esp"),
                    String::from("on"),
                ],
            ),
            "set EFI partition as ESP",
        );
    } else if !efi {
        exec_eval(
            exec("mkfs.ext4", vec![format!("{}1", device)]),
            format!("format {}1 as ext4", device).as_str(),
        );
        exec_eval(
            exec("mkfs.ext4", vec![format!("{}2", device)]),
            format!("format {}2 as ext4", device).as_str(),
        );
        mount(format!("{}2", device).as_str(), "/mnt/", "");
        files_eval(
            files::create_directory("/mnt/boot"),
            "create directory /mnt/boot",
        );
        mount(format!("{}1", device).as_str(), "/mnt/boot", "");
    } else {
        crash("Disk devices must be partitioned with EFI", 1);
    }
}

pub fn mount(partition: &str, mountpoint: &str, options: &str) {
    if !options.is_empty() {
        exec_eval(
            exec(
                "mount",
                vec![
                    String::from(partition),
                    String::from(mountpoint),
                    String::from("-o"),
                    String::from(options),
                ],
            ),
            format!(
                "mount {} with options {} at {}",
                partition, options, mountpoint
            )
            .as_str(),
        );
    } else {
        exec_eval(
            exec(
                "mount",
                vec![String::from(partition), String::from(mountpoint)],
            ),
            format!("mount {} with no options at {}", partition, mountpoint).as_str(),
        );
    }
}


pub fn umount_partitions(device: String) {
    let output = exec_output("lsblk", vec![
        String::from("-J"),
        String::from("-o"),
        String::from("NAME,MOUNTPOINT"),
        format!("/dev/{}", device),
    ]);
        
    if let Ok(output) = output {
        if let Ok(output_str) = String::from_utf8(output.into()) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&output_str) {
                if let Some(blockdevices) = json["blockdevices"].as_array() {
                    for blockdevice in blockdevices {
                        if let Some(children) = blockdevice["children"].as_array() {
                            for child in children {
                                if let Some(mountpoint) = child["mountpoint"].as_str() {
                                    log::debug!("Unmounting {}", mountpoint);
                                    umount(mountpoint);
                                }
                            }
                        }
                        if let Some(mountpoint) = blockdevice["mountpoint"].as_str() {
                            log::debug!("Unmounting {}", mountpoint);
                            umount(mountpoint);
                        }
                    }
                }
            }
        }
    }
}


pub fn umount(mountpoint: &str) {
    let _ = exec("umount", vec![String::from(mountpoint)]);
    let _ = format!("unmount {}", mountpoint).as_str();
}
