
use crate::args::DesktopSetup;
use crate::internal::exec::*;
use crate::internal::*;

/// Will install the provided desktop on the installation
pub fn install_desktop_setup(desktop_setup: DesktopSetup) {
    log::debug!("Installing {:?}", desktop_setup);
    match desktop_setup {
        DesktopSetup::Kde => install_kde(),
        DesktopSetup::Calla => install_calla(),
        DesktopSetup::Sleex => install_sleex(),
        DesktopSetup::Theom => install_theom(),
        DesktopSetup::None => log::debug!("No desktop setup selected"),
    }
    install_networkmanager();
}

fn install_networkmanager() {
    install(vec![String::from("networkmanager")], true);
    exec_eval(
        exec_chroot(
            "systemctl",
            vec![String::from("enable"), String::from("NetworkManager")],
        ),
        "Enable network manager",
    );
}

fn install_calla() {
    install(vec![
        String::from("calla"),
        String::from("alacritty"),
        String::from("nautilus"),
        String::from("polkit-gnome"),
        String::from("cbatticon"),
        String::from("blueman"),
        String::from("ttf-roboto"),
        String::from("noto-fonts-emoji"),
        String::from("ttf-material-icons-git"),
        String::from("ttf-material-design-icons-extended"),
        String::from("playerctl"),
        String::from("redshift"),
        String::from("xsettingsd"),
        String::from("galculator"),
        String::from("baobab"),
        String::from("gnome-characters"),
        String::from("mousepad"),
        String::from("gparted"),
        String::from("wmctrl"),
        String::from("libinput-gestures"),
        String::from("lollypop"),
    ], true);
    enable_dm("sddm");
}


fn install_kde() {
    install(vec![
        String::from("plasma-meta"),
        String::from("konsole"),
        String::from("kate"),
        String::from("dolphin"),
        String::from("ark"),
        String::from("plasma-workspace"),
        String::from("axskel"),
        String::from("papirus-icon-theme"), 
    ], true);
    enable_dm("sddm");
}


fn install_sleex() {
    install(vec![
        // Hyprland stuff
            String::from("hyprland"),
            String::from("hyprlang"),
            String::from("hyprcursor"),
            String::from("hyprutils"),
            String::from("hyprlock"),
            String::from("hyprpicker"),
            String::from("hyprwayland-scanner"),
            // AxOS stuff
            String::from("sleex"),
            String::from("sleex-optional"),

            // Other stuff
            String::from("fastfetch"),
            String::from("firefox"),
            String::from("pipewire-pulse"),
            String::from("papirus-icon-theme"),
            String::from("inxi"),
            String::from("power-profiles-daemon"),
            String::from("fwupd"),
            String::from("gnome-autoar"),
            String::from("gnome-system-monitor"),
            String::from("baobab"),
            String::from("gparted"),
            String::from("gnome-calculator"),
            String::from("loupe"),
            String::from("nwg-displays") ], true);
    enable_dm("sddm");
    set_sddm_sleex_default();
}

fn install_theom() {
    install(vec![
        String::from("theom"),
        String::from("gammastep"),
        String::from("mousepad")
        ], true);
    enable_dm("sddm");
    set_sddm_theom_default();
}

fn set_sddm_sleex_default() {
    exec_eval(
        exec_chroot(
            "mv",
            vec![
                String::from("/usr/share/wayland-sessions/hyprland.desktop"),
                String::from("/usr/share/wayland-sessions/hyprland.desktop.hidden"),
            ],
        ),
        "Rename hyprland.desktop to hyprland.desktop.hidden",
    );
    exec_eval(
        exec_chroot(
            "mv",
            vec![
                String::from("/usr/share/wayland-sessions/hyprland-uwsm.desktop"),
                String::from("/usr/share/wayland-sessions/hyprland-uwsm.desktop.hidden"),
            ],
        ),
        "Rename hyprland-uwsm.desktop to hyprland-uwsm.desktop.hidden",
    );
}

fn set_sddm_theom_default() {
    soft_exec_eval(
        exec_chroot(
            "mv",
            vec![
                String::from("/usr/share/xsessions/i3.desktop"),
                String::from("/usr/share/xsessions/i3.desktop.hidden"),
            ],
        ),
        "Rename i3.desktop to i3.desktop.hidden",
    ); 
    soft_exec_eval(
        exec_chroot(
            "mv",
            vec![
                String::from("/usr/share/xsessions/i3-with-shmlog.desktop"),
                String::from("/usr/share/xsessions/i3-with-shmlog.desktop.hidden"),
            ],
        ),
        "Rename i3-with-shmlog.desktop to i3-with-shmlog.desktop.hidden",
    ); 
}

fn enable_dm(dm: &str) {
    log::debug!("Enabling {}", dm);
    exec_eval(
        exec_chroot("systemctl", vec![String::from("enable"), String::from(dm)]),
        format!("Enable {}", dm).as_str(),
    );
}
