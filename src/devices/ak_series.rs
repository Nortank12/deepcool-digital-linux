use crate::monitor::cpu;
use hidapi::HidApi;
use std::{thread::sleep, time::Duration};

const VENDOR: u16 = 0x3633;
const POLLING_RATE: u64 = 750;

pub struct Display {
    product_id: u16,
    fahrenheit: bool,
    alarm: bool,
}

impl Display {
    pub fn new(product_id: u16, fahrenheit: bool, alarm: bool) -> Self {
        Display {
            product_id,
            fahrenheit,
            alarm,
        }
    }

    pub fn run(&self, api: &HidApi, mode: &str, cpu_temp_sensor: &str) {
        // Connect to device
        let device = api.open(VENDOR, self.product_id).expect("Failed to open HID device");

        // Data packet
        let mut data: [u8; 64] = [0; 64];
        data[0] = 16;

        // Init sequence
        {
            let mut init_data = data.clone();
            init_data[1] = 170;
            device.write(&init_data).expect("Failed to write data");
        }

        // Display loop
        if mode == "auto" {
            loop {
                for _ in 0..8 {
                    device
                        .write(&self.status_message(&data, "temp", &cpu_temp_sensor))
                        .expect("Failed to write data");
                }
                for _ in 0..8 {
                    device
                        .write(&self.status_message(&data, "usage", &cpu_temp_sensor))
                        .expect("Failed to write data");
                }
            }
        } else {
            loop {
                device
                    .write(&self.status_message(&data, &mode, &cpu_temp_sensor))
                    .expect("Failed to write data");
            }
        }
    }

    /// Reads the CPU status information and returns the data packet.
    fn status_message(&self, inital_data: &[u8; 64], mode: &str, cpu_temp_sensor: &str) -> [u8; 64] {
        // Clone the data packet
        let mut data = inital_data.clone();

        // Read CPU utilization
        let cpu_instant = cpu::read_instant();

        // Wait
        sleep(Duration::from_millis(POLLING_RATE));

        // Calculate usage & temperature
        let usage = cpu::get_usage(cpu_instant);
        let temp = cpu::get_temp(cpu_temp_sensor, self.fahrenheit);

        // Main display
        match mode {
            "temp" => {
                data[1] = if self.fahrenheit { 35 } else { 19 };
                data[3] = temp / 100;
                data[4] = temp % 100 / 10;
                data[5] = temp % 10;
            }
            "usage" => {
                data[1] = 76;
                data[3] = usage / 100;
                data[4] = usage % 100 / 10;
                data[5] = usage % 10;
            }
            _ => (),
        }
        // Status bar
        data[2] = if usage < 20 { 1 } else { (usage as f32 / 10 as f32).round() as u8 };
        // Alarm
        data[6] = (self.alarm && temp > if self.fahrenheit { 185 } else { 85 }) as u8;

        data
    }
}
