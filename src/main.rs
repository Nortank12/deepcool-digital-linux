mod devices;
mod monitor;
mod utils;

use colored::*;
use devices::*;
use hidapi::HidApi;
use monitor::{cpu, gpu};
use std::process::exit;
use utils::{args::Args, status::*};

/// Common warning checks for command arguments.
mod common_warnings {
    use crate::{devices::Mode, utils::args::Args, warning};

    pub fn mode_change(args: &Args) {
        if args.mode != Mode::Default {
            warning!("Display mode cannot be changed, value will be ignored");
        }
    }

    pub fn secondary_mode(args: &Args) {
        if args.secondary != Mode::Default {
            warning!("Secondary display mode is not supported, value will be ignored");
        }
    }

    pub fn fahrenheit(args: &Args) {
        if args.fahrenheit {
            warning!("Displaying ˚F is not supported, value will be ignored");
        }
    }

    pub fn alarm(args: &Args) {
        if args.alarm {
            warning!("Alarm is not supported, value will be ignored");
        }
    }

    pub fn alarm_hardcoded(args: &Args) {
        if args.alarm {
            warning!("The alarm is hard-coded in your device, value will be ignored");
        }
    }

    pub fn rotate(args: &Args) {
        if args.rotate > 0 {
            warning!("Display rotation is not supported, value will be ignored");
        }
    }
}

