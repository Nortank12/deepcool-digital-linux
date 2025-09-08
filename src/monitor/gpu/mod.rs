//! Reads live data from an AMD, Intel, or NVIDIA GPU.

mod amd;
mod intel;
mod nvidia;
pub mod pci;

use crate::{error, warning};
use std::{fs::read_to_string, process::exit};

pub enum Gpu {
    Amd(amd::Gpu),
    Intel(intel::Gpu),
    Nvidia(nvidia::Gpu),
    None,
}

impl Gpu {
    pub fn new() -> Self {
        match get_vendor().as_str() {
            "amd" => Gpu::Amd(amd::Gpu::new()),
            "nvidia" => Gpu::Nvidia(nvidia::Gpu::new()),
            "intel" => Gpu::Intel(intel::Gpu::new()),
            _ => {
                warning!("No dedicated GPU was found");
                eprintln!("         GPU information will not be displayed.");
                return Gpu::None;
            }
        }
    }

    pub fn get_temp(&self, fahrenheit: bool) -> u8 {
        match &self {
            Gpu::Amd(amd) => amd.get_temp(fahrenheit),
            Gpu::Intel(intel) => intel.get_temp(fahrenheit),
            Gpu::Nvidia(nvidia) => nvidia.get_temp(fahrenheit),
            Gpu::None => 0,
        }
    }

    pub fn get_usage(&self) -> u8 {
        match &self {
            Gpu::Amd(amd) => amd.get_usage(),
            Gpu::Intel(intel) => intel.get_usage(),
            Gpu::Nvidia(nvidia) => nvidia.get_usage(),
            Gpu::None => 0,
        }
    }

    pub fn get_power(&self) -> u16 {
        match &self {
            Gpu::Amd(amd) => amd.get_power(),
            Gpu::Intel(intel) => intel.get_power(),
            Gpu::Nvidia(nvidia) => nvidia.get_power(),
            Gpu::None => 0,
        }
    }

    pub fn get_frequency(&self) -> u16 {
        match &self {
            Gpu::Amd(amd) => amd.get_frequency(),
            Gpu::Intel(intel) => intel.get_frequency(),
            Gpu::Nvidia(nvidia) => nvidia.get_frequency(),
            Gpu::None => 0,
        }
    }
}

/// Get GPU vendor from PCI device list.
fn get_vendor() -> String {
    let pci_devices = read_to_string("/proc/bus/pci/devices").unwrap_or_else(|_| {
        error!("Cannot read PCI devices");
        exit(1);
    });

    for device in pci_devices.lines() {
        if device.ends_with("amdgpu") {
            return "amd".to_owned();
        } else if device.ends_with("nvidia") {
            return "nvidia".to_owned();
        } else if device.ends_with("i915") {
            let pci_id = device.split("\t").nth(1).unwrap();
            // Check the first 2 digits of the device ID:
            // 56xx: Arc A-Series
            // E2xx: Arc B-Series
            if ["56", "e2"].contains(&&pci_id[4..6]) {
                return "intel".to_owned();
            }
        }
    }

    "".to_owned()
}
