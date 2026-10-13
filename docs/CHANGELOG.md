# Changelog

Notable changes to macrdp. Entries are `## [<branch>-<n>] — <date>`, where `<n>` is
one more than the highest existing suffix for that branch.

## [main-3] — 2026-10-13

### Fixed

- GFX/EGFX capability negotiation no longer collapses when a client advertises a
  capability-set version this build does not recognize. `ironrdp-pdu` is now vendored and
  patched (`crates/ironrdp-pdu-patched/`) with IronRDP's lossless capability-set
  representation: `CapabilityVersion` is a plain `u32` and `RawCapabilitySet` keeps an
  unrecognized version's bytes, so the whole `CapabilitiesAdvertise` PDU can still be
  decoded and re-encoded. Previously a single unknown version failed that PDU, left
  `GfxState::is_ready()` false, and silently suppressed all video — the session connected
  (input and audio worked) but the screen stayed blank. Modern Microsoft clients advertise
  versions `0x000B0101` / `0x000B0300` that this build had never heard of.

### Changed

- The GFX negotiation now confirms the highest-versioned capability set the client
  advertised and echoes it back verbatim (so an unrecognized version round-trips on the
  wire), and treats an unrecognized/newer version as AVC420/AVC444 capable.
- The display loop logs a one-time warning when it suppresses all frames because GFX/AVC
  was never negotiated, instead of failing silently.

## [main-2] — 2026-10-05

### Added

- Internationalisation for the UI. `macrdp-ui/src/i18n/` introduces a
  `LanguageProvider` / `useI18n()` context backed by `en` and `zh` string tables, a
  `t(key, vars)` lookup with `{name}` interpolation, and a `locale` used for date
  formatting. The UI previously hard-coded Chinese strings; it now renders in English by
  default, and Settings gained a **Language** selector (English / 中文). The choice is
  persisted in `localStorage` under `macrdp.lang`.

### Changed

- Every page and layout component resolves its labels through `t()` instead of literal
  strings: `App`, `Sidebar`, `StatusBar`, `ThemeToggle`, `ChartPanel`, `Dashboard`,
  `Settings`, `Permissions`, `Logs`, `Statistics`, `About` and `Popover`. `Statistics`
  now formats dates with the active locale instead of a fixed `zh-CN`.

## [main-1] — 2026-09-26

### Fixed

- The bundled `macrdp.app` no longer aborts inside `dyld` before `main()` when it is
  started from Finder or `launchd`:
  `Library not loaded: @rpath/libswift_Concurrency.dylib`. `macrdp-ui/src-tauri/build.rs`
  now bakes `-Wl,-rpath,/usr/lib/swift` into the UI binary. The `DYLD_LIBRARY_PATH`
  workaround in `.cargo/config.toml` only ever covered `cargo test` / `cargo run`,
  because a `.app` launched by `launchd` inherits none of cargo's environment.

### Added

- `cleanup.md` — how to reclaim the ~5.5 GB of generated build output, the macOS
  `.DS_Store` race that makes `make clean-ui` fail, and how to restore a working tree
  afterwards.

### Changed

- Regenerated the Tauri ACL / capability schemas under
  `macrdp-ui/src-tauri/gen/schemas/`.
- `.gitignore` now ignores `macrdp-ui/src-tauri/macrdp-server`, the sidecar symlink
  that `make link-cli` points at `target/debug/`.
