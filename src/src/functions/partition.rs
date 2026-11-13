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
            exec("mkfs.ext4", vec![String::from("-F"), String::from(blockdevice)]),
            format!("Formatting {blockdevice} as ext4").as_str(),
        ),
        "ext3" => exec_eval(
            exec("mkfs.ext3", vec![String::from("-F"), String::from(blockdevice)]),
            format!("Formatting {blockdevice} as ext3").as_str(),
        ),
        "ext2" => exec_eval(
            exec("mkfs.ext2", vec![String::from("-F"), String::from(blockdevice)]),
            format!("Formatting {blockdevice} as ext2").as_str(),
        ),
        "fat32" | "vfat" | "fat" | "msdos" => exec_eval(
            exec("mkfs.fat", vec![String::from("-F32"), String::from(blockdevice)]),
            format!("Formatting {blockdevice} as fat32").as_str(),
        ),
        "btrfs" => exec_eval(
            exec("mkfs.btrfs", vec![String::from("-f"), String::from(blockdevice)]),
            format!("Formatting {blockdevice} as btrfs").as_str(),
        ),
        "xfs" => exec_eval(
            exec("mkfs.xfs", vec![String::from("-f"), String::from(blockdevice)]),
            format!("Formatting {blockdevice} as xfs").as_str(),
        ),
        "f2fs" => exec_eval(
            exec("mkfs.f2fs", vec![String::from("-f"), String::from(blockdevice)]),
            format!("Formatting {blockdevice} as f2fs").as_str(),
        ),
        "ntfs" => exec_eval(
            exec("mkfs.ntfs", vec![String::from("-f"), String::from(blockdevice)]),
            format!("Formatting {blockdevice} as ntfs").as_str(),
        ),
        "bfs" => exec_eval(
            exec("mkfs.bfs", vec![String::from(blockdevice)]),
            format!("Formatting {blockdevice} as bfs").as_str(),
        ),
        "cramfs" => exec_eval(
            exec("mkfs.cramfs", vec![String::from(blockdevice)]),
            format!("Formatting {blockdevice} as cramfs").as_str(),
        ),
        "minix" => exec_eval(
            exec("mkfs.minix", vec![String::from(blockdevice)]),
            format!("Formatting {blockdevice} as minix").as_str(),
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

pub fn split_partition(blockdevice: &str) -> (String, String) {
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

fn normalize_mountpoint(mp: &str) -> String {
    let trimmed = mp.trim();
    if trimmed.is_empty() || trimmed == "none" || trimmed == "/mnt/none" {
        return String::new();
    }
    let with_mnt = if trimmed == "/" || trimmed == "/mnt" || trimmed == "/mnt/" {
        String::from("/mnt")
    } else if trimmed.starts_with("/mnt/") {
        trimmed.trim_end_matches('/').to_string()
    } else if trimmed.starts_with('/') {
        format!("/mnt{}", trimmed.trim_end_matches('/'))
    } else {
        format!("/mnt/{}", trimmed.trim_end_matches('/'))
    };
    with_mnt
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
            if efi {
                partition_with_efi(&device);
            } else {
                partition_no_efi(&device);
            }

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
            partitions.retain(|p| {
                let mp = p.mountpoint.trim();
                !mp.is_empty() && mp != "none" && mp != "/mnt/none"
            });
            for p in partitions.iter_mut() {
                p.mountpoint = normalize_mountpoint(&p.mountpoint);
            }
            partitions.retain(|p| !p.mountpoint.is_empty());
            partitions.sort_by(|a, b| a.mountpoint.len().cmp(&b.mountpoint.len()));
            for partition in partitions {
                fmt_mount(
                    &partition.mountpoint,
                    &partition.filesystem,
                    &partition.blockdevice,
                );
                let (disk, part_num) = split_partition(&partition.blockdevice);
                if (partition.mountpoint == "/boot/efi"
                    || partition.mountpoint == "/mnt/boot/efi")
                    && !disk.is_empty()
                    && !part_num.is_empty()
                {
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

fn partition_no_efi(device: &Path) {
    let device = device.to_string_lossy().to_string();
    exec_eval(
        exec(
            "parted",
            vec![
                String::from("-s"),
                String::from(&device),
                String::from("mklabel"),
                String::from("msdos"),
            ],
        ),
        format!("create msdos label on {}", &device).as_str(),
    );
    exec_eval(
        exec(
            "parted",
            vec![
                String::from("-s"),
                String::from(&device),
                String::from("mkpart"),
                String::from("primary"),
                String::from("ext4"),
                String::from("1MiB"),
                String::from("512MiB"),
            ],
        ),
        "create BIOS boot partition",
    );
    exec_eval(
        exec(
            "parted",
            vec![
                String::from("-s"),
                String::from(&device),
                String::from("set"),
                String::from("1"),
                String::from("boot"),
                String::from("on"),
            ],
        ),
        "set boot flag on partition 1",
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
                String::from("512MiB"),
                String::from("100%"),
            ],
        ),
        "create ext4 root partition",
    );
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
                String::from("1MiB"),
                String::from("512MiB"),
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
                String::from("512MiB"),
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
                vec![String::from("-F"), format!("{}p2", device)],
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
    } else {
        exec_eval(
            exec(
                "mkfs.ext4",
                vec![String::from("-F"), format!("{}p1", device)],
            ),
            format!("format {}p1 as ext4", device).as_str(),
        );
        exec_eval(
            exec(
                "mkfs.ext4",
                vec![String::from("-F"), format!("{}p2", device)],
            ),
            format!("format {}p2 as ext4", device).as_str(),
        );
        mount(format!("{}p2", device).as_str(), "/mnt/", "");
        files_eval(files::create_directory("/mnt/boot"), "create /mnt/boot");
        mount(format!("{}p1", device).as_str(), "/mnt/boot", "");
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
            exec(
                "mkfs.ext4",
                vec![String::from("-F"), format!("{}2", device)],
            ),
            format!("format {}2 as ext4", device).as_str(),
        );
        mount(format!("{}2", device).as_str(), "/mnt", "");
        files_eval(files::create_directory("/mnt/boot"), "create /mnt/boot");
        files_eval(
            files::create_directory("/mnt/boot/efi"),
            "create /mnt/boot/efi",
        );
        mount(format!("{}1", device).as_str(), "/mnt/boot/efi", "");
    } else {
        exec_eval(
            exec(
                "mkfs.ext4",
                vec![String::from("-F"), format!("{}1", device)],
            ),
            format!("format {}1 as ext4", device).as_str(),
        );
        exec_eval(
            exec(
                "mkfs.ext4",
                vec![String::from("-F"), format!("{}2", device)],
            ),
            format!("format {}2 as ext4", device).as_str(),
        );
        mount(format!("{}2", device).as_str(), "/mnt/", "");
        files_eval(
            files::create_directory("/mnt/boot"),
            "create directory /mnt/boot",
        );
        mount(format!("{}1", device).as_str(), "/mnt/boot", "");
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
    let disk = device.trim();
    if disk.is_empty() {
        return;
    }
    let dev = if disk.starts_with("/dev/") {
        disk.to_string()
    } else {
        format!("/dev/{disk}")
    };
    let output = exec_output("lsblk", vec![
        String::from("-J"),
        String::from("-o"),
        String::from("NAME,MOUNTPOINT"),
        dev,
    ]);
        
    if let Ok(output) = output {
        if let Ok(output_str) = String::from_utf8(output.into()) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&output_str) {
                if let Some(blockdevices) = json["blockdevices"].as_array() {
                    let mut mountpoints: Vec<String> = Vec::new();
                    for blockdevice in blockdevices {
                        if let Some(children) = blockdevice["children"].as_array() {
                            for child in children {
                                if let Some(mountpoint) = child["mountpoint"].as_str() {
                                    if !mountpoint.is_empty() {
                                        mountpoints.push(mountpoint.to_string());
                                    }
                                }
                            }
                        }
                        if let Some(mountpoint) = blockdevice["mountpoint"].as_str() {
                            if !mountpoint.is_empty() {
                                mountpoints.push(mountpoint.to_string());
                            }
                        }
                    }
                    mountpoints.sort_by(|a, b| b.len().cmp(&a.len()));
                    for mountpoint in mountpoints {
                        log::debug!("Unmounting {}", mountpoint);
                        umount(&mountpoint);
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
