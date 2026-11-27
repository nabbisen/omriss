# RFC-073 — Implementation Handoff

Related RFC: `../../done/073-complete-localization-and-locale-persistence.md`

Lifecycle status: inherited from RFC-073.

---

## 1. Build what RFC-073 §8 says

§4.3's type is the starting point. §8.1 decides which variants exist: the
failures found by construction, and none that cannot happen. Write the list
of `Err` sources you found into the report, with how each reaches the user.

## 2. Constraints

1. No `Locale` enters `crates/app/src/file/`. Text, or typed reasons, cross
   that boundary; the catalog is read in `shell/` or the components.
2. Exhaustive matches with no wildcard arm, from a failure reason to a catalog
   key. One exception: `io::ErrorKind` is `non_exhaustive`, so its general
   arm is the one wildcard, commented as such.
3. Every new message in both locales. English per RFC-073 §8.2: what happened
   and what to do, never blaming the user. Japanese: the same meaning, not word
   for word.
4. `omriss-core`: only §8.6's positional title and its doc comment. Say in the
   CHANGELOG that a list member's title from core is now its number. Anything
   else that needs core: stop and report.
5. The proof that no English cause survives is a command and its output in
   the report (RFC-073 §6), not a claim.
6. `docs/src/`: any page quoting an old message changes with it. So does
   `known-limitations.md`, if it has an entry this closes.
