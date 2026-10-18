# cuaca CLI

The binary layer of the cuaca weather toolchain: a Waybar weather indicator built on [BMKG](https://www.bmkg.go.id/) (Indonesian weather agency) public forecast data.

## Project Status

v0.4.0. Rendering, typed models, and i18n were extracted into the [`cuaca`](https://crates.io/crates/cuaca) library (v1.0.0); this repo now publishes the `cuaca` **binary** only (package name `cuaca-cli`).

## Two-Crate Architecture

```
crates.io: cuaca 1.0          (pure logic: models, waybar/terminal render, icons, lang, graphs, stats, warnings defs)
                 ↑ uses
this repo: cuaca-cli 0.4      (I/O: HTTP + caches + resolution + CAP/RSS + daemon socket)
                 ↓ output
           Waybar JSON (bar) / text / json / raw
```

```
cuaca CLI → [location resolution] → adm4 code
                ├── --adm4 (direct)
                ├── --lat/--lon → wilayah HTTP API → resolve cache (24h TTL)
                └── --name → wilayah HTTP API
             ↓
            BMKG API → cache_dir()/cuaca-{adm4}.json (10-min cache)
             ↓
            cuaca::Forecast (typed) → cuaca::waybar::render / cuaca::terminal::render
                 (+ BMKG nowcasting CAP warnings when --warnings, matched via cuaca::warnings)
```

## Source Files

| File | Purpose |
|------|---------|
| `src/main.rs` | Dispatch: direct / server / client / stats |
| `src/cli/args.rs` | clap definitions; `From<&Args> for cuaca::Config`; value parsers backed by the library enums |
| `src/cli/run.rs` | Orchestration: resolve → fetch → parse → render via library functions; warnings selection |
| `src/weather.rs` | BMKG fetch + retry, forecast cache wrapper, JSONL archiving |
| `src/cache.rs` | Cache dir resolution (env override, platform dirs) |
| `src/location.rs` | adm4 resolution with TTL cache; `src/location_remote.rs` wilayah API client + circuit breaker |
| `src/warnings/` | RSS + CAP fetching/parsing with cache; polygon filtering delegated to `cuaca::warnings::select_relevant` |
| `src/server.rs` | Unix-socket daemon (socket/PID housekeeping, graceful shutdown) |
| `src/client.rs` | Client with daemon fallback to direct fetch |
| `src/stats.rs` | Reads JSONL archive → `cuaca::stats::summarize` → table/json |
| `src/error.rs` | CLI error type (network/cache/parse/lib bridge) |

## Key Design Decisions

- **Rendering contract unchanged:** identical Waybar JSON and terminal output (produced by the library now).
- **Cache and archive formats unchanged:** `cuaca-{adm4}.json` wrappers and `forecasts.jsonl` keep the `fetched_at`/`adm4`/`forecast` shape, so existing caches and stats history survive the upgrade.
- **Daemon wire format changed:** the socket payload is now the new serde `Args`; old daemons/clients must be upgraded together.
- **Warnings filtering** moved from an in-CLI cascade to `cuaca::warnings::select_relevant` (polygon containment, province-name + validity fallback).
- **License:** MIT. **Attribution:** BMKG credit in tooltip and README.

## Dependencies

- `cuaca` 1.0 (crates.io) — models + rendering + stats + warnings defs
- chrono 0.4.44 (serde)
- clap 4.6.1 (derive)
- reqwest 0.13.3 (blocking, json, rustls)
- quick-xml 0.31 (CAP XML)
- serde 1 (derive)
- serde_json 1.0.149
- thiserror 1
- directories 5
- signal-hook 0.3 (daemon shutdown, unix)

## Publishing

- **GitHub:** https://github.com/rafiyq/cuaca (binaries from GitHub Releases: `cuaca-x.y.z-<platform>.tar.gz`)
- **Library:** https://crates.io/crates/cuaca (the pure-logic crate lives in [`rafiyq/cuaca-rs`](https://github.com/rafiyq/cuaca-rs))
- **AUR:** Not yet

## Roadmap

### Phase 5 — Library extraction (this release)

- [x] Split pure logic into the `cuaca` crate (v1.0.0, published)
- [x] Rebuild CLI on the published library; keep CLI surface + output byte-compatible
- [x] Port stats plumbing to `cuaca::stats` (same table/JSON layout)
- [x] Warnings filtering via `cuaca::warnings::select_relevant`
- [x] Keep BMKG cache/archive formats for existing users

### Phase 6 — Integration

- [ ] Mirror forecast types in `cuaca-worker` (TS) so Rust and worker JSON contracts stay in lockstep
- [ ] Optional: combined integration rig that runs CLI + worker fixture against the same sample payloads

### Phase 7 — Distribution

- [ ] AUR package (PKGBUILD)
- [ ] Merge the policy from `install.sh`/`install.ps1` with a versioned/download-verified flow (already in place)

See `CHANGELOG.md` for version history.
