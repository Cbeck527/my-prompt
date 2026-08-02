# my-prompt

A spiritual fork of [prmt](https://github.com/3axap4eHko/prmt) which is an ultra-fast, customizable shell prompt.

I'm stripping it down and adding features to try and replicate the prompt that I like with [starship](https://starship.rs), but faster!

## Claude Code Statusline

`my-prompt` can be used as a statusline for [Claude Code](https://code.claude.com/docs/en/statusline). The `--claude` flag produces a prompt without the username or trailing `$`, and displays session information (model, context usage).

### Setup

Add the following to your Claude Code settings file (`~/.claude/settings.json`):

```json
{
  "statusLine": {
    "type": "command",
    "command": "/absolute/path/to/my-prompt --claude"
  }
}
```

Claude Code runs the configured command in a shell and sends JSON on standard input.
Replace `/absolute/path/to/my-prompt` with the absolute path to the installed
executable.

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
- `12k`, `200k` (10k-1M)
- `1.0M`, `1.5M` (1M-10M)
- `10M` (10M+)

## Fish shell prompt

The shipped Fish helper defines `fish_prompt`. It requires `my-prompt` to be
available on your `PATH`. Add the following to `~/.config/fish/config.fish`,
replacing the example path with the location of this repository:

```fish
source /absolute/path/to/my-prompt/etc/my-prompt.fish
```

For example, after installing the binary with `cargo install --path .`, ensure
Cargo's bin directory is on `PATH` before starting Fish.

## Git Backends

By default, `my-prompt` shells out to the `git` binary for branch and status
information. A library-based backend is also available via `--git-backend`:

| Backend | Flag | Description |
|---------|------|-------------|
| `binary` | `--git-backend binary` | Default. Shells out to `git`. Requires `git` on `$PATH`. |
| `gix` | `--git-backend gix` | Pure Rust via [gitoxide](https://github.com/GitoxideLabs/gitoxide). No external dependencies. |

Prompt modules use a Rayon thread pool capped at four threads or the host's
available parallelism, whichever is lower. A positive `RAYON_NUM_THREADS`
value is honored within that cap; zero and invalid values use the capped host
limit.
The pool is initialized only after CLI parsing, so `--help`, `--version`, and
argument errors do not start worker threads.

## Development with Nix

Enter the development shell to get Rust, Cargo, formatting and linting tools,
Cargo audit, Git, and Fish:

```bash
nix develop
```

Run existing commands without entering an interactive shell:

```bash
nix develop -c cargo test --verbose
```

## Building

```bash
cargo build --release
```

For maximum performance on your local machine, enable native CPU optimizations:

```bash
RUSTFLAGS="-C target-cpu=native" cargo build --release
```

This tells LLVM to use the full instruction set of your specific CPU (e.g., Apple Silicon features on M-series Macs). Do **not** use this for cross-compiled or distributed builds -- the resulting binary will only run on CPUs with the same (or newer) feature set.

## Benchmarking

Use the release binary for representative process measurements:

```bash
cargo build --release --locked
cargo run --release -- --bench --no-color
```

`--bench` reports cold startup from the beginning of `main` through the first
render, followed by 100 warm render timings. The cold metric includes CLI
parsing, Claude input parsing when applicable, and Rayon initialization; it
does not include the operating system's process launch time.

## Platform support

Prebuilt releases are available for Linux x86_64 (GNU and musl) and macOS on
Apple Silicon (arm64). Intel macOS is unsupported.

## License

License [The MIT License](./LICENSE)

Copyright (c) 2026 Chris Becker
