pub mod args;
pub mod status;

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

#[macro_export]
macro_rules! assign_if_some {
    ($variable:expr, $option:expr) => {
        match $option {
            Some(value) => {
                $variable = value;
            }
            None => {} // Do nothing
        }
    };
}
