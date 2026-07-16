# CI and Release Integrity Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make pull-request CI target `main` and make public releases require exact Cargo/tag version parity plus the complete quality workflow at the release commit.

**Architecture:** Keep `.github/workflows/ci.yml` as the single quality-gate definition and make it callable from `.github/workflows/release.yml`. The release workflow will enforce a fail-closed dependency chain from version parity through reusable CI and the existing build matrix to publication, while a guarded manual trigger provides a non-publishing test path.

**Tech Stack:** GitHub Actions YAML, Bash, Cargo metadata, `jq`, `yq`, `actionlint`, Cargo, Nix.

## Global Constraints

- Preserve `push.branches: ["**"]`, every existing CI job, validation command, runner, and matrix value.
- Preserve all immutable action commit SHAs.
- Keep workflow permissions read-only except for the existing `create-release` job's `contents: write` permission.
- Preserve the three existing release targets, artifact names, smoke tests, exact asset validation, and `SHA256SUMS` generation.
- Do not query or poll historical workflow runs.
- Do not change `Cargo.toml`, `Cargo.lock`, or the package version.
- Do not publish a release during negative-path testing.

---

## File structure

- `.github/workflows/ci.yml` — owns normal CI triggers and the complete reusable quality workflow.
- `.github/workflows/release.yml` — owns tag/manual triggers, version parity, the reusable quality dependency, release builds, and guarded publication.
- `docs/superpowers/specs/2026-07-15-release-integrity-design.md` — approved behavioral design; do not modify during implementation.

### Task 1: Correct the PR base branch and expose reusable CI

**Files:**
- Modify: `.github/workflows/ci.yml:3-8`
- Test: focused `yq` policy assertions and `actionlint`

**Interfaces:**
- Consumes: GitHub `push`, `pull_request`, and `workflow_call` events.
- Produces: the unchanged CI job set as a same-repository reusable workflow at `./.github/workflows/ci.yml`.

- [ ] **Step 1: Run the focused assertion before editing**

```bash
yq -e '
  .on.pull_request.branches == ["main"] and
  .on.push.branches == ["**"] and
  (.on | has("workflow_call"))
' .github/workflows/ci.yml
```

Expected: exit 1 because the PR filter is `trunk` and `workflow_call` is absent.

- [ ] **Step 2: Replace the CI trigger block**

Replace `.github/workflows/ci.yml:3-8` with:

```yaml
on:
  push:
    branches: ["**"]
  pull_request:
    branches: [main]
  workflow_call:
```

Do not change any content below the trigger block.

- [ ] **Step 3: Run the focused assertion after editing**

```bash
yq -e '
  .on.pull_request.branches == ["main"] and
  .on.push.branches == ["**"] and
  (.on | has("workflow_call")) and
  (.jobs | keys == ["current-toolchain", "dependency-policy", "security-audit", "test"]) and
  (.jobs.test.strategy.matrix.os == ["ubuntu-latest", "macos-15"]) and
  (.jobs.test.strategy.matrix.rust == ["stable"])
' .github/workflows/ci.yml
```

Expected: exit 0 and print `true`.

- [ ] **Step 4: Validate the CI workflow syntax**

```bash
nix shell nixpkgs#actionlint -c actionlint .github/workflows/ci.yml
```

Expected: exit 0 with no workflow diagnostics.

- [ ] **Step 5: Commit the independently testable CI change**

```bash
git add .github/workflows/ci.yml
git commit -m "fix pull request CI branch"
```

### Task 2: Add version parity and reusable quality gates to releases

**Files:**
- Modify: `.github/workflows/release.yml:3-191`
- Test: focused `yq` graph assertions, extracted Bash version checks, and `actionlint`

**Interfaces:**
- Consumes: `github.ref_name` for tag pushes, `inputs.release_tag` for manual runs, root-package metadata from `Cargo.toml`, and `./.github/workflows/ci.yml` from Task 1.
- Produces: `version-parity` and `quality-gate` jobs whose successful completion is required by `build-and-release`; `create-release` remains reachable only from a real `v*` tag push.

- [ ] **Step 1: Run the release graph assertion before editing**

```bash
yq -e '
  (.on | has("workflow_dispatch")) and
  (.jobs | has("version-parity")) and
  (.jobs | has("quality-gate")) and
  (.jobs["quality-gate"].needs == "version-parity") and
  (.jobs["quality-gate"].uses == "./.github/workflows/ci.yml") and
  (.jobs["build-and-release"].needs == "quality-gate") and
  (.jobs["create-release"]["if"] == "github.event_name == '\''push'\'' && github.ref_type == '\''tag'\'' && startsWith(github.ref_name, '\''v'\'')")
' .github/workflows/release.yml
```

Expected: exit 1 because the manual trigger and gate jobs are absent.

- [ ] **Step 2: Add the guarded manual trigger**

Replace `.github/workflows/release.yml:3-7` with:

