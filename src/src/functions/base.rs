use crate::internal::exec::*;
use crate::internal::*;
use log::warn;
use std::path::PathBuf;

/// ### Installs the base packages for the system to proprely work. 
/// This functions takes the kernel to install as an argument. 
/// - eg: `install_base_packages("linux-zen")`.
pub fn install_base_packages(kernel: String) {
    std::fs::create_dir_all("/mnt/etc").unwrap();
    if std::path::Path::new("/etc/axos-version").exists() {
        files::copy_file("/etc/axos-version", "/mnt/etc/axos-version");
    }
    let kernel_to_install = if kernel.is_empty() {
        "linux"
    } else {
        match kernel.as_str() {
            "linux" => "linux",
            "linux-lts" => "linux-lts",
            "linux-zen" => "linux-zen",
            "linux-hardened" => "linux-hardened",
            _ => {
                warn!("Unknown kernel: {}, using default instead", kernel);
                "linux"
            }
        }
    };
    install::install(vec![
        // Base Arch
        String::from("base"),
        String::from(kernel_to_install),
        format!("{kernel_to_install}-headers"),
        String::from("linux-firmware"),
        String::from("sof-firmware"), // required for newer sound cards [ESSENTIAL]
        String::from("man-db"),
        String::from("man-pages"),
        String::from("nano"),
        String::from("sudo"),
        String::from("curl"),
        String::from("wget"),
        String::from("archlinux-keyring"),
        String::from("grep"),
        // Base AxOS
        String::from("about-axos"),
        String::from("axos-hooks"),
        String::from("axos-hooks-extra"),
        String::from("plymouth-theme-axos"),
        String::from("epsilon"),
        String::from("axos-wallpaper-collection"),
        String::from("grub-theme-axos"),
        String::from("axctl"),
        // Extra goodies
        String::from("fastfetch"), // stable now contains the AxOS ASCII logo
        String::from("axos/plymouth"),
        String::from("plymouth-theme-axos"),
        // Fonts
        String::from("noto-fonts"),
        String::from("noto-fonts-cjk"),
        String::from("noto-fonts-extra"),
        String::from("ttf-nerd-fonts-symbols-common"),
        // Common packages for all desktops
        String::from("pipewire"),
        String::from("pipewire-pulse"),
        String::from("pipewire-alsa"),
        // String::from("pipewire-jack"),
        String::from("wireplumber"),
        String::from("power-profiles-daemon"),
        String::from("cups"),
        String::from("cups-pdf"),
        String::from("bluez"),
        String::from("bluez-cups"),
        String::from("zsh-completions"),
        String::from("ttf-liberation"),
        String::from("dnsmasq"),
        String::from("xdg-user-dirs"),
        String::from("firefox"),
        String::from("bash"),
        String::from("bash-completion"),
        String::from("inxi"),
        String::from("acpi"),
        String::from("htop"),
        String::from("fwupd"),
        String::from("ntp"),
        String::from("kf6"),
        String::from("packagekit-qt6"),
        String::from("gnome-packagekit"),
        String::from("packagekit"),
        String::from("unzip"),
        // Graphic drivers
        String::from("xf86-video-amdgpu"),
        String::from("xf86-video-intel"),
        String::from("xf86-video-nouveau"),
        // String::from("xf86-video-vmware"),
        String::from("xf86-video-vesa"),
        String::from("mesa"),
        String::from("vulkan-intel"),
        String::from("vulkan-radeon"),
        String::from("vulkan-icd-loader"),
        // String::from("virtualbox-guest-utils"),
        // Chaotic-AUR
        String::from("chaotic-keyring"),
        String::from("chaotic-mirrorlist"),
        // Display manager
        String::from("sddm"),
        String::from("sddm-theme-axos"),
        // MAC System
        String::from("apparmor"),
    ], true);
    files::copy_file("/etc/pacman.conf", "/mnt/etc/pacman.conf");
    if !std::path::Path::new("/mnt/etc/pacman.d/chaotic-mirrorlist").exists() {
        strip_chaotic_aur("/mnt/etc/pacman.conf");
    }

    exec_eval(
        exec_chroot(
            "systemctl",
            vec![String::from("enable"), String::from("bluetooth")],
        ),
        "Enable bluetooth",
    );

    exec_eval(
        exec_chroot(
            "systemctl",
            vec![String::from("enable"), String::from("cups")],
        ),
        "Enable CUPS",
    );
}

