use crate::{error, monitor::cpu::Cpu};
use hidapi::HidApi;
use std::{process::exit, thread::sleep, time::Duration};

const VENDOR: u16 = 0x3633;
const POLLING_RATE: u64 = 750;

pub struct Display {
    product_id: u16,
    fahrenheit: bool,
    cpu: Cpu,
}

impl Display {
    pub fn new(product_id: u16, fahrenheit: bool) -> Self {
        Display {
            product_id,
            fahrenheit,
            cpu: Cpu::new(),
        }
    }

    pub fn run(&self, api: &HidApi) {
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
        loop {
            // Clone the data packet
            let mut status_data = data.clone();

            // Read CPU utilization & energy consumption
            let cpu_instant = self.cpu.read_instant();
            let cpu_energy = self.cpu.read_energy();

            // Wait
            sleep(Duration::from_millis(POLLING_RATE));

            // ----- Write data to the package -----
            // Utilization
            let usage = self.cpu.get_usage(cpu_instant);
            status_data[2] = usage / 100;
            status_data[3] = usage % 100 / 10;
            status_data[4] = usage % 10;

            // Temperature
            let temp = self.cpu.get_temp(self.fahrenheit);
            status_data[1] = if self.fahrenheit { 35 } else { 19 };
            status_data[5] = temp / 100;
            status_data[6] = temp % 100 / 10;
            status_data[7] = temp % 10;

            // Power consumption
            let power = self.cpu.get_power(cpu_energy, POLLING_RATE);
            status_data[8] = (power / 100) as u8;
            status_data[9] = (power % 100 / 10) as u8;
            status_data[10] = (power % 10) as u8;

            // Alarm
            // Actual values [176, 194 °F] | [80, 90 °C]
            status_data[11] = if self.fahrenheit {
                if temp > 122 { 2 }
                else if temp > 104 { 1 }
                else { 0 }
            } else {
                if temp > 50 { 2 }
                else if temp > 40 { 1 }
                else { 0 }
            };

            println!("USG: {usage}% | TMP: {temp}°C/F | POW: {power}W");
            device.write(&status_data).unwrap();
        }
    }
}
