use crate::monitor::cpu::Cpu;
use super::{device_error, Mode};
use hidapi::HidApi;
use std::{thread::sleep, time::Duration};

pub const DEFAULT_MODE: Mode = Mode::Temperature;
pub const POLLING_RATE: u64 = 750;
pub const TEMP_LIMIT_C: u8 = 90;
pub const TEMP_LIMIT_F: u8 = 194;

pub struct Display {
    mode: Mode,
    fahrenheit: bool,
    alarm: bool,
    cpu: Cpu,
}

impl Display {
    pub fn new(mode: &Mode, fahrenheit: bool, alarm: bool) -> Self {
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
            fahrenheit,
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

        // Init sequence
        {
            let mut init_data = data.clone();
            init_data[1] = 170;
            device.write(&init_data).unwrap();
        }

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

        // Read CPU utilization
        let cpu_instant = self.cpu.read_instant();

        // Wait
        sleep(Duration::from_millis(POLLING_RATE));

        // Calculate usage & temperature
        let usage = self.cpu.get_usage(cpu_instant);
        let temp = self.cpu.get_temp(self.fahrenheit);

        // Main display
        match mode {
            Mode::Temperature => {
                data[1] = if self.fahrenheit { 35 } else { 19 };
                data[3] = temp / 100;
                data[4] = temp % 100 / 10;
                data[5] = temp % 10;
            }
            Mode::Usage => {
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
}
