#!/usr/bin/env bash
# Fills a PKGBUILD template from packaging/arch/ (RFC-072 §5.3).
#
#   scripts/render-pkgbuild.sh PACKAGE VERSION SHA256 [SOURCE_URL [PKGNAME]] > PKGBUILD
#   scripts/render-pkgbuild.sh --source-url PACKAGE VERSION
#
# PACKAGE is omriss or omriss-bin: which template. SHA256 is the checksum of the
# source the PKGBUILD downloads. SOURCE_URL defaults to the GitHub URL for VERSION;
# a local proof passes a file:// URL, a publish job never does. PKGNAME is the name
# the package is published under and defaults to PACKAGE; only a rehearsal on a
# scratch AUR package (scripts/aur-names.sh) ever differs. The templates are not
# valid PKGBUILDs until this has run: they carry @PKGNAME@, @PKGVER@, @SHA256@ and
# @SOURCE_URL@, and this refuses to emit one that still does.
#
# --source-url prints the default SOURCE_URL and nothing else, so that the one place
# that knows where a package's source lives is this file.
set -euo pipefail
cd "$(dirname "$0")/.."

base=https://github.com/nabbisen/omriss

source_url() { # PACKAGE VERSION
  if [ "$1" = omriss-bin ]; then
    echo "$base/releases/download/$2/omriss-$2-x86_64-unknown-linux-gnu.tar.gz"
  else
    echo "$base/archive/refs/tags/$2.tar.gz"
  fi
}

check_package_and_version() {
  case "$1" in omriss|omriss-bin) ;; *) echo "unknown package: $1" >&2; exit 1 ;; esac
  [[ "$2" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]] || { echo "version must be X.Y.Z: $2" >&2; exit 1; }
}

if [ "${1:-}" = --source-url ]; then
  pkg=${2:?package: omriss or omriss-bin}
  version=${3:?version}
  check_package_and_version "$pkg" "$version"
  source_url "$pkg" "$version"
  exit 0
fi

pkg=${1:?package: omriss or omriss-bin}
version=${2:?version}
sha=${3:?sha256 of the source}
check_package_and_version "$pkg" "$version"
[[ "$sha" =~ ^[0-9a-f]{64}$ ]] || { echo "sha256 must be 64 lowercase hex digits" >&2; exit 1; }

url=${4:-$(source_url "$pkg" "$version")}
name=${5:-$pkg}
# The AUR's rule for a package name: lowercase letters and digits and @ . _ + -,
# not starting with a hyphen or a dot.
[[ "$name" =~ ^[a-z0-9@_+][a-z0-9@._+-]*$ ]] || { echo "not an AUR package name: $name" >&2; exit 1; }

out=$(sed -e "s|@PKGNAME@|$name|g" -e "s|@PKGVER@|$version|g" -e "s|@SHA256@|$sha|g" -e "s|@SOURCE_URL@|$url|g" "packaging/arch/$pkg/PKGBUILD")
if grep -q '@[A-Z0-9_]*@' <<<"$out"; then
  echo "unfilled placeholder left in packaging/arch/$pkg/PKGBUILD" >&2
  exit 1
fi
printf '%s\n' "$out"
