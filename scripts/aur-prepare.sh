#!/usr/bin/env bash
# Renders the PKGBUILD one AUR package is published from (RFC-072 §5.3).
#
#   scripts/aur-prepare.sh PACKAGE VERSION AUR_NAME REHEARSAL DIST_DIR OUT_DIR
#
# PACKAGE is omriss or omriss-bin (which template); AUR_NAME is the name it is
# published under (scripts/aur-names.sh decides it); REHEARSAL is true or false;
# DIST_DIR holds the Linux job's tarball and .sha256; OUT_DIR gets PKGBUILD.
#
# The checksum is what the package is checked against by every user, so it comes
# from the file the package downloads, not from a copy of it:
#
#  - omriss-bin downloads the GitHub release's tarball. The release's file is
#    fetched and must be byte for byte the one the Linux job built, or this fails:
#    a release that does not serve what CI built must not reach the AUR.
#  - omriss downloads GitHub's archive of the tag, which only GitHub produces, so
#    its checksum can only come from fetching it.
#
# A rehearsal publishes to a scratch package, and the release it would point at
# does not exist (a dry run creates none), so it fetches nothing: omriss-bin takes the
# built checksum, omriss takes 64 zeros, which no file has. A scratch package built
# from that fails its checksum and installs nothing; it exists to exercise the key,
# the environment and the push.
set -euo pipefail
cd "$(dirname "$0")/.."

pkg=${1:?package: omriss or omriss-bin}
version=${2:?version}
aur_name=${3:?AUR package name}
rehearsal=${4:?rehearsal: true or false}
dist=${5:?directory with the Linux tarball and its .sha256}
out=${6:?output directory}

case "$rehearsal" in true|false) ;; *) echo "rehearsal must be true or false: $rehearsal" >&2; exit 1 ;; esac
# The real packages are written only by a real run, whatever else is asked.
if [ "$rehearsal" = true ] && { [ "$aur_name" = omriss ] || [ "$aur_name" = omriss-bin ]; }; then
  echo "::error::a rehearsal must not render for the real package $aur_name" >&2
  exit 1
fi

url=$(scripts/render-pkgbuild.sh --source-url "$pkg" "$version")

fetch_sha256() { # URL: the checksum of what it serves; any failure fails the caller
  curl --fail --silent --show-error --location --retry 3 --retry-delay 5 "$1" | sha256sum | cut -d' ' -f1
}

built=
if [ "$pkg" = omriss-bin ]; then
  shopt -s nullglob
  sums=("$dist"/omriss-"$version"-x86_64-unknown-linux-gnu.tar.gz.sha256)
  [ "${#sums[@]}" -eq 1 ] || { echo "::error::expected exactly one omriss-$version-….tar.gz.sha256 in $dist" >&2; exit 1; }
  built=$(cut -d' ' -f1 < "${sums[0]}")
fi

if [ "$rehearsal" = true ]; then
  echo "::notice::REHEARSAL for $aur_name: nothing is fetched, and the release this PKGBUILD names does not exist"
  if [ "$pkg" = omriss-bin ]; then sha=$built; else sha=$(printf '0%.0s' {1..64}); fi
elif [ "$pkg" = omriss-bin ]; then
  published=$(fetch_sha256 "$url")
  if [ "$published" != "$built" ]; then
    echo "::error::$url serves $published, but the Linux job built $built; not publishing to the AUR" >&2
    exit 1
  fi
  sha=$built
else
  sha=$(fetch_sha256 "$url")
fi

mkdir -p "$out"
scripts/render-pkgbuild.sh "$pkg" "$version" "$sha" "$url" "$aur_name" > "$out/PKGBUILD"
grep -qx "pkgname=$aur_name" "$out/PKGBUILD" || { echo "::error::rendered PKGBUILD does not name $aur_name" >&2; exit 1; }
echo "Rendered $out/PKGBUILD: $aur_name $version, source $url, sha256 $sha"
