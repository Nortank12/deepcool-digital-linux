pub mod ag_series;
pub mod ak_series;
pub mod ak400_pro;
pub mod ak620_pro;
pub mod ch_series;
pub mod ch_series_gen2;
pub mod ch510;
pub mod ld_series;
pub mod lp_series;
pub mod lq_series;
pub mod ls_series;

use crate::error;
use std::{process::exit, time::Duration};
use hidapi::HidDevice;

pub const DEFAULT_VENDOR_ID: u16 = 13875;
pub const CH510_VENDOR_ID: u16 = 13523;
pub const CH510_PRODUCT_ID: u16 = 4352;

pub const AUTO_MODE_INTERVAL: Duration = Duration::from_millis(5000);

#[derive(PartialEq)]
pub enum Mode {
    Default,
    Auto,
    CpuTemperature,
    CpuUsage,
    CpuPower,
    CpuFrequency,
    CpuFan,
    GpuTemperature,
    GpuUsage,
    GpuPower,
    Cpu,
    Gpu,
    Psu,
}

impl Mode {
    pub const fn symbol(&self) -> &'static str {
        match self {
            Mode::Default => "",
            Mode::Auto => "auto",
            Mode::CpuTemperature => "cpu_temp",
            Mode::CpuUsage => "cpu_usage",
            Mode::CpuPower => "cpu_power",
            Mode::CpuFrequency => "cpu_freq",
            Mode::CpuFan => "cpu_fan",
            Mode::GpuTemperature => "gpu_temp",
            Mode::GpuUsage => "gpu_usage",
            Mode::GpuPower => "gpu_power",
            Mode::Cpu => "cpu",
            Mode::Gpu => "gpu",
            Mode::Psu => "psu",
        }
    }

    pub fn get(symbol: &str) -> Option<Mode> {
        match symbol {
            "auto" => Some(Self::Auto),
            "cpu_temp" => Some(Self::CpuTemperature),
            "cpu_usage" => Some(Self::CpuUsage),
            "cpu_power" => Some(Self::CpuPower),
            "cpu_freq" => Some(Self::CpuFrequency),
            "cpu_fan" => Some(Self::CpuFan),
            "gpu_temp" => Some(Self::GpuTemperature),
            "gpu_usage" => Some(Self::GpuUsage),
            "gpu_power" => Some(Self::GpuPower),
            "cpu" => Some(Self::Cpu),
            "gpu" => Some(Self::Gpu),
            "psu" => Some(Self::Psu),
            _ => None,
        }
    }

    pub fn support_error(&self) -> Mode {
        error!(format!("Display mode \"{}\" is not supported on your device", self.symbol()));
        exit(1);
    }

    pub fn support_error_secondary(&self) -> Mode {
        error!(format!("Secondary display mode \"{}\" is not supported on your device", self.symbol()));
        exit(1);
    }
}

pub fn device_error() -> HidDevice {
    error!("Failed to access the USB device");
    eprintln!("       Try to run the program as root or give permission to the neccesary resources.");
    eprintln!("       You can find instructions about rootless mode on GitHub.");
    exit(1);
}
