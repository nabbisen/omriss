<!--
Project: omriss — Omriss Editor
Document Set: RFC detailed design bundle
Generated for architecture/design review
Language: English
-->
# RFC-031: Performance Targets and Measurement Plan

**Project:** omriss — Omriss Editor  
**Milestone:** M7 — Performance and Large Document Readiness  
**Status.** Implemented (v0.10.0), with its central deliverable never produced — see below  
**Document type:** Detailed RFC design  
**Primary audience:** Architect, Rust developer, UI/UX designer, QA engineer  


> **Unmet criterion, recorded 2026-09-01.** §4 says "exact millisecond
> thresholds should be calibrated after M0/M1 measurements" and §7's acceptance
> criterion is "regression thresholds documented before enforcement." No
> thresholds were ever calibrated, documented, or enforced. Benchmarks exist but
> do not run in CI.
>
> The first performance numbers on record for this project were produced by the
> 0.17.0 full-project audit, not by this RFC's plan. RFC-067 carries the
> measurements; thresholds should be set from them and this header updated when
> they are.
>
> Recorded in the pattern RFC-016's header established: keep the status honest
> rather than the folder tidy.

---

## 1. Summary

Define measurable performance budgets and benchmarking fixtures.

## 2. Goals

- Set document size targets.
- Measure indexing, editing, rendering, and memory.
- Create regression thresholds.
- Avoid premature optimization.

## 3. Non-Goals

- No hard real-time guarantee.
- No extremely large database-like document target.
- No optimization before measurement.

## 4. Design

### Initial Budgets

| Scenario | Target |
|---|---|
| Open 10k-word Markdown | responsive enough for interactive use |
| Re-index after section commit | normally below noticeable delay for ordinary docs |
| Typing in focus editor | no full-document mutation per keystroke |
| Save | bounded by filesystem and document size |

Exact millisecond thresholds should be calibrated after M0/M1 measurements.

### Measurement Points

```text
file read time
UTF-8 decode time
index build time
focus snapshot creation time
section replacement time
re-index time
Dioxus render/update time
save write time
memory peak
```

## 5. Internal Design Notes

### Benchmark Harness

Use `criterion` or a simple stable benchmark runner. Keep performance fixtures checked in or generated deterministically.

## 6. Validation and Test Plan

- Benchmark command runs in CI optional mode.
- Performance fixtures load successfully.
- Regression thresholds documented before enforcement.

## 7. Acceptance Criteria

- Performance discussions refer to measured data.
- Input latency is protected by local edit buffer design.
- Large-document readiness has explicit fixture coverage.

## 8. Dependencies

- RFC-012
- RFC-032
- RFC-034

---

## Implementation Reminder

This RFC must preserve the project-wide invariant: editing one section must not rewrite unrelated Markdown source bytes unless the RFC explicitly describes and justifies a structural source transformation.
