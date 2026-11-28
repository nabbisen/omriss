#!/usr/bin/env bash
# Prints the CHANGELOG.md section of one release: the lines under
# "## [VERSION] - DATE", up to the next "## [" heading (RFC-072 §9.3: the notes of
# a GitHub Release). Fails if there is no such section, or it is empty, so a
# release cannot go out without its notes.
#
#   scripts/changelog-section.sh X.Y.Z
set -euo pipefail
cd "$(dirname "$0")/.."

version=${1:?version, X.Y.Z}
[[ "$version" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]] || { echo "version must be X.Y.Z: $version" >&2; exit 1; }

section=$(awk -v v="$version" '
  index($0, "## [" v "] - ") == 1 { inside = 1; next }
  inside && /^## \[/               { exit }
  inside                           { print }
' CHANGELOG.md)

# Strip the blank lines around the body.
section=$(printf '%s\n' "$section" | sed -e :a -e '/^[[:space:]]*$/{$d;N;ba' -e '}' | sed '/./,$!d')
if [ -z "$section" ]; then
  echo "CHANGELOG.md has no non-empty section headed \"## [$version] - DATE\"" >&2
  echo "(entries live under [Unreleased] until the tag is cut; that heading becomes this one)" >&2
  exit 1
fi
printf '%s\n' "$section"
