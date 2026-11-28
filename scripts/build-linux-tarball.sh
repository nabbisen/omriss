#!/usr/bin/env bash
# Builds the Linux release tarball and its checksum (RFC-072 §5.1).
#
#   scripts/build-linux-tarball.sh [VERSION]
#
# The one script that makes the artifact. CI's Linux job calls it and the local
# proof of the PKGBUILDs calls it, so the two cannot drift: what `omriss-bin`
# installs is what this produced. VERSION, if given, must equal Cargo.toml's
# (CI passes the tag, so a mismatch fails here, not on a user's machine).
#
# Output, in $OUT_DIR (default target/dist):
#   omriss-VERSION-x86_64-unknown-linux-gnu.tar.gz
#   omriss-VERSION-x86_64-unknown-linux-gnu.tar.gz.sha256   ("<hash>  <file>")
#
# The tarball holds one directory of the same name: the release binary, LICENSE,
# README.md, the .desktop file, the 256x256 icon and the AppStream metainfo.
# Set SKIP_BUILD=1 to package an existing target/release/omriss (local checks).
set -euo pipefail
cd "$(dirname "$0")/.."

triple=x86_64-unknown-linux-gnu
out_dir=${OUT_DIR:-target/dist}
version=$(grep -m1 '^version = ' Cargo.toml | cut -d'"' -f2)

fail() { echo "build-linux-tarball: $*" >&2; exit 1; }

[ -z "${1:-}" ] || [ "$1" = "$version" ] || fail "VERSION $1 does not match Cargo.toml's $version"
[ "$(rustc -vV | sed -n 's/^host: //p')" = "$triple" ] || fail "host is not $triple"

# The AppStream release list is how software centres show a version; a release
# without its entry would ship a package that claims to be the previous one.
grep -q "<release version=\"$version\" date=\"[0-9-]*\"" packaging/linux/io.github.nabbisen.omriss.metainfo.xml \
  || fail "packaging/linux/io.github.nabbisen.omriss.metainfo.xml has no <release> for $version (add it when the CHANGELOG heading gets its date)"

if [ "${SKIP_BUILD:-}" != 1 ]; then
  cargo build --release --locked -p omriss
fi
[ -x target/release/omriss ] || fail "target/release/omriss is missing"

name="omriss-$version-$triple"
stage=$(mktemp -d)
trap 'rm -rf "$stage"' EXIT
mkdir "$stage/$name"
install -m755 target/release/omriss "$stage/$name/omriss"
install -m644 LICENSE README.md "$stage/$name/"
install -m644 packaging/linux/io.github.nabbisen.omriss.desktop \
              packaging/linux/io.github.nabbisen.omriss.png \
              packaging/linux/io.github.nabbisen.omriss.metainfo.xml "$stage/$name/"

mkdir -p "$out_dir"
# A fixed order, owner and time: the same inputs make the same bytes, so a
# checksum means something. The time is the commit's, not the build's.
mtime=${SOURCE_DATE_EPOCH:-$(git log -1 --format=%ct)}
tar --sort=name --owner=0 --group=0 --numeric-owner --mtime="@$mtime" \
    -C "$stage" -cf - "$name" | gzip -n -9 > "$out_dir/$name.tar.gz"
(cd "$out_dir" && sha256sum "$name.tar.gz" > "$name.tar.gz.sha256")

echo "$out_dir/$name.tar.gz"
cat "$out_dir/$name.tar.gz.sha256"