/// ### This will setup the archlinux keyring using the `pacman-key` command.
/// It will first initialize it with the `--init` flag, then populate it with the `--populate` flag.
pub fn setup_archlinux_keyring() {
    exec_eval(
        exec_chroot("pacman-key", vec![String::from("--init")]),
        "Initialize pacman keyring",
    );
    exec_eval(
        exec_chroot(
            "pacman-key",
            vec![String::from("--populate")],
        ),
        "Populate pacman keyring",
    );
}

/// ### Function to add the Flathub remote for Flatpak.
/// This is not available in axinstall GUI because I don't like flatpak
pub fn install_flatpak() {
    install(vec![String::from("flatpak")], false);
    exec_eval(
        exec_chroot(
            "flatpak",
            vec![
                String::from("remote-add"),
                String::from("--if-not-exists"),
                String::from("flathub"),
                String::from("https://dl.flathub.org/repo/flathub.flatpakrepo"),
            ],
        ),
        "Add Flathub remote",
    );
}

/// Generates the file system tab in `/mnt/etc/fstab` using the `genfstab` command.
pub fn genfstab() {
    exec_eval(
        exec(
            "bash",
            vec![
                String::from("-c"),
                String::from("genfstab -U /mnt >> /mnt/etc/fstab"),
            ],
        ),
        "Generate fstab",
    );
}

/// ### Install GRUB
/// This will first install the patched grub from the AxOS repo, then install efibootmgr and os-prober.
/// 
/// Then, it will use `grub-install` to install the grub in the efidir, provided as an arg to this function.
pub fn install_bootloader_efi(efidir: PathBuf) {
    install::install(vec![String::from("axos/grub"), String::from("efibootmgr"), String::from("os-prober")], true);
    let raw = efidir.to_string_lossy();
    let trimmed = raw.trim_start_matches("/mnt").trim_start_matches('/');
    let efi_str = format!("/{}", trimmed.trim_end_matches('/'));
    let host_efi = format!("/mnt{}", efi_str);
    if !std::path::Path::new(&host_efi).exists() {
        crash(format!("The efidir {host_efi} doesn't exist"), 1);
    }
    exec_eval(
        exec_chroot(
            "grub-install",
            vec![
                String::from("--target=x86_64-efi"),
                format!("--efi-directory={}", efi_str),
                String::from("--bootloader-id=axos"),
                String::from("--removable"),
            ],
        ),
        "install grub as efi with --removable",
    );
    exec_eval(
        exec_chroot(
            "grub-install",
            vec![
                String::from("--target=x86_64-efi"),
                format!("--efi-directory={}", efi_str),
                String::from("--bootloader-id=axos"),
            ],
        ),
        "install grub as efi without --removable",
    );
    exec_eval(
        exec_chroot(
            "systemctl",
            vec![String::from("enable"), String::from("apparmor.service")],
        ),
        "enable AppArmor service",
    );
    files::update_grub_cmdline("lsm", "lsm=landlock,lockdown,yama,integrity,apparmor,bpf");
    exec_eval(
        exec_chroot(
            "grub-mkconfig",
            vec![String::from("-o"), String::from("/boot/grub/grub.cfg")],
        ),
        "create grub.cfg",
    );
}

/// Install GRUB in legacy BIOS mode
pub fn install_bootloader_legacy(device: PathBuf) {
    install::install(vec![String::from("axos/grub"), String::from("os-prober")], true);
    if !device.exists() {
        crash(format!("The device {device:?} does not exist"), 1);
    }
    let device = device.to_string_lossy().to_string();
    exec_eval(
        exec_chroot(
            "grub-install",
            vec![String::from("--target=i386-pc"), device],
        ),
        "install grub as legacy",
    );
    exec_eval(
        exec_chroot(
            "systemctl",
            vec![String::from("enable"), String::from("apparmor.service")],
        ),
        "enable AppArmor service",
    );
    files::update_grub_cmdline("lsm", "lsm=landlock,lockdown,yama,integrity,apparmor,bpf");
    exec_eval(
        exec_chroot(
            "grub-mkconfig",
            vec![String::from("-o"), String::from("/boot/grub/grub.cfg")],
        ),
        "create grub.cfg",
    );
}

pub fn strip_chaotic_aur(pacman_conf_path: &str) {
    if let Ok(content) = std::fs::read_to_string(pacman_conf_path) {
        let mut out = Vec::new();
        let mut in_chaotic = false;
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with('[') && trimmed.ends_with(']') {
                if trimmed == "[chaotic-aur]" {
                    in_chaotic = true;
                    continue;
                } else {
                    in_chaotic = false;
                }
            }
            if in_chaotic {
                continue;
            }
            if trimmed.starts_with("#Chaotic aur") {
                continue;
            }
            out.push(line);
        }
        let _ = std::fs::write(pacman_conf_path, out.join("\n") + "\n");
    }
}

