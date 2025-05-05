use crate::monitor::{cpu::Cpu, gpu::Gpu};
use super::{device_error, Mode};
use hidapi::HidApi;
use std::{thread::sleep, time::Duration};

pub const DEFAULT_MODE: Mode = Mode::CpuTemperature;

pub struct Display {
    pub mode: Mode,
    pub secondary: Mode,
    update: Duration,
    fahrenheit: bool,
    cpu: Cpu,
    gpu: Gpu,
}

impl Display {
    pub fn new(mode: &Mode, secondary: &Mode, update: Duration, fahrenheit: bool) -> Self {
        // Verify the display mode
        let mode = match mode {
            Mode::Default => DEFAULT_MODE,
            Mode::Auto => Mode::Auto,
            Mode::CpuTemperature => Mode::CpuTemperature,
            Mode::CpuUsage => Mode::CpuUsage,
            _ => mode.support_error(),
        };

        let secondary = match secondary {
            Mode::Default => match mode {
                Mode::CpuTemperature => Mode::GpuTemperature,
                Mode::CpuUsage => Mode::GpuUsage,
                _ => Mode::Auto,
            },
            Mode::GpuTemperature => Mode::GpuTemperature,
            Mode::GpuUsage => Mode::GpuUsage,
            _ => secondary.support_error_secondary(),
        };

        Display {
            mode,
            secondary,
            update,
            fahrenheit,
            cpu: Cpu::new(),
            gpu: Gpu::new(),
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
                    device.write(&self.status_message(&data, &Mode::CpuTemperature)).unwrap();
                }
                for _ in 0..8 {
                    device.write(&self.status_message(&data, &Mode::CpuUsage)).unwrap();
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
        sleep(self.update);

        // Calculate CPU & GPU usage
        let cpu_usage = self.cpu.get_usage(cpu_instant);
        let gpu_usage = self.gpu.get_usage();

        // Main display
        match mode {
            Mode::CpuTemperature => {
                // CPU
                let unit = if self.fahrenheit { 35 } else { 19 };
                let cpu_temp = self.cpu.get_temp(self.fahrenheit);
                data[1] = unit;
                data[3] = cpu_temp / 100;
                data[4] = cpu_temp % 100 / 10;
                data[5] = cpu_temp % 10;
                // GPU
                if self.secondary == Mode::Auto {
                    let gpu_temp = self.gpu.get_temp(self.fahrenheit);
                    data[6] = unit;
                    data[8] = gpu_temp / 100;
                    data[9] = gpu_temp % 100 / 10;
                    data[10] = gpu_temp % 10;
                }
            }
            Mode::CpuUsage => {
                // CPU
                data[1] = 76;
                data[3] = cpu_usage / 100;
                data[4] = cpu_usage % 100 / 10;
                data[5] = cpu_usage % 10;
                // GPU
                if self.secondary == Mode::Auto {
                    data[6] = 76;
                    data[8] = gpu_usage / 100;
                    data[9] = gpu_usage % 100 / 10;
                    data[10] = gpu_usage % 10;
                }
            }
            _ => (),
        }
        if data[6] == 0 {
            match self.secondary {
                Mode::GpuTemperature => {
                    let gpu_temp = self.gpu.get_temp(self.fahrenheit);
                    data[6] = if self.fahrenheit { 35 } else { 19 };
                    data[8] = gpu_temp / 100;
                    data[9] = gpu_temp % 100 / 10;
                    data[10] = gpu_temp % 10;
                }
                Mode::GpuUsage => {
                    data[6] = 76;
                    data[8] = gpu_usage / 100;
                    data[9] = gpu_usage % 100 / 10;
                    data[10] = gpu_usage % 10;
                }
                _ => (),
            }
        }
        // Status bar
        data[2] = if cpu_usage < 15 { 1 } else { (cpu_usage as f32 / 10.0).round() as u8 };
        data[7] = if gpu_usage < 15 { 1 } else { (gpu_usage as f32 / 10.0).round() as u8 };

        data
    }
}
