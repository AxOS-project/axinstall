// use crate::functions::partition::umount;
use crate::internal::*;
use std::process::Command;

pub fn install(pkgs: Vec<String>) {
    exec_eval(
        Command::new("pacman")
        .arg("--root=/mnt")
        .arg("--cachedir=/var/cache/pacman/pkg")
        .arg("--noconfirm")
        .args(&pkgs)
        .status(),
        format!("Install packages {}", pkgs.join(", ")).as_str(),
    );
    // umount("/mnt/dev");
}