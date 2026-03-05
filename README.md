# my-prompt

A spiritual fork of [prmt](https://github.com/3axap4eHko/prmt) which is an ultra-fast, customizable shell prompt.

I'm stripping it down and adding features to try and replicate the prompt that I like with [starship](https://starship.rs), but faster!

## Claude Code Statusline

`my-prompt` can be used as a statusline for [Claude Code](https://docs.anthropic.com/en/docs/claude-code). The `--claude` flag produces a prompt without the username or trailing `$`, and displays session information (model, context usage).

### Setup

Add the following to your Claude Code settings file (`~/.claude/settings.json`):

```json
{
  "statusline": "/path/to/my-prompt --claude"
}
```

### Output

The statusline displays:
- Current directory (with `~` for home)
- Git branch and status
- Claude session info: `[Model used/total (percentage%)]`

Example:
```
~/src/my-project [main+?] [Opus 12k/200k (6%)]
```

Token counts are formatted for readability:
- `845` (under 1k)
- `5.0k` (1k-10k)
- `12k`, `200k` (10k+)

## Git Backends

By default, `my-prompt` shells out to the `git` binary for branch and status information. Two experimental library-based backends are also available via `--git-backend`:

| Backend | Flag | Description |
|---------|------|-------------|
| `binary` | `--git-backend binary` | Default. Shells out to `git`. Requires `git` on `$PATH`. |
| `gix` | `--git-backend gix` | Pure Rust via [gitoxide](https://github.com/GitoxideLabs/gitoxide). No external dependencies. |
| `git2` | `--git-backend git2` | [libgit2](https://libgit2.org/) bindings (vendored). No external dependencies. |

## Building

```bash
cargo build --release
```

For maximum performance on your local machine, enable native CPU optimizations:

```bash
RUSTFLAGS="-C target-cpu=native" cargo build --release
```

This tells LLVM to use the full instruction set of your specific CPU (e.g., Apple Silicon features on M-series Macs). Do **not** use this for cross-compiled or distributed builds -- the resulting binary will only run on CPUs with the same (or newer) feature set.

## License

License [The MIT License](./LICENSE)

Copyright (c) 2026 Chris Becker
