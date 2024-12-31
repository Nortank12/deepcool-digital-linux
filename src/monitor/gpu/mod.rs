//! Reads live data from an AMD or NVIDIA GPU.

mod amd;
mod nvidia;

use crate::error;
use std::{fs::read_to_string, process::exit};

pub enum Gpu {
    Amd(amd::Gpu),
    Nvidia(nvidia::Gpu),
}

impl Gpu {
    pub fn new() -> Self {
        match get_vendor().as_str() {
            "amd" => Gpu::Amd(amd::Gpu::new()),
            "nvidia" => Gpu::Nvidia(nvidia::Gpu::new()),
            _ => {
                error!("No supported GPU was found");
                exit(1);
            }
        }
    }

    pub fn get_temp(&self, fahrenheit: bool) -> u8 {
        match &self {
            Gpu::Amd(amd) => amd.get_temp(fahrenheit),
            Gpu::Nvidia(nvidia) => nvidia.get_temp(fahrenheit),
        }
    }

    pub fn get_usage(&self) -> u8 {
        match &self {
            Gpu::Amd(amd) => amd.get_usage(),
            Gpu::Nvidia(nvidia) => nvidia.get_usage(),
        }
    }

    pub fn get_power(&self) -> u16 {
        match &self {
            Gpu::Amd(amd) => amd.get_power(),
            Gpu::Nvidia(nvidia) => nvidia.get_power(),
        }
    }

    pub fn get_frequency(&self) -> u16 {
        match &self {
            Gpu::Amd(amd) => amd.get_frequency(),
            Gpu::Nvidia(nvidia) => nvidia.get_frequency(),
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
        }
    }

    "".to_owned()
}
