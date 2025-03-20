//! Reads live data from an AMD, NVIDIA, or Intel Arc GPU.

mod amd;
mod intel;
mod nvidia;

use crate::{error, warning};
use std::{fs::read_to_string, process::exit};

pub enum Gpu {
    Amd(amd::Gpu),
    Nvidia(nvidia::Gpu),
    Intel(intel::Gpu),
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
            Gpu::Nvidia(nvidia) => nvidia.get_temp(fahrenheit),
            Gpu::Intel(intel) => intel.get_temp(fahrenheit),
            Gpu::None => 0,
        }
    }

    pub fn get_usage(&self) -> u8 {
        match &self {
            Gpu::Amd(amd) => amd.get_usage(),
            Gpu::Nvidia(nvidia) => nvidia.get_usage(),
            Gpu::Intel(intel) => intel.get_usage(),
            Gpu::None => 0,
        }
    }

    pub fn get_power(&self) -> u16 {
        match &self {
            Gpu::Amd(amd) => amd.get_power(),
            Gpu::Nvidia(nvidia) => nvidia.get_power(),
            Gpu::Intel(intel) => intel.get_power(),
            Gpu::None => 0,
        }
    }

    pub fn get_frequency(&self) -> u16 {
        match &self {
            Gpu::Amd(amd) => amd.get_frequency(),
            Gpu::Nvidia(nvidia) => nvidia.get_frequency(),
            Gpu::Intel(intel) => intel.get_frequency(),
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
        } else if device.contains("808656") || device.contains("8086E2") {
            //56xx (ARC A series PCI_ID) and E2xx (Arc B series PCI_ID)
            // can't use the name "i915", because it is used by the intel IGPUS too
            return "intel".to_owned();
        }
    }
    "".to_owned()
}
