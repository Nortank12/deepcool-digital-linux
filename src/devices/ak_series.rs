//! Display module for:
//! - AK400 DIGITAL & DIGITAL SE
//! - AK500 DIGITAL
//! - AK500S DIGITAL & DIGITAL SE
//! - AK620 DIGITAL & DIGITAL SE

use crate::{devices::AUTO_MODE_INTERVAL, monitor::cpu::Cpu};
use super::{device_error, Mode};
use hidapi::{HidApi, HidDevice};
use std::{thread::sleep, time::{Duration, Instant}};

pub const DEFAULT_MODE: Mode = Mode::CpuTemperature;
pub const TEMP_LIMIT_C: u8 = 90;
pub const TEMP_LIMIT_F: u8 = 194;

pub struct Display {
    cpu: Cpu,
    pub mode: Mode,
    update: Duration,
    fahrenheit: bool,
    alarm: bool,
    se_mode: bool,
}

impl Display {
    pub fn new(cpu: Cpu, mode: &Mode, update: Duration, fahrenheit: bool, alarm: bool) -> Self {
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
            fahrenheit,
            alarm,
            se_mode: false,
        }
    }

    pub fn run(&mut self, api: &HidApi, vid: u16, pid: u16) {
        // Connect to device
        let device = api.open(vid, pid).unwrap_or_else(|_| device_error());

        // Identify device type based on product name. E.g.:
        // AK400 -> AK400 DIGITAL
        // A400  -> AK400 DIGITAL SE
        let prod_name = device.get_product_string().unwrap().unwrap();
        if !prod_name.starts_with("AK") {
            self.se_mode = true;
        }

        // Display warning if a required module is missing
        self.cpu.warn_temp();

        // Data packet
        let mut data: [u8; 64] = [0; 64];
        data[0] = 16;

        // Init sequence
        {
            let mut init_data = data.clone();
            init_data[1] = 170;
            self.write(&device, init_data);
        }

        // Display loop
        match self.mode {
            Mode::Auto => {
                let mut initial_update = self.update;
                let mut mode = Mode::CpuTemperature;
                loop {
                    // Initial update
                    self.write(&device, self.status_message(&data, &mode, initial_update));

                    // Update until timeout
                    let timeout = Instant::now() + AUTO_MODE_INTERVAL;
                    while Instant::now() + self.update < timeout {
                        self.write(&device, self.status_message(&data, &mode, self.update));
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
                self.write(&device, self.status_message(&data, &self.mode, self.update));
            }
        }
    }

    /// Reads the CPU status information and returns the data packet.
    fn status_message(&self, inital_data: &[u8; 64], mode: &Mode, update: Duration) -> [u8; 64] {
        // Clone the data packet
        let mut data = inital_data.clone();

        // Read CPU utilization
        let cpu_instant = self.cpu.read_instant();

        // Wait
        sleep(update);

        // Calculate usage & temperature
        let usage = self.cpu.get_usage(cpu_instant);
        let temp = self.cpu.get_temp(self.fahrenheit);

        // Main display
        match mode {
            Mode::CpuTemperature => {
                data[1] = if self.fahrenheit { 35 } else { 19 };
                data[3] = temp / 100;
                data[4] = temp % 100 / 10;
                data[5] = temp % 10;
            }
            Mode::CpuUsage => {
                data[1] = 76;
                data[3] = usage / 100;
                data[4] = usage % 100 / 10;
                data[5] = usage % 10;
            }
            _ => (),
        }
        // Status bar
        data[2] = if usage < 15 { 1 } else { (usage as f32 / 10.0).round() as u8 };
        // Alarm
        data[6] = (self.alarm && temp >= if self.fahrenheit { TEMP_LIMIT_F } else { TEMP_LIMIT_C }) as u8;

        data
    }

    fn write(&self, device: &HidDevice, mut data: [u8; 64]) {
        // On "DIGITAL SE" devices, the report ID is not sent, making a -1 offset necessary for all bytes.
        if self.se_mode {
            data.copy_within(1.., 0);
        }
        device.write(&data).unwrap();
    }
}
