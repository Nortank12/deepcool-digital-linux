//! Reads live GPU data from the Linux kernel.

//! Reads live GPU data from the Linux kernel (Intel i915 + xe).

use crate::error;
use std::{
    fs::{read_dir, read_to_string},
    process::exit,
};

#[derive(Debug, Clone, Copy)]
enum IntelDriver {
    I915,
    Xe,
}

pub struct Gpu {
    drm_dir: String,
    hwmon_dir: String,
    driver: IntelDriver,
}

impl Gpu {
    pub fn new(pci_address: &str) -> Self {
        let path = format!("/sys/bus/pci/devices/{pci_address}");

        let (drm_dir, driver) = find_drm_dir(&path).unwrap_or_else(|| {
            error!(format!(
                "Failed to access Intel GPU at PCI_ADDR={pci_address}"
            ));
            exit(1);
        });

        let hwmon_dir = find_hwmon_dir(&path, driver).unwrap_or_else(|| {
            error!("Failed to locate Intel GPU hwmon");
            exit(1);
        });

        Self {
            drm_dir,
            hwmon_dir,
            driver,
        }
    }

    /// Reads the GPU temperature in °C or °F.
    pub fn get_temp(&self, fahrenheit: bool) -> u16 {
        let data = read_to_string(format!("{}/temp1_input", self.hwmon_dir)).unwrap_or_else(|_| {
            error!("Failed to get GPU temperature");
            exit(1);
        });

        let millideg = data.trim().parse::<u32>().unwrap_or(0);
        let c = millideg as f32 / 1000.0;

        let value = if fahrenheit {
            c * 9.0 / 5.0 + 32.0
        } else {
            c
        };

        value.round() as u16
    }

    /// Estimates GPU usage based on frequency scaling.
    /// NOTE: This is *not* true utilization.
    pub fn get_usage(&self) -> u8 {
        let (cur, max) = match self.driver {
            IntelDriver::I915 => {
                let cur = read_to_string(format!(
                    "{}/device/gt_cur_freq_mhz",
                    self.drm_dir
                ))
                .ok()
                .and_then(|v| v.trim().parse::<u32>().ok())
                .unwrap_or(0);

                let max = read_to_string(format!(
                    "{}/device/gt_max_freq_mhz",
                    self.drm_dir
                ))
                .ok()
                .and_then(|v| v.trim().parse::<u32>().ok())
                .unwrap_or(0);

                (cur as f32, max as f32)
            }

            IntelDriver::Xe => {
                let cur = read_to_string(format!("{}/device/freq0_cur", self.drm_dir))
                    .ok()
                    .and_then(|v| v.trim().parse::<u64>().ok())
                    .unwrap_or(0);

                let max = read_to_string(format!("{}/device/freq0_max", self.drm_dir))
                    .ok()
                    .and_then(|v| v.trim().parse::<u64>().ok())
                    .unwrap_or(0);

                // Hz → MHz
                (
                    (cur / 1_000_000) as f32,
                    (max / 1_000_000) as f32,
                )
            }
        };

        if max > 0.0 {
            ((cur / max) * 100.0).round().min(100.0) as u8
        } else {
            0
        }
    }

    /// Reads GPU power consumption in Watts.
    pub fn get_power(&self) -> u16 {
        let avg = format!("{}/power1_average", self.hwmon_dir);
        let input = format!("{}/power1_input", self.hwmon_dir);

        let data = read_to_string(&avg)
            .or_else(|_| read_to_string(&input))
            .unwrap_or_else(|_| {
                error!("Failed to get GPU power");
                exit(1);
            });

        let microwatts = data.trim().parse::<u64>().unwrap_or(0);
        (microwatts / 1_000_000) as u16
    }

    /// Reads GPU core frequency in MHz.
    pub fn get_frequency(&self) -> u16 {
        match self.driver {
            IntelDriver::I915 => {
                let data =
                    read_to_string(format!("{}/device/gt_cur_freq_mhz", self.drm_dir))
                        .unwrap_or_else(|_| {
                            error!("Failed to get GPU frequency");
                            exit(1);
                        });

                data.trim().parse::<u16>().unwrap_or(0)
            }

            IntelDriver::Xe => {
                let data = read_to_string(format!("{}/device/freq0_cur", self.drm_dir))
                    .unwrap_or_else(|_| {
                        error!("Failed to get GPU frequency");
                        exit(1);
                    });

                let hz = data.trim().parse::<u64>().unwrap_or(0);
                (hz / 1_000_000) as u16
            }
        }
    }
}

/// Finds DRM directory and detects Intel driver (i915 or xe).
fn find_drm_dir(path: &str) -> Option<(String, IntelDriver)> {
    let uevent = read_to_string(format!("{path}/uevent")).ok()?;

    let driver = if uevent.lines().any(|l| l == "DRIVER=i915") {
        IntelDriver::I915
    } else if uevent.lines().any(|l| l == "DRIVER=xe") {
        IntelDriver::Xe
    } else {
        return None;
    };

    for dir in read_dir(format!("{path}/drm")).ok()? {
        let name = dir.ok()?.file_name().into_string().ok()?;
        if name.starts_with("card") {
            return Some((format!("{path}/drm/{name}"), driver));
        }
    }

    None
}

/// Finds hwmon directory for Intel GPU.
fn find_hwmon_dir(path: &str, driver: IntelDriver) -> Option<String> {
    let expected = match driver {
        IntelDriver::I915 => "i915",
        IntelDriver::Xe => "xe",
    };

    for entry in read_dir(format!("{path}/hwmon")).ok()? {
        let p = entry.ok()?.path();
        let name = read_to_string(p.join("name")).ok()?;
        if name.trim() == expected {
            return Some(p.to_string_lossy().into_owned());
        }
    }

    None
}
