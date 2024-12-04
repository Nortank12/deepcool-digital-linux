//! Reads live CPU data from the Linux kernel.

use crate::error;
use cpu_monitor::CpuInstant;
use std::{fs::read_dir, fs::read_to_string, process::exit};

pub struct Cpu {
    temp_sensor: String,
    pub rapl_max_uj: u64,
}

impl Cpu {
    pub fn new() -> Self {
        Cpu {
            temp_sensor: find_temp_sensor(),
            rapl_max_uj: get_max_energy(),
        }
    }

    /// Reads the value of the CPU temperature sensor and calculates it to be `˚C` or `˚F`.
    pub fn get_temp(&self, fahrenheit: bool) -> u8 {
        // Read sensor data
        let data = read_to_string(&self.temp_sensor).unwrap_or_else(|_| {
            error!("Failed to get CPU temperature");
            exit(1);
        });

        // Calculate temperature
        let mut temp = data.trim_end().parse::<u32>().unwrap();
        if fahrenheit {
            temp = temp * 9 / 5 + 32000
        }

        (temp as f32 / 1000.0).round() as u8
    }

    /// Reads the energy consumption of the CPU in microjoules.
    pub fn read_energy(&self) -> u64 {
        let data = read_to_string("/sys/class/powercap/intel-rapl/intel-rapl:0/energy_uj").unwrap_or_else(|_| {
            error!("Failed to get CPU power");
            exit(1);
        });

        data.trim_end().parse::<u64>().unwrap()
    }

    /// Reads the energy consumption one more time and calculates the CPU power by using the inital energy and the delta time.
    ///
    /// Formula: `W = ΔμJ / (Δms * 1000)`
    pub fn get_power(&self, initial_energy: u64, delta_millisec: u64) -> u16 {
        let current_energy = self.read_energy();
        let delta_energy = if current_energy > initial_energy {
            current_energy - initial_energy
        } else {
            // Offset the current measurement if the counter resets
            (self.rapl_max_uj + current_energy) - initial_energy
        };

        (delta_energy as f64 / (delta_millisec * 1000) as f64).round() as u16
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

/// Looks for the appropriate CPU temperature sensor datastream in the hwmon folder.
fn find_temp_sensor() -> String {
    match read_dir("/sys/class/hwmon") {
        Ok(sensors) => {
            for sensor in sensors {
                let path = sensor.unwrap().path().to_str().unwrap().to_owned();
                match read_to_string(format!("{path}/name")) {
                    Ok(name) => {
                        if ["coretemp", "k10temp", "zenpower"].contains(&name.trim_end()) {
                            return format!("{path}/temp1_input");
                        }
                    }
                    Err(_) => (),
                }
            }
        }
        Err(_) => (),
    }
    error!("Failed to locate CPU temperature sensor");
    exit(1);
}

/// Gets the limit of the displayed energy value so it can be applied as an offset when the counter resets.
///
/// Errors will be handled per device since not all of them require power consumption readings.
fn get_max_energy() -> u64 {
    match read_to_string("/sys/class/powercap/intel-rapl/intel-rapl:0/max_energy_range_uj") {
        Ok(data) => data.trim_end().parse::<u64>().unwrap(),
        Err(_) => 0,
    }
}
