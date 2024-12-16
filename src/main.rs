mod devices;
mod monitor;

use colored::*;
use hidapi::HidApi;
use std::{env::args, process::exit};

const VENDOR: u16 = 0x3633;

enum TemperatureUnit {
    Celsius,
    Fahrenheit,
}

impl TemperatureUnit {
    fn symbol(&self) -> &'static str {
        match self {
            TemperatureUnit::Celsius => "°C",
            TemperatureUnit::Fahrenheit => "°F",
        }
    }
}

enum AlarmState {
    Auto,
    On,
    Off,
    NotSupported,
}

struct Alarm {
    state: AlarmState,
    temp_limit: u8,
}

struct Args {
    mode: String,
    pid: u16,
    fahrenheit: bool,
    alarm: bool,
}

#[macro_export]
macro_rules! warning {
    ($input:expr) => {
        use colored::*;
        eprintln!("{}", format!("{} {}", "Warning!".yellow(), $input).bold());
    };
}

#[macro_export]
macro_rules! error {
    ($input:expr) => {
        use colored::*;
        eprintln!("{}", format!("{} {}", "Error!".red(), $input).bold());
    };
}

fn main() {
    // Read args
    let args = read_args();

    // Find device
    let api = HidApi::new().unwrap_or_else(|err| {
        error!(err);
        exit(1);
    });
    let mut product_id = 0;
    for device in api.device_list() {
        if device.vendor_id() == VENDOR {
            if args.pid == 0 || device.product_id() == args.pid {
                product_id = device.product_id();
                println!("Device found: {}", device.product_string().unwrap().bright_green());
                println!("-----");
                break;
            }
        }
    }
    if product_id == 0 {
        if args.pid > 0 {
            error!("No DeepCool device was found with the specified PID");
        } else {
            error!("No DeepCool device was found");
        }
        exit(1);
    }

    // Connect to device and send datastream
    match product_id {
        // AK Series
        1..=4 => {
            if args.mode == "power" {
                error!("Display mode \"power\" is not supported on your device");
                exit(1);
            }
            // Write info
            display_configuration_info(
                &args.mode,
                if args.fahrenheit { TemperatureUnit::Fahrenheit } else { TemperatureUnit::Celsius },
                Alarm {
                    state: if args.alarm { AlarmState::On } else { AlarmState::Off },
                    temp_limit: if args.fahrenheit {
                        devices::ak_series::TEMP_LIMIT_F
                    } else {
                        devices::ak_series::TEMP_LIMIT_C
                    },
                },
                devices::ak_series::POLLING_RATE,
            );
            // Display loop
            let ak_device = devices::ak_series::Display::new(product_id, args.fahrenheit, args.alarm);
            ak_device.run(&api, &args.mode);
        }
        // LS Series
        6 => {
            if args.mode == "usage" {
                error!("Display mode \"usage\" is not supported on your device");
                exit(1);
            }
            // Write info
            display_configuration_info(
                &args.mode,
                if args.fahrenheit { TemperatureUnit::Fahrenheit } else { TemperatureUnit::Celsius },
                Alarm {
                    state: if args.alarm { AlarmState::On } else { AlarmState::Off },
                    temp_limit: if args.fahrenheit {
                        devices::ls_series::TEMP_LIMIT_F
                    } else {
                        devices::ls_series::TEMP_LIMIT_C
                    },
                },
                devices::ls_series::POLLING_RATE,
            );
            // Display loop
            let ls_device = devices::ls_series::Display::new(product_id, args.fahrenheit, args.alarm);
            ls_device.run(&api, &args.mode);
        }
        // AG Series
        8 => {
            if args.mode == "power" {
                error!("Display mode \"power\" is not supported on your device");
                exit(1);
            }
            // Write info
            display_configuration_info(
                &args.mode,
                TemperatureUnit::Celsius,
                Alarm {
                    state: if args.alarm { AlarmState::On } else { AlarmState::Off },
                    temp_limit: devices::ag_series::TEMP_LIMIT_C,
                },
                devices::ag_series::POLLING_RATE,
            );
            if args.fahrenheit {
                warning!("Displaying ˚F is not supported, value will be ignored");
            }
            // Display loop
            let ag_device = devices::ag_series::Display::new(product_id, args.alarm);
            ag_device.run(&api, &args.mode);
        }
        // LD Series
        10 => {
            // Write info
            display_configuration_info(
                "auto",
                if args.fahrenheit { TemperatureUnit::Fahrenheit } else { TemperatureUnit::Celsius },
                Alarm {
                    state: AlarmState::Auto,
                    temp_limit: if args.fahrenheit {
                        devices::ld_series::TEMP_LIMIT_F
                    } else {
                        devices::ld_series::TEMP_LIMIT_C
                    },
                },
                devices::ld_series::POLLING_RATE,
            );
            if args.mode != "temp" {
                warning!("Display mode cannot be changed, value will be ignored");
            }
            if args.alarm {
                warning!("The alarm is hard-coded in your device, value will be ignored");
            }
            // Display loop
            let ld_device = devices::ld_series::Display::new(product_id, args.fahrenheit);
            ld_device.run(&api);
        }
        // CH Series & MORPHEUS
        5 | 7 | 21 => {
            if args.mode == "power" {
                error!("Display mode \"power\" is not supported on your device");
                exit(1);
            }
            // Write info
            display_configuration_info(
                &args.mode,
                if args.fahrenheit { TemperatureUnit::Fahrenheit } else { TemperatureUnit::Celsius },
                Alarm { state: AlarmState::NotSupported, temp_limit: 0 },
                devices::ch_series::POLLING_RATE,
            );
            if args.alarm {
                warning!("Alarm is not supported, value will be ignored");
            }
            // Display loop
            let ch_device = devices::ch_series::Display::new(product_id, args.fahrenheit);
            ch_device.run(&api, &args.mode);
        }
        _ => {
            println!("Device not yet supported!");
            println!("\nPlease create an issue on GitHub providing your device name and the following information:");
            let device = api.open(VENDOR, product_id).unwrap_or_else(|_| {
                error!("Failed to access the USB device");
                eprintln!("       Try to run the program as root or give permission to the neccesary resources.");
                eprintln!("       You can find instructions about rootless mode on GitHub.");
                exit(1);
            });
            let info = device.get_device_info().unwrap();
            println!("Vendor ID: {}", info.vendor_id().to_string().bright_cyan());
            println!("Device ID: {}", info.product_id().to_string().bright_cyan());
            println!("Vendor name: {}", info.manufacturer_string().unwrap().bright_cyan());
            println!("Device name: {}", info.product_string().unwrap().bright_cyan());
        }
    }
}

