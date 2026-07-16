# Contributing to `meta-magic_mount-rs`

Thank you for contributing!

This document explains how to prepare your environment, make changes, and submit pull requests that are easy to review.

---

## Table of contents

- [Code of conduct](#code-of-conduct)
- [Ways to contribute](#ways-to-contribute)
- [Development setup](#development-setup)
- [Build and run](#build-and-run)
- [Project layout](#project-layout)
- [Coding guidelines](#coding-guidelines)
- [Testing and validation](#testing-and-validation)
- [Commit and PR guidelines](#commit-and-pr-guidelines)
- [Review expectations](#review-expectations)

---

## Code of conduct

- Be respectful and constructive in issues and pull requests.
- Assume good intent and focus on technical outcomes.

---

## Ways to contribute

You can help by:

- Reporting bugs (with reproduction steps and logs).
- Proposing or implementing features.
- Improving docs, examples, and developer tooling.
- Reviewing open pull requests.

For larger changes (new behavior, refactors, architecture-impacting work), open an issue first so we can align on scope.

---

## Development setup

The toolchain used by CI is:

- Rust nightly toolchain
- Android NDK r29
- `cargo-ndk`
- Node.js 24 or later
- `pnpm` 11 (for `webui`; the exact version is declared in `webui/package.json`)

Recommended environment variables:

```bash
export ANDROID_NDK_HOME=<path/to/ndk>
export ANDROID_NDK_ROOT=$ANDROID_NDK_HOME
```

---

## Build and run

From the repository root:

```bash
cargo xtask build
```

To build a specific package variant, use one of the targets exercised by CI:

```bash
cargo xtask build -t arm64
cargo xtask build -t armv7
cargo xtask build -t x86-64
cargo xtask build -t universal
```

Build artifacts are written to `output/magic_mount_rs-*.zip`.

---

## Project layout

- `src/`: core Rust implementation.
- `xtask/`: build orchestration and helper tasks.
- `module/`: packaging scripts and module assets.
- `webui/`: frontend UI for configuration.
- `docs/`: user-facing documentation.

Keep changes scoped to the minimum set of files required for the problem.

---

## Coding guidelines

### Rust

Before submitting, run:

```bash
cargo fmt --all
cargo ndk -t arm64-v8a -t armeabi-v7a clippy -- -D warnings
```

Guidelines:

- Prefer small, focused functions and clear naming.
- Avoid unrelated refactors in feature/fix PRs.
- Do not add new dependencies unless necessary.
- Preserve backward compatibility unless the PR explicitly documents a breaking change.

### WebUI

In `webui/`:

```bash
pnpm install --frozen-lockfile
pnpm lint
pnpm typecheck
pnpm build
```

Follow the existing ESLint/Prettier conventions.

---

## Testing and validation

At minimum, validate the paths touched by your change.

Suggested checks:

```bash
cargo test
```

Changes to shell scripts under `module/` must also pass ShellCheck. New source and
configuration files must carry the GPL-v3 license header defined in
`licenserc.toml` unless they are explicitly excluded there.

When applicable, include:

- Manual verification steps.
- Environment/device details.
- Before/after behavior summary.

If fixing a bug, add or document a reproducible regression check whenever possible.

---

## Commit and PR guidelines

### Commits

- Follow the repository's established subject format:

  ```text
  <type>(<optional-scope>): <imperative description>
  ```

- Common types in the history include `build`, `ci`, `docs`, `test`, `style`,
  `chore`, and `deps`. Subsystem names such as `mount`,
  `parser`, `webui`, `xtask`, `module`, or `metainstall` may also be used as the
  type when they describe the change more clearly.
- Use a scope for a distinct area when useful, for example `deps(webui): ...`.
- Keep the subject concise, start it with a lowercase type, and do not end it
  with a period.
- Keep commits atomic and logically grouped.
- Keep release bookkeeping (`Cargo.toml`, `module/module.prop`, and `update/*`)
  out of ordinary feature and fix commits unless the change is part of a release.

### Pull requests

Please include:

1. **What changed** (summary)
2. **Why** (problem/context)
3. **How it was tested** (commands + results)
4. **Compatibility impact** (if any)
5. **Screenshots** (for visible WebUI changes)

Also:

- Link related issues (for example, `Fixes #123`).
- Keep PRs focused: one concern per PR when possible.

---

## Review expectations

- Maintainers may ask for additional tests, simplification, or scope reduction.
- Prefer follow-up commits over force-push during active review (unless asked otherwise).
- If feedback is unclear, ask for clarification directly in the PR.

Thanks again for helping improve `meta-magic_mount-rs`.
