fn main() {
    tauri_build::build();

    // The screencapturekit crate links Apple's Swift concurrency runtime, and
    // the linker records that one library as `@rpath/libswift_Concurrency.dylib`.
    // Nothing emits an LC_RPATH to resolve it, so an app started from Finder or
    // launchd aborts inside dyld before main() ever runs:
    //
    //   Library not loaded: @rpath/libswift_Concurrency.dylib
    //   Referenced from: .../Contents/MacOS/macrdp-ui
    //   Reason: no LC_RPATH's found
    //
    // `.cargo/config.toml` sets DYLD_LIBRARY_PATH as a workaround, but that only
    // covers `cargo test` / `cargo run`, because cargo passes its environment to
    // the child process. A bundled .app launched by launchd inherits none of it,
    // so the search path has to be baked into the binary instead.
    //
    // `/usr/lib/swift` is the correct target: macOS serves the Swift runtime from
    // the dyld shared cache, so `libswift_Concurrency.dylib` need not exist as a
    // file on disk there. That is precisely why the `.exists()` probe in
    // crates/macrdp-server/build.rs misses it and falls back to the Xcode
    // toolchain copy.
    //
    // NOTE: `cargo:rustc-link-arg` applies only to the crate that emits it, so
    // every final binary in the workspace needs its own copy of this. This is the
    // UI binary; crates/macrdp-server has an equivalent block of its own.
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("macos") {
        println!("cargo:rustc-link-arg=-Wl,-rpath,/usr/lib/swift");
    }
}
