#!/usr/bin/env bash
set -euo pipefail

if [ "$#" -gt 1 ]; then
  echo "usage: $0 [output-file]" >&2
  exit 2
fi

output=${1:-THIRD_PARTY_LICENSES.html}
raw=$(mktemp)
trap 'rm -f "$raw"' EXIT

cargo about generate --locked --fail --output-file "$raw" about.hbs
LC_ALL=C tr -d '\r' < "$raw" | sed 's/[[:blank:]]*$//' > "$output"
