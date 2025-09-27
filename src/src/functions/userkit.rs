use std::vec;

use crate::args::UserKit;
use crate::internal::*;

pub fn install_userkit(kit: UserKit) {
    log::debug!("Installing {:?}", kit);

    match kit {
        UserKit::Developer => install_dev(),
        UserKit::Hacker => install_hacks(),
        UserKit::Artist => install_artist(),
        UserKit::Entertainment => install_entertainment(),
        UserKit::Office => install_office(),
    }
}

fn install_dev() {
    install(vec![String::from("axos-developer-kit")], false);
}

fn install_hacks() {
    install(vec![String::from("axos-hacker-kit")], false);
}

fn install_artist() {
    install(vec![String::from("axos-artist-kit")], false);
}

fn install_office() {
    install(vec![String::from("axos-office-kit")], false);
}

fn install_entertainment() {
    install(vec![String::from("axos-entertainment-kit")], false)
}