pub struct RgbColor {
    red: u8,
    green: u8,
    blue: u8,
}

impl RgbColor {
    pub fn new(red: u8, green: u8, blue: u8) -> RgbColor {
        RgbColor { red, green, blue }
    }

    pub fn rgb(&self) -> i32 {
        ((self.red as i32) << 16) | ((self.green as i32) << 8) | (self.blue as i32)
    }
}
