# CB-159: Rust quality gate design

## Goal

Restore a green, complete, and reproducible Stable Rust quality gate without
changing the MSRV job or suppressing lint diagnostics.

## Scope

- Refactor `find_upward_from` in `src/modules/utils.rs` to propagate the
  missing-parent case with `?`.
- Resolve all diagnostics from the strict Clippy command in production and
  test code with idiomatic source changes.
- Add focused unit coverage for `find_upward_from` finding an ancestor and
  returning `None` when the file is absent.
- Make the Stable CI job run the canonical strict Clippy command and lock its
  build and test dependency resolution.
- Document the canonical Clippy command in `AGENTS.md`.

The MSRV `cargo check` command remains unchanged.

## Implementation design

### Rust source and tests

`find_upward_from` will keep its public signature and traversal behavior. At
the filesystem root, the missing parent will end the search by propagating
`None` with `?` rather than manually matching the option.

The remaining strict-Clippy diagnostics are test-only readability and style
findings. They will be resolved directly: digit separators in large literals,
character patterns, captured format arguments, and range containment. No
`allow` attributes or lint-policy changes will be added.

The new `utils.rs` tests will use a temporary directory tree. One test will
prove that a marker in an ancestor is found from a descendant. The other will
prove that a missing marker returns `None`.

### CI and documentation

The Stable test job in `.github/workflows/ci.yml` will use this exact command:

```sh
cargo clippy --all-targets --all-features --locked -- -D warnings
```

Both debug and release `cargo build` commands, plus `cargo test`, will gain
`--locked`. Formatting remains unchanged. The MSRV job continues to run its
existing `cargo check --verbose` command.

`AGENTS.md` will replace its short Clippy command with the same canonical
strict command so local and hosted validation agree.

## Validation

Run the following after implementation:

```sh
cargo fmt -- --check
cargo clippy --all-targets --all-features --locked -- -D warnings
cargo test --locked --verbose
```

Inspect the CI workflow to confirm the Stable job invokes the exact strict
Clippy command and that its build and test commands use `--locked`.

## Non-goals

- Pinning Stable Rust or changing the rolling-Stable CI policy.
- Changing the MSRV job.
- Adding a task runner, Cargo alias, or new CI job.
- Broad or local lint suppressions.
