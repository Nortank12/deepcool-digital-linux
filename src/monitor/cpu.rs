//! Reads live CPU data from the Linux kernel.

use crate::error;
use cpu_monitor::CpuInstant;
use std::{borrow::Borrow, fs::{read_dir, read_to_string, File}, process::exit};

#[derive(PartialEq)]
pub enum CpuType {
    Intel,
    Amd,
    Other,
}

impl CpuType {
    pub fn to_string(&self) -> String {
        match self {
            CpuType::Intel => "Intel".to_string(),
            CpuType::Amd => "AMD".to_string(),
            CpuType::Other => "Other".to_string(),
        }
    }
}

pub struct Cpu {
    temp_sensor: String,
}

impl Cpu {
    pub fn new() -> Self {
        Cpu {
            temp_sensor: find_temp_sensor(),
        }
    }

    pub fn get_type(&self) -> CpuType {
        let data = read_to_string("/proc/cpuinfo").unwrap_or_else(|_| {
            error!("Failed to get CPU type");
            exit(1);
        });
        let lines = data.lines().collect::<Vec<&str>>();
        let vendor_id_line = lines.iter().find(|line| line.starts_with("vendor_id")).unwrap_or_else(|| {
            error!("Failed to get vendor_id");
            exit(1);
        });
        if vendor_id_line.contains("Intel") {
            CpuType::Intel
        } else if vendor_id_line.contains("AMD") {
            CpuType::Amd
        } else {
            CpuType::Other
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

    pub fn get_power_with_command(&self) -> u64 {
        let output = std::process::Command::new("perf")
            .arg("stat")
            .arg("-e")
            .arg("power/energy-pkg/")
            .arg("sleep")
            .arg("0.750")
            .output();
        match output {
            Ok(output) => {
                if output.status.success() {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    let result = stdout.to_string() + &stderr.to_string();
                    let joules_line = result.lines().find(|line| line.contains("Joules power/energy-pkg/")).unwrap_or_else(|| {
                        "Could not find Joules power/energy-pkg"
                    }).to_string();
                    let joules = joules_line.trim_start().split_whitespace().nth(0).unwrap().parse::<f32>().unwrap();
                    //println!("Joules: {}", joules);
                    return joules as u64;
                    // let microwatt_hours = ((joules * 1_000_000.0) / 3600.0) as u64;
                    // return microwatt_hours;
                } else {
                    panic!("Failed to execute perf command")
                }
            }
            Err(_) => {
                error!("Failed to execute perf command");
                exit(1);
            }
        }
    }

    /// Reads the energy consumption one more time and calculates the CPU power by using the inital energy and the delta time.
    ///
    /// Formula: `W = ΔμJ / (Δms * 1000)`
    pub fn get_power(&self, initial_energy: u64, delta_millisec: u64) -> u16 {
        let delta_energy = self.read_energy() - initial_energy;

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
