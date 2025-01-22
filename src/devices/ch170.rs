use crate::{error, monitor::{cpu::Cpu, gpu::Gpu}};
use super::{device_error, Mode};
use hidapi::HidApi;
use std::{process::exit, thread::sleep, time::Duration};

pub const DEFAULT_MODE: Mode = Mode::CpuFrequency;
pub const POLLING_RATE: u64 = 750;

pub struct Display {
    pub mode: Mode,
    fahrenheit: bool,
    cpu: Cpu,
    gpu: Gpu,
}

impl Display {
    pub fn new(mode: &Mode, fahrenheit: bool) -> Self {
        // Verify the display mode
        let mode = match mode {
            Mode::Default => DEFAULT_MODE,
            Mode::CpuFrequency => Mode::CpuFrequency,
            Mode::CpuFan => Mode::CpuFan,
            Mode::Gpu => Mode::Gpu,
            Mode::Psu => Mode::Psu,
            _ => mode.support_error(),
        };

        Display {
            mode,
            fahrenheit,
            cpu: Cpu::new(),
            gpu: Gpu::new(),
        }
    }

    pub fn run(&self, api: &HidApi, vid: u16, pid: u16) {
        // Connect to device
        let device = api.open(vid, pid).unwrap_or_else(|_| device_error());

        // Check if `rapl_max_uj` was read correctly
        if matches!(self.mode, Mode::CpuFrequency | Mode::CpuFan) && self.cpu.rapl_max_uj == 0 {
            error!("Failed to get CPU power details");
            exit(1);
        }

        // Data packet
        let mut data: [u8; 64] = [0; 64];
        data[0] = 16;
        data[1] = 104;
        data[2] = 1;
        data[3] = 6;
        data[4] = 35;
        data[5] = 1;
        data[6] = match self.mode {
            Mode::CpuFrequency => 2,
            Mode::CpuFan => 3,
            Mode::Gpu => 4,
            Mode::Psu => 5,
            _ => 0,
        };
        data[9] = if self.fahrenheit { 1 } else { 0 };

        // Display loop
        loop {
            // Initialize the packet
            let mut status_data = data.clone();

            match self.mode {
                Mode::CpuFrequency | Mode::CpuFan => {
                    // Read CPU utilization & energy consumption
                    let cpu_instant = self.cpu.read_instant();
                    let cpu_energy = self.cpu.read_energy();

                    // Wait
                    sleep(Duration::from_millis(POLLING_RATE));

                    // Power consumption
                    let power = (self.cpu.get_power(cpu_energy, POLLING_RATE)).to_be_bytes();
                    status_data[7] = power[0];
                    status_data[8] = power[1];

                    // Temperature
                    let temp = (self.cpu.get_temp(self.fahrenheit) as f32).to_be_bytes();
                    status_data[10] = temp[0];
                    status_data[11] = temp[1];
                    status_data[12] = temp[2];
                    status_data[13] = temp[3];

                    // Utilization
                    status_data[14] = self.cpu.get_usage(cpu_instant);

                    // Frequency
                    if self.mode == Mode::CpuFrequency {
                        let frequency = (self.cpu.get_frequency()).to_be_bytes();
                        status_data[15] = frequency[0];
                        status_data[16] = frequency[1];
                    }
                }
                Mode::Gpu => {
                    // Wait
                    sleep(Duration::from_millis(POLLING_RATE));

                    // Power consumption
                    let power = (self.gpu.get_power()).to_be_bytes();
                    status_data[19] = power[0];
                    status_data[20] = power[1];

                    // Temperature
                    let temp = (self.gpu.get_temp(self.fahrenheit) as f32).to_be_bytes();
                    status_data[21] = temp[0];
                    status_data[22] = temp[1];
                    status_data[23] = temp[2];
                    status_data[24] = temp[3];

                    // Utilization
                    status_data[25] = self.gpu.get_usage();

                    // Frequency
                    let frequency = (self.gpu.get_frequency()).to_be_bytes();
                    status_data[26] = frequency[0];
                    status_data[27] = frequency[1];
                }
                Mode::Psu => {
                    // Wait
                    sleep(Duration::from_millis(POLLING_RATE));
                }
                _ => (),
            }

            // Checksum & termination byte
            let checksum: u16 = status_data[1..=39].iter().map(|&x| x as u16).sum();
            status_data[40] = (checksum % 256) as u8;
            status_data[41] = 22;

            device.write(&status_data).unwrap();
        }
    }
}
