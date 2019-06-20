use std::str::FromStr;
use serde_derive::Deserialize;

/// The 8 standard colors.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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
    pub fn to_fg_str(&self) -> &str {
        match *self {
            Color::Black => "30",
            Color::Red => "31",
            Color::Green => "32",
            Color::Yellow => "33",
            Color::Blue => "34",
            Color::Magenta => "35",
            Color::Cyan => "36",
            Color::White => "37",
            Color::BrightBlack => "90",
            Color::BrightRed => "91",
            Color::BrightGreen => "92",
            Color::BrightYellow => "93",
            Color::BrightBlue => "94",
            Color::BrightMagenta => "95",
            Color::BrightCyan => "96",
            Color::BrightWhite => "97",
        }
    }

    //     pub fn to_bg_str(&self) -> &str {
    //         match *self {
    //             Color::Black => "40",
    //             Color::Red => "41",
    //             Color::Green => "42",
    //             Color::Yellow => "43",
    //             Color::Blue => "44",
    //             Color::Magenta => "45",
    //             Color::Cyan => "46",
    //             Color::White => "47",
    //             Color::BrightBlack => "100",
    //             Color::BrightRed => "101",
    //             Color::BrightGreen => "102",
    //             Color::BrightYellow => "103",
    //             Color::BrightBlue => "104",
    //             Color::BrightMagenta => "105",
    //             Color::BrightCyan => "106",
    //             Color::BrightWhite => "107",
    //         }
    //     }
}

impl FromStr for Color {
    type Err = ();

    fn from_str(src: &str) -> Result<Self, Self::Err> {
        let src = src.to_lowercase();

        match src.as_ref() {
            "black" => Ok(Color::Black),
            "red" => Ok(Color::Red),
            "green" => Ok(Color::Green),
            "yellow" => Ok(Color::Yellow),
            "blue" => Ok(Color::Blue),
            "magenta" => Ok(Color::Magenta),
            "cyan" => Ok(Color::Cyan),
            "white" => Ok(Color::White),
            "bright black" => Ok(Color::BrightBlack),
            "bright red" => Ok(Color::BrightRed),
            "bright green" => Ok(Color::BrightGreen),
            "bright yellow" => Ok(Color::BrightYellow),
            "bright blue" => Ok(Color::BrightBlue),
            "bright magenta" => Ok(Color::BrightMagenta),
            "bright cyan" => Ok(Color::BrightCyan),
            "bright white" => Ok(Color::BrightWhite),
            _ => Err(()),
        }
    }
}

// mod tests {
//     pub use super::*;
//
//     mod from_str {
//         pub use super::*;
//
//         macro_rules! make_test {
//             ( $( $name:ident: $src:expr => $dst:expr),* ) => {
//
//                 $(
//                     #[test]
//                     fn $name() {
//                         let color : Color = $src.into();
//                         assert_eq!($dst, color)
//                     }
//                 )*
//             }
//         }
//
//         make_test!(
//             black: "black" => Color::Black,
//             red: "red" => Color::Red,
//             green: "green" => Color::Green,
//             yellow: "yellow" => Color::Yellow,
//             blue: "blue" => Color::Blue,
//             magenta: "magenta" => Color::Magenta,
//             cyan: "cyan" => Color::Cyan,
//             white: "white" => Color::White,
//             brightblack: "bright black" => Color::BrightBlack,
//             brightred: "bright red" => Color::BrightRed,
//             brightgreen: "bright green" => Color::BrightGreen,
//             brightyellow: "bright yellow" => Color::BrightYellow,
//             brightblue: "bright blue" => Color::BrightBlue,
//             brightmagenta: "bright magenta" => Color::BrightMagenta,
//             brightcyan: "bright cyan" => Color::BrightCyan,
//             brightwhite: "bright white" => Color::BrightWhite,
//
//             invalid: "invalid" => Color::White,
//             capitalized: "BLUE" => Color::Blue,
//             mixed_case: "bLuE" => Color::Blue
//         );
//     }
//
//     mod from_string {
//         pub use super::*;
//
//         macro_rules! make_test {
//             ( $( $name:ident: $src:expr => $dst:expr),* ) => {
//
//                 $(
//                     #[test]
//                     fn $name() {
//                         let src = String::from($src);
//                         let color : Color = src.into();
//                         assert_eq!($dst, color)
//                     }
//                 )*
//             }
//         }
//
//         make_test!(
//             black: "black" => Color::Black,
//             red: "red" => Color::Red,
//             green: "green" => Color::Green,
//             yellow: "yellow" => Color::Yellow,
//             blue: "blue" => Color::Blue,
//             magenta: "magenta" => Color::Magenta,
//             cyan: "cyan" => Color::Cyan,
//             white: "white" => Color::White,
//             brightblack: "bright black" => Color::BrightBlack,
//             brightred: "bright red" => Color::BrightRed,
//             brightgreen: "bright green" => Color::BrightGreen,
//             brightyellow: "bright yellow" => Color::BrightYellow,
//             brightblue: "bright blue" => Color::BrightBlue,
//             brightmagenta: "bright magenta" => Color::BrightMagenta,
//             brightcyan: "bright cyan" => Color::BrightCyan,
//             brightwhite: "bright white" => Color::BrightWhite,
//
//             invalid: "invalid" => Color::White,
//             capitalized: "BLUE" => Color::Blue,
//             mixed_case: "bLuE" => Color::Blue
//         );
//     }
//
//     mod fromstr {
//         pub use super::*;
//
//         #[test]
//         fn parse() {
//             let color: Result<Color, _> = "blue".parse();
//             assert_eq!(Ok(Color::Blue), color)
//         }
//
//         #[test]
//         fn error() {
//             let color: Result<Color, ()> = "bloublou".parse();
//             assert_eq!(Err(()), color)
//         }
//
//     }
// }

