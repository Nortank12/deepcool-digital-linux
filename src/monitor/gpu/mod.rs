//! Reads live data from an AMD, NVIDIA, or Intel Arc GPU.

mod amd;
mod nvidia;
mod intel;

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
        } else if device.contains("i915") {
            let fields: Vec<&str> = device.split_whitespace().collect(); // creates a vector of substrings containing the PCI device information.
            if fields.len() > 2 {
                let pci_class = &fields[1][..2]; // PCI class code (first 2 hex digits)

                // Exclude iGPUs: Class code 03 (Display Controller) + subclass 80 (iGPU)
                if pci_class != "03" {
                    return "intel".to_owned();
                }
            }

        }
    }

    "".to_owned()
}
