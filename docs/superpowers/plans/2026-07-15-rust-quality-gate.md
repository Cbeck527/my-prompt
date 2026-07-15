# CB-159 Rust Quality Gate Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Restore a green, complete, reproducible Stable Rust quality gate without changing the MSRV job.

**Architecture:** Keep the production behavior unchanged while expressing the end-of-directory traversal through `Option` propagation. Resolve every current strict-Clippy diagnostic directly in source and tests, then make the Stable CI workflow and contributor guidance run the same canonical lint command with locked dependency resolution.

**Tech Stack:** Rust 2024, Cargo, Clippy, GitHub Actions, `tempfile` dev-dependency.

## Global Constraints

- Keep the rolling-Stable toolchain policy; do not pin Rust.
- Do not change the MSRV job or its `cargo check --verbose` command.
- Do not add `allow` attributes or weaken the repository's denied lint policy.
- Do not add dependencies or change `Cargo.toml` or `Cargo.lock`.
- Use the canonical lint command exactly: `cargo clippy --all-targets --all-features --locked -- -D warnings`.
- Keep `find_upward_from`'s public signature and filesystem-root behavior unchanged.

---

## File structure

- `src/modules/utils.rs` — upward file search implementation and focused unit tests.
- `src/modules/claude.rs` — readable numeric literals in existing tests.
- `src/modules/fail.rs` — character-pattern assertions in existing tests.
- `src/modules/path.rs` — idiomatic format arguments and character pattern in existing tests.
- `src/modules/time.rs` — idiomatic format arguments and inclusive-range assertion in existing tests.
- `src/main.rs` — readable context-window literals in existing parser tests.
- `.github/workflows/ci.yml` — Stable lint command and locked debug, test, and release builds.
- `AGENTS.md` — canonical local Clippy validation command.

### Task 1: Preserve and cover upward file search

**Files:**
- Modify: `src/modules/utils.rs:11-25`
- Test: `src/modules/utils.rs` (new `#[cfg(test)]` module after `find_upward_from`)

**Interfaces:**
- Consumes: `find_upward_from(start_dir: &Path, name: &str) -> Option<PathBuf>`.
- Produces: the same `Option<PathBuf>` result for existing callers, including `DirenvModule`.

- [ ] **Step 1: Add focused behavior tests**

Append this complete test module to `src/modules/utils.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn finds_file_in_ancestor_directory() {
        let temp_dir = tempfile::tempdir().expect("create temp dir");
        let ancestor = temp_dir.path().join("ancestor");
        let descendant = ancestor.join("descendant");
        let marker = ancestor.join(".envrc");

        fs::create_dir_all(&descendant).expect("create descendant directory");
        fs::write(&marker, "").expect("create marker file");

        assert_eq!(find_upward_from(&descendant, ".envrc"), Some(marker));
    }

    #[test]
    fn returns_none_when_file_is_absent() {
        let temp_dir = tempfile::tempdir().expect("create temp dir");
        let descendant = temp_dir.path().join("ancestor/descendant");

        fs::create_dir_all(&descendant).expect("create descendant directory");

        assert_eq!(find_upward_from(&descendant, ".envrc"), None);
    }
}
```

- [ ] **Step 2: Run the new tests before the refactor**

Run: `cargo test modules::utils::tests --locked --verbose`

Expected: both new tests pass. The behavior already exists; these tests protect it while the implementation becomes idiomatic.

- [ ] **Step 3: Replace the manual parent match with `?`**

Replace the loop body in `find_upward_from` with this complete implementation:

```rust
    loop {
        let potential = current.join(name);
        if potential.exists() {
            return Some(potential);
        }

        let parent = current.parent()?;
        current = parent.to_path_buf();
    }
```

- [ ] **Step 4: Verify the implementation and production lint target**

Run: `cargo test modules::utils::tests --locked --verbose`

Expected: both upward-search tests pass.

Run: `cargo clippy --lib --locked -- -D warnings`

Expected: passes. The `clippy::question_mark` error is gone while test-target diagnostics remain for Task 2.

- [ ] **Step 5: Commit the behavior-preserving refactor**

```bash
git add src/modules/utils.rs
git commit -m "refactor upward file search"
```

### Task 2: Resolve strict test-target diagnostics

**Files:**
- Modify: `src/modules/claude.rs:94-109, 121-150`
- Modify: `src/modules/fail.rs:58-71`
- Modify: `src/modules/path.rs:149-202`
- Modify: `src/modules/time.rs:51-78`
- Modify: `src/main.rs:232,258,275,284,293,311,327,344,372,408,451`

**Interfaces:**
- Consumes: existing module test output and the repository's denied pedantic lint policy.
- Produces: warning-free code under the canonical full-target Clippy command without changing prompt output or test assertions.

- [ ] **Step 1: Reproduce the remaining full-target lint failures**

Run: `cargo clippy --all-targets --all-features --locked -- -D warnings`

Expected: fails on the current test-only diagnostics: unreadable numeric literals, single-character string patterns, uninlined format arguments, and manual range containment.

- [ ] **Step 2: Apply the direct source edits**

