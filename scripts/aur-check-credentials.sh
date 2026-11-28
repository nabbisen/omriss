#!/usr/bin/env bash
# The AUR jobs' credentials must be present, and the key must look like a private
# key (RFC-072 §5.4). Reads AUR_SSH_PRIVATE_KEY, AUR_COMMIT_USERNAME and
# AUR_COMMIT_EMAIL from the environment, names what is missing, and never prints
# a value.
#
# 0.17.0's Store job ran with every secret unset: the values expanded to nothing
# and the job died on a message about something else. An unset credential must say
# so, and a public key pasted where the private one belongs must say so too.
set -u

missing=()
for name in AUR_SSH_PRIVATE_KEY AUR_COMMIT_USERNAME AUR_COMMIT_EMAIL; do
  [ -n "${!name:-}" ] || missing+=("$name")
done
if [ ${#missing[@]} -ne 0 ]; then
  echo "::error::Not set on the 'aur' environment (secret AUR_SSH_PRIVATE_KEY; variables AUR_COMMIT_USERNAME and AUR_COMMIT_EMAIL): ${missing[*]}"
  exit 1
fi

case "$AUR_SSH_PRIVATE_KEY" in
  '-----BEGIN '*'PRIVATE KEY-----'*) ;;
  *)
    echo "::error::AUR_SSH_PRIVATE_KEY is not a private key (it should start with '-----BEGIN OPENSSH PRIVATE KEY-----'). The public half goes on the AUR account, the private half here."
    exit 1
    ;;
esac
echo "The AUR credentials are present."