```yaml
on:
  push:
    tags:
      - "v*"
  workflow_dispatch:
    inputs:
      release_tag:
        description: Candidate release tag to validate without publishing
        required: true
        type: string
```

- [ ] **Step 3: Add the version and quality jobs**

Insert the following jobs immediately after the existing workflow-level
`permissions: contents: read` block and before `build-and-release`:

```yaml
  version-parity:
    name: Verify Version Parity
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@93cb6efe18208431cddfb8368fd83d5badbf9bfd # v5

      - name: Verify tag matches package version
        shell: bash
        env:
          RELEASE_TAG: ${{ github.event_name == 'workflow_dispatch' && inputs.release_tag || github.ref_name }}
        run: |
          set -euo pipefail

          manifest_path="$(pwd -P)/Cargo.toml"
          package_version="$(
            cargo metadata --locked --offline --no-deps --format-version 1 |
              jq -er --arg manifest "$manifest_path" '
                .packages
                | map(select(.manifest_path == $manifest))
                | if length == 1 then
                    .[0].version
                  else
                    error("expected exactly one root package")
                  end
              '
          )"
          expected_tag="v${package_version}"

          if [ "$RELEASE_TAG" != "$expected_tag" ]; then
            echo "release tag $RELEASE_TAG does not match package version $package_version" >&2
            exit 1
          fi

          echo "release tag $RELEASE_TAG matches package version $package_version"

  quality-gate:
    name: Quality Gate
    needs: version-parity
    permissions:
      contents: read
    uses: ./.github/workflows/ci.yml
```

- [ ] **Step 4: Gate the build and publication jobs**

Add this field directly below `build-and-release:`:

```yaml
    needs: quality-gate
```

Keep the existing `create-release.needs: build-and-release` field and add this
job-level condition directly below it:

```yaml
    if: github.event_name == 'push' && github.ref_type == 'tag' && startsWith(github.ref_name, 'v')
```

Do not change any existing build, artifact, checksum, smoke-test, permission,
or `gh release create` step.

- [ ] **Step 5: Run the release graph assertion after editing**

```bash
yq -e '
  (.on.push.tags == ["v*"]) and
  (.on.workflow_dispatch.inputs.release_tag.required == true) and
  (.on.workflow_dispatch.inputs.release_tag.type == "string") and
  (.jobs["version-parity"].permissions == null) and
  (.jobs["quality-gate"].needs == "version-parity") and
  (.jobs["quality-gate"].permissions.contents == "read") and
  (.jobs["quality-gate"].uses == "./.github/workflows/ci.yml") and
  (.jobs["build-and-release"].needs == "quality-gate") and
  (.jobs["create-release"].needs == "build-and-release") and
  (.jobs["create-release"]["if"] == "github.event_name == '\''push'\'' && github.ref_type == '\''tag'\'' && startsWith(github.ref_name, '\''v'\'')") and
  (.jobs["create-release"].permissions.contents == "write")
' .github/workflows/release.yml
```

Expected: exit 0 and print `true`.

- [ ] **Step 6: Exercise matching and mismatched versions locally**

```bash
version_check="$(
  yq -r '.jobs["version-parity"].steps[] | select(.name == "Verify tag matches package version").run' \
    .github/workflows/release.yml
)"

RELEASE_TAG=v0.0.5 bash -c "$version_check"

if RELEASE_TAG=v0.0.6 bash -c "$version_check"; then
  echo "mismatched release tag unexpectedly passed" >&2
  exit 1
fi
```

Expected: `v0.0.5` succeeds, `v0.0.6` prints the mismatch diagnostic and exits
nonzero, and the wrapper command exits 0.

- [ ] **Step 7: Validate both workflow files**

```bash
nix shell nixpkgs#actionlint -c actionlint
```

Expected: exit 0 with no workflow diagnostics.

- [ ] **Step 8: Review the focused workflow diff**

```bash
git diff --check
git diff -- .github/workflows/ci.yml .github/workflows/release.yml
```

Expected: no whitespace errors; only the approved triggers, version job,
quality call, dependencies, and publication condition are new.

- [ ] **Step 9: Commit the independently testable release gate**

```bash
git add .github/workflows/release.yml
git commit -m "gate tagged releases"
```

### Task 3: Run the complete local verification gate

**Files:**
- Verify: `.github/workflows/ci.yml`
- Verify: `.github/workflows/release.yml`
- Verify: `Cargo.toml`
- Verify: `Cargo.lock`

**Interfaces:**
- Consumes: the two committed workflow changes from Tasks 1 and 2.
- Produces: evidence that workflow syntax, Rust quality, Nix evaluation, dependency policy, and security policy remain green.

- [ ] **Step 1: Validate GitHub Actions and the final policy graph**