Make these exact replacements:

```rust
// src/modules/claude.rs
assert_eq!(format_tokens(100_000), "100k");
assert_eq!(format_tokens(200_000), "200k");
context_total: 200_000,

// src/modules/fail.rs
assert!(output.contains('['));
assert!(output.contains(']'));

// src/modules/path.rs
value.ends_with(' '),
"Expected trailing space, got: {value}",
"Expected path to start with ~/my_prompt_test_project_, got: {path}",
let base = home.join(format!("my_prompt_test_base_{unique}"));
"Expected path to start with ~/my_prompt_test_base_, got: {path}",
"Expected path to end with /alpine, got: {path}",

// src/modules/time.rs
"Expected plain [hh:MMAM/PM] format, got: {output}",
(1..=12).contains(&hour),
"12h format hour should be 1-12, got: {hour}",

// src/main.rs
"context_window_size": 200_000,
assert_eq!(session.context_total, 200_000);
```

Update both `context_total` literals in `src/modules/claude.rs` and every `200000` literal at the listed `src/main.rs` locations to `200_000`. Preserve every assertion's expected value and message text apart from captured-format syntax.

- [ ] **Step 3: Run the complete local quality gate**

Run: `cargo fmt -- --check`

Expected: passes.

Run: `cargo clippy --all-targets --all-features --locked -- -D warnings`

Expected: passes with no warnings.

Run: `cargo test --locked --verbose`

Expected: all unit, integration, and doc tests pass, including the new upward-search tests.

- [ ] **Step 4: Commit the lint cleanup**

```bash
git add src/main.rs src/modules/claude.rs src/modules/fail.rs src/modules/path.rs src/modules/time.rs
git commit -m "fix strict clippy diagnostics"
```

### Task 3: Make Stable CI and contributor guidance reproducible

**Files:**
- Modify: `.github/workflows/ci.yml:44-55`
- Modify: `AGENTS.md:15-20`

**Interfaces:**
- Consumes: the canonical full-target Clippy command verified in Task 2.
- Produces: Stable CI and contributor instructions that invoke the same lint gate and lock build/test dependency resolution.

- [ ] **Step 1: Update the Stable CI commands**

Replace the four `run` values in the Stable test job with:

```yaml
      - name: Run clippy
        run: cargo clippy --all-targets --all-features --locked -- -D warnings
        if: matrix.rust == 'stable'

      - name: Build
        run: cargo build --locked --verbose

      - name: Run tests
        run: cargo test --locked --verbose

      - name: Build release
        run: cargo build --release --locked --verbose
```

Do not change the formatting step or the `msrv` job.

- [ ] **Step 2: Update the contributor-facing Clippy command**

Replace the `AGENTS.md` Clippy entry with:

```markdown
- `cargo clippy --all-targets --all-features --locked -- -D warnings` — run the repository's denied pedantic lints across every target and feature.
```

- [ ] **Step 3: Verify workflow alignment and the final local gate**

Run: `rg -n -F 'cargo clippy --all-targets --all-features --locked -- -D warnings' .github/workflows/ci.yml AGENTS.md`

Expected: one match in the Stable CI job and one match in `AGENTS.md`.

Run: `rg -n -F 'cargo build --locked --verbose' .github/workflows/ci.yml`

Expected: one match in the debug build step.

Run: `rg -n -F 'cargo test --locked --verbose' .github/workflows/ci.yml`

Expected: one match in the test step.

Run: `rg -n -F 'cargo build --release --locked --verbose' .github/workflows/ci.yml`

Expected: one match in the release build step.

Run: `cargo fmt -- --check && cargo clippy --all-targets --all-features --locked -- -D warnings && cargo test --locked --verbose`

Expected: all three commands pass.

- [ ] **Step 4: Commit the reproducible CI gate**

```bash
git add .github/workflows/ci.yml AGENTS.md
git commit -m "lock rust quality gate"
```

### Task 4: Final review

**Files:**
- Verify: `src/modules/utils.rs`
- Verify: `src/modules/claude.rs`
- Verify: `src/modules/fail.rs`
- Verify: `src/modules/path.rs`
- Verify: `src/modules/time.rs`
- Verify: `src/main.rs`
- Verify: `.github/workflows/ci.yml`
- Verify: `AGENTS.md`

**Interfaces:**
- Consumes: completed Tasks 1-3.
- Produces: a clean working tree and a change set that meets every CB-159 acceptance criterion.

- [ ] **Step 1: Review the final diff for prohibited scope changes**

Run: `git diff HEAD~3..HEAD -- Cargo.toml Cargo.lock .github/workflows/ci.yml src/modules AGENTS.md`

Expected: no changes to `Cargo.toml` or `Cargo.lock`; the MSRV `cargo check --verbose` command is unchanged; no `allow` attributes were added.

- [ ] **Step 2: Verify the committed quality gate**

Run: `cargo fmt -- --check && cargo clippy --all-targets --all-features --locked -- -D warnings && cargo test --locked --verbose`

Expected: all commands pass.

- [ ] **Step 3: Confirm repository status**

Run: `git status --short`

Expected: no output.
