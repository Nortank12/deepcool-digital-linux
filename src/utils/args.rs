use crate::{assign_if_some, devices::Mode, error,  monitor::gpu::pci::{get_gpu_list, Vendor}, CH510_PRODUCT_ID, CH510_VENDOR_ID, DEFAULT_VENDOR_ID};
use colored::*;
use hidapi::HidApi;
use std::{collections::HashMap, process::exit, time::Duration};
use clap::{Parser};

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Cli {

    ///Change the display mode of your device
    #[arg(short, long)]
    mode: Option<String>,

    ///Change the secondary display mode of your device (if supported)
    #[arg(short, long)]
    secondary: Option<String>,

    ///Specify the Product ID if multiple devices are connected
    #[arg( long)]
    pid: Option<u16>,

    ///Specify the nth GPU of a specific vendor to monitor (use ID 0 for integrated GPU)
    #[arg( long)]
    gpuid: Option<String>,

    ///Change the update interval of the display [default: 1000]
    #[arg(short, long)]
    update: Option<u64>,

    ///Rotate the display (LP Series only)
    #[arg(short, long)]
    rotate: Option<u16>,

    ///Change the temperature unit to °F
    #[arg(short, long)]
    fahrenheit: bool,

    ///Enable the alarm
    #[arg(short, long)]
    alarm: bool,

    ///Print Product ID of the connected devices
    #[arg(short, long)]
    list: bool,

    ///Print all available GPUs
    #[arg(short, long)]
    gpulist: bool,
}

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
    fn gpu_list() {
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

    fn list() {
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
	


    pub fn read() -> Self {
	//default value
        let mut mode = Mode::Default;
        let mut secondary = Mode::Default;
        let mut pid = 0;
        let mut gpuid = None;
        let mut update = Duration::from_millis(1000);
        let mut rotate = 0;

	let cli: Cli = Cli::parse();

	if cli.mode.is_some() {
	    let mut cli_mode = String::from("");
	    assign_if_some!(cli_mode, cli.mode);
	    match Mode::get(&cli_mode) {
		Some(value) => {mode = value}, 
		None => {
		    error!("Invalid display mode");
		    exit(1);
		}
	    }
	}

	if cli.secondary.is_some() {
	    let mut cli_secondary = String::from("");
	    assign_if_some!(cli_secondary, cli.secondary);
	    match Mode::get(&cli_secondary) {
		Some(value) => {secondary = value},
		None => {
		    error!("Invalid secondary display mode");
                    exit(1);
		}
	    }
	}

	assign_if_some!(pid, cli.pid);

	if cli.gpuid.is_some() {
	    let mut cli_gpuid = String::from("");
	    assign_if_some!(cli_gpuid, cli.gpuid);
	    let mut gpuid_str = cli_gpuid.split(':');
            let vendor = Vendor::get(gpuid_str.next().unwrap_or(""));
            let id = gpuid_str.next().unwrap_or("").parse::<u8>().ok();
            match (vendor, id) {
		(Some(vendor), Some(id)) => {
                    gpuid = Some((vendor, id));
		}
		_ => {
                    error!("Invalid GPUID");
                    exit(1);
		}
            }
	}
	
	match cli.update {
	    Some(value) => {
		if value >= 100 && value <= 2000 {
		    update = Duration::from_millis(value);
		} else {
		    error!("Update interval must be between 100 and 2000");
                    exit(1);
		}
	    },
	    None => {
	    }
	}

	let fahrenheit = cli.fahrenheit;
	let alarm = cli.alarm;

	assign_if_some!(rotate, cli.rotate);
	if ![0, 90, 180, 270].contains(&rotate) {
	     error!("Invalid rotation value");
            exit(1);
	}

	if cli.list {
	    Self::list();
	}

	if cli.gpulist {
	    Self::gpu_list();
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
