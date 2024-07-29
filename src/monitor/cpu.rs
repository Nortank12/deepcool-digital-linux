//! Reads live CPU data from the Linux kernel.

use cpu_monitor::CpuInstant;
use std::{fs::read_to_string, process::exit};

/// Looks for the appropriate CPU temperature sensor datastream in the hwmon folder.
pub fn find_temp_sensor() -> String {
    let mut i = 0;
    loop {
        match read_to_string(format!("/sys/class/hwmon/hwmon{i}/name")) {
            Ok(data) => {
                let hwname = data.trim_end();
                if ["coretemp", "k10temp", "zenpower"].contains(&hwname) {
                    return format!("/sys/class/hwmon/hwmon{i}/temp1_input");
                }
            }
            Err(_) => {
                println!("CPU temperature sensor not found!");
                exit(1);
            }
        }
        i += 1;
    }
}

/// Reads the value of the CPU temperature sensor and calculates it to be `˚C` or `˚F`.
pub fn get_temp(temp_sensor: &str, fahrenheit: bool) -> u8 {
    // Read sensor data
    let data = read_to_string(temp_sensor).expect("CPU temperature cannot be read!");

    // Calculate temperature
    let mut temp = data.trim_end().parse::<u32>().unwrap();
    if fahrenheit {
        temp = temp * 9 / 5 + 32000
    }

    (temp as f32 / 1000.0).round() as u8
}

/// Reads the energy consumption of the CPU in microjoules.
pub fn read_energy() -> u64 {
    let data = read_to_string("/sys/class/powercap/intel-rapl/intel-rapl:0/energy_uj")
        .expect("CPU energy consumption cannot be read!");

    data.trim_end().parse::<u64>().unwrap()
}

/// Reads the energy consumption one more time and calculates the CPU power by using the inital energy and the delta time.
///
/// Formula: `W = ΔμJ / (Δms * 1000)`
pub fn get_power(initial_energy: u64, delta_millisec: u64) -> u16 {
    let delta_energy = read_energy() - initial_energy;

    (delta_energy as f64 / (delta_millisec * 1000) as f64).round() as u16
}

/// Reads the CPU instant and provides usage statistics.
pub fn read_instant() -> CpuInstant {
    CpuInstant::now().expect("CPU time cannot be read!")
}

/// Reads the CPU instant one more time and calculates the utilization as a `0-100` number.
pub fn get_usage(initial_instant: CpuInstant) -> u8 {
    let usage = (read_instant() - initial_instant).non_idle() * 100.0;

    (usage).round() as u8
}
