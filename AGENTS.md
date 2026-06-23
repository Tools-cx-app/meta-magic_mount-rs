# Repository Guide

## Toolchain

- Use Rust nightly. The root manifest enables unstable Cargo features, and Android builds also require Android NDK r29 plus `cargo-ndk`.
- The frontend requires Node >=24 and pnpm 11.7.0; install it with `pnpm install --frozen-lockfile` in `webui/`.
- `.cargo/config.toml` defines `cargo xtask` as `cargo run --package xtask`. To pass help to xtask rather than Cargo, use `cargo xtask -- --help`.

## Project Boundaries

- The Cargo workspace contains the root `magic_mount_rs` binary, the shared `api` model crate, the `daemon` HTTP server, and `xtask`.
- `src/main.rs` performs privileged mounts with no argument and exposes only the `emulated-soft-reboot` maintenance command. The WebUI contract lives in `daemon/src/api.rs` under `/api/v1`.
- Mount flow is `config/scanner/parser -> magic_mount::{utils,node} -> magic_mount::MagicMount`, followed by custom binds in `bind_mount.rs`. Changes here affect the Android mount namespace and cannot be exercised by ordinary host unit tests.
- `module/service.sh` supervises the daemon during the late-start service stage. The daemon binds to loopback, publishes its random port and bearer token in `/data/adb/magic_mount/daemon.json`, and accepts requests only from the module WebView origin.
- `webui/src/lib/api.ts` uses the KernelSU bridge only to read the daemon discovery file; normal operations then use the authenticated HTTP client in `webui/src/lib/api.client.ts`.
- `module/` is the install-package template. `xtask` combines it with architecture binaries and the WebUI into ignored `output/*.zip` artifacts.

## Generated Side Effects

- Any root Cargo build runs `build.rs`, which regenerates ignored `module/module.prop` from `Cargo.toml` and Git history and sets the compile-time `MODULE_ID`.
- `pnpm run build` in `webui/` writes directly to ignored `module/webroot/`; it does not use a local `webui/dist/` directory.
- Do not edit or commit `module/module.prop`, `module/webroot/`, `module/bin/`, `output/`, or `target/`; they are generated/ignored.
- Keep both `Cargo.lock` and `webui/pnpm-lock.yaml` updated when their respective dependencies change.

## Commands

- Host Rust tests: `cargo test --workspace`
- One exact Rust test: `cargo test parser::tests::parse_bind_valid -- --exact`
- Rust formatting check: `cargo fmt --all -- --check`
- CI-equivalent Android lint: `cargo ndk -t arm64-v8a -t armeabi-v7a clippy --workspace --exclude xtask -- -D warnings`
- Frontend checks, from `webui/`: `pnpm test`, `pnpm run lint`, `pnpm run typecheck`, `pnpm run build`
- Shell changes under `module/`: run ShellCheck; CI intentionally disables SC2034, SC2086, and SC2115.
- Package one target: `cargo xtask build -t arm64|armv7|x86-64|universal`; artifacts are `output/magic_mount_rs-*.zip`.

## Testing And Runtime Gotchas

- Files under `tests/unit/` are included into the binary with `#[path = ...]`; test names therefore use module paths such as `config::tests::...`, not integration-test targets.
- Host tests cover parsers, scanning, configuration, and path preparation, but not real tmpfs/bind/move mounts. Mount changes need an Android root environment and explicit manual verification.
- `/data/adb/magic_mount/config.toml` and `/data/adb/magic_mount/custom` are persistent runtime inputs. Default install values differ by platform: `module/config.toml` uses `KSU`, while `module/config_apatch.toml` uses `APatch`.
- The daemon's `GET /api/v1/modules` endpoint scans modules on demand. The mount binary still refreshes `/data/adb/magic_mount/scan.ret` during startup, but that cache is not the WebUI data source.
- `/data/adb/magic_mount/daemon.json` contains a runtime bearer token and must remain owner-only. The companion `daemon.lock` is both the single-instance lock and the PID source used by `module/uninstall.sh`.
- `parser::COMMAND_LIST` and the include-file visited list are process-global. Tests that invoke recursive `file`/`add` parsing must isolate or clear parser state as existing parser tests do.
- Rust Clippy denies standard `HashMap`/`HashSet` and standard sync locks via `clippy.toml`; use `FxHashMap`/`FxHashSet` and `parking_lot` locks.
