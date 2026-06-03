# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.6] - 2026-06-03

### Added

- Comprehensive test suite (36 tests) covering terminal rendering, format helpers, warnings parsing, and polygon math.
- `core` module: pure weather logic independent of I/O and display.
- `cli` module: separate argument parsing (`args`) and orchestration (`run`).
- `display` module: terminal renderer with layout, colorization, and icon helpers.
- Structured error handling via `CuacaError` using `thiserror`.
- Location resolution (`resolve`) with GPS cache and fallback to `wilayah` database.
- Weather fetching (`fetch_weather`) with retry logic and disk cache.
- Warnings modularization: `warnings/cache`, `rss`, `cap`, `polygon`, `fetch` with proper `Result` propagation.
- `error_json` utility for Waybar-compatible error output.
- Module documentation for core components.

### Changed

- `main` reduced to thin wrapper calling `cli::run`.
- Terminal rendering now uses dedicated `layout` and `colorize` helpers.
- `format_wind_dir_icon` centralized in `format` module, respecting `nerd` flag.
- Error handling throughout uses `Result` instead of `process::exit`.
- Warnings fetch now returns `Result<Vec<Warning>, CuacaError>`; callers fall back to empty on error.
- Color handling separated from ASCII icons (dynamic coloring based on weather code).
- Consistent formatting with `cargo fmt`.
- Release workflow: macOS runners changed to `macos-latest`; automatic release creation removed (build-only workflow).

### Fixed

- Clippy `collapsible-match` lint by refactoring `ansi_to_pango`.
- NaN panic in chart scaling via `total_cmp`.
- UTF-8 safe truncation in terminal title field using `chars().take()`.
- JSON injection safety in `error_json` using `serde_json::json!`.
- Release workflow token requirement eliminated (build-only artifacts).

## [0.2.5] - 2026-05-24

### Added

- Tooltip: multi‑line ASCII weather icon with preserved 256‑color palette and monospace font as a left‑hand column beside description.
- Terminal: hourly forecast table for Tomorrow and Day After Tomorrow (columns: Time, Temp, Description, Wind, Cloud, Precip, Vis) with fixed‑width formatting and 2‑space left margin. Day headers bold; column headers dim.
- Added 2‑space left margin to title ("Weather Report:"), date line, and source line for visual consistency.

### Fixed

- Tooltip ASCII art colors: fixed `ansi_to_pango` CSI parsing (strip trailing `m`).
- Terminal output: removed day‑summary lines; corrected table separator placement and column alignment.

### Changed

- Installers (`install.sh`, `install.ps1`) expect binary at archive root (no `dist/` subdirectory).
- Release workflow: cache whole `target/`, inject `GITHUB_TOKEN` for `wilayah` API calls, and use newline‑separated file list for `action-gh-release`.

## [0.2.4] - 2026-05-22

### Added

- Install scripts (`install.sh`, `install.ps1`) with checksum verification and auto-upgrade.
- Release workflow now publishes `.sha256` checksum files for each asset.

## [0.2.3] - 2026-05-21

### Fixed

- Tooltip spacing: blank line after title instead of after description; day header spacing
- Code quality: clippy warnings cleanup and minor refactors

## [0.2.2] - 2026-05-20

### Added

- Polygon‑based alert matching with fallback to area name.
- `--warnings-ttl` flag to configure warnings cache TTL (default 15 minutes).
- Tooltip shows warning validity period (HH:MM–HH:MM) and infographic link.

## [0.2.1] - 2026-05-20

### Added

- BMKG nowcast weather warnings via `--warnings` flag (province‑level matching, 15‑minute RSS TTL, per‑alert caching).
- Colored ASCII weather icons with 256‑color support.
- Wider column chart bars with fixed 8‑slot time axis, spanning multiple days.
- Localized headers and dates based on language (EN/ID).
- Indonesian translations for wind unit (km/j), average label (rata-rata), and total (Jumlah).

### Changed

- Cache strategy: all caches now respect `CUACA_CACHE_DIR` and store under `$CUACA_CACHE_DIR/cuaca` subdirectory (default `$TEMP/cuaca`).
- Warnings cache uses per‑alert files and 15‑minute RSS TTL for efficiency.
- Temperature panel title now includes unit (°C/°F).
- Visibility panel title now includes unit (km).
- Wind speed formatting uses localized unit.
- Day summary line now displays total rain label and average humidity label per language.

### Fixed

- Icons are no longer double‑wrapped with color codes.
- ANSI codes are stripped in no‑color mode.
- Tooltip now properly escapes Pango markup to avoid blank rendering.

## [0.2.0] - 2026-05-20

### Added

- Colored ASCII weather icons with 256‑color support.
- Wider column chart bars with fixed 8‑slot time axis, spanning multiple days.
- Localized headers and dates based on language (EN/ID).
- Indonesian translations for wind unit (km/j), average label (rata-rata), and total (Jumlah).

### Changed

- Temperature panel title now includes unit (°C/°F).
- Visibility panel title now includes unit (km).
- Wind speed formatting uses localized unit.
- Day summary line now displays total rain label and average humidity label per language.

### Fixed

- Icons are no longer double‑wrapped with color codes.
- ANSI codes are stripped in no‑color mode.

## [0.1.0] - 2026-05-17

- Initial implementation
- Multi-platform builds (Linux, macOS, Windows)
- Auto-release on git tag push
- 13 unit tests, zero clippy warnings
- EN/ID support, 14 weather codes, emoji + nerd font icons
- Custom indicator expressions, 10-min caching, exp backoff retry
- Cross-platform cache paths, Day 3 label, mutual `--lat`/`--lon` validation
- `--fahrenheit` temperature conversion
- Graceful cache corruption handling with API fallback