fn main() {
    // Read args
    let args = Args::read();
    println!("--- Deepcool Digital Linux ---");

    // Find dedicated or integrated GPU
    let pci_device = {
        // Get list of GPUs
        let gpus = gpu::pci::get_gpu_list();
        if gpus.is_empty() {
            None
        } else {
            match args.gpuid {
                // Look for the nth GPU of the specified vendor
                Some((vendor, id)) => {
                    let mut nth = 1;
                    let mut device = None;
                    if id > 0 {
                        // Match dedicated GPU
                        for gpu in gpus.iter() {
                            if gpu.vendor == vendor && gpu.bus > 0 {
                                if nth == id { device = Some(gpu.clone()); break; }
                                else { nth += 1; }
                            }
                        }
                    } else {
                        // Match integrated (first) GPU
                        let first_gpu = gpus.first().unwrap();
                        if first_gpu.vendor == vendor && first_gpu.bus == 0 { device = Some(first_gpu.clone()) }
                    }
                    device.or_else(|| { error!("No GPU was found with the specified GPUID"); exit(1) })
                },
                // Find the first dedicated GPU if present; otherwise, use the iGPU
                None => gpus.iter().find(|gpu| gpu.bus > 0).cloned().or_else(|| gpus.first().cloned()),
            }
        }
    };

    // Display CPU and GPU name
    match cpu::get_name() {
        Some(cpu_name) => println!("CPU MON.: {}", cpu_name.bright_green()),
        None => println!("CPU MON.: {}", "Unknown CPU".bright_green()),
    }
    match &pci_device {
        Some(gpu) => println!("GPU MON.: {}", gpu.name.bright_green()),
        None => println!("GPU MON.: {}", "none".bright_black()),
    };
    println!("-----");

    // Find DeepCool device
    let api = HidApi::new().unwrap_or_else(|err| {
        error!(err);
        exit(1);
    });
    let mut product_id = 0;
    for device in api.device_list() {
        if device.vendor_id() == DEFAULT_VENDOR_ID {
            if args.pid == 0 || device.product_id() == args.pid {
                product_id = device.product_id();
                println!("Device found: {}", device.product_string().unwrap().bright_green());
                break;
            }
        } else if device.vendor_id() == CH510_VENDOR_ID && device.product_id() == CH510_PRODUCT_ID {
            if args.pid == 0 || device.product_id() == args.pid {
                product_id = device.product_id();
                println!("Device found: {}", "CH510-MESH-DIGITAL".bright_green());
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

    // Initialize CPU & GPU monitoring
    let cpu = cpu::Cpu::new();
    let gpu = gpu::Gpu::new(pci_device);

    // Connect to device and send datastream
    match product_id {
        // AK Series
        1..=4 => {
            println!("Supported modes: {} [default: {}]", "auto cpu_temp cpu_usage".bold(), ak_series::DEFAULT_MODE.symbol());
            // Connect to device
            let ak_device = ak_series::Display::new(cpu, &args.mode, args.update, args.fahrenheit, args.alarm);
            // Print current configuration & warnings
            print_device_status(
                &ak_device.mode,
                None,
                None,
                if args.fahrenheit { TemperatureUnit::Fahrenheit } else { TemperatureUnit::Celsius },
                Alarm {
                    state: if args.alarm { AlarmState::On } else { AlarmState::Off },
                    temp_limit: if args.fahrenheit {
                        ak_series::TEMP_LIMIT_F
                    } else {
                        ak_series::TEMP_LIMIT_C
                    },
                    temp_warning: 0,
                },
                args.update,
            );
            common_warnings::secondary_mode(&args);
            common_warnings::rotate(&args);
            // Display loop
            ak_device.run(&api, DEFAULT_VENDOR_ID, product_id);
        }
        // LS Series
        6 => {
            println!("Supported modes: {} [default: {}]", "auto cpu_temp cpu_power".bold(), ls_series::DEFAULT_MODE.symbol());
            // Connect to device
            let ls_device = ls_series::Display::new(cpu, &args.mode, args.update, args.fahrenheit, args.alarm);
            // Print current configuration & warnings
            print_device_status(
                &ls_device.mode,
                None,
                None,
                if args.fahrenheit { TemperatureUnit::Fahrenheit } else { TemperatureUnit::Celsius },
                Alarm {
                    state: if args.alarm { AlarmState::On } else { AlarmState::Off },
                    temp_limit: if args.fahrenheit {
                        ls_series::TEMP_LIMIT_F
                    } else {
                        ls_series::TEMP_LIMIT_C
                    },
                    temp_warning: 0,
                },
                args.update,
            );
            common_warnings::secondary_mode(&args);
            common_warnings::rotate(&args);
            // Display loop
            ls_device.run(&api, DEFAULT_VENDOR_ID, product_id);
        }
        // AG Series
        8 => {
            println!("Supported modes: {} [default: {}]", "auto cpu_temp cpu_usage".bold(), ag_series::DEFAULT_MODE.symbol());
            // Connect to device
            let ag_device = ag_series::Display::new(cpu, &args.mode, args.update, args.alarm);
            // Print current configuration & warnings
            print_device_status(
                &ag_device.mode,
                None,
                None,
                TemperatureUnit::Celsius,
                Alarm {
                    state: if args.alarm { AlarmState::On } else { AlarmState::Off },
                    temp_limit: ag_series::TEMP_LIMIT_C,
                    temp_warning: 0,
                },
                args.update,
            );
            common_warnings::secondary_mode(&args);
            common_warnings::fahrenheit(&args);
            common_warnings::rotate(&args);
            // Display loop
            ag_device.run(&api, DEFAULT_VENDOR_ID, product_id);
        }
        // LD Series
        10 => {
            println!("Supported modes: {}", "auto".bold());
            // Connect to device
            let ld_device = ld_series::Display::new(cpu, args.update, args.fahrenheit);
            // Print current configuration & warnings
            print_device_status(
                &ld_series::DEFAULT_MODE,
                None,
                None,
                if args.fahrenheit { TemperatureUnit::Fahrenheit } else { TemperatureUnit::Celsius },
                Alarm {
                    state: AlarmState::Auto,
                    temp_limit: if args.fahrenheit {
                        ld_series::TEMP_LIMIT_F
                    } else {
                        ld_series::TEMP_LIMIT_C
                    },
                    temp_warning: 0,
                },
                args.update,
            );
            common_warnings::mode_change(&args);
            common_warnings::secondary_mode(&args);
            common_warnings::alarm_hardcoded(&args);
            common_warnings::rotate(&args);
            // Display loop
            ld_device.run(&api, DEFAULT_VENDOR_ID, product_id);
        }
        // LP Series
        12 => {
            println!(
                "Supported modes: {} [default: {}]",
                "cpu_usage cpu_temp cpu_power gpu_usage gpu_temp gpu_power".bold(),
                lp_series::DEFAULT_MODE.symbol()
            );
            println!(
                "Supported secondary: {}",
                "cpu_usage cpu_temp cpu_power gpu_usage gpu_temp gpu_power".bold()
            );
            // Connect to device
            let lp_device = lp_series::Display::new(cpu, gpu, &args.mode, &args.secondary, args.update, args.fahrenheit, args.rotate);
            // Print current configuration & warnings
            print_device_status(
                &lp_device.mode,
                lp_device.secondary.as_ref(),
                Some(args.rotate),
                if args.fahrenheit { TemperatureUnit::Fahrenheit } else { TemperatureUnit::Celsius },
                Alarm { state: AlarmState::NotSupported, temp_limit: 0, temp_warning: 0 },
                args.update,
            );
            common_warnings::alarm(&args);
            // Display loop
            lp_device.run(&api, DEFAULT_VENDOR_ID, product_id);
        }
        // LQ Series & ASSASSIN IV
        13 | 15 | 31 => {
            println!("Supported modes: {}", "auto".bold());
            // Connect to device
            let lq_device = devices::lq_series::Display::new(cpu, args.update, args.fahrenheit);
            // Print current configuration & warnings
            print_device_status(
                &lq_series::DEFAULT_MODE,
                None,
                None,
                if args.fahrenheit { TemperatureUnit::Fahrenheit } else { TemperatureUnit::Celsius },
                Alarm {
                    state: AlarmState::Auto,
                    temp_limit: if args.fahrenheit {
                        lq_series::TEMP_LIMIT_F
                    } else {
                        lq_series::TEMP_LIMIT_C
                    },
                    temp_warning: if args.fahrenheit {
                        lq_series::TEMP_WARNING_F
                    } else {
                        lq_series::TEMP_WARNING_C
                    },
                },
                args.update,
            );
            common_warnings::mode_change(&args);
            common_warnings::secondary_mode(&args);
            common_warnings::alarm_hardcoded(&args);
            common_warnings::rotate(&args);
            // Display loop
            lq_device.run(&api, DEFAULT_VENDOR_ID, product_id);
        }
        // AK400 PRO
        16 => {
            println!("Supported modes: {}", "auto".bold());
            // Connect to device
            let ak400_pro = devices::ak400_pro::Display::new(cpu, args.update, args.fahrenheit);
            // Print current configuration & warnings
            print_device_status(
                &ak400_pro::DEFAULT_MODE,
                None,
                None,
                if args.fahrenheit { TemperatureUnit::Fahrenheit } else { TemperatureUnit::Celsius },
                Alarm {
                    state: AlarmState::Auto,
                    temp_limit: if args.fahrenheit {
                        ak400_pro::TEMP_LIMIT_F
                    } else {
                        ak400_pro::TEMP_LIMIT_C
                    },
                    temp_warning: if args.fahrenheit {
                        ak400_pro::TEMP_WARNING_F
                    } else {
                        ak400_pro::TEMP_WARNING_C
                    },
                },
                args.update,
            );
            common_warnings::mode_change(&args);
            common_warnings::secondary_mode(&args);
            common_warnings::alarm_hardcoded(&args);
            common_warnings::rotate(&args);
            // Display loop
            ak400_pro.run(&api, DEFAULT_VENDOR_ID, product_id);
        }
        // AK500 / AK620 PRO
        17 | 18 => {
            println!("Supported modes: {}", "auto".bold());
            // Connect to device
            let ak620_pro = devices::ak620_pro::Display::new(cpu, args.update, args.fahrenheit);
            // Print current configuration & warnings
            print_device_status(
                &ak620_pro::DEFAULT_MODE,
                None,
                None,
                if args.fahrenheit { TemperatureUnit::Fahrenheit } else { TemperatureUnit::Celsius },
                Alarm {
                    state: AlarmState::Auto,
                    temp_limit: if args.fahrenheit {
                        ak620_pro::TEMP_LIMIT_F
                    } else {
                        ak620_pro::TEMP_LIMIT_C
                    },
                    temp_warning: if args.fahrenheit {
                        ak620_pro::TEMP_WARNING_F
                    } else {
                        ak620_pro::TEMP_WARNING_C
                    },
                },
                args.update,
            );
            common_warnings::mode_change(&args);
            common_warnings::secondary_mode(&args);
            common_warnings::alarm_hardcoded(&args);
            common_warnings::rotate(&args);
            // Display loop
            ak620_pro.run(&api, DEFAULT_VENDOR_ID, product_id);
        }
        // CH170 | CH270 | CH690
        19 | 22 | 27 => {
            println!(
                "Supported modes: {} {} {} {} [default: {}]",
                "auto cpu_freq".bold(),
                "cpu_fan".bright_black().strikethrough(),
                "gpu".bold(),
                "psu".bright_black().strikethrough(),
                ch_series_gen2::DEFAULT_MODE.symbol()
            );
            // Connect to device
            let ch_gen2_device = ch_series_gen2::Display::new(cpu, gpu, &args.mode, args.update, args.fahrenheit);
            // Print current configuration & warnings
            print_device_status(
                &ch_gen2_device.mode,
                None,
                None,
                if args.fahrenheit { TemperatureUnit::Fahrenheit } else { TemperatureUnit::Celsius },
                Alarm { state: AlarmState::NotSupported, temp_limit: 0, temp_warning: 0 },
                args.update,
            );
            common_warnings::secondary_mode(&args);
            common_warnings::alarm(&args);
            common_warnings::rotate(&args);
            // Display loop
            ch_gen2_device.run(&api, DEFAULT_VENDOR_ID, product_id);
        }
        // CH Series & MORPHEUS
        5 | 7 | 21 => {
            println!("Supported modes: {} [default: {}]", "auto cpu_temp cpu_usage".bold(), ch_series::DEFAULT_MODE.symbol());
            println!("Supported secondary: {}", "gpu_temp gpu_usage".bold());
            // Connect to device
            let ch_device = ch_series::Display::new(cpu, gpu, &args.mode, &args.secondary, args.update, args.fahrenheit);
            // Print current configuration & warnings
            print_device_status(
                &ch_device.mode,
                Some(&ch_device.secondary),
                None,
                if args.fahrenheit { TemperatureUnit::Fahrenheit } else { TemperatureUnit::Celsius },
                Alarm { state: AlarmState::NotSupported, temp_limit: 0, temp_warning: 0 },
                args.update,
            );
            common_warnings::alarm(&args);
            common_warnings::rotate(&args);
            // Display loop
            ch_device.run(&api, DEFAULT_VENDOR_ID, product_id);
        }
        // CH510
        CH510_PRODUCT_ID => {
            println!("Supported modes: {} [default: {}]", "cpu gpu".bold(), ch510::DEFAULT_MODE.symbol());
            // Connect to device
            let ch510 = ch510::Display::new(cpu, gpu, &args.mode, args.update, args.fahrenheit);
            // Print current configuration & warnings
            print_device_status(
                &ch510.mode,
                None,
                None,
                if args.fahrenheit { TemperatureUnit::Fahrenheit } else { TemperatureUnit::Celsius },
                Alarm { state: AlarmState::NotSupported, temp_limit: 0, temp_warning: 0 },
                args.update,
            );
            common_warnings::secondary_mode(&args);
            common_warnings::alarm(&args);
            common_warnings::rotate(&args);
            // Display loop
            ch510.run(&api, CH510_VENDOR_ID, product_id);
        }
        _ => {
            println!("Device not yet supported!");
            println!("\nPlease create an issue on GitHub providing your device name and the following information:");
            let device = api.open(DEFAULT_VENDOR_ID, product_id).unwrap_or_else(|_| device_error());
            let info = device.get_device_info().unwrap();
            println!("Vendor ID: {}", info.vendor_id().to_string().bright_cyan());
            println!("Device ID: {}", info.product_id().to_string().bright_cyan());
            println!("Vendor name: {}", info.manufacturer_string().unwrap().bright_cyan());
            println!("Device name: {}", info.product_string().unwrap().bright_cyan());
        }
    }
}
