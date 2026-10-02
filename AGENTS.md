# Repository Guide

## Toolchain

- Use Rust nightly. The root manifest enables unstable Cargo features, and Android builds also require Android NDK r29 plus `cargo-ndk`.
- The frontend requires Node >=24; use the pnpm version pinned in `webui/package.json` (`11.24.0` currently). Install dependencies with `pnpm install --frozen-lockfile` in `webui/`.
- `.cargo/config.toml` defines `cargo xtask` as `cargo run --package xtask`. To pass help to xtask rather than Cargo, use `cargo xtask -- --help`.

## Project Boundaries

- The Cargo workspace contains the root `magic_mount_rs` binary and `xtask`; `daemon/` only contains ignored `.bak` remnants and is not a package.
- `src/main.rs` is both the runtime entrypoint and CLI used by the WebUI. With no argument it scans modules and performs privileged mounts; `show-config`, `save-config`, `modules`, and `version` are the WebUI contract.
- Mount flow is `config/scanner/parser -> magic_mount::collect_module_files -> magic_mount::MagicMount`, followed by custom binds in `bind_mount.rs`. Collection and mounting live in `src/magic_mount.rs`. Changes to actual mounts affect the Android mount namespace and cannot be exercised by ordinary host unit tests.
- `webui/src/lib/api.ts` invokes the installed binary through the KernelSU bridge. Its binary path comes from `MODULE_ID`, injected by `xtask`; direct frontend builds default it to `test`.
- WebUI configuration has two implementations: `webui/src/ui/miuix/pages/config.vue` and `webui/src/ui/md3/pages/config.vue`. Keep both in sync through the shared `configStore` and API; update all six locale JSON files when adding user-facing text.
- Both configuration pages edit `extra_mount` after `partitions`. Name validation is shared in `webui/src/lib/extraMount.ts`; MD3 uses the optional `ChipInput` validation callback. Keep validation aligned with `Config::extra_mount_partitions`; filesystem eligibility remains a backend check.
- `module/` is the install-package template. `xtask` combines it with architecture binaries and the WebUI into ignored `output/*.zip` artifacts.

## Generated Side Effects

- Any root Cargo build runs `build.rs`, which regenerates ignored `module/module.prop` from `Cargo.toml` and Git history and sets the compile-time `MODULE_ID`.
- `pnpm run build` in `webui/` writes directly to ignored `module/webroot/`; it does not use a local `webui/dist/` directory.
- Do not edit or commit `module/module.prop`, `module/webroot/`, `module/bin/`, `output/`, or `target/`; they are generated/ignored.
- Keep both `Cargo.lock` and `webui/pnpm-lock.yaml` updated when their respective dependencies change.

## Commands

- Host Rust tests: `cargo test`
- Extra mount regression tests: `cargo test extra_mount`
- One exact Rust test: `cargo test parser::tests::parse_bind_valid -- --exact`
- Rust formatting check: `cargo fmt --all -- --check`
- CI-equivalent Android lint: `cargo ndk -t arm64-v8a -t armeabi-v7a clippy -- -D warnings`
- Frontend checks, from `webui/`: `pnpm run lint`, `pnpm run typecheck`, `pnpm run build`
- Shell changes under `module/`: run ShellCheck; CI intentionally disables SC2034, SC2086, and SC2115.
- Package one target: `cargo xtask build -t arm64|armv7|x86-64|universal`; artifacts are `output/magic_mount_rs-*.zip`.

## Testing And Runtime Gotchas

- Files under `tests/unit/` are included into the binary with `#[path = ...]`; test names therefore use module paths such as `config::tests::...`, not integration-test targets.
- Host tests cover parsers, scanning, configuration, and path preparation, but not real tmpfs/bind/move mounts. Mount changes need an Android root environment and explicit manual verification.
- `/data/adb/magic_mount/config.toml` and `/data/adb/magic_mount/custom` are persistent runtime inputs. Default install values differ by platform: `module/config.toml` uses `KSU`, while `module/config_apatch.toml` uses `APatch`.
- `extra_mount` defaults to `[]` and reads `<module>/<partition>/`, unlike `partitions`, which handles entries under `<module>/system/`. The root target must be an existing directory, and `/system/<partition>` must have no entry, including dangling symlinks. Names must be nonempty single components other than `.`, `..`, or `system`, with no slash or NUL. Invalid or ineligible entries are logged and skipped.
- Modules with only top-level `extra_mount` content need no `system/` directory; `disable`, `remove`, and `skip_mount` markers still apply. Configuration changes require a reboot to affect mounts. Verify add/remove, duplicate prevention, invalid-name feedback, and save/reload in both WebUI themes; actual mounting requires a rooted Android device.
- The `modules` CLI command reads `/data/adb/magic_mount/scan.ret`, which the no-argument startup path refreshes; it is a cached snapshot, not an on-demand scan.
- `parser::COMMAND_LIST` and the include-file visited list are process-global. Tests that invoke recursive `file`/`add` parsing must isolate or clear parser state as existing parser tests do.
- Rust Clippy denies standard `HashMap`/`HashSet` and standard sync locks via `clippy.toml`; use `FxHashMap`/`FxHashSet` and `parking_lot` locks.
