# Changelog

Notable changes to macrdp. Entries are `## [<branch>-<n>] — <date>`, where `<n>` is
one more than the highest existing suffix for that branch.

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
