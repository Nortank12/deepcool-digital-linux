use crate::{error, monitor::cpu::Cpu};
use hidapi::HidApi;
use std::{process::exit, thread::sleep, time::Duration};
use crate::custom_mode::CustomModeItem;

const VENDOR: u16 = 0x3633;
const POLLING_RATE: u64 = 750;

pub struct Display {
    product_id: u16,
    fahrenheit: bool,
    alarm: bool,
    cpu: Cpu,
}

impl Display {
    pub fn new(product_id: u16, fahrenheit: bool, alarm: bool) -> Self {
        Display {
            product_id,
            fahrenheit,
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
        } else if mode.starts_with("custom:") {
            let modes = CustomModeItem::parse_modes(mode);
            loop {
                for mode in modes.iter() {
                    for _ in 0..(mode.tick_length) {
                        device.write(&self.status_message(&data, mode.mode.as_str())).unwrap();
                    }
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

        // Read CPU utilization
        let cpu_instant = self.cpu.read_instant();

        // Wait
        sleep(Duration::from_millis(POLLING_RATE));

        // Calculate usage & temperature
        let usage = self.cpu.get_usage(cpu_instant);
        let temp = self.cpu.get_temp(self.fahrenheit);

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
            other => {
                if other.starts_with("number.") {
                    let value = other.split(".").skip(1).next().unwrap().parse::<u16>().unwrap().min(999);
                    data[1] = 19;
                    data[3] = (value / 100) as u8;
                    data[4] = (value % 100 / 10) as u8;
                    data[5] = (value % 10) as u8;
                }
            }
        }
        // Status bar
        data[2] = if usage < 15 { 1 } else { (usage as f32 / 10.0).round() as u8 };
        // Alarm
        data[6] = (self.alarm && temp > if self.fahrenheit { 185 } else { 85 }) as u8;

        data
    }
}
