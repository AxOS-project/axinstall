use std::process::Command;

/// Executes a command directly on the host system.
///
/// This is a thin wrapper around [`std::process::Command::status`] that runs
/// the given command with the provided arguments.
pub fn exec(command: &str, args: Vec<String>) -> Result<std::process::ExitStatus, std::io::Error> {
    let returncode = Command::new(command).args(args).status();
    returncode
}

/// Executes a command inside the target system using `arch-chroot`.
///
/// Wraps the command in a `bash -c "arch-chroot /mnt ..."` invocation,
/// so that it runs inside the `/mnt` chroot (the target installation root).
///
/// ### Notes
/// - Assumes `/mnt` is already prepared as a valid chroot environment.
/// - Relies on `arch-chroot` being available in `$PATH`.
pub fn exec_chroot(
    command: &str,
    args: Vec<String>,
) -> Result<std::process::ExitStatus, std::io::Error> {
    let returncode = Command::new("bash")
        .args([
            "-c",
            format!("arch-chroot /mnt {} {}", command, args.join(" ")).as_str(),
        ])
        .status();
    returncode
}

/*
pub fn exec_workdir(
    command: &str,
    workdir: &str,
    args: Vec<String>,
) -> Result<std::process::ExitStatus, std::io::Error> {
    let returncode = Command::new(command)
        .args(args)
        .current_dir(workdir)
        .status();
    returncode
}
*/
