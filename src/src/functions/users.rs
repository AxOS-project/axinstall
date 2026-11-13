use crate::internal::exec::*;
use crate::internal::*;
use std::process::Command;

pub fn new_user(username: &str, hasroot: bool, password: &str, do_hash_pass: bool, shell: &str) {
    let shell: &str = shell;
    let password_str = if do_hash_pass {
        let hashed_pass = hash_pass(password).stdout;
        match String::from_utf8(hashed_pass) {
            Ok(v) => v,
            Err(e) => panic!("Failed to hash password, invalid UTF-8 sequence {}", e),
        }
    } else {
        password.to_string()
    };
    let clean_pass = password_str.trim().trim_matches('\'');
    let quoted_pass = format!("'{clean_pass}'");

    let shell_to_install = match shell {
        "bash" => "bash",
        "csh" => "tcsh",
        "fish" => "fish",
        "tcsh" => "tcsh",
        "zsh" => "zsh",
        &_ => "bash",
    };
    install::install(vec![String::from(shell_to_install)], true);
    let shell_path = match shell {
        "bash" => "/bin/bash",
        "csh" => "/usr/bin/csh",
        "fish" => "/usr/bin/fish",
        "tcsh" => "/usr/bin/tcsh",
        "zsh" => "/usr/bin/zsh",
        &_ => "/usr/bin/bash",
    };
    exec_eval(
        exec_chroot(
            "useradd",
            vec![
                String::from("-m"),
                String::from("-s"),
                String::from(shell_path),
                String::from("-p"),
                quoted_pass,
                String::from(username),
            ],
        ),
        format!("Create user {}", username).as_str(),
    );
    if hasroot {
        exec_eval(
            exec_chroot(
                "usermod",
                vec![
                    String::from("-aG"),
                    String::from("wheel"),
                    String::from(username),
                ],
            ),
            format!("Add user {} to wheel group", username).as_str(),
        );
        files_eval(
            files::sed_file(
                "/mnt/etc/sudoers",
                "# %wheel ALL=(ALL:ALL) ALL",
                "%wheel ALL=(ALL:ALL) ALL",
            ),
            "Add wheel group to sudoers",
        );
        files_eval(
            files::append_file("/mnt/etc/sudoers", "\nDefaults pwfeedback\n"),
            "Add pwfeedback to sudoers",
        );
        files_eval(
            files::create_directory("/mnt/var/lib/AccountsService/users/"),
            "Create /mnt/var/lib/AccountsService",
        );
        files::create_file(&format!("/mnt/var/lib/AccountsService/users/{}", username));
        let session_line =
            if std::path::Path::new("/mnt/usr/share/wayland-sessions/sleex.desktop").exists() {
                "Session=sleex\n"
            } else if std::path::Path::new("/mnt/usr/share/xsessions/theom.desktop").exists() {
                "Session=theom\n"
            } else if std::path::Path::new("/mnt/usr/share/wayland-sessions/plasma.desktop")
                .exists()
                || std::path::Path::new("/mnt/usr/share/xsessions/plasma.desktop").exists()
            {
                "Session=plasma\n"
            } else {
                ""
            };
        files_eval(
            files::append_file(
                &format!("/mnt/var/lib/AccountsService/users/{}", username),
                &format!("[User]\n{}SystemAccount=false", session_line),
            ),
            format!("Populate AccountsService user file for {}", username).as_str(),
        )
    }
}

pub fn hash_pass(password: &str) -> std::process::Output {
    let output = Command::new("openssl")
        .args(["passwd", "-6", password])
        .output()
        .expect("Failed to hash password");
    output
}

pub fn root_pass(root_pass: &str) {
    let clean_pass = root_pass.trim().trim_matches('\'');
    if clean_pass.is_empty() {
        log::info!("Root password is empty, locking root account");
        exec_eval(
            exec_chroot("passwd", vec![String::from("-l"), String::from("root")]),
            "lock root account",
        );
        return;
    }
    exec_eval(
        exec_chroot(
            "usermod",
            vec![
                String::from("--password"),
                format!("'{}'", clean_pass),
                String::from("root"),
            ],
        ),
        "set root password",
    );
}
