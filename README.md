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

## License

License [The MIT License](./LICENSE)

Copyright (c) 2026 Chris Becker