use log::Level;
use std::fmt;

/// Extension crate allowing the use of `.colored` on Levels.
trait ColoredLogLevel {
    /// Colors this log level with the given color.
    fn colored(&self, color: Color) -> WithFgColor<Level>;
}

/// Opaque structure which represents some text data and a color to display it
/// with.
///
/// This implements [`fmt::Display`] to displaying the inner text (usually a
/// log level) with ANSI color markers before to set the color and after to
/// reset the color.
///
/// `WithFgColor` instances can be created and displayed without any allocation.
// this is necessary in order to avoid using colored::ColorString, which has a
// Display implementation involving many allocations, and would involve two
// more string allocations even to create it.
//
// [`fmt::Display`]: https://doc.rust-lang.org/std/fmt/trait.Display.html
pub struct WithFgColor<T>
where
    T: fmt::Display,
{
    text: T,
    color: Color,
}

impl<T> fmt::Display for WithFgColor<T>
where
    T: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "\x1B[{}m", self.color.to_fg_str())?;
        fmt::Display::fmt(&self.text, f)?;
        write!(f, "\x1B[0m")?;
        Ok(())
    }
}

#[derive(Copy, Clone)]
#[must_use = "builder methods take config by value and thus must be reassigned to variable"]
pub struct ColoredLevelConfig {
    /// The color to color logs with the [`Error`] level.
    ///
    /// [`Error`]: https://docs.rs/log/0.4/log/enum.Level.html#variant.Error
    pub error: Color,
    /// The color to color logs with the [`Warn`] level.
    ///
    /// [`Warn`]: https://docs.rs/log/0.4/log/enum.Level.html#variant.Warn
    pub warn: Color,
    /// The color to color logs with the [`Info`] level.
    ///
    /// [`Info`]: https://docs.rs/log/0.4/log/enum.Level.html#variant.Info
    pub info: Color,
    /// The color to color logs with the [`Debug`] level.
    ///
    /// [`Debug`]: https://docs.rs/log/0.4/log/enum.Level.html#variant.Debug
    pub debug: Color,
    /// The color to color logs with the [`Trace`] level.
    ///
    /// [`Trace`]: https://docs.rs/log/0.4/log/enum.Level.html#variant.Trace
    pub trace: Color,
}

impl ColoredLevelConfig {
    /// Creates a new ColoredLevelConfig with the default colors.
    ///
    /// This matches the behavior of [`ColoredLevelConfig::default`].
    ///
    /// [`ColoredLevelConfig::default`]: #method.default
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    // /// Overrides the [`Error`] level color with the given color.
    // ///
    // /// The default color is [`Color::Red`].
    // ///
    // /// [`Error`]: https://docs.rs/log/0.4/log/enum.Level.html#variant.Error
    // /// [`Color::Red`]: https://docs.rs/colored/1/colored/enum.Color.html#variant.Red
    // pub fn error(mut self, error: Color) -> Self {
    //     self.error = error;
    //     self
    // }
    //
    // /// Overrides the [`Warn`] level color with the given color.
    // ///
    // /// The default color is [`Color::Yellow`].
    // ///
    // /// [`Warn`]: https://docs.rs/log/0.4/log/enum.Level.html#variant.Warn
    // /// [`Color::Yellow`]: https://docs.rs/colored/1/colored/enum.Color.html#variant.Yellow
    // pub fn warn(mut self, warn: Color) -> Self {
    //     self.warn = warn;
    //     self
    // }
    //
    // /// Overrides the [`Info`] level color with the given color.
    // ///
    // /// The default color is [`Color::Blue`].
    // ///
    // /// [`Info`]: https://docs.rs/log/0.4/log/enum.Level.html#variant.Info
    // /// [`Color::White`]: https://docs.rs/colored/1/colored/enum.Color.html#variant.White
    // pub fn info(mut self, info: Color) -> Self {
    //     self.info = info;
    //     self
    // }
    //
    // /// Overrides the [`Debug`] level color with the given color.
    // ///
    // /// The default color is [`Color::Magenta`].
    // ///
    // /// [`Debug`]: https://docs.rs/log/0.4/log/enum.Level.html#variant.Debug
    // /// [`Color::White`]: https://docs.rs/colored/1/colored/enum.Color.html#variant.White
    // pub fn debug(mut self, debug: Color) -> Self {
    //     self.debug = debug;
    //     self
    // }
    //
    // /// Overrides the [`Trace`] level color with the given color.
    // ///
    // /// The default color is [`Color::White`].
    // ///
    // /// [`Trace`]: https://docs.rs/log/0.4/log/enum.Level.html#variant.Trace
    // /// [`Color::White`]: https://docs.rs/colored/1/colored/enum.Color.html#variant.White
    // pub fn trace(mut self, trace: Color) -> Self {
    //     self.trace = trace;
    //     self
    // }

