#[derive(Debug, Eq, PartialEq)]
pub(crate) enum Color {
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
}

impl Color {
    fn ansi_code(&self) -> &'static str {
        match self {
            Self::Red => "\x1b[31m",
            Self::Green => "\x1b[32m",
            Self::Yellow => "\x1b[33m",
            Self::Blue => "\x1b[34m",
            Self::Magenta => "\x1b[35m",
            Self::Cyan => "\x1b[36m",
            Self::White => "\x1b[37m",
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct AnsiStyle {
    color: Color,
    bold: bool,
}

impl AnsiStyle {
    pub(crate) const RESET: &'static str = "\x1b[0m";

    #[must_use]
    pub(crate) fn new(color: Color, bold: bool) -> Self {
        Self { color, bold }
    }

    #[must_use]
    pub(crate) fn start_codes(&self) -> String {
        let mut buffer = self.color.ansi_code().to_owned();
        if self.bold {
            buffer.push_str("\x1b[1m");
        }
        buffer
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normal_color_uses_ansi_foreground_code() {
        assert_eq!(AnsiStyle::new(Color::Red, false).start_codes(), "\x1b[31m");
        assert_eq!(AnsiStyle::new(Color::Blue, false).start_codes(), "\x1b[34m");
    }

    #[test]
    fn bold_appends_the_ansi_bold_modifier() {
        assert_eq!(
            AnsiStyle::new(Color::Red, true).start_codes(),
            "\x1b[31m\x1b[1m"
        );
    }

    #[test]
    fn reset_constant_clears_ansi_styling() {
        assert_eq!(AnsiStyle::RESET, "\x1b[0m");
    }
}
