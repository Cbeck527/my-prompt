# Repository Guidelines

## Project Priorities

`my-prompt` is a personal project in a private GitHub repository and is used
only by me. Optimize for the project's actual needs rather than hypothetical
downstream users. Do not preserve compatibility just for its own sake:
breaking changes are acceptable when they are The Right Thing To Do. Update
affected documentation and tests, and call out user-visible behavior changes
clearly.

The prompt runs frequently, so speed matters. Prefer a simple, readable,
pragmatic implementation until measurement shows that performance is a real
problem. Use `--debug` and `--bench` to measure prompt behavior before adding
complexity or optimizing speculative paths.

Keep code easy to scan. Prefer boring, explicit solutions over clever
abstractions, and keep each change focused on the requested behavior.

## Project Structure & Module Organization

`my-prompt` is a Rust CLI with no library target. The executable entry point is
`src/main.rs`, which declares the internal prompt modules. Reusable rendering
logic lives in `src/prompt.rs` and `src/module_trait.rs`; individual prompt
components are organized under `src/modules/` (for example, `git.rs`,
`path.rs`, and `claude.rs`). Black-box CLI tests are in `tests/`, with focused
unit tests kept next to their implementation behind `#[cfg(test)]`. The Fish
setup helper is `src/init/my-prompt.fish`; `README.md` documents user-facing setup
and behavior.

## Build, Test, and Development Commands

The minimum supported Rust version is 1.96. CI verifies Rust 1.96 alongside the
current stable toolchain. Raise `package.rust-version` only when required by the
project or its dependencies, and update this guidance and `README.md` together.

- `cargo build --locked --verbose` — compile the debug binary.
- `cargo build --release --locked --verbose` — build the optimized, stripped release binary.
- `cargo test --locked --verbose` — run unit and integration tests.
- `cargo fmt -- --check` — verify standard Rust formatting without changing files.
- `cargo clippy --all-targets --all-features --locked -- -D warnings` — run the repository's denied pedantic lints across every target and feature.
- `nix flake check --no-build --all-systems` — verify the declared Nix systems.
- `nix develop -c cargo deny --locked check` — check dependency policy in the project dev shell.
- `cargo audit` — check dependencies for known security advisories (used in CI).
- `cargo run -- --no-color` — render the prompt directly for a readable smoke test.
- `cargo run -- --debug` — render the prompt and print module/timing diagnostics.
- `cargo run -- --bench` — render the prompt repeatedly and report timing statistics.

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

Name tests after observable behavior, such as
`render_prompt_returns_output_for_default_format`.
Add unit tests beside module logic and integration tests in `tests/` when
checking CLI behavior. This project is sufficiently simple that tests
should stay proportional to the change; do not pursue coverage or elaborate
harnesses for their own sake. For a straightforward CLI or prompt-formatting
change, it may be faster and clearer to build or run the binary and inspect
representative output. Add focused tests when they clarify non-trivial logic or
protect against a likely regression, and use the full test suite for broader,
release, or cross-module changes.

## Commit & Pull Request Guidelines

Use short, imperative, lowercase commit subjects consistent with history, for
example `fix staged-change detection` or `update dependencies`. Keep commits
focused. Since this is a private, single-user project, a breaking change does
not need a compatibility layer when it is the clearest solution; document the
new behavior instead. Pull requests should describe the user-visible change,
explain the implementation briefly, list validation commands and results, and
link a related issue when one exists. Include representative prompt output for
formatting or CLI behavior changes.

## Release and Security Notes

Tagged `v*` pushes trigger release builds for Linux x86_64 (GNU and musl) and
macOS on Apple Silicon (arm64). Intel macOS is unsupported.
Do not commit `target/`, generated binaries, credentials, or local environment
files. Run `cargo audit` when changing dependencies and update `Cargo.lock`
with dependency changes.
