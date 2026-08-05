# Contributing

Thanks for taking a look. Bug reports and focused improvements are welcome.

This is my personal prompt, so its segment order, colors, character, username
alias, CLI-only architecture, and deliberately small configuration surface are
intentional. If you want a substantially different prompt or a general-purpose
configuration system, a fork is the best path—and exactly the kind of reuse this
public repository is meant to support.

Before opening a pull request:

1. Keep the change small, readable, and tied to observable behavior.
2. Add focused tests when they clarify non-trivial logic or prevent a likely
   regression.
3. Update user-facing documentation and `CHANGELOG.md` for interface changes.
4. Run the validation commands in the README.

Pull requests should explain the user-visible effect, the implementation
trade-off, and the commands used for validation. Representative prompt output is
helpful for rendering changes.

This project does not preserve backward compatibility by default. Flag breaking
changes explicitly and remove obsolete paths instead of adding compatibility
layers.
