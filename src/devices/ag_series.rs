use crate::monitor::cpu::Cpu;
use super::{device_error, Mode};
use hidapi::HidApi;
use std::{thread::sleep, time::Duration};

pub const DEFAULT_MODE: Mode = Mode::Temperature;
pub const POLLING_RATE: u64 = 750;
pub const TEMP_LIMIT_C: u8 = 90;

pub struct Display {
    mode: Mode,
    alarm: bool,
    cpu: Cpu,
}

impl Display {
    pub fn new(mode: &Mode, alarm: bool) -> Self {
        // Verify the display mode
        let mode = match mode {
            Mode::Default => DEFAULT_MODE,
            Mode::Auto => Mode::Auto,
            Mode::Temperature => Mode::Temperature,
            Mode::Usage => Mode::Usage,
            _ => mode.support_error(),
        };

        Display {
            mode,
            alarm,
            cpu: Cpu::new(),
        }
    }

    pub fn run(&self, api: &HidApi, vid: u16, pid: u16) {
        // Connect to device
        let device = api.open(vid, pid).unwrap_or_else(|_| device_error());

        // Data packet
        let mut data: [u8; 64] = [0; 64];
        data[0] = 16;

        // Display loop
        match self.mode {
            Mode::Auto => loop {
                for _ in 0..8 {
                    device.write(&self.status_message(&data, &Mode::Temperature)).unwrap();
                }
                for _ in 0..8 {
                    device.write(&self.status_message(&data, &Mode::Usage)).unwrap();
                }
            }
            _ => loop {
                device.write(&self.status_message(&data, &self.mode)).unwrap();
            }
        }
    }

    /// Reads the CPU status information and returns the data packet.
    fn status_message(&self, inital_data: &[u8; 64], mode: &Mode) -> [u8; 64] {
        // Clone the data packet
        let mut data = inital_data.clone();

        if mode == &Mode::Usage {
            // Read CPU utilization
            let cpu_instant = self.cpu.read_instant();

            // Wait
            sleep(Duration::from_millis(POLLING_RATE));

            // Calculate & write usage
            let usage = self.cpu.get_usage(cpu_instant);
            data[1] = 76;
            data[3] = if usage < 100 { usage % 100 / 10 } else { 9 };
            data[4] = if usage < 100 { usage % 10 } else { 9 };
        } else {
            // If display mode is not usage, simply wait
            sleep(Duration::from_millis(POLLING_RATE));
        }

        // Calculate temperature
        let temp = self.cpu.get_temp(false);

        if mode == &Mode::Temperature {
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