    /// Colors the given log level with the color in this configuration
    /// corresponding to it's level.
    ///
    /// The structure returned is opaque, but will print the Level surrounded
    /// by ANSI color codes when displayed. This will work correctly for
    /// UNIX terminals, but due to a lack of support from the [`colored`]
    /// crate, this will not function in Windows.
    ///
    /// [`colored`]: https://github.com/mackwic/colored
    pub fn color(&self, level: Level) -> WithFgColor<Level> {
        level.colored(self.get_color(level))
    }

    /// Retrieves the color that a log level should be colored as.
    pub fn get_color(&self, level: Level) -> Color {
        match level {
            Level::Error => self.error,
            Level::Warn => self.warn,
            Level::Info => self.info,
            Level::Debug => self.debug,
            Level::Trace => self.trace,
        }
    }
}

impl Default for ColoredLevelConfig {
    /// Retrieves the default configuration. This has:
    ///
    /// - [`Error`] as [`Color::Red`]
    /// - [`Warn`] as [`Color::Yellow`]
    /// - [`Info`] as [`Color::White`]
    /// - [`Debug`] as [`Color::White`]
    /// - [`Trace`] as [`Color::White`]
    ///
    /// [`Error`]: https://docs.rs/log/0.4/log/enum.Level.html#variant.Error
    /// [`Warn`]: https://docs.rs/log/0.4/log/enum.Level.html#variant.Warn
    /// [`Info`]: https://docs.rs/log/0.4/log/enum.Level.html#variant.Info
    /// [`Debug`]: https://docs.rs/log/0.4/log/enum.Level.html#variant.Debug
    /// [`Trace`]: https://docs.rs/log/0.4/log/enum.Level.html#variant.Trace
    /// [`Color::White`]: https://docs.rs/colored/1/colored/enum.Color.html#variant.White
    /// [`Color::Yellow`]: https://docs.rs/colored/1/colored/enum.Color.html#variant.Yellow
    /// [`Color::Red`]: https://docs.rs/colored/1/colored/enum.Color.html#variant.Red
    fn default() -> Self {
        ColoredLevelConfig {
            error: Color::Red,
            warn: Color::Yellow,
            info: Color::Blue,
            debug: Color::Magenta,
            trace: Color::White,
        }
    }
}

impl ColoredLogLevel for Level {
    fn colored(&self, color: Color) -> WithFgColor<Level> {
        WithFgColor { text: *self, color }
    }
}

// #[cfg(test)]
// mod test {
//     use colored::Colorize;
//     use crate::Color::*;
//
//     use super::WithFgColor;
//
//     #[test]
//     fn fg_color_matches_colored_behavior() {
//         for &color in &[
//             Black,
//             Red,
//             Green,
//             Yellow,
//             Blue,
//             Magenta,
//             Cyan,
//             White,
//             BrightBlack,
//             BrightRed,
//             BrightGreen,
//             BrightYellow,
//             BrightBlue,
//             BrightMagenta,
//             BrightCyan,
//             BrightWhite,
//         ] {
//             assert_eq!(
//                 format!("{}", "test".color(color)),
//                 format!(
//                     "{}",
//                     WithFgColor {
//                         text: "test",
//                         color: color,
//                     }
//                 )
//             );
//         }
//     }
//
//     #[test]
//     fn fg_color_respects_formatting_flags() {
//         let s = format!(
//             "{:^8}",
//             WithFgColor {
//                 text: "test",
//                 color: Yellow,
//             }
//         );
//         assert!(s.contains("  test  "));
//         assert!(!s.contains("   test  "));
//         assert!(!s.contains("  test   "));
//     }
// }


static ID_COLORS: &'static [&str] = &["green", "white", "yellow", "white", "blue", "magenta", "cyan", ];

/// Pick a color from: "green", "white", "yellow", "white", "blue", "magenta"
/// or "cyan" based on a provided text.
pub fn pick_color(text: &str) -> &str {
    let mut total: u16 = 0;
    for b in text.to_string().into_bytes() {
        total += b as u16;
    }
    ID_COLORS[(total as usize) % ID_COLORS.len()]
}


#[test]
fn pick_color_test() {
    let text = "main";
    let color = pick_color(text);

    assert_eq!(color, "white");

    let text = "simple";
    let color = pick_color(text);

    assert_eq!(color, "cyan");
}
