# cuaca

a weather indicator for [Waybar](https://github.com/Alexays/Waybar/) using [BMKG](https://www.bmkg.go.id/) weather data.

## Installation

Compile yourself using `cargo build --release`, or download a precompiled binary from the [releases](https://github.com/bjesus/cuaca/releases) page.

## Usage

- `--adm4 CODE` - **required** BMKG adm4 code for your location (e.g. `31.71.03.1001` for Kemayoran, Jakarta Pusat)
- `--lang en|id` - language for tooltip labels [default: `en`]
- `--custom-indicator EXPR` - custom bar display using `{KEY}` placeholders. Available keys: `t`, `hu`, `tcc`, `tp`, `ws`, `wd_deg`, `wd`, `weather_desc`, `weather_desc_en`, `vs_text`, and `{ICON}`
- `--date-format FMT` - strftime format for dates [default: `%Y-%m-%d`]
- `--ampm` - display time in AM/PM format
- `--nerd` - use nerd font symbols instead of emojis
- `--hide-details` - show shorter per-slot lines (hide cloud cover, precipitation, and visibility)

Example:

```
cuaca --adm4 31.71.03.1001 --ampm --hide-details
```

## Waybar configuration

Assuming `cuaca` is in your path:

```json
"custom/weather": {
    "format": "{}°",
    "tooltip": true,
    "interval": 3600,
    "exec": "cuaca --adm4 31.71.03.1001",
    "return-type": "json"
}
```

Custom styling based on current conditions:

```css
#custom-weather.sunny {
  background-color: yellow;
}
```

## How it works

cuaca fetches 3-day forecast data from BMKG's public weather API for a given administrative area code (adm4). The data includes 3-hourly forecast slots with temperature, humidity, cloud cover, precipitation, wind, and visibility.

The first available forecast slot is used as the bar display. The tooltip shows the full 3-day forecast with all available data fields. Results are cached for 10 minutes to avoid excessive API calls.

## Data Source

Weather data is provided by [BMKG](https://www.bmkg.go.id/) (Badan Meteorologi, Klimatologi, dan Geofisika) through their [public forecast API](https://data.bmkg.go.id/prakiraan-cuaca/). Attribution is included in the tooltip.

Administrative area codes (adm4) follow Keputusan Menteri Dalam Negeri Nomor 100.1.1-6117 Tahun 2022. See [BMKG's data portal](https://data.bmkg.go.id/prakiraan-cuaca/) and [GitHub repository](https://github.com/infoBMKG/data-cuaca) for more information.

