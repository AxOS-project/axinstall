use crate::args;
use crate::args::{DesktopSetup, PartitionMode};
use crate::functions::*;
use crate::internal::*;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Serialize, Deserialize)]
struct Config {
    partition: Partition,
    bootloader: Bootloader,
    locale: Locale,
    networking: Networking,
    users: Vec<Users>,
    rootpass: String,
    desktop: String,
    swap: u64,
    nvidia: bool,
    extra_packages: Vec<String>,
    kernel: String,
    artist_uk: bool,
    devel_uk: bool,
    hacker_uk: bool,
    office_uk: bool,
    entertainment_uk: bool,
    flatpak: bool,
}

#[derive(Serialize, Deserialize)]
struct Partition {
    device: String,
    mode: PartitionMode,
    efi: bool,
    partitions: Vec<String>,
}

#[derive(Serialize, Deserialize)]
struct Bootloader {
    r#type: String,
    location: String,
}

#[derive(Serialize, Deserialize)]
struct Locale {
    locale: Vec<String>,
    keymap: String,
    timezone: String,
}

#[derive(Serialize, Deserialize)]
struct Networking {
    hostname: String,
    ipv6: bool,
}

#[derive(Serialize, Deserialize)]
struct Users {
    name: String,
    password: String,
    hasroot: bool,
    shell: String,
}

pub fn read_config(configpath: PathBuf) {
    let data = std::fs::read_to_string(&configpath);
    match &data {
        Ok(_) => {
            log::debug!("[ \x1b[2;1;32mOK\x1b[0m ] Read config file {configpath:?}");
        }
        Err(e) => {
            crash(
                format!("Read config file {configpath:?}  ERROR: {}", e),
                e.raw_os_error().unwrap(),
            );
        }
    }
    let config: std::result::Result<Config, serde_json::Error> =
        serde_json::from_str(&data.unwrap());
    match &config {
        Ok(_) => {
            log::debug!("[ \x1b[2;1;32mOK\x1b[0m ] Parse config file {configpath:?}",);
        }
        Err(e) => {
            crash(format!("Parse config file {configpath:?}  ERROR: {}", e), 1);
        }
    }
    let config: Config = config.unwrap();
    log::info!("Block device to use : /dev/{}", config.partition.device);
    log::info!("Partitioning mode : {:?}", config.partition.mode);
    log::info!("Partitioning for EFI : {}", config.partition.efi);
    let mut partitions: Vec<args::Partition> = Vec::new();
    for partition in config.partition.partitions {
        partitions.push(args::Partition::new(
            partition.split(':').collect::<Vec<&str>>()[0].to_string(),
            partition.split(':').collect::<Vec<&str>>()[1].to_string(),
            partition.split(':').collect::<Vec<&str>>()[2].to_string(),
        ));
    }
    let device = PathBuf::from("/dev/").join(config.partition.device.as_str());
    partition::partition(
        device,
        config.partition.mode,
        config.partition.efi,
        &mut partitions,
    );
    base::install_base_packages(config.kernel);
    base::setup_archlinux_keyring();
    if config.flatpak {
        base::install_flatpak();
    }
    base::genfstab();
    println!();
    log::info!("Installing bootloader : {}", config.bootloader.r#type);
    log::info!("Installing bootloader to : {}", config.bootloader.location);
    base::install_bootloader_efi(PathBuf::from(config.bootloader.location));    
    println!();
    log::info!("Adding Locales : {:?}", config.locale.locale);
    log::info!("Using keymap : {}", config.locale.keymap);
    log::info!("Setting timezone : {}", config.locale.timezone);
    locale::set_locale(config.locale.locale.join(" "));
    locale::set_keyboard(config.locale.keymap.as_str());
    locale::set_timezone(config.locale.timezone.as_str());
    println!();
    log::info!("Hostname : {}", config.networking.hostname);
    log::info!("Enabling ipv6 : {}", config.networking.ipv6);
    network::set_hostname(config.networking.hostname.as_str());
    network::create_hosts();
    if config.networking.ipv6 {
        network::enable_ipv6();
    }
    println!();
    println!("---------");
    log::info!("Installing desktop : {:?}", config.desktop);
    /*if let Some(desktop) = &config.desktop {
        desktops::install_desktop_setup(*desktop);
    }*/
    match config.desktop.to_lowercase().as_str() {
        "kde" => desktops::install_desktop_setup(DesktopSetup::Kde),
        "plasma" => desktops::install_desktop_setup(DesktopSetup::Kde),
        "calla" => desktops::install_desktop_setup(DesktopSetup::Calla),
        "sleex" => desktops::install_desktop_setup(DesktopSetup::Sleex),
        "theom" => desktops::install_desktop_setup(DesktopSetup::Theom),
        "none/diy" => desktops::install_desktop_setup(DesktopSetup::None),
        _ => log::info!("No desktop setup selected!"),
    }
    println!();
    println!("---------");
    for i in 0..config.users.len() {
        log::info!("Creating user : {}", config.users[i].name);
        log::info!("Setting use password : {}", config.users[i].password);
        log::info!("Enabling root for user : {}", config.users[i].hasroot);
        log::info!("Setting user shell : {}", config.users[i].shell);
        users::new_user(
            config.users[i].name.as_str(),
            config.users[i].hasroot,
            config.users[i].password.as_str(),
            false,
            config.users[i].shell.as_str(),
        );
        println!("---------");
    }
    println!();
    log::info!("Setting root password : {}", config.rootpass);
    users::root_pass(config.rootpass.as_str());

    println!();
    log::info!("Copying live config");
    base::copy_live_config();
    println!();
    log::info!("Enabling nvidia : {}", config.nvidia);
    if config.nvidia {
        base::install_nvidia();
    }
    log::info!("Enabling swap: {}M ", config.swap);
    if config.swap > 0 {
        base::enable_swap(config.swap);
    }
    log::info!("Installing user kits");
    if config.artist_uk {
        userkit::install_userkit(args::UserKit::Artist);
    }
    if config.devel_uk {
        userkit::install_userkit(args::UserKit::Developer);
    }
    if config.hacker_uk {
        userkit::install_userkit(args::UserKit::Hacker);
    }
    if config.office_uk {
        userkit::install_userkit(args::UserKit::Office);
    }
    if config.entertainment_uk {
        userkit::install_userkit(args::UserKit::Entertainment);
    }
    log::info!("Extra packages : {:?}", config.extra_packages);
    let extra_packages: Vec<String> = config.extra_packages.clone();
    install(extra_packages);
    println!();
    println!("Installation finished! You may reboot now!")
}
