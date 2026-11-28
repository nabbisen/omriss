#!/usr/bin/env bash
# Fills a PKGBUILD template from packaging/arch/ (RFC-072 §5.3).
#
#   scripts/render-pkgbuild.sh PACKAGE VERSION SHA256 [SOURCE_URL] > PKGBUILD
#
# PACKAGE is omriss or omriss-bin. SHA256 is the checksum of the source the
# PKGBUILD downloads. SOURCE_URL defaults to the GitHub URL for VERSION; a local
# proof passes a file:// URL, a publish job never does. The templates are not
# valid PKGBUILDs until this has run: they carry @PKGVER@, @SHA256@ and
# @SOURCE_URL@, and this refuses to emit one that still does.
set -euo pipefail
cd "$(dirname "$0")/.."

pkg=${1:?package: omriss or omriss-bin}
version=${2:?version}
sha=${3:?sha256 of the source}
case "$pkg" in omriss|omriss-bin) ;; *) echo "unknown package: $pkg" >&2; exit 1 ;; esac
[[ "$version" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]] || { echo "version must be X.Y.Z: $version" >&2; exit 1; }
[[ "$sha" =~ ^[0-9a-f]{64}$ ]] || { echo "sha256 must be 64 lowercase hex digits" >&2; exit 1; }

base=https://github.com/nabbisen/omriss
if [ "$pkg" = omriss-bin ]; then
  default_url="$base/releases/download/$version/omriss-$version-x86_64-unknown-linux-gnu.tar.gz"
else
  default_url="$base/archive/refs/tags/$version.tar.gz"
fi
url=${4:-$default_url}

out=$(sed -e "s|@PKGVER@|$version|g" -e "s|@SHA256@|$sha|g" -e "s|@SOURCE_URL@|$url|g" "packaging/arch/$pkg/PKGBUILD")
if grep -q '@[A-Z0-9_]*@' <<<"$out"; then
  echo "unfilled placeholder left in packaging/arch/$pkg/PKGBUILD" >&2
  exit 1
fi
printf '%s\n' "$out"
