mod device;
use device::{startup_message, status_message};

use std::process::exit;
use libc::geteuid;
use hidapi::HidApi;
use clap::Parser;


const VENDOR: u16 = 0x3633;
const PRODUCT_NAMES: [&str; 4] = ["AK400", "AK620", "AK500", "AK500S"];

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Change the display mode between "temp, usage, auto"
    #[arg(short, long, default_value_t = String::from("temp"))]
    mode: String,

    /// Change temperature unit to Fahrenheit
    #[arg(short, long)]
    fahrenheit: bool,

    /// Enable the alarm (80˚C or 176˚F)
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
            println!("Device found: {}-DIGITAL", PRODUCT_NAMES[(product_id - 1) as usize]);
            break;
        }
    }
    if product_id == 0 {
        println!("Device not found!");
        exit(1);
    }
    
    // Connect
    let device = api.open(VENDOR, product_id).expect("Failed to open HID device");

    // Start
    device.write(&startup_message()).expect("Failed to write data");

    // Write info
    println!("-----");
    println!("DISP. MODE: {}", args.mode);
    if args.mode != "usage" {
        println!("TEMP. UNIT: {}", if args.fahrenheit {"˚F"} else {"˚C"});
    }
    println!("ALARM:      {}", if args.alarm {"on"} else {"off"});
    println!("-----");
    println!("Update interval: 750ms");
    println!("\nPress Ctrl + C to terminate");

    // Display loop
    if args.mode == "auto" {
        loop {
            for _ in 0..8 {
                device.write(&status_message("temp", args.fahrenheit, args.alarm)).expect("Failed to write data");
            }
            for _ in 0..8 {
                device.write(&status_message("usage", args.fahrenheit, args.alarm)).expect("Failed to write data");
            }
        }
    } else {
        loop {
            device.write(&status_message(&args.mode, args.fahrenheit, args.alarm)).expect("Failed to write data");
        }
    }
}
