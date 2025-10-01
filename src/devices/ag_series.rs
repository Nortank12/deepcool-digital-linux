//! Display module for:
//! - AG300 DIGITAL
//! - AG400 DIGITAL
//! - AG500 DIGITAL
//! - AG620 DIGITAL

use crate::monitor::cpu::Cpu;
use super::{device_error, Mode, AUTO_MODE_INTERVAL};
use hidapi::HidApi;
use std::{thread::sleep, time::{Duration, Instant}};

pub const DEFAULT_MODE: Mode = Mode::CpuTemperature;
pub const TEMP_LIMIT_C: u8 = 90;

pub struct Display {
    cpu: Cpu,
    pub mode: Mode,
    update: Duration,
    alarm: bool,
}

impl Display {
    pub fn new(cpu: Cpu, mode: &Mode, update: Duration, alarm: bool) -> Self {
        // Verify the display mode
        let mode = match mode {
            Mode::Default => DEFAULT_MODE,
            Mode::Auto => Mode::Auto,
            Mode::CpuTemperature => Mode::CpuTemperature,
            Mode::CpuUsage => Mode::CpuUsage,
            _ => mode.support_error(),
        };

        Display {
            cpu,
            mode,
            update,
            alarm,
        }
    }

    pub fn run(&self, api: &HidApi, vid: u16, pid: u16) {
        // Connect to device
        let device = api.open(vid, pid).unwrap_or_else(|_| device_error());

        // Display warning if a required module is missing
        self.cpu.warn_temp();

        // Data packet
        let mut data: [u8; 64] = [0; 64];
        data[0] = 16;

        // Display loop
        match self.mode {
            Mode::Auto => {
                let mut initial_update = self.update;
                let mut mode = Mode::CpuTemperature;
                loop {
                    // Initial update
                    device.write(&self.status_message(&data, &mode, initial_update)).unwrap();

                    // Update until timeout
                    let timeout = Instant::now() + AUTO_MODE_INTERVAL;
                    while Instant::now() + self.update < timeout {
                        device.write(&self.status_message(&data, &mode, self.update)).unwrap();
                    }

                    // Make the next initial update faster to fit the timeframe
                    initial_update = timeout - Instant::now();

                    // Switch to the next display mode
                    mode = match mode {
                        Mode::CpuTemperature => Mode::CpuUsage,
                        Mode::CpuUsage => Mode::CpuTemperature,
                        _ => DEFAULT_MODE,
                    }
                }
            }
            _ => loop {
                device.write(&self.status_message(&data, &self.mode, self.update)).unwrap();
            }
        }
    }

    /// Reads the CPU status information and returns the data packet.
    fn status_message(&self, inital_data: &[u8; 64], mode: &Mode, update: Duration) -> [u8; 64] {
        // Clone the data packet
        let mut data = inital_data.clone();

        if mode == &Mode::CpuUsage {
            // Read CPU utilization
            let cpu_instant = self.cpu.read_instant();

            // Wait
            sleep(update);

            // Calculate & write usage
            let usage = self.cpu.get_usage(cpu_instant);
            data[1] = 76;
            data[3] = if usage < 100 { usage % 100 / 10 } else { 9 };
            data[4] = if usage < 100 { usage % 10 } else { 9 };
        } else {
            // If display mode is not usage, simply wait
            sleep(update);
        }

        // Calculate temperature
        let temp = self.cpu.get_temp(false);

        if mode == &Mode::CpuTemperature {
            // Write temperature
            data[1] = 19;
            data[3] = if temp < 100 { temp % 100 / 10 } else { 9 };
            data[4] = if temp < 100 { temp % 10 } else { 9 };
        }

        // Alarm
        data[5] = (self.alarm && temp >= TEMP_LIMIT_C) as u8;

        data
    }
}
