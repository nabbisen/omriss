#!/usr/bin/env bash
# Tests of the scripts that decide what reaches the AUR (RFC-072): which package
# names a run may write to, and what is rendered for them. They are the guard in
# front of an action that pushes to a public repository, so each refusal is a case
# here and not only a line in the script. Runs in CI's lint job; no network.
set -uo pipefail
cd "$(dirname "$0")/.."

fails=0
ok()  { echo "ok   $*"; }
bad() { echo "FAIL $*"; fails=$((fails + 1)); }

# names DRY SRC BIN -> prints the outputs; the exit status is the script's
names() { DRY_RUN=$1 SOURCE_NAME=$2 BIN_NAME=$3 scripts/aur-names.sh 2>/dev/null; }

expect_names() { # DRY SRC BIN EXPECTED_OUTPUT_ONE_LINE
  local got
  got=$(names "$1" "$2" "$3" | tr '\n' ' ') || { bad "names '$1' '$2' '$3' refused, expected: $4"; return; }
  [ "$got" = "$4 " ] && ok "names '$1' '$2' '$3'" || bad "names '$1' '$2' '$3': got '$got'"
}
expect_refused() { # DRY SRC BIN
  if names "$1" "$2" "$3" > /dev/null; then bad "names '$1' '$2' '$3' accepted"; else ok "names '$1' '$2' '$3' refused"; fi
}

REAL="aur_source_name=omriss aur_bin_name=omriss-bin aur_rehearsal=false"
expect_names "" "" "" "$REAL"                       # a tag push
expect_names false "" "" "$REAL"                    # a real manual run
expect_names true "" "" "$REAL"                     # a plain dry run: no rehearsal
expect_names true omriss omriss-bin "$REAL"         # the real names, spelled out
expect_names true scratch-a scratch-b "aur_source_name=scratch-a aur_bin_name=scratch-b aur_rehearsal=true"
expect_refused true scratch-a scratch-a             # one name for two packages
expect_refused true scratch-a ""                    # half a rehearsal
expect_refused true "" scratch-b
expect_refused true omriss-bin scratch-b            # a real name in the wrong slot
expect_refused true scratch-a omriss
expect_refused true omriss scratch-b
expect_refused false scratch-a scratch-b            # a real run never takes other names
expect_refused "" scratch-a scratch-b
expect_refused true "Not A Name" scratch-b
expect_refused true -scratch scratch-b
expect_refused true scratch-a ".hidden"

# aur-prepare.sh, with a curl that serves fixed bytes and no network
work=$(mktemp -d); trap 'rm -rf "$work"' EXIT
mkdir -p "$work/bin" "$work/dist"
cat > "$work/bin/curl" <<'STUB'
#!/usr/bin/env bash
# serves $FAKE_BODY for any URL, or fails like curl --fail does
[ -z "${FAKE_FAIL:-}" ] || { echo "curl: (22) The requested URL returned error: 404" >&2; exit 22; }
printf '%s' "$FAKE_BODY"
STUB
chmod +x "$work/bin/curl"
V=9.9.9
TAR="omriss-$V-x86_64-unknown-linux-gnu.tar.gz"
printf 'the tarball CI built' > "$work/dist/$TAR"
BUILT=$(sha256sum "$work/dist/$TAR" | cut -d' ' -f1)
echo "$BUILT  $TAR" > "$work/dist/$TAR.sha256"
SERVED=$(printf 'the tag archive' | sha256sum | cut -d' ' -f1)
ZEROS=$(printf '0%.0s' {1..64})

prep() { PATH="$work/bin:$PATH" scripts/aur-prepare.sh "$@" > "$work/log" 2>&1; }
sha_in() { grep -o "^sha256sums=('[0-9a-f]*')" "$1/PKGBUILD" | grep -o "[0-9a-f]\{64\}"; }

# real run, omriss-bin: the release serves what CI built
FAKE_BODY='the tarball CI built' prep omriss-bin $V omriss-bin false "$work/dist" "$work/o1" \
  && [ "$(sha_in "$work/o1")" = "$BUILT" ] && grep -qx 'pkgname=omriss-bin' "$work/o1/PKGBUILD" \
  && ok "real omriss-bin: built checksum, published equals built" || bad "real omriss-bin: $(cat "$work/log")"
# ... and when it serves something else
FAKE_BODY='something else' prep omriss-bin $V omriss-bin false "$work/dist" "$work/o2" \
  && bad "real omriss-bin accepted a release that serves other bytes" || ok "real omriss-bin refuses a release that serves other bytes"
