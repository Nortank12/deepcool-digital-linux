use std::{
    fs::File,
    io::{BufRead, BufReader},
    process::exit,
    thread::sleep,
    time::Duration
};
use cpu_monitor::CpuInstant;

/// Looks for the appropriate CPU temperature sensor datastream in the hwmon folder.
pub fn find_cpu_sensor() -> String {
    let mut i = 0;
    loop {
        let mut line = String::new();

        match File::open(format!("/sys/class/hwmon/hwmon{i}/name")) {
            Ok(file) => {
                let mut reader = BufReader::new(file);
                reader.read_line(&mut line).unwrap();
                let hwname = line.trim_end();
                if hwname == "k10temp" || hwname == "coretemp" {
                    return format!("/sys/class/hwmon/hwmon{i}/temp1_input");
                }
            },
            Err(_) => {
                println!("CPU temperature sensor not found!");
                exit(1);
            },
        }
        i += 1;
    }
}

/// Reads the value of the `k10temp` sensor and returns it as a rounded integer.
fn get_temp(cpu_sensor: &str, fahrenheit: bool) -> u8 {
    // Read sensor data
    let mut line = String::new();
    let file = File::open(cpu_sensor)
        .expect("Sensor data not found!");
    let mut reader = BufReader::new(file);
    reader.read_line(&mut line).unwrap();

    // Calculate temperature
    let mut k10temp = line.trim().parse::<u32>().unwrap();
    if fahrenheit {
        k10temp = k10temp * 9/5 + 32000
    }
    
    (k10temp as f32 / 1000 as f32).round() as u8
}

/// Calculates the CPU usage in the duration of the provided `milliseconds` and returns
/// the percentage as a rounded integer.
fn get_usage(duration_ms: u64) -> u8 {
    // Read CPU load
    let start = CpuInstant::now();
    sleep(Duration::from_millis(duration_ms));
    let end = CpuInstant::now();

    // Calculate duration
    let duration = end.unwrap() - start.unwrap();
    
    (duration.non_idle() * 100 as f64).round() as u8
}

/// Reads CPU information and converts the data into a digestible format for the device.
pub fn status_message(cpu_sensor: &str, mode: &str, fahrenheit: bool, alarm: bool) -> [u8; 64] {
    let mut data: [u8; 64] = [0; 64];
    data[0] = 16;

    let usage = get_usage(750);
    let temp = get_temp(cpu_sensor, fahrenheit);
    data[2] = (usage as f32 / 10 as f32).round() as u8;

    match mode {
        "temp" => {
            data[1] = if fahrenheit {35} else {19};
            data[3] = temp / 100;
            data[4] = temp % 100 / 10;
            data[5] = temp % 10;
            data[6] = (alarm && temp > if fahrenheit {176} else {80}) as u8;
        },
        "usage" => {
            data[1] = 76;
            data[3] = usage / 100;
            data[4] = usage % 100 / 10;
            data[5] = usage % 10;
            data[6] = (alarm && temp > if fahrenheit {176} else {80}) as u8;
        }
        _ => ()
    }

    data
}

/// Startup message to trigger the status bar animation.
pub fn startup_message() -> [u8; 64] {
    let mut data: [u8; 64] = [0; 64];
    data[0] = 16;
    data[1] = 170;

    data
}
