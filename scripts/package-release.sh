#!/usr/bin/env bash
set -euo pipefail

if [ "$#" -ne 4 ]; then
  echo "usage: $0 <binary> <release-tag> <target> <output-directory>" >&2
  exit 2
fi

binary=$1
release_tag=$2
target=$3
output_dir=$4

test -s "$binary"
test -x "$binary"
mkdir -p "$output_dir"
output_dir=$(cd "$output_dir" && pwd -P)

archive="my-prompt-${release_tag}-${target}.tar.gz"
directory="my-prompt-${release_tag}-${target}"
work_dir=$(mktemp -d)
trap 'rm -rf "$work_dir"' EXIT
staging="$work_dir/$directory"

mkdir "$staging"
install -m 0755 "$binary" "$staging/my-prompt"
install -m 0644 LICENSE THIRD_PARTY_NOTICES.md THIRD_PARTY_LICENSES.json "$staging/"
tar -C "$work_dir" -czf "$output_dir/$archive" "$directory"

printf '%s\n' "$output_dir/$archive"
