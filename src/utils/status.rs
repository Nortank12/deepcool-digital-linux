use crate::Mode;
use colored::*;
use std::time::Duration;

pub enum TemperatureUnit {
    Celsius,
    Fahrenheit,
}

impl TemperatureUnit {
    const fn symbol(&self) -> &'static str {
        match self {
            TemperatureUnit::Celsius => "°C",
            TemperatureUnit::Fahrenheit => "°F",
        }
    }
}

pub enum AlarmState {
    Auto,
    On,
    Off,
    NotSupported,
}

pub struct Alarm {
    pub state: AlarmState,
    pub temp_limit: u8,
    pub temp_warning: u8,
}

pub fn print_device_status(
    mode: &Mode,
    secondary: Option<&Mode>,
    rotation: Option<u16>,
    temp_unit: TemperatureUnit,
    alarm: Alarm,
    update: Duration,
) {
    println!("-----");
    match secondary {
        Some(s) => println!("DISP. MODE: {} | {}", mode.symbol().bright_cyan(), s.symbol().bright_cyan()),
        None => println!("DISP. MODE: {}", mode.symbol().bright_cyan()),
    }
    if let Some(r) = rotation {
        if r > 0 {
            println!("ROTATION:   {}", format!("{r}°").bright_cyan());
        } else {
            println!("ROTATION:   {}", "none".bright_black());
        }
    }
    println!("TEMP. UNIT: {}", temp_unit.symbol().bright_cyan());
    match alarm.state {
        AlarmState::Auto => {
            if alarm.temp_warning > 0 {
                println!(
                    "ALARM:      {} | {} [warning: {}]",
                    "auto".bright_green(),
                    (alarm.temp_limit.to_string() + temp_unit.symbol()).bright_cyan(),
                    (alarm.temp_warning.to_string() + temp_unit.symbol()).bright_cyan()
                );
            } else {
                println!(
                    "ALARM:      {} | {}",
                    "auto".bright_green(),
                    (alarm.temp_limit.to_string() + temp_unit.symbol()).bright_cyan()
                );
            }
        }
        AlarmState::On => println!(
            "ALARM:      {} | {}",
            "on".bright_green(),
            (alarm.temp_limit.to_string() + temp_unit.symbol()).bright_cyan()
        ),
        AlarmState::Off => println!("ALARM:      {}", "off".bright_red()),
        AlarmState::NotSupported => println!("ALARM:      {}", "not supported".bright_black().italic()),
    }
    println!("-----");
    println!("Update interval: {}", format!("{:?}", update).bright_cyan());
    println!("\nPress {} to terminate", "Ctrl+C".bold());
}
