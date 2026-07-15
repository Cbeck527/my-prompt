# Repository Guidelines

## Project Structure & Module Organization

`my-prompt` is a Rust library and CLI. The executable entry point is
`src/main.rs`; reusable prompt rendering and public types live in `src/lib.rs`,
`src/prompt.rs`, and `src/module_trait.rs`. Individual prompt components are
organized under `src/modules/` (for example, `git.rs`, `path.rs`, and
`claude.rs`). Integration tests are in `tests/`, with focused unit tests kept
next to their implementation behind `#[cfg(test)]`. The Fish setup helper is
`etc/my-prompt.fish`; `README.md` documents user-facing setup and behavior.

## Build, Test, and Development Commands

- `cargo build` — compile the debug binary and library.
- `cargo build --release` — build the optimized, stripped release binary.
- `cargo test --verbose` — run unit and integration tests.
- `cargo fmt -- --check` — verify standard Rust formatting without changing files.
- `cargo clippy --all-targets --all-features --locked -- -D warnings` — run the repository's denied pedantic lints across every target and feature.
- `cargo audit` — check dependencies for known security advisories (used in CI).
- `cargo run -- --debug` — render the prompt and print module/timing diagnostics.

For local benchmarking, use the CLI's `--bench` flag. `RUSTFLAGS="-C
target-cpu=native" cargo build --release` is appropriate only for binaries
that will run on the same CPU family.

## Coding Style & Naming Conventions

Use `rustfmt` defaults, four-space indentation, and idiomatic Rust naming:
`snake_case` for functions/modules, `UpperCamelCase` for types, and
`SCREAMING_SNAKE_CASE` for constants. Keep prompt modules small and make
backend-specific behavior explicit. `Cargo.toml` denies Rust warnings and
Clippy's pedantic lint group, so new code should be warning-free rather than
silencing lints broadly.

## Testing Guidelines

Name tests after observable behavior, such as `test_render_prompt_full`.
Add unit tests beside module logic and integration tests in `tests/` when
checking public API behavior. Run `cargo test` locally; no separate coverage
threshold is configured.

## Commit & Pull Request Guidelines

Use short, imperative, lowercase commit subjects consistent with history, for
example `fix staged-change detection` or `update dependencies`. Keep commits
focused. Pull requests should describe the user-visible change, explain the
implementation briefly, list validation commands and results, and link a
related issue when one exists. Include representative prompt output for
formatting or CLI behavior changes.

## Release and Security Notes

Tagged `v*` pushes trigger multi-platform release builds in GitHub Actions.
Do not commit `target/`, generated binaries, credentials, or local environment
files. Run `cargo audit` when changing dependencies and update `Cargo.lock`
with dependency changes.
