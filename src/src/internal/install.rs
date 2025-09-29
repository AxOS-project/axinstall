// use crate::functions::partition::umount;
use crate::internal::*;
use std::process::Command;



/// Installs a list of packages into the `/mnt` directory using `pacstrap`.
///
/// - If `attempt_retries` is `true`, the command will be retried up to 3 times
///   with exponential backoff (1s, 2s, 4s).
/// - If retries are disabled, the command is executed once in "soft" mode:
///   errors are logged, but will not crash the program.
///
/// The result is passed to [`exec_eval`] (fatal on failure) or
/// [`soft_exec_eval`] (non-fatal on failure), depending on the retry mode.
pub fn install(pkgs: Vec<String>, attempt_retries: bool) {
    if attempt_retries {
        let result = retry(3, || Command::new("pacstrap").arg("/mnt").args(&pkgs).status());
        exec_eval(result, &format!("Install packages {}", pkgs.join(", ")));
    } else {
        soft_exec_eval(
            Command::new("pacstrap").arg("/mnt").args(&pkgs).status(),
            format!("Install packages {}", pkgs.join(", ")).as_str(),
        );
    }
    // umount("/mnt/dev");
}

/// Retries a fallible operation a fixed number of times with exponential backoff.
///
/// - Starts with a 1 second delay, then doubles (1s, 2s, 4s, ...).
/// - Logs a warning on each failed attempt.
/// - Returns the first successful result, or the final error if all attempts fail.
fn retry<F, T>(mut attempts: u32, mut f: F) -> std::io::Result<T>
where
    F: FnMut() -> std::io::Result<T>,
{
    let mut delay = 1;
    loop {
        match f() {
            Ok(result) => return Ok(result),
            Err(e) if attempts > 1 => {
                log::warn!("Operation failed: {}. Retrying in {}s...", e, delay);
                std::thread::sleep(std::time::Duration::from_secs(delay));
                attempts -= 1;
                delay *= 2;
            }
            Err(e) => return Err(e),
        }
    }
}
