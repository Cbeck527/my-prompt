#[derive(Debug, Clone, PartialEq)]
pub enum Color {
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
    BrightBlack,
    BrightRed,
    BrightGreen,
    BrightYellow,
    BrightBlue,
    BrightMagenta,
    BrightCyan,
    BrightWhite,
}

impl Color {
    pub(crate) fn push_ansi_code(&self, buf: &mut String) {
        match self {
            Color::Black => buf.push_str("\x1b[30m"),
            Color::Red => buf.push_str("\x1b[31m"),
            Color::Green => buf.push_str("\x1b[32m"),
            Color::Yellow => buf.push_str("\x1b[33m"),
            Color::Blue => buf.push_str("\x1b[34m"),
            Color::Magenta => buf.push_str("\x1b[35m"),
            Color::Cyan => buf.push_str("\x1b[36m"),
            Color::White => buf.push_str("\x1b[37m"),
            Color::BrightBlack => buf.push_str("\x1b[90m"),
            Color::BrightRed => buf.push_str("\x1b[91m"),
            Color::BrightGreen => buf.push_str("\x1b[92m"),
            Color::BrightYellow => buf.push_str("\x1b[93m"),
            Color::BrightBlue => buf.push_str("\x1b[94m"),
            Color::BrightMagenta => buf.push_str("\x1b[95m"),
            Color::BrightCyan => buf.push_str("\x1b[96m"),
            Color::BrightWhite => buf.push_str("\x1b[97m"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AnsiStyle {
    pub color: Color,
    pub bold: bool,
}

impl AnsiStyle {
    pub const RESET: &'static str = "\x1b[0m";

    pub fn new(color: Color, bold: bool) -> Self {
        Self { color, bold }
    }

    pub fn start_codes(&self) -> String {
        let mut buf = String::new();
        self.color.push_ansi_code(&mut buf);
        if self.bold {
            buf.push_str("\x1b[1m");
        }
        buf
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normal_colors() {
        let red = AnsiStyle::new(Color::Red, false);
        assert_eq!(red.start_codes(), "\x1b[31m");

        let blue = AnsiStyle::new(Color::Blue, false);
        assert_eq!(blue.start_codes(), "\x1b[34m");
    }

    #[test]
    fn test_bright_colors() {
        let bright_red = AnsiStyle::new(Color::BrightRed, false);
        assert_eq!(bright_red.start_codes(), "\x1b[91m");

        let bright_blue = AnsiStyle::new(Color::BrightBlue, false);
        assert_eq!(bright_blue.start_codes(), "\x1b[94m");
    }

    #[test]
    fn test_bold_modifier() {
        let bold_red = AnsiStyle::new(Color::Red, true);
        assert_eq!(bold_red.start_codes(), "\x1b[31m\x1b[1m");
    }

    #[test]
    fn test_reset_constant() {
        assert_eq!(AnsiStyle::RESET, "\x1b[0m");
    }
}
