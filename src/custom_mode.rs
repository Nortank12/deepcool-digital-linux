pub struct CustomModeItem {
    pub mode: String,
    pub tick_length: u8,
}

impl CustomModeItem {
    pub fn parse(input: &str) -> Self {
        let mut parts = input.split(",");
        let Some(mode) = parts.next().map(&str::to_string) else { panic!("Need at least the mode parameter") };
        let tick_length = match parts.next() {
            None => 8u8,
            Some(val) => val.parse().unwrap(),
        };
        Self {
            mode: mode.to_string(),
            tick_length,
        }
    }

    pub fn parse_modes(input: &str) -> Vec<Self> {
        input.split(":").skip(1).map(CustomModeItem::parse).collect()
    }
}