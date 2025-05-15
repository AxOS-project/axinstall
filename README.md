
# Axinstall-cli

axinstall-cli is an installer backend for AxOS.

## Backend usage

### Autopartition the drive
```sh
# autopartition /dev/sda with efi enabled
# axinstall-cli partition auto /dev/sda --efi

# autopartition /dev/nvmen0 with efi disabled
# axinstall-cli partition auto /dev/nvmen0
```

### Install base packages
```sh
# axinstall-cli install-base
```

### Install bootloader
```sh
# install as efi with esp being /boot/efi
# axinstall-cli bootloader grub-efi /boot/efi

# install as legacy on /dev/sda
# axinstall-cli bootloader grub-legacy /dev/sda
```

### Generate fstab
```sh
# axinstall-cli genfstab
```

### Configuring locale settings
```sh
# set the keyboard layout to colemak, the timezone to Europe/Berlin and set en_US.UTF-8 as the locale
# axinstall-cli locale colemak Europe/Berlin en_US.UTF-8 UTF-8
```

### Configure network settings
```sh
# set the hostname to getcryst.al with ipv6 disabled
# axinstall-cli networking getcryst.al 

# set the hostname to getcryst.al with ipv6 enabled
# axinstall-cli networking getcryst.al --ipv6
```

### Setup zramd
```sh
# install and enable zramd
# axinstall-cli zramd
```

### Configure users
```sh
# make a new user called nonRootHaver, without sudo, easytohack as the password and bash as the default shell
# axinstall-cli users new-user nonRootHaver easytohack bash

# make a user called rootHaver, with sudo, omgsosuperhardtohack as the password and fish as the default shell
# axinstall-cli users new-user rootHaver omgsuperhardtohack fish --hasroot
```

### Set root password
```sh
# set the root password to 'muchSecurity,veryHardToHack'
# axinstall-cli users root-password muchSecurity,veryHardToHack
```

### Install a desktop environment
```sh
# install onyx
# axinstall-cli desktops onyx

# install gnome
# axinstall-cli desktops gnome
```

### Setup nvidia
```sh
# axinstall-cli nvidia
```

### Setup keyring
```sh
# axinstall-cli setup-keyring
```

### Install a user kit
```sh
# axinstall-cli user-kit artist
# axinstall-cli user-kit developer
# axinstall-cli user-kit hacker
```

### Debug logging

debug messages:
```sh
# axinstall-cli -v
```

traces:
```sh
# axinstall-cli -vv
```


## Non-secret Secret
$ echo "axinstall-cli_UWU=true" >> ~/.zshrc 

$ echo "axinstall-cli_UWU=true" >> ~/.bashrc 

$ set -Ux axinstall-cli_UWU true 


if you want to have your log and crash output be “cute”

## 🙌 Contributing

This project uses `rustup`, to set up `cargo` for **Jade** development, please follow the guidelines below:


#### Debug/development builds

- `cargo build`

#### Optimised/release builds

- `cargo build --release`



## 📜 License

[GPLv3-only](https://choosealicense.com/licenses/gpl-3.0/)

## Credits

This project is forked from Crystal Jade
