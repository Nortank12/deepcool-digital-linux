mod devices;
mod monitor;

use clap::Parser;
use hidapi::HidApi;
use libc::geteuid;
use monitor::cpu::find_temp_sensor;
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

fn main() {
    // Check root
    unsafe {
        if geteuid() != 0 {
            println!("Try to run the program as root!");
            exit(1);
        }
    }

    // Read args
    let args = Args::parse();
    if !["temp", "usage", "auto"].contains(&args.mode.as_str()) {
        println!("Invalid mode!");
        exit(1);
    }

    // Find device
    let api = HidApi::new().expect("Failed to initialize HID API");
    let mut product_id = 0;
    for device in api.device_list() {
        if device.vendor_id() == VENDOR {
            product_id = device.product_id();
            println!("Device found: {}", device.product_string().unwrap());
            println!("-----");
            break;
        }
    }
    if product_id == 0 {
        println!("No DeepCool device found!");
        exit(1);
    }

    // Find CPU temp. sensor
    let cpu_hwmon_path = find_temp_sensor();

    // Connect to device and send datastream
    match product_id {
        1..=4 => {
            // Write info
            println!("DISP. MODE: {}", args.mode);
            if args.mode != "usage" {
                println!("TEMP. UNIT: {}", if args.fahrenheit { "˚F" } else { "˚C" });
            }
            println!("ALARM:      {}", if args.alarm { "on" } else { "off" });
            println!("-----");
            println!("Update interval: 750ms");
            println!("\nPress Ctrl + C to terminate");

            // Display loop
            let ak_device = devices::ak_series::Display::new(product_id, args.fahrenheit, args.alarm);
            ak_device.run(&api, &args.mode, &cpu_hwmon_path);
        }
        8 => {
            // Write info
            println!("DISP. MODE: {}", args.mode);
            if args.mode != "usage" {
                println!("TEMP. UNIT: ˚C (˚F not supported)");
            }
            println!("ALARM:      {}", if args.alarm { "on" } else { "off" });
            println!("-----");
            println!("Update interval: 750ms");
            println!("\nPress Ctrl + C to terminate");

            // Display loop
            let ag_device = devices::ag_series::Display::new(product_id, args.alarm);
            ag_device.run(&api, &args.mode, &cpu_hwmon_path);
        }
        10 => {
            // Write info
            println!("DISP. MODE: not supported");
            if args.mode != "usage" {
                println!("TEMP. UNIT: {}", if args.fahrenheit { "˚F" } else { "˚C" });
            }
            println!("ALARM:      built-in (85˚C | 185˚F)");
            println!("-----");
            println!("Update interval: 1 second");
            println!("\nPress Ctrl + C to terminate");

            // Display loop
            let ld_device = devices::ld_series::Display::new(product_id, args.fahrenheit);
            ld_device.run(&api, &cpu_hwmon_path);
        }
        _ => {
            println!("Device not yet supported!");
            println!("\nPlease create an issue on GitHub providing your device name and the following information:");
            let device = api.open(VENDOR, product_id).unwrap();
            let info = device.get_device_info().unwrap();
            println!("Vendor ID: {}", info.vendor_id());
            println!("Device ID: {}", info.product_id());
            println!("Vendor name: {}", info.manufacturer_string().unwrap());
            println!("Device name: {}", info.product_string().unwrap());
        }
    }
}
