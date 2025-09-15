//! Reads live data from an AMD, Intel, or NVIDIA GPU.

mod amd;
mod intel;
mod nvidia;
pub mod pci;

use crate::monitor::gpu::pci::PciDevice;

pub enum Gpu {
    Amd(amd::Gpu),
    Intel(intel::Gpu),
    Nvidia(nvidia::Gpu),
    None,
}

impl Gpu {
    pub fn new(pci_device: Option<PciDevice>) -> Self {
        match pci_device {
            Some(gpu) => match gpu.vendor {
                pci::Vendor::Amd => Gpu::Amd(amd::Gpu::new(&gpu.address)),
                pci::Vendor::Intel => Gpu::Intel(intel::Gpu::new(&gpu.address)),
                pci::Vendor::Nvidia => Gpu::Nvidia(nvidia::Gpu::new(&gpu.address)),
            }
            None => Gpu::None,
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
