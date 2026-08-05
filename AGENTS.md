# Repository Guidelines

## Project priorities

`my-prompt` is Chris Becker's personal Rust shell prompt and a public portfolio
project. Optimize for its real behavior, clarity, and reliability rather than
hypothetical downstream configuration needs. Forks are welcome.

Do not preserve backward compatibility for its own sake. Breaking changes are
acceptable when they are the clearest long-term design; update documentation and
tests, add a changelog entry, and call them out explicitly. Remove obsolete paths
instead of adding compatibility layers or migrations.

The prompt runs frequently, so speed matters. Prefer simple, readable code until
measurement shows a real problem. Use `--debug`, the built-in `bench` subcommand,
and `hyperfine` before adding performance complexity.

The prompt order, colors, `$` character, username alias, CLI-only shape, and
best-effort omission of unavailable modules are intentional product choices.

## Structure

The project has one binary and no library target. `src/main.rs` owns CLI and
process I/O; `src/claude.rs` parses Claude status input; `src/prompt.rs` owns
ordered parallel rendering; `src/module_trait.rs` defines the render snapshot
and module trait; and prompt components live under `src/modules/`. Black-box CLI
tests are in `tests/`. Fish setup is embedded from `src/init/my-prompt.fish`;
shared release smoke and packaging logic lives in `scripts/`.

Nix packaging lives in `flake.nix` and `nix/`. Public behavior belongs in
`README.md`; release-visible changes belong in `CHANGELOG.md`.

## Build and validation

Rust 1.96 is the minimum supported version. If it changes, keep `Cargo.toml`, CI,
README, and this file aligned.

- `cargo build --locked --verbose`
- `cargo build --release --locked --verbose`
- `cargo test --locked --verbose`
- `cargo fmt -- --check`
- `cargo clippy --all-targets --all-features --locked -- -D warnings`
- `cargo check --all-targets --locked --verbose` with Rust 1.96
- `nix flake check --no-build --all-systems`
- `nix flake check --print-build-logs`
- `nix develop -c cargo deny --locked check`
- `nix develop -c cargo audit`
- `nix develop -c actionlint`
- `nix develop -c scripts/generate-third-party-licenses.sh /tmp/THIRD_PARTY_LICENSES.html`
- `diff -u THIRD_PARTY_LICENSES.html /tmp/THIRD_PARTY_LICENSES.html`
- `cargo run --locked -- --no-color`
- `cargo run --locked -- --debug`
- `cargo run --release --locked -- bench --no-color`

The strict Clippy command is the authoritative Rust lint gate. Dependency
notices must be regenerated when `Cargo.lock` changes.

## Rust style

Use rustfmt defaults and idiomatic naming. Keep modules small and backend-specific
behavior explicit. Prefer borrowing to cloning, static dispatch where practical,
checked error propagation, and pure helpers for logic that would otherwise need
global process mutation in tests. Production code forbids `unsafe`.

Warnings and Clippy pedantic lints are denied. Fix findings instead of silencing
them broadly. When a lint exception is justified, use a narrow `#[expect(...,
reason = "...")]`.

Keep tests proportional. Put focused unit tests beside implementation logic and
use integration tests for observable CLI/Fish behavior. A direct smoke run is
often clearer than an elaborate harness for straightforward rendering changes.

## Nix and release policy

Keep flake inputs pinned and make shared inputs follow `nixpkgs`. All declared
systems must evaluate; the native system must build both the package and Home
Manager activation check.

Tagged `v*` pushes publish versioned archives for Linux x86_64 GNU, Linux x86_64
musl, and Apple Silicon macOS. Intel macOS is unsupported. Release archives must
contain the binary, `LICENSE`, `THIRD_PARTY_NOTICES.md`, and
`THIRD_PARTY_LICENSES.html`. macOS binaries must be Developer ID signed and
notarized. Published archives require checksums and GitHub build provenance.

Do not commit `target/`, `.direnv/`, generated binaries, credentials, signing
material, or local environment files.

## Commits and pull requests

Use short, imperative, lowercase commit subjects. Keep commits focused. Pull
requests should state the user-visible change, explain the implementation and
trade-offs, list validation commands and outcomes, identify breaking changes,
and include representative output when rendering changes.