pub fn configure_chaotic_aur(enable: bool) {
    let pacman_conf = "/mnt/etc/pacman.conf";
    if !enable || !std::path::Path::new("/mnt/etc/pacman.d/chaotic-mirrorlist").exists() {
        strip_chaotic_aur(pacman_conf);
    } else if let Ok(content) = std::fs::read_to_string(pacman_conf) {
        if !content.contains("[chaotic-aur]") {
            let _ = files::append_file(
                pacman_conf,
                "[chaotic-aur]\nSigLevel = Optional TrustAll\nInclude = /etc/pacman.d/chaotic-mirrorlist",
            );
        }
    }
}

/// Copy some files from the live to the installation
pub fn copy_live_config() {
    files::copy_file("/etc/pacman.conf", "/mnt/etc/pacman.conf");
    if !std::path::Path::new("/mnt/etc/pacman.d/chaotic-mirrorlist").exists() {
        strip_chaotic_aur("/mnt/etc/pacman.conf");
    }
    if std::path::Path::new("/etc/axos-version").exists() {
        files::copy_file("/etc/axos-version", "/mnt/etc/axos-version");
    }
    if std::path::Path::new("/mnt/usr/bin/reload-releases").exists() {
        soft_exec_eval(
            exec_chroot("/usr/bin/reload-releases", vec![]),
            "Reload AxOS release info",
        );
    }
    std::fs::create_dir_all("/mnt/etc/sddm.conf.d").unwrap();
    if std::path::Path::new("/etc/sddm.conf.d/settings.conf").exists() {
        files::copy_file(
            "/etc/sddm.conf.d/settings.conf",
            "/mnt/etc/sddm.conf.d/settings.conf",
        );
    }
    if std::path::Path::new("/etc/sddm.conf").exists() {
        files::copy_file("/etc/sddm.conf", "/mnt/etc/sddm.conf");
    }
}

/// Will install basic NVIDIA packages, and then patch the grub and initcpio config to accept nvidia GPUs it's modules.
pub fn install_nvidia() {
    install(vec![
        String::from("dkms"),
        String::from("nvidia-dkms"),
        String::from("nvidia-utils"),
        String::from("egl-wayland"),
    ], true);

    // Apply nvidia module in grub
    files::update_grub_cmdline("nvidia-drm.modeset", "nvidia-drm.modeset=1");

    // Apply initcpio modules
    let mkinitcpio_content = std::fs::read_to_string("/mnt/etc/mkinitcpio.conf").unwrap_or_default();
    let mut mkinitcpio_conf_found = false;
    let mapped_lines: Vec<String> = mkinitcpio_content.lines().map(|line| {
        if line.trim_start().starts_with("MODULES=") && !line.trim_start().starts_with("#") {
            mkinitcpio_conf_found = true;
            "MODULES=(nvidia nvidia_modeset nvidia_uvm nvidia_drm)".to_string()
        } else {
            line.to_string()
        }
    }).collect();
    
    let mut final_lines = mapped_lines;
    if !mkinitcpio_conf_found {
        final_lines.push("MODULES=(nvidia nvidia_modeset nvidia_uvm nvidia_drm)".to_string());
    }
    let new_initcpio_content = final_lines.join("\n") + "\n";
    std::fs::write("/mnt/etc/mkinitcpio.conf", new_initcpio_content).unwrap();

    exec_eval(
        exec_chroot("mkinitcpio", vec![String::from("-P")]),
        "Regenerate initramfs with Nvidia modules",
    );
    exec_eval(
        exec_chroot(
            "grub-mkconfig",
            vec![String::from("-o"), String::from("/boot/grub/grub.cfg")],
        ),
        "Regenerate grub.cfg with Nvidia kernel parameters",
    );
}

/// Will enable swap (self explainatory)
pub fn enable_swap(size: u64) {
    let _ = std::fs::remove_file("/mnt/swapfile");
    let size_mb = size.to_string();
    exec_eval(
        exec(
            "fallocate",
            vec![
                String::from("-l"),
                format!("{}M", size_mb),
                String::from("/mnt/swapfile"),
            ],
        ),
        "Create swapfile",
    );
    exec_eval(
        exec(
            "chmod",
            vec![String::from("600"), String::from("/mnt/swapfile")],
        ),
        "Set swapfile permissions",
    );
    exec_eval(
        exec("mkswap", vec![String::from("/mnt/swapfile")]),
        "Format swapfile",
    );
    files::append_file("/mnt/etc/fstab", "\n/swapfile none swap defaults 0 0\n").unwrap();
}
