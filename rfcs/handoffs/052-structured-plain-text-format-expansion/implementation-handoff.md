# RFC-052 Implementation Handoff Stub

## Summary

RFC-052 expands the product direction toward local-first structured plain-text
editing while keeping Markdown primary. This is a planning and terminology
handoff stub only.

## Scope followed

In scope for this stub:

- Reserve future product language for structured plain-text files.
- Preserve Markdown-first positioning.
- Require future format work to use RFC-053 canonical names.

Out of scope for M10:

- JSON/TOML/YAML implementation.
- Adapter trait implementation.
- UI behavior beyond the M10 split.

## Files changed

Generated this stub package under
`rfcs/handoffs/052-structured-plain-text-format-expansion/`.

## Design decisions and assumptions

- M10 must not block future structured formats.
- M10 must not implement structured format support.
- RFC-053 is the canonical type authority for later work.

## Tests and gates run

No implementation gates apply to this stub. Future RFC-052 work must define and
run its own gates.

## Generated artifacts

Stub handoff files only.

## Known limitations

This is not a full implementation handoff.

## Recommended next step

After M10 is accepted and stable, replace this stub with a full RFC-052 handoff.
