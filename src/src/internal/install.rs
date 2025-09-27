// use crate::functions::partition::umount;
use crate::internal::*;
use std::process::Command;

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
