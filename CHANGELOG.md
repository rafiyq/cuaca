# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.0] - 2026-05-20

 ### Added

 - BMKG nowcast weather warnings integration via `--warnings` flag, with province-level matching and 5‑minute caching.
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
