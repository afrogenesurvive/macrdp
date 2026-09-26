# Cleanup — reclaiming build artifacts

Measured and cleaned **2026-09-26** on `macrdp` (`~/Documents/GitHub/macrdp`).

## Result

| Directory | Before |
| --- | --- |
| `target/` (root workspace, debug) | 2.7 GB |
| `macrdp-ui/src-tauri/target/` (second workspace, release) | 2.5 GB |
| `macrdp-ui/node_modules/` | 245 MB |
| `macrdp-ui/dist/` | 536 KB |
| **Whole repo** | **~5.5 GB → 8.4 MB** |

Free space on `~` went from **16 Gi → 21 Gi**.

`.git` is only **2.1 MB** — the repo's *history* was never the problem. Everything
above is generated output, all of it ignored by `.gitignore`.

## The commands that were run

```bash
cd ~/Documents/GitHub/macrdp

# 1. Root Cargo workspace (macrdp-server + all of crates/)
make clean-cli              # == cargo clean

# 2. Tauri workspace + frontend bundle
make clean-ui               # == rm -rf macrdp-ui/dist macrdp-ui/src-tauri/target

# 3. Frontend dependencies
rm -rf macrdp-ui/node_modules
```

## Gotcha: `make clean-ui` fails on macOS

`make clean-ui` exits with:

```
rm: macrdp-ui/src-tauri/target/release: Directory not empty
rm: macrdp-ui/src-tauri/target: Directory not empty
make: *** [clean-ui] Error 1
```

This is **not** a permissions problem and nothing holds the files open. Finder /
Spotlight re-creates a `.DS_Store` inside the directory while `rm -rf` is deleting
it, so the final `rmdir` sees a non-empty directory. It also leaves `make`'s `&&`
chain stopped, so following steps in that chain silently do not run.

Use this instead — sweep `.DS_Store` first and retry in a loop, because Finder can
race you more than once:

```bash
cd ~/Documents/GitHub/macrdp

for dir in macrdp-ui/src-tauri/target macrdp-ui/node_modules; do
  for i in 1 2 3 4 5; do
    find "$dir" -name .DS_Store -delete 2>/dev/null
    rm -rf "$dir" 2>/dev/null
    [ -d "$dir" ] || break
  done
done
```

## Restoring a working tree afterwards

```bash
cd ~/Documents/GitHub/macrdp

make install       # == cd macrdp-ui && npm install   (~245 MB)
make build-cli     # rebuilds target/ and re-creates the binary symlinks
make dev-ui        # full Tauri dev run
```

`make build-cli` depends on `make link-cli`, which recreates
`macrdp-ui/src-tauri/macrdp-server` and
`macrdp-ui/src-tauri/target/debug/macrdp-server` as **symlinks** into
`target/debug/`. They are not copies and cost no disk space.

Note that the first `make build-cli` after a clean is a full cold build of the
whole dependency graph (IronRDP, AWS-LC, OpenH264, screencapturekit, …), so expect
several minutes.

Until that rebuild runs, `macrdp-ui/src-tauri/macrdp-server` is a **dangling
symlink**: it points at `../../target/debug/macrdp-server`, which no longer exists.
That is expected and harmless — `make build-cli` depends on `link-cli`, which
recreates it — but anything that tries to execute the sidecar before then will fail
with a confusing "no such file" error. Check with:

```bash
readlink macrdp-ui/src-tauri/macrdp-server      # ../../target/debug/macrdp-server
[ -e macrdp-ui/src-tauri/macrdp-server ] && echo "resolves OK" || echo DANGLING
```

## Before / after check

```bash
cd ~/Documents/GitHub/macrdp

du -sh .git target macrdp-ui/node_modules macrdp-ui/dist macrdp-ui/src-tauri/target 2>/dev/null
du -h -d1 target/debug | sort -h      # breaks down deps/ build/ incremental/
df -h "$HOME" | tail -1
```

## Optional: stop it growing back

Not applied — all of these edit tracked files, so apply deliberately.

1. **Biggest win.** Drop debug info from dependencies, which is what makes
   `target/debug/deps` 2.2 GB. In the root `Cargo.toml`:

   ```toml
   [profile.dev.package."*"]
   debug = false        # or debug = 1 for line tables only
   ```

2. **Stop `incremental/` growing** (190 MB here, and it grows on every edit):

   ```bash
   export CARGO_INCREMENTAL=0
   ```

3. **Make the Tauri release profile an actual release** in
   `macrdp-ui/src-tauri/Cargo.toml` — it is currently `opt-level = 0`,
   `lto = false`, `strip = false`, so the "release" binary keeps every symbol:

   ```toml
   [profile.release]
   opt-level = 3
   lto = "thin"
   strip = true
   codegen-units = 16
   ```

4. **Prune stale artifacts by age** instead of nuking everything:

   ```bash
   cargo install cargo-sweep
   cargo sweep -t 30d                          # root workspace
   cargo sweep -t 30d --manifest-path macrdp-ui/src-tauri/Cargo.toml
   ```

5. **Structural, not a script.** `macrdp-ui/src-tauri/Cargo.toml` declares its own
   bare `[workspace]`, so it is *not* a member of the root workspace. macrdp-core,
   both vendored IronRDP forks (`crates/ironrdp-server-gfx`,
   `crates/ironrdp-acceptor-patched`) and the whole transitive graph are therefore
   compiled **twice**, into two different target dirs. There is no
   `CARGO_TARGET_DIR` and no `sccache` anywhere in the repo. Folding `src-tauri`
   into the root workspace (merging the two `[patch.crates-io]` tables) would
   remove the duplicate build permanently, at the cost of a real refactor.

6. `macrdp-ui`'s `node_modules` is 245 MB largely because the `shadcn` CLI is a
   dependency and drags in a server-side tree (`@modelcontextprotocol/sdk`, `msw`,
   `express`, `hono`, `graphql`, `ts-morph`, `jose`, `@ecies`, `@noble/*`).
   Running it on demand via `npx shadcn@latest` instead of keeping it installed
   avoids that entirely.

---

## Appendix: finding build artifacts elsewhere on disk

These two commands name no paths themselves — run them from whichever directory
you want to inspect:

```bash
# Build/dependency directories at any depth, over 100 MB
find . -type d \( -name node_modules -o -name .build -o -name .venv -o -name venv \
  -o -name .next -o -name DerivedData -o -name target -o -name build -o -name Pods \
  -o -name dist -o -name .dart_tool \) -prune -exec du -sm {} + 2>/dev/null \
  | awk '$1>100' | sort -n

# Oversized git object stores. These are HISTORY, not build output — never
# delete them; run `git gc` instead.
find . -maxdepth 2 -type d \( -name .git -o -name '*.git' \) -prune \
  -exec du -sm {} + 2>/dev/null | awk '$1>100' | sort -n
```

Always confirm a directory is disposable before removing it — `ignored=yes` and
`tracked_files=0` means generated output, not source:

```bash
for p in <repo>/<dir>; do
  d=${p%%/*}
  printf '%-48s ignored=%-4s tracked=%s\n' "$p" \
    "$(git -C "$d" check-ignore -q "${p#*/}" && echo yes || echo NO)" \
    "$(git -C "$d" ls-files "${p#*/}" | wc -l | tr -d ' ')"
done
```

Inside this repo, the equivalent surfaces are the two `target/` trees plus
`macrdp-ui/node_modules` — see the table at the top of this document.
