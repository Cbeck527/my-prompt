#!/usr/bin/env bash
set -euo pipefail

if [ "$#" -ne 2 ]; then
  echo "usage: $0 <binary> <version>" >&2
  exit 2
fi

binary=$(cd "$(dirname "$1")" && pwd -P)/$(basename "$1")
version=$2

test -s "$binary"
test -x "$binary"
"$binary" --version | grep -Fx "my-prompt $version"

smoke_dir=$(mktemp -d)
trap 'rm -rf "$smoke_dir"' EXIT

regular_output="$smoke_dir/regular.txt"
transient_output="$smoke_dir/transient.txt"
claude_output="$smoke_dir/claude.txt"
helper="$smoke_dir/my-prompt.fish"

(
  cd "$smoke_dir"
  NO_COLOR=1 "$binary" --no-color > "$regular_output"
  NO_COLOR=1 "$binary" --final-rendering --no-color > "$transient_output"
  printf '%s\n' '{"model":{"display_name":"Opus"},"context_window":{"context_window_size":200000,"used_percentage":6,"current_usage":{"input_tokens":8500,"cache_creation_input_tokens":5000,"cache_read_input_tokens":2000}}}' |
    NO_COLOR=1 "$binary" claude --no-color > "$claude_output"
)

test -s "$regular_output"
test -s "$transient_output"
grep -F '[Opus 15k/200k (6%)]' "$claude_output"
if LC_ALL=C grep -q $'\033' "$regular_output" "$transient_output" "$claude_output"; then
  echo "no-color smoke output contains ANSI escapes" >&2
  exit 1
fi

"$binary" init > "$helper"
fish -n "$helper"