```bash
nix shell nixpkgs#actionlint -c actionlint

yq -e '
  .on.pull_request.branches == ["main"] and
  .on.push.branches == ["**"] and
  (.on | has("workflow_call"))
' .github/workflows/ci.yml

yq -e '
  (.jobs["quality-gate"].needs == "version-parity") and
  (.jobs["build-and-release"].needs == "quality-gate") and
  (.jobs["create-release"].needs == "build-and-release") and
  (.jobs["create-release"].permissions.contents == "write")
' .github/workflows/release.yml
```

Expected: all three commands exit 0.

- [ ] **Step 2: Run the repository's Rust quality commands**

```bash
cargo fmt -- --check
cargo clippy --all-targets --all-features --locked -- -D warnings
cargo test --locked --verbose
```

Expected: formatting, strict Clippy, unit tests, integration tests, and doc tests
all pass.

- [ ] **Step 3: Run Nix and dependency/security policy checks**

```bash
nix flake check --no-build --all-systems
nix develop -c cargo deny --locked check
cargo audit
```

Expected: all declared Nix systems evaluate; dependency policy and RustSec
audit pass.

- [ ] **Step 4: Prove prohibited files and hardened release content stayed unchanged**

```bash
git diff HEAD~2..HEAD -- Cargo.toml Cargo.lock
git diff HEAD~2..HEAD -- .github/workflows/ci.yml .github/workflows/release.yml
rg -n '^\s*uses:' .github/workflows/ci.yml .github/workflows/release.yml
git status --short --branch
```

Expected:

- no `Cargo.toml` or `Cargo.lock` diff;
- all third-party `uses:` entries still use 40-character commit SHAs;
- the release target matrix, exact asset names, checksum block, smoke tests, and
  `contents: write` boundary are unchanged;
- the branch is ahead only by the intended local commits and the worktree is
  clean.

### Task 4: Exercise remote negative paths after landing

**Files:**
- Verify remotely: `.github/workflows/release.yml` on the default branch

**Interfaces:**
- Consumes: the dispatchable release workflow after it exists on `main` and explicit authorization to push/dispatch remote workflows.
- Produces: GitHub Actions run evidence that version mismatch and failed quality checks cannot upload artifacts or publish a release.

- [ ] **Step 1: Dispatch a mismatched tag from healthy `main`**

```bash
gh workflow run release.yml --ref main -f release_tag=v0.0.6
```

Expected: `version-parity` fails because `Cargo.toml` still reports `0.0.5`;
quality, build, upload, and publication jobs are skipped.

- [ ] **Step 2: Inspect the mismatch run and prove it uploaded nothing**

```bash
run_id="$(
  gh run list --workflow release.yml --event workflow_dispatch --limit 1 \
    --json databaseId --jq '.[0].databaseId'
)"

gh run view "$run_id" --json conclusion,jobs,url
gh api "repos/Cbeck527/my-prompt/actions/runs/$run_id/artifacts" --jq '.total_count'
```

Expected: the run conclusion is `failure`, every downstream release job is
skipped, and the artifact count is `0`.

- [ ] **Step 3: Create and dispatch an authorized temporary failure branch**

Run only after receiving explicit authorization for the temporary remote
branch:

```bash
git switch -c cb/release-gate-negative-test
```

Use `apply_patch` to apply this deliberately unformatted, behavior-preserving
change:

```diff
*** Begin Patch
*** Update File: src/lib.rs
@@
-pub use prompt::{CLAUDE_FORMAT, PROMPT_FORMAT, PromptModule, TRANSIENT_FORMAT, render_prompt};
+pub use prompt::{CLAUDE_FORMAT,
+PROMPT_FORMAT, PromptModule, TRANSIENT_FORMAT, render_prompt};
*** End Patch
```

Then publish and dispatch only the temporary branch:

```bash
git add src/lib.rs
git commit -m "test release quality gate failure"
git push -u origin cb/release-gate-negative-test
gh workflow run release.yml \
  --ref cb/release-gate-negative-test \
  -f release_tag=v0.0.5
```

Expected: `version-parity` succeeds, the reusable CI call fails at formatting,
and every release build, upload, and publication job is skipped.

- [ ] **Step 4: Inspect the quality-failure run and prove it uploaded nothing**

```bash
failure_run_id="$(
  gh run list \
    --workflow release.yml \
    --event workflow_dispatch \
    --branch cb/release-gate-negative-test \
    --limit 1 \
    --json databaseId \
    --jq '.[0].databaseId'
)"

gh run watch "$failure_run_id"
gh run view "$failure_run_id" --json conclusion,jobs,url
gh api "repos/Cbeck527/my-prompt/actions/runs/$failure_run_id/artifacts" --jq '.total_count'
```

Expected: the quality gate is `failure`, every downstream release job is
skipped, and the artifact count is `0`. `gh run watch` may exit nonzero because
the failure is intentional.

- [ ] **Step 5: Remove the temporary test branch after recording the run URL**

```bash
git switch main
git push origin --delete cb/release-gate-negative-test
git branch -D cb/release-gate-negative-test
```

Expected: no intentional formatting failure remains locally or remotely, and
no GitHub release was created by either manual run.
