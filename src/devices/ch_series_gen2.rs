use crate::{monitor::{cpu::Cpu, gpu::Gpu}, warning};
use super::{device_error, Mode, AUTO_MODE_INTERVAL};
use hidapi::HidApi;
use std::{thread::sleep, time::{Duration, Instant}};

pub const DEFAULT_MODE: Mode = Mode::CpuFrequency;

pub struct Display {
    cpu: Cpu,
    gpu: Gpu,
    pub mode: Mode,
    update: Duration,
    fahrenheit: bool,
}

impl Display {
    pub fn new(cpu: Cpu, gpu: Gpu, mode: &Mode, update: Duration, fahrenheit: bool) -> Self {
        // Verify the display mode
        let mode = match mode {
            Mode::Default => DEFAULT_MODE,
            Mode::Auto => Mode::Auto,
            Mode::CpuFrequency => Mode::CpuFrequency,
            Mode::CpuFan => Mode::CpuFan,
            Mode::Gpu => Mode::Gpu,
            Mode::Psu => Mode::Psu,
            _ => mode.support_error(),
        };

        Display {
            cpu,
            gpu,
            mode,
            update,
            fahrenheit,
        }
    }

    pub fn run(&self, api: &HidApi, vid: u16, pid: u16) {
        // Connect to device
        let device = api.open(vid, pid).unwrap_or_else(|_| device_error());

        // Display warning to address limitated display modes
        match self.mode {
            Mode::CpuFan => { warning!("CPU fan speed monitoring is not yet supported"); }
            Mode::Psu => { warning!("PSU monitoring is not yet supported"); }
            Mode::Auto => { warning!("Display mode \"auto\" only cycles between fully supported modes"); }
            _ => (),
        }

        // Display warning if a required module is missing
        if matches!(self.mode, Mode::CpuFrequency | Mode::CpuFan | Mode::Auto) {
            self.cpu.warn_temp();
            self.cpu.warn_rapl();
        }
        if matches!(self.mode, Mode::Gpu | Mode::Auto) {
            self.gpu.warn_missing();
        }

        // Data packet
        let mut data: [u8; 64] = [0; 64];
        data[0] = 16;
        data[1] = 104;
        data[2] = 1;
        data[3] = 6;
        data[4] = 35;
        data[5] = 1;
        data[9] = if self.fahrenheit { 1 } else { 0 };

        // Display loop
        match self.mode {
            Mode::Auto => {
                let mut initial_update = self.update;
                let mut mode = Mode::CpuFrequency;
                loop {
                    // Initial update
                    device.write(&self.status_message(&data, &mode, initial_update)).unwrap();

                    // Update until timeout
                    let timeout = Instant::now() + AUTO_MODE_INTERVAL;
                    while Instant::now() + self.update < timeout {
                        device.write(&self.status_message(&data, &mode, self.update)).unwrap();
                    }

                    // Make the next initial update faster to fit the timeframe
                    initial_update = timeout - Instant::now();

                    // Switch to the next display mode
                    mode = match mode {
                        Mode::CpuFrequency => Mode::Gpu,
                        Mode::Gpu => Mode::CpuFrequency,
                        _ => DEFAULT_MODE,
                    }
                }
            }
            _ => loop {
                device.write(&self.status_message(&data, &self.mode, self.update)).unwrap();
            }
        }
    }

    /// Reads the system status information and returns the data packet.
    fn status_message(&self, inital_data: &[u8; 64], mode: &Mode, update: Duration) -> [u8; 64] {
        // Clone the data packet
        let mut data = inital_data.clone();

        // Set the display mode
        data[6] = match mode {
            Mode::CpuFrequency => 2,
            Mode::CpuFan => 3,
            Mode::Gpu => 4,
            Mode::Psu => 5,
            _ => 0,
        };

        // Main display
        match mode {
            Mode::CpuFrequency | Mode::CpuFan => {
                // Read CPU utilization & energy consumption
                let cpu_instant = self.cpu.read_instant();
                let cpu_energy = self.cpu.read_energy();

                // Wait
                sleep(update);

                // Power consumption
                let power = (self.cpu.get_power(cpu_energy, update.as_millis() as u64)).to_be_bytes();
                data[7] = power[0];
                data[8] = power[1];

                // Temperature
                let temp = (self.cpu.get_temp(self.fahrenheit) as f32).to_be_bytes();
                data[10] = temp[0];
                data[11] = temp[1];
                data[12] = temp[2];
                data[13] = temp[3];

                // Utilization
                data[14] = self.cpu.get_usage(cpu_instant);

                // Frequency
                if matches!(mode, Mode::CpuFrequency) {
                    let frequency = (self.cpu.get_frequency()).to_be_bytes();
                    data[15] = frequency[0];
                    data[16] = frequency[1];
                }
            }
            Mode::Gpu => {
                // Wait
                sleep(update);

                // Power consumption
                let power = (self.gpu.get_power()).to_be_bytes();
                data[19] = power[0];
                data[20] = power[1];

                // Temperature
                let temp = (self.gpu.get_temp(self.fahrenheit) as f32).to_be_bytes();
                data[21] = temp[0];
                data[22] = temp[1];
                data[23] = temp[2];
                data[24] = temp[3];

                // Utilization
                data[25] = self.gpu.get_usage();

                // Frequency
                let frequency = (self.gpu.get_frequency()).to_be_bytes();
                data[26] = frequency[0];
                data[27] = frequency[1];
            }
            Mode::Psu => {
                // Wait
                sleep(update);
            }
            _ => (),
        }

        // Checksum & termination byte
        let checksum: u16 = data[1..=39].iter().map(|&x| x as u16).sum();
        data[40] = (checksum % 256) as u8;
        data[41] = 22;

        data
    }
}
