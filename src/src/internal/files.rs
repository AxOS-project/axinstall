use crate::internal::*;
use std::fs::{self, File, OpenOptions};
use std::io::prelude::*;

pub fn create_file(path: &str) {
    let returncode = File::create(path);
    match returncode {
        Ok(_) => {
            log::info!("Create {}", path);
        }
        Err(e) => {
            crash(format!("Create {}: Failed with error {}", path, e), 1);
        }
    }
}

pub fn copy_file(path: &str, destpath: &str) {
    let return_code = std::fs::copy(path, destpath);
    match return_code {
        Ok(_) => {
            log::info!("Copy {} to {}", path, destpath);
        }
        Err(e) => {
            crash(
                format!("Copy {} to {}: Failed with error {}", path, destpath, e),
                1,
            );
        }
    }
}

pub fn append_file(path: &str, content: &str) -> std::io::Result<()> {
    log::info!("Append '{}' to file {}", content.trim_end(), path);
    let mut file = OpenOptions::new().append(true).open(path)?;
    file.write_all(format!("\n{content}\n").as_bytes())?;
    Ok(())
}

pub fn sed_file(path: &str, find: &str, replace: &str) -> std::io::Result<()> {
    log::info!("Sed '{}' to '{}' in file {}", find, replace, path);
    let contents = fs::read_to_string(path)?;
    let new_contents = contents.replace(find, replace);
    let mut file = OpenOptions::new().write(true).truncate(true).open(path)?;
    file.write_all(new_contents.as_bytes())?;
    Ok(())
}

pub fn create_directory(path: &str) -> std::io::Result<()> {
    std::fs::create_dir_all(path)?;
    log::info!("Create directory {}", path);
    Ok(())
}

pub fn write_file(path: &str, content: &str) -> std::io::Result<()> {
    log::info!("Write file {}", path);

    if let Some(parent) = std::path::Path::new(path).parent() {
        std::fs::create_dir_all(parent)?;
    }

    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(path)?;
    file.write_all(content.as_bytes())?;
    Ok(())
}

/// Helper function to update GRUB_CMDLINE_LINUX_DEFAULT in /mnt/etc/default/grub
pub fn update_grub_cmdline(key: &str, value: &str) {
    let grub_path = "/mnt/etc/default/grub";
    let content = std::fs::read_to_string(grub_path).unwrap_or_else(|_| String::from("GRUB_CMDLINE_LINUX_DEFAULT=\"\""));
    
    let mut found = false;
    let mut new_lines: Vec<String> = content.lines().map(|line| {
        if line.trim().starts_with("GRUB_CMDLINE_LINUX_DEFAULT=") {
            found = true;
            
            let current_params = line.split('"').nth(1).unwrap_or("");
            let mut params: Vec<&str> = current_params.split_whitespace().collect();

            // Remove existing instances of the key (e.g., if we're setting lsm=, remove old lsm=)
            let prefix = if key.contains('=') { key.split('=').next().unwrap() } else { key };
            params.retain(|p| !p.starts_with(prefix));

            params.push(value);

            format!("GRUB_CMDLINE_LINUX_DEFAULT=\"{}\"", params.join(" "))
        } else {
            line.to_string()
        }
    }).collect();

    if !found {
        new_lines.push(format!("GRUB_CMDLINE_LINUX_DEFAULT=\"{}\"", value));
    }

    std::fs::write(grub_path, new_lines.join("\n")).unwrap();
}