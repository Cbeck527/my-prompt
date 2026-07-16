# CB-175 and CB-176: CI and release integrity design

## Goal

Make pull-request CI follow the repository's `main` integration branch and
make every public release prove version parity and pass the complete quality
gate at the exact release commit before any release artifact is uploaded.

## Confirmed failures

- `.github/workflows/ci.yml` filters pull requests to the retired `trunk`
  branch even though `main` is the repository's default branch.
- `.github/workflows/release.yml` accepts every `v*` tag without comparing it
  with the root package version in `Cargo.toml`.
- The release build and publication jobs have no dependency on CI. The v0.0.5
  release was published before CI for the same commit completed.
- CI filters push events by branch, so a tag push does not independently start
  the quality workflow.

## CI trigger design

Change only the pull-request base filter from `trunk` to `main`. Keep the
existing `push.branches: ["**"]` policy, job names, validation commands,
runner matrix, and permissions unchanged.

Add `workflow_call` as a third trigger. This lets the release workflow invoke
the same quality jobs without copying them or selecting a separate historical
run. Normal branch pushes and pull requests continue to use the existing
triggers.

## Release gate design

Keep the tag-push trigger and add a manual `workflow_dispatch` trigger with a
required `release_tag` input for non-publishing gate tests.

The release workflow will use this dependency graph:

```text
version-parity
      |
      v
quality-gate
      |
      v
build-and-release
      |
      v
create-release
```

### Version parity

The `version-parity` job will:

1. Check out the triggering commit using the existing immutable checkout SHA.
2. Derive the root package version with locked, offline Cargo metadata.
3. Require the candidate release tag to equal `v${package_version}` exactly.

For tag pushes, the candidate is `github.ref_name`. For manual runs, it is the
required `release_tag` input. The shell receives this value through a quoted
environment variable rather than interpolating an untrusted ref into the
script.

A mismatch fails before the quality workflow, release builds, artifact
uploads, or publication can start.

### Quality gate

The `quality-gate` job will depend on `version-parity` and call
`./.github/workflows/ci.yml`. A same-repository reusable workflow resolves from
the caller's commit, so every existing CI job runs against the exact candidate
release commit.

`build-and-release` will depend on `quality-gate`. GitHub's normal dependency
semantics skip it when any called CI job fails, is cancelled, or remains
incomplete. The existing build matrix, immutable action references, binary
smoke tests, artifact names, and upload validation remain unchanged.

### Publication boundary

`create-release` will retain its dependency on the successful build matrix and
remain the only job with `contents: write`. A job-level condition will also
require a real tag-push event. Manual runs can exercise version checks, CI,
builds, and artifact validation, but cannot create a public release.

The existing explicit three-binary allowlist, `SHA256SUMS`, smoke tests, and
release command remain unchanged.

## Validation design

Before editing, focused `yq` assertions will demonstrate that the `main` PR
filter, reusable trigger, version job, and release dependencies are absent.
After editing:

1. Run `actionlint` against both workflows.
2. Use `yq` assertions to verify the exact PR/push triggers and release job
   dependency graph.
3. Extract and run the version-parity shell block locally with a matching tag
   and a deliberately mismatched tag. The mismatch must exit nonzero.
4. Run formatting, strict Clippy, locked tests, and the dependency/security
   checks represented by CI.
5. After the dispatchable workflow exists on the default branch, run two
   non-publishing manual checks:
   - a healthy commit with a mismatched `release_tag`, proving every downstream
     job is skipped and no artifact is uploaded;
   - a temporary commit with a matching `release_tag` and an intentional
     formatting or test failure, proving the quality gate blocks every build,
     upload, and publication job.

The next real matching tag remains the end-to-end success-path verification
for the three binaries and `SHA256SUMS`.

## Trade-off

Each release tag reruns the complete CI matrix even when the same commit
already passed branch CI. This adds Actions time, but avoids polling,
additional API permissions, ambiguous rerun selection, and trigger-order
races.

## Non-goals

- Do not change the CI validation matrix or commands.
- Do not narrow push CI from all branches to only `main`.
- Do not change release targets, asset names, checksums, or smoke tests.
- Do not query or poll historical workflow runs.
- Do not bump the package version as part of these two tickets.
