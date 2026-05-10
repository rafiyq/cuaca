use clap::Parser;

use crate::lang::Lang;

#[derive(Parser, Debug)]
#[command(
    name = "cuaca",
    version,
    about = "A weather indicator for Waybar using BMKG weather data",
    long_about = None
)]
pub struct Args {
    #[arg(
        long,
        help = "BMKG adm4 code for the desired location (e.g. 31.71.03.1001)"
    )]
    pub adm4: String,

    #[arg(
        value_enum,
        short,
        long,
        default_value = "en",
        help = "Language for tooltip labels"
    )]
    pub lang: Lang,

    #[arg(
        long,
        help = "Custom expression with {KEY} placeholders for the bar display. Uses forecast slot keys (t, hu, weather_desc_en, weather_desc, wd, ws, etc.) and {ICON}"
    )]
    pub custom_indicator: Option<String>,

    #[arg(
        long,
        default_value = "%Y-%m-%d",
        help = "strftime format for dates. see https://docs.rs/chrono/latest/chrono/format/strftime/index.html"
    )]
    pub date_format: String,

    #[arg(long, help = "Display time in AM/PM format")]
    pub ampm: bool,

    #[arg(long, help = "Use nerd font symbols instead of emojis")]
    pub nerd: bool,

    #[arg(
        long,
        help = "Show shorter per-slot lines (hide cloud cover, precipitation, and visibility)"
    )]
    pub hide_details: bool,
}
