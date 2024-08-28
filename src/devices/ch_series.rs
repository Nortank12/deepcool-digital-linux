use crate::monitor::{cpu::Cpu, gpu::Gpu};
use hidapi::HidApi;
use std::{thread::sleep, time::Duration};

const VENDOR: u16 = 0x3633;
const POLLING_RATE: u64 = 750;

pub struct Display {
    product_id: u16,
    fahrenheit: bool,
    cpu: Cpu,
    gpu: Gpu,
}

impl Display {
    pub fn new(product_id: u16, fahrenheit: bool) -> Self {
        Display {
            product_id,
            fahrenheit,
            cpu: Cpu::new(),
            gpu: Gpu::new(),
        }
    }

    pub fn run(&self, api: &HidApi, mode: &str) {
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
                        .write(&self.status_message(&data, "temp"))
                        .expect("Failed to write data");
                }
                for _ in 0..8 {
                    device
                        .write(&self.status_message(&data, "usage"))
                        .expect("Failed to write data");
                }
            }
        } else {
            loop {
                device
                    .write(&self.status_message(&data, &mode))
                    .expect("Failed to write data");
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

        // Calculate CPU & GPU usage
        let cpu_usage = self.cpu.get_usage(cpu_instant);
        let gpu_usage = self.gpu.get_usage();

        // Main display
        match mode {
            "temp" => {
                let unit = if self.fahrenheit { 35 } else { 19 };
                let cpu_temp = self.cpu.get_temp(self.fahrenheit);
                let gpu_temp = self.gpu.get_temp(self.fahrenheit);
                // CPU
                data[1] = unit;
                data[3] = cpu_temp / 100;
                data[4] = cpu_temp % 100 / 10;
                data[5] = cpu_temp % 10;
                // GPU
                data[6] = unit;
                data[8] = gpu_temp / 100;
                data[9] = gpu_temp % 100 / 10;
                data[10] = gpu_temp % 10;
            }
            "usage" => {
                // CPU
                data[1] = 76;
                data[3] = cpu_usage / 100;
                data[4] = cpu_usage % 100 / 10;
                data[5] = cpu_usage % 10;
                // GPU
                data[6] = 76;
                data[8] = gpu_usage / 100;
                data[9] = gpu_usage % 100 / 10;
                data[10] = gpu_usage % 10;
            }
            _ => (),
        }
        // Status bar
        data[2] = if cpu_usage < 20 { 1 } else { (cpu_usage as f32 / 10.0).round() as u8 };
        data[7] = if gpu_usage < 20 { 1 } else { (gpu_usage as f32 / 10.0).round() as u8 };

        data
    }
}