# ... and when the release is not there
FAKE_FAIL=1 prep omriss-bin $V omriss-bin false "$work/dist" "$work/o3" \
  && bad "real omriss-bin accepted a missing release" || ok "real omriss-bin refuses a missing release"
# real run, omriss: the checksum is the archive's
FAKE_BODY='the tag archive' prep omriss $V omriss false "$work/dist" "$work/o4" \
  && [ "$(sha_in "$work/o4")" = "$SERVED" ] && grep -qx 'pkgname=omriss' "$work/o4/PKGBUILD" \
  && ok "real omriss: checksum of the fetched archive" || bad "real omriss: $(cat "$work/log")"
FAKE_FAIL=1 prep omriss $V omriss false "$work/dist" "$work/o5" \
  && bad "real omriss accepted a missing archive" || ok "real omriss refuses a missing archive"
# rehearsal: fetches nothing (a curl that always fails proves it), names the scratch package
FAKE_FAIL=1 prep omriss-bin $V scratch-b true "$work/dist" "$work/o6" \
  && [ "$(sha_in "$work/o6")" = "$BUILT" ] && grep -qx 'pkgname=scratch-b' "$work/o6/PKGBUILD" \
  && ok "rehearsal omriss-bin: scratch name, built checksum, nothing fetched" || bad "rehearsal omriss-bin: $(cat "$work/log")"
FAKE_FAIL=1 prep omriss $V scratch-a true "$work/dist" "$work/o7" \
  && [ "$(sha_in "$work/o7")" = "$ZEROS" ] && grep -qx 'pkgname=scratch-a' "$work/o7/PKGBUILD" \
  && grep -qx '_name=omriss' "$work/o7/PKGBUILD" \
  && ok "rehearsal omriss: scratch name, zero checksum, upstream name kept" || bad "rehearsal omriss: $(cat "$work/log")"
# a rehearsal never renders for a real package
prep omriss-bin $V omriss-bin true "$work/dist" "$work/o8" \
  && bad "rehearsal rendered for the real omriss-bin" || ok "rehearsal refuses the real name omriss-bin"
prep omriss $V omriss true "$work/dist" "$work/o9" \
  && bad "rehearsal rendered for the real omriss" || ok "rehearsal refuses the real name omriss"
# no checksum file, or two
rm "$work/dist/$TAR.sha256"
prep omriss-bin $V scratch-b true "$work/dist" "$work/o10" \
  && bad "omriss-bin rendered without the built checksum" || ok "omriss-bin refuses a missing .sha256"

# aur-check-credentials.sh: names what is missing, rejects a public key, prints no value
KEY='-----BEGIN OPENSSH PRIVATE KEY-----
b3BlbnNzaC1rZXktdjEAAAAA-not-a-real-key
-----END OPENSSH PRIVATE KEY-----'
creds() { env -i PATH="$PATH" "$@" scripts/aur-check-credentials.sh > "$work/log" 2>&1; }
creds AUR_SSH_PRIVATE_KEY="$KEY" AUR_COMMIT_USERNAME=u AUR_COMMIT_EMAIL=e@example.invalid \
  && ok "credentials: all present" || bad "credentials: all present refused"
creds AUR_COMMIT_USERNAME=u AUR_COMMIT_EMAIL=e@example.invalid \
  && bad "credentials: missing key accepted" || { grep -q 'AUR_SSH_PRIVATE_KEY' "$work/log" && ok "credentials: missing key named" || bad "credentials: missing key not named"; }
creds AUR_SSH_PRIVATE_KEY="$KEY" AUR_COMMIT_USERNAME=u \
  && bad "credentials: missing email accepted" || { grep -q 'AUR_COMMIT_EMAIL' "$work/log" && ok "credentials: missing email named" || bad "credentials: missing email not named"; }
creds AUR_SSH_PRIVATE_KEY='ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIsecretlooking me@host' AUR_COMMIT_USERNAME=u AUR_COMMIT_EMAIL=e@example.invalid \
  && bad "credentials: a public key accepted" || { ! grep -q 'AAAAC3' "$work/log" && ok "credentials: a public key refused, not echoed" || bad "credentials: value echoed"; }

[ "$fails" -eq 0 ] && echo "all AUR script tests passed" || { echo "$fails AUR script test(s) FAILED"; exit 1; }
