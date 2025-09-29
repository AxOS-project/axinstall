use crate::internal::exec::*;
use crate::internal::*;

/// Will set the timezone by creating a symlink between the `/usr/share/zoneinfo/wanted_timezone` and `/etc/localtime`.
/// 
/// Then, `hwclock` will sync the clock
pub fn set_timezone(timezone: &str) {
    exec_eval(
        exec_chroot(
            "ln",
            vec![
                "-sf".to_string(),
                format!("/usr/share/zoneinfo/{}", timezone),
                "/etc/localtime".to_string(),
            ],
        ),
        "Set timezone",
    );
    exec_eval(
        exec_chroot("hwclock", vec!["--systohc".to_string()]),
        "Set system clock",
    );
}

/// Will set the system locale by appending values to locale.gen and locale.conf
pub fn set_locale(locale: String) {
    files_eval(
        files::append_file("/mnt/etc/locale.gen", "en_US.UTF-8 UTF-8"),
        "add en_US.UTF-8 UTF-8 to locale.gen",
    );
    files::create_file("/mnt/etc/locale.conf");
    files_eval(
        files::append_file("/mnt/etc/locale.conf", "LANG=en_US.UTF-8"),
        "edit locale.conf",
    );
    for i in (0..locale.split(' ').count()).step_by(2) {
        files_eval(
            files::append_file(
                "/mnt/etc/locale.gen",
                &format!(
                    "{} {}\n",
                    locale.split(' ').collect::<Vec<&str>>()[i],
                    locale.split(' ').collect::<Vec<&str>>()[i + 1]
                ),
            ),
            "add locales to locale.gen",
        );
        if locale.split(' ').collect::<Vec<&str>>()[i] != "en_US.UTF-8" {
            files_eval(
                files::sed_file(
                    "/mnt/etc/locale.conf",
                    "en_US.UTF-8",
                    locale.split(' ').collect::<Vec<&str>>()[i],
                ),
                format!(
                    "Set locale {} in /etc/locale.conf",
                    locale.split(' ').collect::<Vec<&str>>()[i]
                )
                .as_str(),
            );
        }
    }
    exec_eval(exec_chroot("locale-gen", vec![]), "generate locales");
}

/// Will set the kb layout by editing `vconsole.conf` and `/etc/X11/xorg.conf.d/00-keyboard.conf`.
pub fn set_keyboard(keyboard: &str) {
    files::create_file("/mnt/etc/vconsole.conf");
    files_eval(
        files::append_file(
            "/mnt/etc/vconsole.conf",
            format!("KEYMAP={}", keyboard).as_str(),
        ),
        "set keyboard layout in vconsole",
    );

    files_eval(
        files::write_file(
            "/mnt/etc/X11/xorg.conf.d/00-keyboard.conf",
            &format!(
                "Section \"InputClass\"\n\
                 Identifier \"system-keyboard\"\n\
                 MatchIsKeyboard \"on\"\n\
                 Option \"XkbLayout\" \"{}\"\n\
                 EndSection\n",
                keyboard
            ),
        ),
        "set X11 keyboard layout",
    );
}