//! CLI arguments definition.

use clap::{Parser, Subcommand, ValueEnum};
use serde::{Deserialize, Serialize};

use crate::core::color::ColorMode;
use crate::lang::Lang;

#[derive(Debug, Clone, Copy, ValueEnum, Serialize, Deserialize)]
pub enum OutputFormat {
    #[value(name = "bar")]
    Bar,
    #[value(name = "text")]
    Text,
    #[value(name = "json")]
    Json,
}

/// Direct mode arguments (also used as client request payload).
#[derive(Parser, Debug, Clone, Serialize, Deserialize)]
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

    #[arg(
        long,
        help = "Output raw forecast JSON instead of formatted Waybar output (client and direct modes only)"
    )]
    pub raw: bool,
}

/// Server subcommand options.
#[derive(Parser, Debug, Clone)]
pub struct ServerOpts {
    #[arg(
        long,
        help = "Enable archiving of forecasts to forecasts.jsonl for later analytics"
    )]
    pub archive: bool,

    #[arg(
        long,
        help = "Path to Unix socket for client connections (overrides CUACA_SOCKET and default)"
    )]
    pub socket: Option<std::path::PathBuf>,

    #[arg(
        long,
        default_value = "10",
        help = "Cache TTL in minutes for forecasts (default: 10)"
    )]
    pub ttl: Option<u64>,
}

/// Stats subcommand options.
#[derive(Parser, Debug, Clone)]
pub struct StatsOpts {
    #[arg(long, help = "Filter by adm4 code (default: all)")]
    pub adm4: Option<String>,

    #[arg(long, help = "Start date for forecast slots (YYYY-MM-DD, local time)")]
    pub start: Option<chrono::NaiveDate>,

    #[arg(long, help = "End date for forecast slots (YYYY-MM-DD, local time)")]
    pub end: Option<chrono::NaiveDate>,

    #[arg(
        long,
        default_value = "t,hu,tp,ws,tcc",
        help = "Comma-separated list of variables to include in statistics"
    )]
    pub variables: String,

    #[arg(
        long,
        value_enum,
        default_value = "table",
        help = "Output format: table or json"
    )]
    pub format: Option<OutputFormat>,
}

/// Top-level command enumeration.
#[derive(Subcommand, Debug, Clone)]
pub enum Command {
    /// Start the cuaca server daemon (Unix only)
    Server(ServerOpts),

    /// Query the server (fallback to direct if unavailable)
    Client {
        #[arg(long, help = "Output raw forecast JSON instead of Waybar format")]
        raw: bool,
    },

    /// Analyze forecast archive for volatility and consistency
    Stats(StatsOpts),
}

/// Root command with optional subcommand.
#[derive(Parser, Debug)]
pub struct Root {
    #[command(flatten)]
    pub args: Args,

    #[command(subcommand)]
    pub cmd: Option<Command>,
}

// Helper to convert Args into the request for client (cloning is fine)
impl From<&Args> for serde_json::Value {
    fn from(args: &Args) -> Self {
        serde_json::to_value(args).unwrap()
    }
}
