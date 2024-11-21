use crate::{error, monitor::cpu::Cpu};
use hidapi::HidApi;
use std::{process::exit, thread::sleep, time::Duration};

const VENDOR: u16 = 0x3633;
const POLLING_RATE: u64 = 1000;

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

        // Check if `rapl_max_uj` was read correctly
        if self.cpu.rapl_max_uj == 0 {
            error!("Failed to get CPU power details");
            exit(1);
        }

        // Data packet
        let mut data: [u8; 64] = [0; 64];
        data[0] = 16;
        data[1] = 104;
        data[2] = 1;
        data[3] = 1;

        // Init sequence
        {
            let mut init_data = data.clone();
            init_data[4] = 2;
            init_data[5] = 3;
            init_data[6] = 1;
            init_data[7] = 112;
            init_data[8] = 22;
            device.write(&init_data).unwrap();
            init_data[5] = 2;
            init_data[7] = 111;
            device.write(&init_data).unwrap();
        }

        // Display loop
        data[4] = 11;
        data[5] = 1;
        data[6] = 2;
        data[7] = 5;
        loop {
            // Initialize the packet
            let mut status_data = data.clone();

            // Read CPU utilization & energy consumption
            let cpu_instant = self.cpu.read_instant();
            let cpu_energy = self.cpu.read_energy();

            // Wait
            sleep(Duration::from_millis(POLLING_RATE));

            // ----- Write data to the package -----
            // Power consumption
            let power = (self.cpu.get_power(cpu_energy, POLLING_RATE)).to_be_bytes();
            status_data[8] = power[0];
            status_data[9] = power[1];

            // Temperature
            let temp = (self.cpu.get_temp(self.fahrenheit) as f32).to_be_bytes();
            status_data[10] = if self.fahrenheit { 1 } else { 0 };
            status_data[11] = temp[0];
            status_data[12] = temp[1];
            status_data[13] = temp[2];
            status_data[14] = temp[3];

            // Utilization
            status_data[15] = self.cpu.get_usage(cpu_instant);

            // Checksum & termination byte
            let checksum: u16 = status_data[1..=15].iter().map(|&x| x as u16).sum();
            status_data[16] = (checksum % 256) as u8;
            status_data[17] = 22;

            device.write(&status_data).unwrap();
        }
    }
}
