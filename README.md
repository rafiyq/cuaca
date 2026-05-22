# cuaca

a weather indicator for [Waybar](https://github.com/Alexays/Waybar/) using [BMKG](https://www.bmkg.go.id/) weather data.

## Installation

1. Compile yourself using `cargo build --release`, or
2. Download a precompiled binary from the [releases](https://github.com/rafiyq/cuaca/releases) page, or
3. Use the provided installer:
   - Linux/macOS: `curl -fsSL https://raw.githubusercontent.com/rafiyq/cuaca/main/install.sh | bash`
   - Windows (PowerShell): `irm https://raw.githubusercontent.com/rafiyq/cuaca/main/install.ps1 | iex`

The installer verifies SHA256 checksums, supports auto‑upgrade, and allows options like `--bin-dir`, `--dry-run`, `--no-verify`, and `--version`.

## Usage

- `--adm4 CODE` - **required** BMKG adm4 code for your location (e.g. `31.71.03.1001` for Kemayoran, Jakarta Pusat)
- `--lang en|id` - language for tooltip labels [default: `en`]
- `--custom-indicator EXPR` - custom bar display using `{KEY}` placeholders. Available keys: `t`, `hu`, `tcc`, `tp`, `ws`, `wd_deg`, `wd`, `weather_desc`, `weather_desc_en`, `vs_text`, and `{ICON}`
- `--date-format FMT` - strftime format for dates [default: `%Y-%m-%d`]
- `--ampm` - display time in AM/PM format
- `--nerd` - use nerd font symbols instead of emojis
- `--hide-details` - show shorter per-slot lines (hide cloud cover, precipitation, and visibility)
- `--warnings` - include BMKG nowcast weather warnings
- `--warnings-ttl MINUTES` - warnings cache TTL in minutes (default: 15)

 Example:

 ```
 cuaca --adm4 31.71.03.1001 --ampm --hide-details
 ```

## Weather Warnings

 With the `--warnings` flag, cuaca fetches BMKG nowcast weather warnings ( CAP XML ) that affect your location. Matching is performed using precise polygon containment when the alert defines a polygon; otherwise the province name is used as fallback. The tooltip will show each warning's validity period (HH:MM–HH:MM) and a link to the BMKG infographic. Warnings are cached per‑alert for 15 minutes (configurable via `--warnings-ttl`) to respect rate limits.

 Note: The tooltip properly escapes visibility values like `< 8 km` to avoid rendering issues.

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
