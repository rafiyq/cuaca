# cuaca

A weather indicator for Waybar using BMKG (Indonesian weather agency) public forecast API.

## Project Status

Built and verified. Ready for polish and distribution.

## Architecture

```
cuaca (CLI) → [location resolution] → adm4 code
                 ├── --adm4 (direct)
                 ├── --lat/--lon → wilayah lib → /tmp/cuaca-gps.json (24h cache)
                 └── --name → wilayah lib
              ↓
             BMKG API → Waybar JSON output
              ↓
         /tmp/cuaca-{adm4}.json (10-min cache)
```

## Source Files

| File | Purpose |
|------|---------|
| `src/main.rs` | Orchestrator: location resolution, fetch, cache, parse, build Waybar JSON |
| `src/cli.rs` | CLI argument parser (clap) — 10 flags |
| `src/lang.rs` | EN/ID i18n for tooltip labels |
| `src/constants.rs` | 14 weather codes → emoji/nerd font icon mappings |
| `src/format.rs` | Formatting utilities + custom indicator engine + 12 tests |

## Key Design Decisions

- **Data source:** BMKG public API (`api.bmkg.go.id/publik/prakiraan-cuaca?adm4=`)
- **Output:** Waybar JSON (`text`, `tooltip`, `class`)
- **Caching:** 10-minute file cache in `/tmp/`
- **Retry:** Exponential backoff, up to 20 retries (500ms base)
- **Weather codes:** 14 codes (0,1=sunny; 2=partly cloudy; 3=mostly cloudy; 4=cloudy; 5,10=mist; 45=fog; 60,61=light rain; 63=moderate rain; 80=heavy rain; 95,97=thunderstorm)
- **Forecast display:** 3 day-groups, 3-hourly slots, all shown (no time filtering)
- **Languages:** EN (weather_desc_en) and ID (weather_desc)
- **License:** MIT
- **Attribution:** BMKG credit in tooltip and README

## Dependencies

- chrono 0.4.44 (unstable-locales)
- clap 4.6.1 (derive)
- reqwest 0.13.3 (blocking, json, rustls)
- serde 1 (derive)
- serde_json 1.0.149
- wilayah (path: ../locate) — embedded SQLite DB for GPS→adm4 resolution

## Publishing

- **GitHub:** Published at (your repo URL)
- **AUR:** Not yet
- **crates.io:** Skipped (binary, not library)

## Relationship to wilayah

`cuaca` uses [`wilayah`](https://crates.io/crates/wilayah) from crates.io as a library to resolve location input into BMKG adm4 codes.
Users can provide their location via `--adm4`, `--lat`/`--lon`, or `--name`.
GPS resolutions are cached to `/tmp/cuaca-gps.json` (24h TTL) to avoid reopening the database on every Waybar refresh.

## Roadmap

### Phase 1 — Polish
- [ ] Add `repository`, `homepage`, `authors` to Cargo.toml
- [ ] Fix `--hide-details` nerd font icons (currently hardcoded emoji)
- [ ] Test against real BMKG API with actual weather data
- [ ] Verify `--version` / `-V` works

### Phase 2 — Features
- [ ] `--fahrenheit` — convert Celsius to Fahrenheit
- [ ] `--vertical-view` — icon on first line, temp on second
- [ ] `--mpers` — wind speed in m/s instead of km/h
- [ ] Day 3 label (currently only Today and Tomorrow)

### Phase 3 — Integration
- [x] `--lat`/`--lon` — resolve GPS coordinates via `wilayah` library
- [x] `--name` — resolve village name via `wilayah` library
- [x] `/tmp/cuaca-gps.json` — 24h GPS resolution cache

### Phase 4 — Distribution
- [ ] AUR package (PKGBUILD)
- [ ] AArch64 build target in CI

## Changelog

### v0.1.0
- Initial implementation
- 12 unit tests, zero clippy warnings
- EN/ID support, 14 weather codes, emoji + nerd font icons
- Custom indicator expressions, 10-min caching, exp backoff retry