mod devices;
mod monitor;

use clap::Parser;
use colored::*;
use hidapi::HidApi;
use std::process::exit;

const VENDOR: u16 = 0x3633;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Change the display mode between "temp, usage, auto"
    #[arg(short, long, default_value_t = String::from("temp"))]
    mode: String,

    /// Change temperature unit to Fahrenheit
    #[arg(short, long)]
    fahrenheit: bool,

    /// Enable the alarm (85˚C | 185˚F)
    #[arg(short, long)]
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
    let args = Args::parse();
    if !["temp", "usage", "auto"].contains(&args.mode.as_str()) {
        error!("Invalid mode");
        exit(1);
    }

    // Find device
    let api = HidApi::new().unwrap_or_else(|err| {
        error!(err);
        exit(1);
    });
    let mut product_id = 0;
    for device in api.device_list() {
        if device.vendor_id() == VENDOR {
            product_id = device.product_id();
            println!("Device found: {}", device.product_string().unwrap().bright_green());
            println!("-----");
            break;
        }
    }
    if product_id == 0 {
        error!("No DeepCool device was found");
        exit(1);
    }

    // Connect to device and send datastream
    match product_id {
        // AK Series
        1..=4 => {
            // Write info
            println!("DISP. MODE: {}", args.mode.bright_cyan());
            if args.mode != "usage" {
                println!("TEMP. UNIT: {}", if args.fahrenheit { "˚F".bright_cyan() } else { "˚C".bright_cyan() });
            }
            println!("ALARM:      {}", if args.alarm { "on".bright_green() } else { "off".bright_red() });
            println!("-----");
            println!("Update interval: {}", "750ms".bright_cyan());
            println!("\nPress {} to terminate", "Ctrl+C".bold());

            // Display loop
            let ak_device = devices::ak_series::Display::new(product_id, args.fahrenheit, args.alarm);
            ak_device.run(&api, &args.mode);
        }
        // AG Series
        8 => {
            // Write info
            println!("DISP. MODE: {}", args.mode.bright_cyan());
            if args.mode != "usage" {
                println!("TEMP. UNIT: {} {}", "˚C".bright_cyan(), "(˚F not supported)".bright_black().italic());
            }
            println!("ALARM:      {}", if args.alarm { "on".bright_green() } else { "off".bright_red() });
            println!("-----");
            println!("Update interval: {}", "750ms".bright_cyan());
            println!("\nPress {} to terminate", "Ctrl+C".bold());
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
            println!("DISP. MODE: {}", "not supported".bright_black().italic());
            println!("TEMP. UNIT: {}", if args.fahrenheit { "˚F".bright_cyan() } else { "˚C".bright_cyan() });
            println!("ALARM:      {}", "built-in".bright_cyan());
            println!("-----");
            println!("Update interval: {}", "1s".bright_cyan());
            println!("\nPress {} to terminate", "Ctrl+C".bold());
            if args.mode != "temp" {
                warning!("Display mode cannot be changed, value will be ignored");
            }
            if args.alarm {
                warning!("The alarm is handled internally, value will be ignored");
            }

            // Display loop
            let ld_device = devices::ld_series::Display::new(product_id, args.fahrenheit);
            ld_device.run(&api);
        }
        // CH Series & MORPHEUS
        5 | 7 | 21 => {
            // Write info
            println!("DISP. MODE: {}", args.mode.bright_cyan());
            if args.mode != "usage" {
                println!("TEMP. UNIT: {}", if args.fahrenheit { "˚F".bright_cyan() } else { "˚C".bright_cyan() });
            }
            println!("ALARM:      {}", "not supported".bright_black().italic());
            println!("-----");
            println!("Update interval: {}", "750ms".bright_cyan());
            println!("\nPress {} to terminate", "Ctrl+C".bold());
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
