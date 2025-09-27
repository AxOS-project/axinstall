use crate::internal::exec::*;
use crate::internal::*;
use log::warn;
use std::path::PathBuf;

pub fn install_base_packages(kernel: String) {
    std::fs::create_dir_all("/mnt/etc").unwrap();
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
    ], true);
    files::copy_file("/etc/pacman.conf", "/mnt/etc/pacman.conf");

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

pub fn setup_archlinux_keyring() {
    exec_eval(
        exec_chroot("pacman-key", vec![String::from("--init")]),
        "Initialize pacman keyring",
    );
    exec_eval(
        exec_chroot(
            "pacman-key",
            vec![String::from("--populate"), String::from("archlinux")],
        ),
        "Populate pacman keyring",
    );
}

// Function to add the Flathub remote for Flatpak
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

pub fn install_bootloader_efi(efidir: PathBuf) {
    install::install(vec![String::from("axos/grub"), String::from("efibootmgr"), String::from("os-prober")], true);
    let efidir = std::path::Path::new("/mnt").join(efidir);
    let efi_str = efidir.to_str().unwrap();
    if !std::path::Path::new(&format!("/mnt{efi_str}")).exists() {
        crash(format!("The efidir {efidir:?} doesn't exist"), 1);
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
            "grub-mkconfig",
            vec![String::from("-o"), String::from("/boot/grub/grub.cfg")],
        ),
        "create grub.cfg",
    );
}

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
            "grub-mkconfig",
            vec![String::from("-o"), String::from("/boot/grub/grub.cfg")],
        ),
        "create grub.cfg",
    );
}

pub fn copy_live_config() {
    files::copy_file("/etc/pacman.conf", "/mnt/etc/pacman.conf");
    files::copy_file("/etc/axos-version", "/mnt/etc/axos-version");
    std::fs::create_dir_all("/mnt/etc/sddm.conf.d").unwrap();
    files::copy_file(
        "/etc/sddm.conf.d/settings.conf",
        "/mnt/etc/sddm.conf.d/settings.conf",
    );
    files::copy_file("/etc/sddm.conf", "/mnt/etc/sddm.conf");
    // files::copy_file("/etc/mkinitcpio.conf", "/mnt/etc/mkinitcpio.conf"); // Why is this even there ???
}

pub fn install_nvidia() {
    install(vec![
        String::from("dkms"),
        String::from("nvidia"),
        String::from("nvidia-dkms"),
        String::from("nvidia-utils"),
        String::from("egl-wayland"),
    ], true);

    // Apply nvidia module in grub
    let grub_cmdline_content = std::fs::read_to_string("/mnt/etc/default/grub").unwrap_or_default();
    let mut grub_conf_found = false;
    let mut lines: Vec<String> = grub_cmdline_content.lines().map(|line| {
        if line.starts_with("GRUB_CMDLINE_LINUX_DEFAULT=") {
            grub_conf_found = true;
            if line.contains("nvidia-drm.modeset=1") {
                line.to_string() // Already there, do nothing
            } else {
                line.replace("GRUB_CMDLINE_LINUX_DEFAULT=\"", "GRUB_CMDLINE_LINUX_DEFAULT=\"nvidia-drm.modeset=1 ")
            }
        } else { line.to_string() }
    }).collect();
    if !grub_conf_found { lines.push("GRUB_CMDLINE_LINUX_DEFAULT=\"nvidia-drm.modeset=1\"".to_string()); }
    let new_grub_content = lines.join("\n");
    std::fs::write("/mnt/etc/default/grub", new_grub_content).unwrap();

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
    let new_initcpio_content = final_lines.join("\n");
    std::fs::write("/mnt/etc/mkinitcpio.conf", new_initcpio_content).unwrap();

}

pub fn enable_swap(size: u64) {
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
    std::fs::write("/mnt/etc/fstab", "\n/swapfile none swap defaults 0 0\n").unwrap();
}
