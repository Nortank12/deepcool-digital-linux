//! Parses the `pci.ids` database and maps the name of all AMD, Intel, and NVIDIA GPUs to their product IDs.

use super::Vendor;
use std::{collections::HashMap, fs::File, io::{BufRead, BufReader}, path::Path};

const PCI_IDS_PATHS: [&str; 3] = [
    "/usr/share/misc/pci.ids",
    "/usr/share/hwdata/pci.ids",
    "/var/lib/pciutils/pci.ids",
];

/// Returns a HashMap of Vendor, Device ID, and Subsystem ID.
///
/// Format: `(vendor_name, device_id, Option<(subsystem_vendor_id, subsystem_device_id)>)`
pub fn get_device_names() -> Option<HashMap<(Vendor, u16, Option<(u16, u16)>), String>> {
    let mut devices: HashMap<(Vendor, u16, Option<(u16, u16)>), String> = HashMap::new();

    let file = PCI_IDS_PATHS.iter().find_map(|path| {
        if Path::new(path).exists() { File::open(path).ok() }
        else { None }
    });

    if let Some(file) = file {
        let reader = BufReader::new(file);
        let mut current_vendor = None;
        let mut current_device = None;

        for line in reader.lines() {
            let line = line.unwrap();
            // Skip comments and empty lines
            if line.starts_with('#') || line.trim().is_empty() { continue; }
            if line.starts_with("\t\t") {
                // Parse Subsystem ID
                if let (Some(vendor), Some(device)) = (current_vendor, current_device) {
                    // Remove generic name
                    devices.remove(&(vendor, device, None));

                    let mut parts = line.split_whitespace();
                    let subsys_vendor = u16::from_str_radix(parts.next().unwrap(), 16).unwrap();
                    let subsys_device = u16::from_str_radix(parts.next().unwrap(), 16).unwrap();
                    let dev_name = {
                        // Use the name in square brackets when available
                        let start = line.find('[').unwrap_or(0) + 1;
                        let end = line.find(']').unwrap_or(0);
                        if end > start {
                            line[start..end].to_owned()
                        } else {
                            parts.collect::<Vec<_>>().join(" ")
                        }
                    };

                    devices.insert((vendor, device, Some((subsys_vendor, subsys_device))), dev_name);
                }
            } else if line.starts_with('\t') {
                // Parse Device ID
                if let Some(vendor) = current_vendor {
                    let mut parts = line.split_whitespace();
                    let dev_id = u16::from_str_radix(parts.next().unwrap(), 16).unwrap();
                    current_device = Some(dev_id);
                    let dev_name = {
                        // Use the name in square brackets when available
                        let start = line.find('[').unwrap_or(0) + 1;
                        let end = line.find(']').unwrap_or(0);
                        if end > start {
                            line[start..end].to_owned()
                        } else {
                            parts.collect::<Vec<_>>().join(" ")
                        }
                    };

                    devices.insert((vendor, dev_id, None), dev_name);
                }
            } else {
                // Parse Vendor ID
                let mut parts = line.split_whitespace();
                let vendor_id = u16::from_str_radix(parts.next().unwrap(), 16).unwrap();
                current_vendor = match vendor_id {
                    0x1002 | 0x1022 => Some(Vendor::Amd),
                    0x8086 => Some(Vendor::Intel),
                    0x10de => Some(Vendor::Nvidia),
                    _ => None,
                }
            }
        }
        if !devices.is_empty() {
            return Some(devices);
        }
    }

    None
}
