use crate::{error, monitor::cpu::Cpu};
use hidapi::HidApi;
use std::{process::exit, thread::sleep, time::Duration};

const VENDOR: u16 = 0x3633;
const POLLING_RATE: u64 = 750;

pub struct Display {
    product_id: u16,
    alarm: bool,
    cpu: Cpu,
}

impl Display {
    pub fn new(product_id: u16, alarm: bool) -> Self {
        Display {
            product_id,
            alarm,
            cpu: Cpu::new(),
        }
    }

    pub fn run(&self, api: &HidApi, mode: &str) {
        // Connect to device
        let device = api.open(VENDOR, self.product_id).unwrap_or_else(|_| {
            error!("Failed to access the USB device");
            eprintln!("       Try to run the program as root or give permission to the neccesary resources.");
            eprintln!("       You can find instructions about rootless mode on GitHub.");
            exit(1);
        });

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
        if mode == "auto" {
            loop {
                for _ in 0..8 {
                    device.write(&self.status_message(&data, "temp")).unwrap();
                }
                for _ in 0..8 {
                    device.write(&self.status_message(&data, "usage")).unwrap();
                }
            }
        } else {
            loop {
                device.write(&self.status_message(&data, &mode)).unwrap();
            }
        }
    }

    /// Reads the CPU status information and returns the data packet.
    fn status_message(&self, inital_data: &[u8; 64], mode: &str) -> [u8; 64] {
        // Clone the data packet
        let mut data = inital_data.clone();

        if mode == "usage" {
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
