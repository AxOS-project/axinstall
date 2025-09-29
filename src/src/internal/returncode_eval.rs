use crate::internal::*;
use crate::functions::partition::umount;

/// ### Evaluates the result of a command execution.
///
/// - If the command succeeds, logs an informational message.
/// - If the command fails or returns an error:
///   - Unmounts `/mnt/boot/efi` and `/mnt/`.
///   - Calls [`crash`] with the provided log message and the exit code or error code.
pub fn exec_eval(
    return_code: std::result::Result<std::process::ExitStatus, std::io::Error>,
    logmsg: &str,
) {
    match return_code {
        Ok(status) => {
            if status.success() {
                log::info!("{}", logmsg);
            } else {
                umount("/mnt/boot/efi");
                umount("/mnt/");
                crash(
                    format!("{}  ERROR: exited with code {}", logmsg, status.code().unwrap_or(-1)),
                    status.code().unwrap_or(1),
                );
            }
        }
        Err(e) => {
            umount("/mnt/boot/efi");
            umount("/mnt/");
            crash(
                format!("{}  ERROR: {}", logmsg, e),
                e.raw_os_error().unwrap_or(1),
            );
        }
    }
}

/// Evaluates the result of a command execution in a non-fatal way.
///
/// - If the command succeeds, logs an informational message.
/// - If the command fails or returns an error:
///   - Unmounts `/mnt/boot/efi` and `/mnt/`.
///   - Logs the error, but does not terminate the program (unlike [`exec_eval`]).
pub fn soft_exec_eval(
    return_code: std::result::Result<std::process::ExitStatus, std::io::Error>,
    logmsg: &str,
) {
    match return_code {
        Ok(status) => {
            if status.success() {
                log::info!("{}", logmsg);
            } else {
                umount("/mnt/boot/efi");
                umount("/mnt/");
                log::error!("{}  ERROR: exited with code {}", logmsg, status.code().unwrap_or(-1));
            }
        }
        Err(e) => {
            umount("/mnt/boot/efi");
            umount("/mnt/");
            log::error!("{}  ERROR: {}", logmsg, e.raw_os_error().unwrap_or(1));
        }
    }
}

pub fn files_eval(return_code: std::result::Result<(), std::io::Error>, logmsg: &str) {
    match &return_code {
        Ok(_) => {
            log::info!("{}", logmsg);
        }
        Err(e) => {
            crash(
                format!("{} ERROR: {}", logmsg, e),
                return_code.unwrap_err().raw_os_error().unwrap(),
            );
        }
    }
}
