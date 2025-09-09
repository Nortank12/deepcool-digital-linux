use crate::{devices::Mode, error, monitor::gpu::pci::{get_gpu_list, Vendor}, CH510_PRODUCT_ID, CH510_VENDOR_ID, DEFAULT_VENDOR_ID};
use colored::*;
use hidapi::HidApi;
use std::{collections::HashMap, env::args, process::exit, time::Duration};

pub struct Args {
    pub mode: Mode,
    pub secondary: Mode,
    pub pid: u16,
    pub gpuid: Option<(Vendor, u8)>,
    pub update: Duration,
    pub fahrenheit: bool,
    pub alarm: bool,
    pub rotate: u16,
}

impl Args {
    pub fn read() -> Self {
        let args: Vec<String> = args().collect();
        let mut mode = Mode::Default;
        let mut secondary = Mode::Default;
        let mut pid = 0;
        let mut gpuid = None;
        let mut update = Duration::from_millis(1000);
        let mut fahrenheit = false;
        let mut alarm = false;
        let mut rotate = 0;

        let mut i = 1;
        while i < args.len() {
            match args[i].as_str() {
                "-m" | "--mode" => {
                    if i + 1 < args.len() {
                        mode = match Mode::get(&args[i + 1]) {
                            Some(mode) => mode,
                            None => {
                                error!("Invalid display mode");
                                exit(1);
                            }
                        };
                        i += 1;
                    } else {
                        error!("--mode requires a value");
                        exit(1);
                    }
                }
                "-s" | "--secondary" => {
                    if i + 1 < args.len() {
                        secondary = match Mode::get(&args[i + 1]) {
                            Some(mode) => mode,
                            None => {
                                error!("Invalid secondary display mode");
                                exit(1);
                            }
                        };
                        i += 1;
                    } else {
                        error!("--secondary requires a value");
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
                "--gpuid" => {
                    if i + 1 < args.len() {
                        let mut gpuid_str = args[i + 1].split(':');
                        let vendor = Vendor::get(gpuid_str.next().unwrap_or(""));
                        let id = gpuid_str.next().unwrap_or("").parse::<u8>().ok();
                        match (vendor, id) {
                            (Some(vendor), Some(id)) => {
                                gpuid = Some((vendor, id));
                                i += 1;
                            }
                            _ => {
                                error!("Invalid GPUID");
                                exit(1);
                            }
                        }
                    } else {
                        error!("--gpuid requires a value");
                        exit(1);
                    }
                }
                "-u" | "--update" => {
                    if i + 1 < args.len() {
                        match args[i + 1].parse::<u64>() {
                            Ok(val) => {
                                if val >= 100 && val <= 2000 {
                                    update = Duration::from_millis(val);
                                    i += 1;
                                } else {
                                    error!("Update interval must be between 100 and 2000");
                                    exit(1);
                                }
                            }
                            Err(_) => {
                                error!("Invalid update interval");
                                exit(1);
                            }
                        }
                    } else {
                        error!("--update requires a value");
                        exit(1);
                    }
                }
                "-f" | "--fahrenheit" => {
                    fahrenheit = true;
                }
                "-a" | "--alarm" => {
                    alarm = true;
                }
                "-r" | "--rotate" => {
                    if i + 1 < args.len() {
                        match args[i + 1].parse::<u16>() {
                            Ok(val) => {
                                if [90, 180, 270].contains(&val) {
                                    rotate = val;
                                    i += 1;
                                } else {
                                    error!("Rotation value must be one of 90, 180, or 270");
                                    exit(1);
                                }
                            }
                            Err(_) => {
                                error!("Invalid rotation value");
                                exit(1);
                            }
                        }
                    } else {
                        error!("--rotate requires a value");
                        exit(1);
                    }
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
                        if device.vendor_id() == DEFAULT_VENDOR_ID {
                            products += 1;
                            println!(
                                "{} | {}",
                                device.product_id().to_string().bright_green().bold(),
                                device.product_string().unwrap().bright_green()
                            );
                        } else if device.vendor_id() == CH510_VENDOR_ID && device.product_id() == CH510_PRODUCT_ID {
                            products += 1;
                            println!(
                                "{} | {}",
                                device.product_id().to_string().bright_green().bold(),
                                "CH510-MESH-DIGITAL".bright_green()
                            );
                        }
                    }
                    if products == 0 {
                        println!("{}", "No DeepCool device was found".bright_black().italic());
                    }
                    exit(0);
                }
                "-g" | "--gpulist" => {
                    println!("GPU list [{} | {} {}]", "ID".bright_green().bold(), "Name".bright_green(), "(PCI Address)".bright_black());
                    println!("-----");
                    let gpus = get_gpu_list();
                    let mut gpu_ids = HashMap::new();
                    for gpu in &gpus {
                        let nth = gpu_ids.entry(&gpu.vendor).or_insert(0 as u8);
                        *nth += 1;
                        println!(
                            "{} | {} {}",
                            format!("{}:{}", gpu.vendor.name().to_lowercase(), *nth).bright_green().bold(),
                            gpu.name.bright_green(),
                            format!("({})", gpu.address).bright_black(),
                        );
                    }
                    if gpus.len() == 0 {
                        println!("{}", "No GPUs were found".bright_black().italic())
                    }
                    exit(0);
                }
                "-h" | "--help" => {
                    println!("{} [OPTIONS]", "Usage: deepcool-digital-linux".bold());
                    println!("\n{}", "Options:".bold());
                    println!("  {}, {} <MODE>       Change the display mode of your device", "-m".bold(), "--mode".bold());
                    println!("  {}, {} <MODE>  Change the secondary display mode of your device (if supported)", "-s".bold(), "--secondary".bold());
                    println!("      {} <ID>          Specify the Product ID if multiple devices are connected", "--pid".bold());
                    println!("      {} <VENDOR:ID> Specify the nth GPU of a specific vendor to monitor (use ID 0 for integrated GPU)", "--gpuid".bold());
                    println!("\n  {}, {} <MILLISEC> Change the update interval of the display [default: 1000]", "-u".bold(), "--update".bold());
                    println!("  {}, {}        Change the temperature unit to °F", "-f".bold(), "--fahrenheit".bold());
                    println!("  {}, {}             Enable the alarm", "-a".bold(), "--alarm".bold());
                    println!("  {}, {} <DEGREE>   Rotate the display (LP Series only)", "-r".bold(), "--rotate".bold());
                    println!("\n{}", "Commands:".bold());
                    println!("  {}, {}         Print Product ID of the connected devices", "-l".bold(), "--list".bold());
                    println!("  {}, {}      Print all available GPUs", "-g".bold(), "--gpulist".bold());
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
                                    mode = match Mode::get(&args[i + 1]) {
                                        Some(mode) => mode,
                                        None => {
                                            error!("Invalid display mode");
                                            exit(1);
                                        }
                                    };
                                    i += 1;
                                } else {
                                    error!("--mode requires a value");
                                    exit(1);
                                }
                            }
                            's' => {
                                if i + 1 < args.len() && args[i].ends_with('s') {
                                    secondary = match Mode::get(&args[i + 1]) {
                                        Some(mode) => mode,
                                        None => {
                                            error!("Invalid secondary display mode");
                                            exit(1);
                                        }
                                    };
                                    i += 1;
                                } else {
                                    error!("--secondary requires a value");
                                    exit(1);
                                }
                            }
                            'u' => {
                                if i + 1 < args.len() && args[i].ends_with('u') {
                                    match args[i + 1].parse::<u64>() {
                                        Ok(val) => {
                                            if val >= 100 && val <= 2000 {
                                                update = Duration::from_millis(val);
                                                i += 1;
                                            } else {
                                                error!("Update interval must be between 100 and 2000");
                                                exit(1);
                                            }
                                        }
                                        Err(_) => {
                                            error!("Invalid update interval");
                                            exit(1);
                                        }
                                    }
                                } else {
                                    error!("--update requires a value");
                                    exit(1);
                                }
                            }
                            'r' => {
                                if i + 1 < args.len() && args[i].ends_with('r') {
                                    match args[i + 1].parse::<u16>() {
                                        Ok(val) => {
                                            if [90, 180, 270].contains(&val) {
                                                rotate = val;
                                                i += 1;
                                            } else {
                                                error!("Rotation value must be one of 90, 180, or 270");
                                                exit(1);
                                            }
                                        }
                                        Err(_) => {
                                            error!("Invalid rotation value");
                                            exit(1);
                                        }
                                    }
                                } else {
                                    error!("--update requires a value");
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
            secondary,
            pid,
            gpuid,
            update,
            fahrenheit,
            alarm,
            rotate,
        }
    }
}
