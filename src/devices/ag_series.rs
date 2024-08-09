use crate::monitor::cpu;
use hidapi::HidApi;
use std::{thread::sleep, time::Duration};

const VENDOR: u16 = 0x3633;
const POLLING_RATE: u64 = 750;

pub struct Display {
    product_id: u16,
    alarm: bool,
}

impl Display {
    pub fn new(product_id: u16, alarm: bool) -> Self {
        Display { product_id, alarm }
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

        if mode == "usage" {
            // Read CPU utilization
            let cpu_instant = cpu::read_instant();

            // Wait
            sleep(Duration::from_millis(POLLING_RATE));

            // Calculate & write usage
            let usage = cpu::get_usage(cpu_instant);
            data[1] = 76;
            data[3] = if usage < 100 { usage % 100 / 10 } else { 9 };
            data[4] = if usage < 100 { usage % 10 } else { 9 };
        } else {
            // If display mode is not usage, simply wait
            sleep(Duration::from_millis(POLLING_RATE));
        }

        // Calculate temperature
        let temp = cpu::get_temp(cpu_temp_sensor, false);

        if mode == "temp" {
            // Write temperature
            data[1] = 19;
            data[3] = if temp < 100 { temp % 100 / 10 } else { 9 };
            data[4] = if temp < 100 { temp % 10 } else { 9 };
        }

        // Alarm
        data[5] = (self.alarm && temp > 85) as u8;

        data
    }
}
