pub mod ag_series;
pub mod ak_series;
pub mod ch_series;
pub mod ld_series;
pub mod ls_series;

use crate::error;
use std::process::exit;
use hidapi::HidDevice;

pub const DEFAULT_VENDOR: u16 = 13875;

#[derive(PartialEq)]
pub enum Mode {
    Default,
    Auto,
    Temperature,
    Usage,
    Power,
}

impl Mode {
    pub fn symbol(&self) -> &'static str {
        match self {
            Mode::Default => "",
            Mode::Auto => "auto",
            Mode::Temperature => "temp",
            Mode::Usage => "usage",
            Mode::Power => "power",
        }
    }

    pub fn get(symbol: &str) -> Option<Mode> {
        match symbol {
            "auto" => Some(Self::Auto),
            "temp" => Some(Self::Temperature),
            "usage" => Some(Self::Usage),
            "power" => Some(Self::Power),
            _ => None,
        }
    }

    pub fn support_error(&self) -> Mode {
        error!(format!("Display mode \"{}\" is not supported on your device", self.symbol()));
        exit(1);
    }
}

pub fn device_error() -> HidDevice {
    error!("Failed to access the USB device");
    eprintln!("       Try to run the program as root or give permission to the neccesary resources.");
    eprintln!("       You can find instructions about rootless mode on GitHub.");
    exit(1);
}
