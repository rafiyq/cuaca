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

**Location selection (one required):**

- `--adm4 CODE` - BMKG adm4 code (e.g. `31.71.03.1001` for Kemayoran, Jakarta Pusat)
- `--lat LAT --lon LON` - GPS coordinates; resolves via remote API (cached 24 h)
- `--name QUERY` - village name (substring match); resolves via remote API (cached 24 h)

**Other options:**

- `--lang en|id` - language for tooltip labels [default: `en`]
- `--custom-indicator EXPR` - custom bar display using `{KEY}` placeholders. Available keys: `t`, `hu`, `tcc`, `tp`, `ws`, `wd_deg`, `wd`, `weather_desc`, `weather_desc_en`, `vs_text`, and `{ICON}`
- `--date-format FMT` - strftime format for dates [default: `%Y-%m-%d`]
- `--ampm` - display time in AM/PM format
- `--nerd` - use nerd font symbols instead of emojis
- `--hide-details` - show shorter per-slot lines (hide cloud cover, precipitation, and visibility)
- `--warnings` - include BMKG nowcast weather warnings
- `--warnings-ttl MINUTES` - warnings cache TTL in minutes (default: 15)
- `--raw` - output raw forecast JSON only (no formatting, no warnings)

Example:

```
cuaca --adm4 31.71.03.1001 --ampm --hide-details
```

## Daemon and Client Modes

For long-running deployments (e.g. Waybar), you can run a daemon that listens on a Unix socket and serves requests, reducing API call overhead and rate limit pressure.

Start the daemon:

```
cuaca server --archive --socket ~/.cache/cuaca/cuaca.sock --ttl 600
```

This writes a PID file next to the socket and gracefully shuts down on SIGTERM/SIGINT. The `--archive` flag appends every fetched forecast to `forecasts.jsonl` in the cache directory for later analysis.

In your Waybar config, use the client instead of direct calls:

```json
"custom/weather": {
    "format": "{}°",
    "tooltip": true,
    "interval": 3600,
    "exec": "cuaca client",
    "return-type": "json"
}
```

If the daemon is not running, the client falls back to a direct fetch.

Environment variables:

- `CUACA_SOCKET` - path to Unix socket (default: `$CUACA_CACHE_DIR/cuaca.sock`).

You can run the daemon as a systemd user service:

```
[Unit]
Description=Cuaca weather daemon

[Service]
ExecStart=%h/.cargo/bin/cuaca server --archive
Restart=on-failure

[Install]
WantedBy=default.target
```

Enable with `systemctl --user enable --start cuaca.service`.

## Statistics

The `cuaca stats` command analyzes the archived forecasts (enable with `--archive` on the server) and prints statistical summaries. By default it prints a table with mean and standard deviation for temperature (°C), humidity (%), precipitation (mm), wind (km/h), and cloud cover (%).

Options:
- `--adm4 FILTER` - only include forecasts for a specific adm4 code.
- `--start DATE` and `--end DATE` - restrict to date range (YYYY-MM-DD).
- `--variables VARS` - comma-separated list of variables to include: `t,hu,tp,ws,tcc`. Default is all.
- `--format table|json` - output format; default is table.

Example:

```
cuaca stats --start 2026-06-01 --end 2026-06-07 --variables t,hu --format json
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

The first available forecast slot is used as the bar display. The tooltip shows the full 3-day forecast with all available data fields. Weather results are cached for 10 minutes.

### Location resolution

- If you provide `--adm4`, it is used directly.
- If you provide `--lat`/`--lon` or `--name`, cuaca queries the [wilayah](https://api.wilayah.workers.dev/) public API to translate coordinates or place names into an adm4 code. This lookup is cached for 24 hours in a platform‑appropriate user cache directory (`$XDG_CACHE_HOME/cuaca` on Linux, `~/Library/Caches/cuaca` on macOS, `%LOCALAPPDATA%\\cuaca\\cache` on Windows). You can override the cache location with the `CUACA_CACHE_DIR` environment variable.
- The remote API base URL can be changed via `WILAYAH_API_BASE` (e.g., for self‑hosting). The default is `https://api.wilayah.workers.dev`.
- A circuit breaker protects against repeated failures when the remote API is unavailable; after consecutive errors, further lookups are short‑circuited for 5 minutes.

Note: Using `--lat`/`--lon` or `--name` sends your coordinates or query to the remote service.

## Data Source

Weather data is provided by [BMKG](https://www.bmkg.go.id/) (Badan Meteorologi, Klimatologi, dan Geofisika) through their [public forecast API](https://data.bmkg.go.id/prakiraan-cuaca/). Attribution is included in the tooltip.

Administrative area codes (adm4) follow Keputusan Menteri Dalam Negeri Nomor 100.1.1-6117 Tahun 2022. See [BMKG's data portal](https://data.bmkg.go.id/prakiraan-cuaca/) and [GitHub repository](https://github.com/infoBMKG/data-cuaca) for more information.
