use crate::{error, monitor::cpu::Cpu};
use super::{device_error, Mode};
use hidapi::HidApi;
use std::{process::exit, thread::sleep, time::Duration};

mod dot_matrix {
    pub enum Unit {
        Percent,
        Celsius,
        Fahrenheit,
        Watt,
        Empty,
    }

    impl Unit {
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
}

pub const DEFAULT_MODE: Mode = Mode::CpuUsage;
pub const POLLING_RATE: u64 = 750;

pub struct Display {
    pub mode: Mode,
    pub secondary: Option<Mode>,
    fahrenheit: bool,
    cpu: Cpu,
}

impl Display {
    pub fn new(mode: &Mode, secondary: &Mode, fahrenheit: bool) -> Self {
        // Verify the display mode
        let mode = match mode {
            Mode::Default => DEFAULT_MODE,
            Mode::CpuUsage => Mode::CpuUsage,
            Mode::CpuTemperature => Mode::CpuTemperature,
            Mode::CpuPower => Mode::CpuPower,
            _ => mode.support_error(),
        };

        let secondary = match secondary {
            Mode::Default => None,
            Mode::CpuUsage => Some(Mode::CpuUsage),
            Mode::CpuTemperature => Some(Mode::CpuTemperature),
            Mode::CpuPower => Some(Mode::CpuPower),
            _ => Some(secondary.support_error_secondary()),
        };

        Display {
            mode,
            secondary,
            fahrenheit,
            cpu: Cpu::new(),
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
            let mut value = 0;
            let mut unit = dot_matrix::Unit::Empty;

            match self.mode {
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
                _ => (),
            }

            // Set the pixels and calculate the bytes for the display
            if value / 100 < 1 {
                // 2-digit number
                dot_matrix::insert_pattern(&mut matrix, dot_matrix::get_number_pattern((value / 10) as u8), 5, 1);
                dot_matrix::insert_pattern(&mut matrix, dot_matrix::get_number_pattern((value % 10) as u8), 5, 5);
                dot_matrix::insert_pattern(&mut matrix, unit.get_pattern(), 5, 9);
            } else {
                // 3-digit number
                dot_matrix::insert_pattern(&mut matrix, dot_matrix::get_number_pattern((value / 100) as u8), 5, 1);
                dot_matrix::insert_pattern(&mut matrix, dot_matrix::get_number_pattern((value % 100 / 10) as u8), 5, 5);
                dot_matrix::insert_pattern(&mut matrix, dot_matrix::get_number_pattern((value % 10) as u8), 5, 9);
                dot_matrix::insert_pattern(&mut matrix, unit.get_pattern(), 5, 13);
            }
            status_data[6..=33].copy_from_slice(&self.matrix_to_bytes(matrix));

            // Checksum & termination byte
            let checksum: u16 = status_data[1..=33].iter().map(|&x| x as u16).sum();
            status_data[34] = (checksum % 256) as u8;
            status_data[35] = 22;

            device.write(&status_data).unwrap();
        }
    }

    /// Converts the 14x14 matrix to be data bytes.
    fn matrix_to_bytes(&self, matrix: [[bool; 14]; 14]) -> [u8; 28] {
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
