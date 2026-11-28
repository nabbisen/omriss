#!/usr/bin/env bash
# Two rules for every workflow under .github/workflows/ (RFC-072 §12):
#
#  1. No `${{ }}` inside the text of a `run:` step. An expression is pasted into
#     the script before the shell sees it, so a value containing a quote, a `$(`
#     or a newline becomes code; through `env:` it is only ever data. The version
#     and every secret reach a script as an environment variable.
#  2. Every `uses:` of a remote action names a 40-hex commit SHA, with its tag in
#     a comment: these jobs run with credentials in scope, and a moving tag could
#     change under us.
#
#   scripts/check-workflows.sh [FILE...]    default: .github/workflows/*.yml
#   scripts/check-workflows.sh --self-test  the checker against fixtures that must
#                                           and must not trip it
#
# Plain awk, no YAML library, so it runs anywhere CI's lint job does. It reads
# `run:` as YAML does for the forms workflows use: a value on the same line, or a
# block scalar (`|`, `>`, with or without `-`/`+`) whose lines are indented deeper
# than the key. A line in a block that merely mentions the text, even in a shell
# comment, is flagged too: the cost of that is a reworded comment.
set -euo pipefail
cd "$(dirname "$0")/.."

check_file() { # prints one line per violation; exit status 0 either way
  awk -v file="$1" -v dq='"' -v sq="'" '
    function indent(s) { match(s, /^ */); return RLENGTH }
    {
      line = $0
      if (inblock) {
        if (line ~ /^[[:space:]]*$/ || indent(line) > keyindent) {
          if (index(line, "${{")) printf "%s:%d: ${{ }} in a run: script: %s\n", file, NR, trim(line)
          next
        }
        inblock = 0
      }
      if (match(line, /^[[:space:]]*(- )?run:[[:space:]]*/)) {
        key = substr(line, 1, RLENGTH)
        rest = substr(line, RLENGTH + 1)
        keyindent = indent(key) + (key ~ /^[[:space:]]*- / ? 2 : 0)
        if (rest ~ /^[|>][-+0-9]*[[:space:]]*(#.*)?$/) { inblock = 1; next }
        if (index(rest, "${{")) printf "%s:%d: ${{ }} in a run: script: %s\n", file, NR, trim(rest)
        next
      }
      if (match(line, /^[[:space:]]*(- )?uses:[[:space:]]*/)) {
        ref = substr(line, RLENGTH + 1); sub(/[[:space:]]*(#.*)?$/, "", ref)
        gsub(dq, "", ref); gsub(sq, "", ref)
        if (ref ~ /^\.\//) next
        n = split(ref, part, "@"); sha = part[n]
        if (n < 2 || length(sha) != 40 || sha !~ /^[0-9a-f]+$/) printf "%s:%d: action not pinned to a 40-hex commit SHA: %s\n", file, NR, ref
      }
    }
    function trim(s) { sub(/^[[:space:]]+/, "", s); return s }
  ' "$1"
}

if [ "${1:-}" = --self-test ]; then
  t=$(mktemp -d); trap 'rm -rf "$t"' EXIT
  SHA=34e114876b0b11c390a56381ad16ebd13914f8d5
  n_fail=0
  case_() { # NAME EXPECTED_VIOLATIONS  (fixture on stdin)
    cat > "$t/f.yml"
    got=$(check_file "$t/f.yml" | wc -l)
    if [ "$got" -eq "$2" ]; then echo "ok   $1"; else echo "FAIL $1: expected $2 violation(s), got $got"; check_file "$t/f.yml"; n_fail=$((n_fail + 1)); fi
  }
  case_ "clean file" 0 <<EOF2
jobs:
  a:
    steps:
      - uses: actions/checkout@$SHA # v4.3.1
      - uses: ./local-action
      - name: x
        env:
          V: \${{ needs.v.outputs.version }}
        run: echo "\$V"
      - run: echo plain
      - run: |
          echo "\$V"
EOF2
  case_ "expression on the run line" 1 <<EOF2
      - run: echo \${{ github.ref }}
EOF2
  case_ "expression in a literal block" 1 <<EOF2
      - name: x
        run: |
          set -e
          echo \${{ github.ref }}
          echo ok
EOF2
  case_ "expression in a folded block" 2 <<EOF2
        run: >
          msstore reconfigure
          --tenantId "\${{ secrets.A }}"
          --sellerId "\${{ secrets.B }}"
        env:
          A: \${{ secrets.A }}
EOF2
  case_ "block ends at the next key; env after it is allowed" 0 <<EOF2
        run: |-
          echo hi
        env:
          A: \${{ secrets.A }}
        if: \${{ always() }}
EOF2
  case_ "a blank line inside a block does not end it" 1 <<EOF2
        run: |
          echo one

          echo \${{ github.sha }}
EOF2
  case_ "run as the first key of a step" 1 <<EOF2
      - run: |
          echo \${{ github.sha }}
EOF2
  case_ "unpinned tag" 1 <<EOF2
      - uses: actions/checkout@v4
EOF2
  case_ "branch ref and short sha" 2 <<EOF2
      - uses: foo/bar@main
      - uses: foo/baz@34e1148
EOF2
  case_ "pinned, with trailing comment, quoted" 0 <<EOF2
      - uses: "actions/checkout@$SHA" # v4
EOF2
  [ "$n_fail" -eq 0 ] && { echo "all workflow-rule self-tests passed"; exit 0; } || { echo "$n_fail self-test(s) FAILED"; exit 1; }
fi

if [ "$#" -gt 0 ]; then files=("$@"); else files=(.github/workflows/*.yml); fi
out=$(for f in "${files[@]}"; do check_file "$f"; done)
if [ -n "$out" ]; then
  printf '%s\n' "$out" >&2
  echo "check-workflows: the lines above break a workflow rule (see the header of scripts/check-workflows.sh)" >&2
  exit 1
fi
echo "check-workflows: ${#files[@]} workflow(s) follow both rules"
