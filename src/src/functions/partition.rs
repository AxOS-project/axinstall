// To be reviewed

use crate::args;
use crate::args::PartitionMode;
use crate::internal::exec::*;
use crate::internal::*;
use std::path::{Path, PathBuf};
use log;

pub fn fmt_mount(mountpoint: &str, filesystem: &str, blockdevice: &str) {
    let fs_command = match filesystem {
        "ext4" => ("mkfs.ext4", vec![String::from(blockdevice)]),
        "fat32" => ("mkfs.fat", vec![String::from("-F32"), String::from(blockdevice)]),
        "btrfs" => ("mkfs.btrfs", vec![String::from("-f"), String::from(blockdevice)]),
        "xfs" => ("mkfs.xfs", vec![String::from(blockdevice)]),
        "noformat" | "don't format" => {
            log::debug!("Not formatting {}", blockdevice);
            return;
        }
        _ => {
            crash(
                format!("Unknown filesystem {filesystem}, used in partition {blockdevice}"),
                1,
            );
        }
    };

    exec_eval(
        exec(fs_command.0, fs_command.1),
        format!("Formatting {blockdevice} as {filesystem}").as_str(),
    );

    exec_eval(
        exec("mkdir", vec![String::from("-p"), String::from(mountpoint)]),
        format!("Creating mountpoint {mountpoint} for {blockdevice}").as_str(),
    );
    mount(blockdevice, mountpoint, "");
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
                if &partition.mountpoint == "/boot/efi" {
                    exec_eval(
                        exec(
                            "parted",
                            vec![
                                String::from("-s"),
                                String::from(&partition.blockdevice),
                                String::from("set"),
                                String::from("1"),
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

pub fn umount(mountpoint: &str) {
    let _ = exec("umount", vec![String::from(mountpoint)]);
    let _ = format!("unmount {}", mountpoint).as_str();
}
