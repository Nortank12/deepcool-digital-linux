use crate::monitor::{cpu::Cpu, gpu::Gpu};
use super::{device_error, Mode};
use hidapi::HidApi;
use std::{thread::sleep, time::Duration};

pub const DEFAULT_MODE: Mode = Mode::Cpu;
pub const POLLING_RATE: u64 = 750;

pub struct Display {
    mode: Mode,
    fahrenheit: bool,
    cpu: Cpu,
    gpu: Gpu,
}

impl Display {
    pub fn new(mode: &Mode, fahrenheit: bool) -> Self {
        // Verify the display mode
        let mode = match mode {
            Mode::Default => DEFAULT_MODE,
            Mode::Cpu => Mode::Cpu,
            Mode::Gpu => Mode::Gpu,
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

        // Get temperature unit
        let unit = if self.fahrenheit { "F" } else { "C" };

        // Display loop
        loop {
            let message = match self.mode {
                Mode::Cpu => {
                    // Get CPU instant & wait
                    let cpu_instant = self.cpu.read_instant();
                    sleep(Duration::from_millis(POLLING_RATE));

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
                    sleep(Duration::from_millis(POLLING_RATE));

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
