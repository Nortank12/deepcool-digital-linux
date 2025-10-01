//! Display module for:
//! - CH510 MESH DIGITAL

use crate::monitor::{cpu::Cpu, gpu::Gpu};
use super::{device_error, Mode};
use hidapi::HidApi;
use std::{thread::sleep, time::Duration};

pub const DEFAULT_MODE: Mode = Mode::Cpu;

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
            Mode::Cpu => Mode::Cpu,
            Mode::Gpu => Mode::Gpu,
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

        // Display warning if a required module is missing
        match self.mode {
            Mode::Cpu => self.cpu.warn_temp(),
            Mode::Gpu => self.gpu.warn_missing(),
            _ => (),
        }

        // Get temperature unit
        let unit = if self.fahrenheit { "F" } else { "C" };

        // Display loop
        loop {
            let message = match self.mode {
                Mode::Cpu => {
                    // Get CPU instant & wait
                    let cpu_instant = self.cpu.read_instant();
                    sleep(self.update);

                    // Return the message
                    format!(
                        "HLXDATA({},{},0,0,{})\r\n",
                        self.cpu.get_usage(cpu_instant),
                        self.cpu.get_temp(self.fahrenheit),
                        unit,
                    )
                }
                Mode::Gpu => {
                    // Wait
                    sleep(self.update);

                    // Return the message
                    format!(
                        "HLXDATA({},{},0,0,{})\r\n",
                        self.gpu.get_usage(),
                        self.gpu.get_temp(self.fahrenheit),
                        unit,
                    )
                }
                _ => "".to_owned(),
            };
            device.write(message.as_bytes()).unwrap();
        }
    }
}
