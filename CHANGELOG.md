# Changelog

All notable changes are documented here. This project follows Semantic
Versioning while remaining pre-1.0, so breaking interface changes increment the
minor version.

## [Unreleased]

### Fixed

- Replaced machine-specific raw dependency metadata with a reproducible,
  human-readable third-party license report.

### Changed

- **Breaking:** release archives contain `THIRD_PARTY_LICENSES.html` instead of
  `THIRD_PARTY_LICENSES.json`.

## [0.2.1] - 2026-08-05

### Changed

- Refreshed dependencies and removed a redundant Fish setup integration test.

## [0.2.0] - 2026-08-05

First supported public release.

### Added

- Nix package and Home Manager module with evaluation checks.
- Claude Code status-line rendering and canonical Fish initialization.
- Strict CI, Dependabot, dependency policy, release provenance, and
  signed/notarized Apple Silicon macOS builds.
- Project, vendored-source, and Rust dependency license notices in release
  archives.

### Changed

- **Breaking:** release assets are versioned target archives rather than raw,
  platform-named binaries.
- **Breaking:** an empty `NO_COLOR` value no longer disables color; the variable
  must be nonempty, matching the NO_COLOR convention.
- Git CLI status uses `--no-optional-locks` and streams output without buffering
  the entire child process result.
- Render-time environment state is captured once before parallel work begins.
- The built-in benchmark reports the direnv path actually selected.

### Removed

- Unused hostname module and unused benchmark/test dependencies.
- Unrelated vendored async-Rust skill.

## [0.1.0] - 2026-07-31

Private milestone before the supported public release.

[Unreleased]: https://github.com/cbeck527/my-prompt/compare/v0.2.1...HEAD
[0.2.1]: https://github.com/cbeck527/my-prompt/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/cbeck527/my-prompt/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/cbeck527/my-prompt/releases/tag/v0.1.0
