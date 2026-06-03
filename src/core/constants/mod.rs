mod ascii;
mod emoji;
mod nerd;

pub use ascii::*;
pub use emoji::*;
pub use nerd::*;

pub const ICON_PLACEHOLDER: &str = "{ICON}";
pub const CLOUD_COVER_ICON: &str = "\u{2601}\u{fe0f}";
pub const PRECIPITATION_ICON: &str = "\u{1f327}\u{fe0f}";
pub const VISIBILITY_ICON: &str = "\u{1f441}\u{fe0f}";
pub const ERROR_ICON: &str = "\u{26d3}\u{fe0f}";
