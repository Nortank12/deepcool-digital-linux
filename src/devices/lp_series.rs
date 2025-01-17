use crate::{error, monitor::{cpu::Cpu, gpu::Gpu}};
use super::{device_error, Mode};
use hidapi::HidApi;
use std::{process::exit, thread::sleep, time::Duration};

/// Helper module for the LP Series.
mod dot_matrix {
    pub enum Unit {
        Percent,
        Celsius,
        Fahrenheit,
        Watt,
        Empty,
    }

    impl Unit {
        /// Returns a 5x5 matrix array representing the unit.
        pub const fn get_pattern(self) -> [[bool; 5]; 5] {
            match self {
                Unit::Percent => [
                    [true, true, false, false, true],
                    [true, true, false, true, false],
                    [false, false, true, false, false],
                    [false, true, false, true, true],
                    [true, false, false, true, true],
                ],
                Unit::Celsius => [
                    [true, false, false, false, false],
                    [false, false, true, true, false],
                    [false, true, false, false, false],
                    [false, true, false, false, false],
                    [false, false, true, true, false],
                ],
                Unit::Fahrenheit => [
                    [true, false, true, true, false],
                    [false, false, true, false, false],
                    [false, false, true, true, false],
                    [false, false, true, false, false],
                    [false, false, true, false, false],
                ],
                Unit::Watt => [
                    [false, false, false, false, false],
                    [true, false, true, false, true],
                    [true, false, true, false, true],
                    [true, false, true, false, true],
                    [false, true, false, true, false],
                ],
                Unit::Empty => [[false; 5]; 5],
            }
        }
    }

    /// Returns a 3x5 matrix array representing the number.
    pub const fn get_number_pattern(num: u8) -> [[bool; 3]; 5] {
        match num {
            0 => [
                [true, true, true],
                [true, false, true],
                [true, false, true],
                [true, false, true],
                [true, true, true],
            ],
            1 => [
                [false, true, false],
                [true, true, false],
                [false, true, false],
                [false, true, false],
                [true, true, true],
            ],
            2 => [
                [true, true, true],
                [false, false, true],
                [false, true, false],
                [true, false, false],
                [true, true, true],
            ],
            3 => [
                [true, true, true],
                [false, false, true],
                [true, true, true],
                [false, false, true],
                [true, true, true],
            ],
            4 => [
                [true, false, true],
                [true, false, true],
                [true, true, true],
                [false, false, true],
                [false, false, true],
            ],
            5 => [
                [true, true, true],
                [true, false, false],
                [true, true, true],
                [false, false, true],
                [true, true, true],
            ],
            6 => [
                [true, true, true],
                [true, false, false],
                [true, true, true],
                [true, false, true],
                [true, true, true],
            ],
            7 => [
                [true, true, true],
                [false, false, true],
                [false, true, false],
                [false, true, false],
                [false, true, false],
            ],
            8 => [
                [true, true, true],
                [true, false, true],
                [true, true, true],
                [true, false, true],
                [true, true, true],
            ],
            9 => [
                [true, true, true],
                [true, false, true],
                [true, true, true],
                [false, false, true],
                [true, true, true],
            ],
            _ => [[false; 3]; 5],
        }
    }

    /// Inserts a pattern into the 14x14 matrix at the defined position.
    pub fn insert_pattern<const M: usize, const N: usize>(
        matrix: &mut [[bool; 14]; 14],
        pattern: [[bool; M]; N],
        row_pos: usize,
        col_pos: usize,
    ) {
        // Calculate the actual dimensions that will fit
        let max_rows = (14 - row_pos).min(N);
        let max_cols = (14 - col_pos).min(M);

        // Insert the pattern
        for i in 0..max_rows {
            for j in 0..max_cols {
                matrix[row_pos + i][col_pos + j] = pattern[i][j];
            }
        }
    }

    /// Converts the 14x14 matrix to be data bytes.
    pub fn matrix_to_bytes(matrix: [[bool; 14]; 14]) -> [u8; 28] {
        let mut bytes: [u8; 28] = [0; 28];

        // Values for each row position (HEX: 10, 20, 40, 80, 01, 02, 04)
        const ROW_VALUES: [u8; 7] = [16, 32, 64, 128, 1, 2, 4];

        // First 14 bytes (odd rows)
        for col in 0..14 {
            let mut byte = 0;
            for row_id in 0..7 {
                if matrix[row_id * 2][col] {
                    byte += ROW_VALUES[row_id];
                }
            }
            bytes[col] = byte;
        }

        // Last 14 bytes (even rows - reversed)
        for col in 0..14 {
            let mut byte = 0;
            for row_id in 0..7 {
                if matrix[row_id * 2 + 1][col] {
                    byte += ROW_VALUES[row_id];
                }
            }
            bytes[27 - col] = byte;
        }

        bytes
    }
}

pub const DEFAULT_MODE: Mode = Mode::CpuUsage;
pub const POLLING_RATE: u64 = 750;

pub struct Display {
    pub mode: Mode,
    pub secondary: Option<Mode>,
    fahrenheit: bool,
    cpu: Cpu,
    gpu: Gpu,
}