fn read_args() -> Args {
    let args: Vec<String> = args().collect();
    let mut mode = "temp".to_string();
    let mut pid = 0;
    let mut fahrenheit = false;
    let mut alarm = false;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "-m" | "--mode" => {
                if i + 1 < args.len() {
                    mode = args[i + 1].clone();
                    if ["temp", "usage", "power", "auto"].contains(&mode.as_str()) {
                        i += 1;
                    } else {
                        error!("Invalid mode");
                        exit(1);
                    }
                } else {
                    error!("--mode requires a value");
                    exit(1);
                }
            }
            "--pid" => {
                if i + 1 < args.len() {
                    match args[i + 1].parse::<u16>() {
                        Ok(id) => {
                            if id > 0 {
                                pid = id;
                                i += 1;
                            } else {
                                error!("Invalid PID");
                                exit(1);
                            }
                        }
                        Err(_) => {
                            error!("Invalid PID");
                            exit(1);
                        }
                    }
                } else {
                    error!("--pid requires a value");
                    exit(1);
                }
            }
            "-f" | "--fahrenheit" => {
                fahrenheit = true;
            }
            "-a" | "--alarm" => {
                alarm = true;
            }
            "-l" | "--list" => {
                println!("Device list [{} | {}]", "PID".bright_green().bold(), "Name".bright_green());
                println!("-----");
                let api = HidApi::new().unwrap_or_else(|err| {
                    error!(err);
                    exit(1);
                });
                let mut products = 0;
                for device in api.device_list() {
                    if device.vendor_id() == VENDOR {
                        products += 1;
                        println!(
                            "{} | {}",
                            device.product_id().to_string().bright_green().bold(),
                            device.product_string().unwrap().bright_green()
                        );
                        break;
                    }
                }
                if products == 0 {
                    error!("No DeepCool device was found");
                    exit(1);
                }
                exit(0);
            }
            "-h" | "--help" => {
                println!("{} [OPTIONS]", "Usage: deepcool-digital-linux".bold());
                println!("\n{}", "Options:".bold());
                println!("  {}, {} <MODE>  Change the display mode between \"temp, usage, power, auto\" [default: temp]", "-m".bold(), "--mode".bold());
                println!("      {} <ID>     Specify the Product ID if you use mutiple devices", "--pid".bold());
                println!("  {}, {}   Change the temperature unit to °F", "-f".bold(), "--fahrenheit".bold());
                println!("  {}, {}        Enable the alarm", "-a".bold(), "--alarm".bold());
                println!("\n{}", "Commands:".bold());
                println!("  {}, {}         Print Product ID of the connected devices", "-l".bold(), "--list".bold());
                println!("  {}, {}         Print help", "-h".bold(), "--help".bold());
                println!("  {}, {}      Print version", "-v".bold(), "--version".bold());
                exit(0);
            }
            "-v" | "--version" => {
                println!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
                exit(0);
            }
            arg if arg.starts_with('-') && arg.len() > 1 => {
                for c in arg.chars().skip(1) {
                    match c {
                        'm' => {
                            if i + 1 < args.len() && args[i].ends_with('m') {
                                if ["temp", "usage", "power", "auto"].contains(&args[i + 1].as_str()) {
                                    mode = args[i + 1].clone();
                                    i += 1;
                                } else {
                                    error!("Invalid mode");
                                    exit(1);
                                }
                            } else {
                                error!("--mode requires a value");
                                exit(1);
                            }
                        }
                        'f' => fahrenheit = true,
                        'a' => alarm = true,
                        _ => {
                            if arg.starts_with("--") {
                                error!(format!("Invalid option {arg}"));
                            } else {
                                error!(format!("Invalid option -{c}"));
                            }
                            exit(1);
                        }
                    }
                }
            }
            _ => {
                error!(format!("Invalid option {}", args[i]));
                exit(1);
            }
        }
        i += 1;
    }

    Args {
        mode,
        pid,
        fahrenheit,
        alarm,
    }
}

fn display_configuration_info(mode: &str, temp_unit: TemperatureUnit, alarm: Alarm, polling_rate: u64) {
    println!("DISP. MODE: {}", mode.bright_cyan());
    println!("TEMP. UNIT: {}", temp_unit.symbol().bright_cyan());
    match alarm.state {
        AlarmState::Auto => println!(
            "ALARM:      {} | {}",
            "auto".bright_green(),
            (alarm.temp_limit.to_string() + temp_unit.symbol()).bright_cyan()
        ),
        AlarmState::On => println!(
            "ALARM:      {} | {}",
            "on".bright_green(),
            (alarm.temp_limit.to_string() + temp_unit.symbol()).bright_cyan()
        ),
        AlarmState::Off => println!("ALARM:      {}", "off".bright_red()),
        AlarmState::NotSupported => println!("ALARM:      {}", "not supported".bright_black().italic()),
    }
    println!("-----");
    println!("Update interval: {}", format!("{}ms", polling_rate).bright_cyan());
    println!("\nPress {} to terminate", "Ctrl+C".bold());
}
