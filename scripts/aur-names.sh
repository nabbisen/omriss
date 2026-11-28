#!/usr/bin/env bash
# Decides which AUR packages a release run may write to (RFC-072 §5.3, §7, §9.4).
#
#   DRY_RUN=true|false SOURCE_NAME=... BIN_NAME=... scripts/aur-names.sh
#
# Prints key=value lines for $GITHUB_OUTPUT: aur_source_name, aur_bin_name and
# aur_rehearsal. The names come from the workflow's manual inputs and are empty on
# a tag push, which means the real packages, omriss and omriss-bin.
#
#  - A real run (not a dry run) always writes to the real packages: asking for any
#    other name is an error, so a typo cannot publish a release under a scratch name.
#  - A dry run writes to no AUR package, unless BOTH names are scratch names:
#    neither the real name of either package, and not each other's. That is the
#    rehearsal (RFC-072 §7), and it can only ever reach packages that are not omriss
#    or omriss-bin. One scratch name alone is an error rather than a half rehearsal.
#
# A force-push to the AUR is visible to every Arch user, so this refuses on doubt.
set -euo pipefail

real_source=omriss
real_bin=omriss-bin
source_name=${SOURCE_NAME:-$real_source}
bin_name=${BIN_NAME:-$real_bin}
dry_run=${DRY_RUN:-false}

fail() { echo "::error::$*" >&2; exit 1; }

# The AUR's rule for a package name.
valid='^[a-z0-9@_+][a-z0-9@._+-]*$'
for name in "$source_name" "$bin_name"; do
  [[ "$name" =~ $valid ]] || fail "'$name' is not an AUR package name (lowercase letters, digits and @ . _ + -; not starting with - or .)"
done

is_real() { [ "$1" = "$real_source" ] || [ "$1" = "$real_bin" ]; }

rehearsal=false
if [ "$dry_run" = true ]; then
  if [ "$source_name" = "$real_source" ] && [ "$bin_name" = "$real_bin" ]; then
    : # a plain dry run: no AUR package is written to
  elif ! is_real "$source_name" && ! is_real "$bin_name"; then
    [ "$source_name" != "$bin_name" ] || fail "the two scratch packages need two different names"
    rehearsal=true
  else
    fail "a rehearsal takes two scratch names, neither of them omriss or omriss-bin (got '$source_name' and '$bin_name'); leave both empty for a dry run that writes to no package"
  fi
else
  { [ "$source_name" = "$real_source" ] && [ "$bin_name" = "$real_bin" ]; } \
    || fail "package names can be changed only in a dry run; a real release publishes omriss and omriss-bin"
fi

echo "aur_source_name=$source_name"
echo "aur_bin_name=$bin_name"
echo "aur_rehearsal=$rehearsal"