impl Display {
    pub fn new(mode: &Mode, secondary: &Mode, fahrenheit: bool) -> Self {
        // Verify the display mode
        let mode = match mode {
            Mode::Default => DEFAULT_MODE,
            Mode::CpuUsage => Mode::CpuUsage,
            Mode::CpuTemperature => Mode::CpuTemperature,
            Mode::CpuPower => Mode::CpuPower,
            Mode::GpuUsage => Mode::GpuUsage,
            Mode::GpuTemperature => Mode::GpuTemperature,
            Mode::GpuPower => Mode::GpuPower,
            _ => mode.support_error(),
        };

        let secondary = match secondary {
            Mode::Default => None,
            Mode::CpuUsage => Some(Mode::CpuUsage),
            Mode::CpuTemperature => Some(Mode::CpuTemperature),
            Mode::CpuPower => Some(Mode::CpuPower),
            Mode::GpuUsage => Some(Mode::GpuUsage),
            Mode::GpuTemperature => Some(Mode::GpuTemperature),
            Mode::GpuPower => Some(Mode::GpuPower),
            _ => Some(secondary.support_error_secondary()),
        };

        Display {
            mode,
            secondary,
            fahrenheit,
            cpu: Cpu::new(),
            gpu: Gpu::new(),
        }
    }

    pub fn run(&self, api: &HidApi, vid: u16, pid: u16) {
        // Connect to device
        let device = api.open(vid, pid).unwrap_or_else(|_| device_error());

        // Check if `rapl_max_uj` was read correctly
        if self.mode == Mode::CpuPower && self.cpu.rapl_max_uj == 0 {
            error!("Failed to get CPU power details");
            exit(1);
        }

        // Data packet
        let mut data: [u8; 64] = [0; 64];
        data[0] = 16;
        data[1] = 104;
        data[2] = 1;
        data[3] = 5;
        data[4] = 29;
        data[5] = 1;

        // Display loop
        loop {
            // Initialize the packet
            let mut status_data = data.clone();
            let mut matrix = [[false; 14]; 14];

            // Set the pixels and calculate the bytes for the display
            match &self.secondary {
                Some(secondary) => {
                    let (value, unit) = self.get_system_info(&self.mode);
                    self.insert_to_matrix(&mut matrix, 1, value, unit);
                    let (value, unit) = self.get_system_info(secondary);
                    self.insert_to_matrix(&mut matrix, 8, value, unit);
                },
                None => {
                    let (value, unit) = self.get_system_info(&self.mode);
                    self.insert_to_matrix(&mut matrix, 5, value, unit);
                }
            }
            status_data[6..=33].copy_from_slice(&dot_matrix::matrix_to_bytes(matrix));

            // Checksum & termination byte
            let checksum: u16 = status_data[1..=33].iter().map(|&x| x as u16).sum();
            status_data[34] = (checksum % 256) as u8;
            status_data[35] = 22;

            device.write(&status_data).unwrap();
        }
    }

    fn get_system_info(&self, mode: &Mode) -> (u16, dot_matrix::Unit) {
        let mut value = 0;
        let mut unit = dot_matrix::Unit::Empty;

        match mode {
            Mode::CpuUsage => {
                // Get CPU instant & wait
                let cpu_instant = self.cpu.read_instant();
                sleep(Duration::from_millis(POLLING_RATE));

                // Set the data
                value = self.cpu.get_usage(cpu_instant) as u16;
                unit = dot_matrix::Unit::Percent;
            }
            Mode::CpuTemperature => {
                // Wait
                sleep(Duration::from_millis(POLLING_RATE));

                // Set the data
                value = self.cpu.get_temp(self.fahrenheit) as u16;
                unit = if self.fahrenheit { dot_matrix::Unit::Fahrenheit } else { dot_matrix::Unit::Celsius };
            }
            Mode::CpuPower => {
                // Get CPU energy & wait
                let cpu_energy = self.cpu.read_energy();
                sleep(Duration::from_millis(POLLING_RATE));

                // Set the data
                value = self.cpu.get_power(cpu_energy, POLLING_RATE);
                unit = dot_matrix::Unit::Watt;
            }
            Mode::GpuUsage => {
                // Wait
                sleep(Duration::from_millis(POLLING_RATE));

                // Set the data
                value = self.gpu.get_usage() as u16;
                unit = dot_matrix::Unit::Percent;
            }
            Mode::GpuTemperature => {
                // Wait
                sleep(Duration::from_millis(POLLING_RATE));

                // Set the data
                value = self.gpu.get_temp(self.fahrenheit) as u16;
                unit = if self.fahrenheit { dot_matrix::Unit::Fahrenheit } else { dot_matrix::Unit::Celsius };
            }
            Mode::GpuPower => {
                // Wait
                sleep(Duration::from_millis(POLLING_RATE));

                // Set the data
                value = self.gpu.get_power();
                unit = dot_matrix::Unit::Watt;
            }
            _ => (),
        }

        (value, unit)
    }

    fn insert_to_matrix(&self, matrix: &mut [[bool; 14]; 14], row_id: usize, value: u16, unit: dot_matrix::Unit) {
        if value / 100 < 1 {
            // 2-digit number
            dot_matrix::insert_pattern(matrix, dot_matrix::get_number_pattern((value / 10) as u8), row_id, 1);
            dot_matrix::insert_pattern(matrix, dot_matrix::get_number_pattern((value % 10) as u8), row_id, 5);
            dot_matrix::insert_pattern(matrix, unit.get_pattern(), 5, 9);
        } else {
            // 3-digit number
            dot_matrix::insert_pattern(matrix, dot_matrix::get_number_pattern((value / 100) as u8), row_id, 1);
            dot_matrix::insert_pattern(matrix, dot_matrix::get_number_pattern((value % 100 / 10) as u8), row_id, 5);
            dot_matrix::insert_pattern(matrix, dot_matrix::get_number_pattern((value % 10) as u8), row_id, 9);
            dot_matrix::insert_pattern(matrix, unit.get_pattern(), 5, 13);
        }
    }
}
