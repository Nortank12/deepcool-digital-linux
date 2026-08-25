//! Reads live CPU data from the Linux kernel.

use crate::{error, warning};
use cpu_monitor::CpuInstant;
use std::{
    fs::{read_dir, read_to_string, File},
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
    process::exit,
};

struct EnergySensor {
    path: PathBuf,
    max_uj: Option<u64>,
}

pub struct Cpu {
    temp_sensor: Option<String>,
    energy_sensor: Option<EnergySensor>,
}

impl Cpu {
    pub fn new() -> Self {
        Cpu {
            temp_sensor: find_temp_sensor(),
            energy_sensor: find_energy_sensor(),
        }
    }

    /// Displays a warning message if temperature sensor is not initialized.
    pub fn warn_temp(&self) {
        if self.temp_sensor == None {
            warning!("No supported CPU temperature sensor was found");
            eprintln!("         CPU temperature will not be displayed, and alarm will be disabled.");
            eprintln!("         Supported kernel modules are: asusec, coretemp, k10temp, and zenpower.");
        }
    }

    /// Displays a warning message if no supported CPU energy counter is available.
    pub fn warn_rapl(&self) {
        if self.energy_sensor.is_none() {
            warning!("No supported CPU energy sensor was found");
            eprintln!("         CPU power consumption will not be displayed.");
        }
    }

    /// Reads the value of the CPU temperature sensor and calculates it to be `˚C` or `˚F`.
    pub fn get_temp(&self, fahrenheit: bool) -> u8 {
        if let Some(sensor) = &self.temp_sensor {
            // Read sensor data
            let data = read_to_string(sensor).unwrap_or_else(|_| {
                error!("Failed to get CPU temperature");
                exit(1);
            });
            // Calculate temperature
            let mut temp = data.trim_end().parse::<u32>().unwrap();
            if fahrenheit {
                temp = temp * 9 / 5 + 32000
            }
            return (temp as f32 / 1000.0).round() as u8;
        }

        0
    }

    /// Reads the energy consumption of the CPU in microjoules.
    pub fn read_energy(&self) -> u64 {
        if let Some(sensor) = &self.energy_sensor {
            let data = read_to_string(&sensor.path).unwrap_or_else(|_| {
                error!("Failed to get CPU power");
                exit(1);
            });
            return data.trim_end().parse::<u64>().unwrap();
        }

        0
    }

    /// Reads the energy consumption one more time and calculates the CPU power by using the inital energy and the delta time.
    ///
    /// Formula: `W = ΔμJ / (Δms * 1000)`
    pub fn get_power(&self, initial_energy: u64, delta_millisec: u64) -> u16 {
        if let Some(sensor) = &self.energy_sensor {
            let current_energy = self.read_energy();
            let delta_energy = if current_energy > initial_energy {
                current_energy - initial_energy
            } else if let Some(max_uj) = sensor.max_uj {
                // Offset the current measurement if a bounded counter wraps.
                (max_uj + current_energy) - initial_energy
            } else {
                // Unbounded hwmon counters only decrease when the sensor resets.
                return 0;
            };
            return (delta_energy as f64 / (delta_millisec * 1000) as f64).round() as u16;
        }

        0
    }

    /// Reads the CPU instant and provides usage statistics.
    pub fn read_instant(&self) -> CpuInstant {
        CpuInstant::now().unwrap_or_else(|_| {
            error!("Failed to get CPU usage");
            exit(1);
        })
    }

    /// Reads the CPU instant one more time and calculates the utilization as a `0-100` number.
    pub fn get_usage(&self, initial_instant: CpuInstant) -> u8 {
        let usage = (self.read_instant() - initial_instant).non_idle() * 100.0;

        (usage).round() as u8
    }

    /// Reads the frequency of all CPU cores and returns the highest one in MHz.
    pub fn get_frequency(&self) -> u16 {
        let cpuinfo = read_to_string("/proc/cpuinfo").unwrap_or_else(|_| {
            error!("Failed to get CPU clock");
            exit(1);
        });

        let mut highest_core = 0.0;
        for info in cpuinfo.lines() {
            if info.starts_with("cpu MHz") {
                let clock = info.split(":").nth(1).unwrap();
                let clock = clock.trim().parse::<f32>().unwrap();
                if clock > highest_core {
                    highest_core = clock;
                }
            }
        }

        highest_core.round() as u16
    }
}

/// Looks for the appropriate CPU temperature sensor datastream in the hwmon directory.
fn find_temp_sensor() -> Option<String> {
    for sensor in read_dir("/sys/class/hwmon").ok()? {
        let path = sensor.ok()?.path().to_str()?.to_owned();
        if let Ok(name) = read_to_string(format!("{path}/name")) {
            if ["asusec", "coretemp", "k10temp", "zenpower"].contains(&name.trim_end()) {
                return Some(format!("{path}/temp1_input"));
            }
        }
    }

    None
}

/// Finds an Intel RAPL or AMD zenergy/amd_energy package energy counter.
fn find_energy_sensor() -> Option<EnergySensor> {
    let intel_rapl = Path::new("/sys/class/powercap/intel-rapl/intel-rapl:0");
    let intel_path = intel_rapl.join("energy_uj");
    if intel_path.is_file() {
        let max_uj = read_to_string(intel_rapl.join("max_energy_range_uj"))
            .ok()
            .and_then(|data| data.trim_end().parse::<u64>().ok());
        return Some(EnergySensor {
            path: intel_path,
            max_uj,
        });
    }

    for entry in read_dir("/sys/class/hwmon").ok()?.flatten() {
        let path = entry.path();
        let Ok(name) = read_to_string(path.join("name")) else {
            continue;
        };
        if !["zenergy", "amd_energy"].contains(&name.trim_end()) {
            continue;
        }

        if let Some(path) = find_socket_energy(&path) {
            return Some(EnergySensor { path, max_uj: None });
        }
    }

    None
}

/// Finds the package/socket energy channel exposed by an AMD hwmon device.
fn find_socket_energy(hwmon: &Path) -> Option<PathBuf> {
    for entry in read_dir(hwmon).ok()?.flatten() {
        let label_path = entry.path();
        let Some(file_name) = label_path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        let Some(channel) = file_name
            .strip_prefix("energy")
            .and_then(|name| name.strip_suffix("_label"))
        else {
            continue;
        };
        let Ok(label) = read_to_string(&label_path) else {
            continue;
        };
        if label.trim_end().starts_with("Esocket") {
            let input_path = hwmon.join(format!("energy{channel}_input"));
            if input_path.is_file() {
                return Some(input_path);
            }
        }
    }

    None
}

/// Gets the CPU model name.
pub fn get_name() -> Option<String> {
    let file = File::open("/proc/cpuinfo").ok()?;
    let reader = BufReader::new(file);
    for line in reader.lines() {
        let line = line.ok()?;
        if line.starts_with("model name") {
            if let Some(colon_pos) = line.find(':') {
                return Some(line[colon_pos + 1..].trim().to_string());
            }
        }
    }

    None
}
