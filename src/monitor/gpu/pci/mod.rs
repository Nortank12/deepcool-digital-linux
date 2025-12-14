//! Identifies and enumarates GPUs as PCI devices.

mod pci_ids;

use crate::error;
use std::{fs::{read_dir, read_to_string}, process::exit};

#[derive(PartialEq, Eq, Hash, Clone, Copy)]
pub enum Vendor {
    Amd,
    Intel,
    Nvidia,
}

impl Vendor {
    pub const fn name(&self) -> &'static str {
        match self {
            Vendor::Amd => "AMD",
            Vendor::Intel => "Intel",
            Vendor::Nvidia => "NVIDIA",
        }
    }

    pub fn get(symbol: &str) -> Option<Vendor> {
        match symbol {
            "amd" => Some(Self::Amd),
            "intel" => Some(Self::Intel),
            "nvidia" => Some(Self::Nvidia),
            _ => None,
        }
    }
}

#[derive(Clone)]
pub struct PciDevice {
    pub vendor: Vendor,
    pub bus: u8,
    pub address: String,
    pub name: String,
}

fn parse_pci_addr(addr: &str) -> Option<(u16, u8, u8, u8)> {
    // PCI Address Format:
    // 0000:00:00.0 | <domain>:<bus>:<device>.<function>
    let mut parts = addr.split(|c| c == ':' || c == '.');
    let domain = u16::from_str_radix(parts.next()?, 16).ok()?;
    let bus = u8::from_str_radix(parts.next()?, 16).ok()?;
    let device = u8::from_str_radix(parts.next()?, 16).ok()?;
    let function = u8::from_str_radix(parts.next()?, 10).ok()?;
    Some((domain, bus, device, function))
}

fn parse_pci_id(id: &str) -> Option<(u16, u16)> {
    // PCI ID Format:
    // 0000:0000 | <vendor>:<device>
    let mut parts = id.split(':');
    let vendor = u16::from_str_radix(parts.next()?, 16).ok()?;
    let device = u16::from_str_radix(parts.next()?, 16).ok()?;
    Some((vendor, device))
}

/// Gets all GPUs from the PCI bus.
pub fn get_gpu_list() -> Vec<PciDevice> {
    let pci_devices = read_dir("/sys/bus/pci/devices").unwrap_or_else(|_| {
        error!("Cannot read PCI devices");
        exit(1);
    });

    let mut gpus = Vec::new();
    let gpu_names = pci_ids::get_device_names();

    for device in pci_devices {
        let dir = device.unwrap();
        let uevent_file = dir.path().join("uevent");

        match read_to_string(uevent_file) {
            Ok(data) => {
                let mut driver = None;
                let mut pci_id = None;
                let mut subsys_id = None;
                for line in data.lines() {
                    if let Some(value) = line.strip_prefix("DRIVER=") {
                        driver = Some(value);
                    } else if let Some(value) = line.strip_prefix("PCI_ID=") {
                        pci_id = Some(value);
                    } else if let Some(value) = line.strip_prefix("PCI_SUBSYS_ID=") {
                        subsys_id = Some(value);
                    }
                }

                if let (Some(driver), Some(pci_id), Some(subsys_id)) = (driver, pci_id, subsys_id) {
                    let vendor = match driver {
                        "amdgpu" => Some(Vendor::Amd),
                        "nvidia" => Some(Vendor::Nvidia),
                        "xe" => Some(Vendor::Intel),
                        "i915" => {
                            // Check the first 2 digits of the device ID:
                            // 56xx: Arc A-Series
                            // E2xx: Arc B-Series
                            if ["56", "E2"].contains(&&pci_id[5..7]) { Some(Vendor::Intel) }
                            else { None }
                        }
                        _ => None,
                    };
                    if let Some(vendor) = vendor {
                        let pci_addr_str = dir.file_name().to_str().unwrap().to_owned();
                        let pci_addr = parse_pci_addr(&pci_addr_str).unwrap();
                        let pci_id = parse_pci_id(pci_id).unwrap();
                        let subsys_id = parse_pci_id(subsys_id).unwrap();
                        let gpu_name = if let Some(gpu_names) = &gpu_names {
                            // Look for subsystem ID (common on AMD devices)
                            if let Some(name) = gpu_names.get(&(vendor, pci_id.1, Some((subsys_id.0, subsys_id.1)))) { Some(name) }
                            // Fallback to device ID
                            else if let Some(name) = gpu_names.get(&(vendor, pci_id.1, None)) { Some(name) }
                            // Fallback to generic name
                            else { None }
                        } else { None };
                        // Unwrap the matched device name or specify generic name
                        let gpu_name = match gpu_name {
                            Some(name) => format!("{} {}", vendor.name(), name.to_owned()),
                            None => format!("{} {}", vendor.name(), if pci_addr.1 > 0 { "GPU" } else { "iGPU" })
                        };
                        gpus.push(
                            PciDevice {
                                vendor,
                                bus: pci_addr.1,
                                address: pci_addr_str,
                                name: gpu_name
                            }
                        );
                    }
                }
            }
            Err(_) => (),
        }
    }

    gpus
}
