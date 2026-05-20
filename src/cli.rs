use clap::{Parser, ValueEnum};

use crate::lang::Lang;

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum OutputFormat {
    #[value(name = "bar")]
    Bar,
    #[value(name = "text")]
    Text,
    #[value(name = "json")]
    Json,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum ColorMode {
    #[value(name = "auto")]
    Auto,
    #[value(name = "always")]
    Always,
    #[value(name = "never")]
    Never,
}

#[derive(Parser, Debug)]
#[command(
    name = "cuaca",
    version,
    about = "A weather indicator using BMKG data",
    long_about = None
)]
pub struct Args {
    #[arg(
        long,
        help = "BMKG adm4 code for the desired location (e.g. 31.71.03.1001)"
    )]
    pub adm4: Option<String>,

    #[arg(long, help = "Latitude for GPS-based location (requires --lon)")]
    pub lat: Option<f64>,

    #[arg(long, help = "Longitude for GPS-based location (requires --lat)")]
    pub lon: Option<f64>,

    #[arg(long, help = "Village name to look up")]
    pub name: Option<String>,

    #[arg(
        value_enum,
        short,
        long,
        default_value = "en",
        help = "Language for labels"
    )]
    pub lang: Lang,

    #[arg(
        value_enum,
        long,
        default_value = "bar",
        help = "Output format: bar (Waybar JSON), text (terminal display), json (raw data)"
    )]
    pub format: OutputFormat,

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

    #[arg(long, help = "Display temperature in Fahrenheit instead of Celsius")]
    pub fahrenheit: bool,

    #[arg(long, help = "Include BMKG weather warnings (nowcasts)")]
    pub warnings: bool,

    #[arg(
         long,
         default_value = "15",
         value_parser = clap::value_parser!(u64).range(1..=60),
         help = "Warnings cache TTL in minutes (RSS and fallback CAP age)"
     )]
    pub warnings_ttl: u64,

    #[arg(
        long,
        default_value = "4",
        help = "Number of Y-axis tick labels per panel (3-6)"
    )]
    pub yticks: usize,

    #[arg(
        value_enum,
        long,
        default_value = "auto",
        help = "ANSI color output: auto, always, or never"
    )]
    pub color: ColorMode,
}
