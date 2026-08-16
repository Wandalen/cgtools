# Malformed-RON-syntax LoadError::Ron test coverage

## Execution State

- **Executor Type:** any
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/verified/
- **actor:** null
- **started_at:** null
- **expires_at:** null
- **round:** 1
- **state:** 🎯 (Verified)
- **closes:** null
- **repo_identity:** self
- **unit_type:** module
- **unit:** lib/yrd_gamedev/cgtools/module/helper/tilemap_scene
- **verified_by:** null
- **verification_date:** null
- **blocked_by:** null

## Goal

Add native test coverage proving `RenderSpec::from_ron_str` /
`SceneSnapshot::from_ron_str` return `LoadError::Ron` — not a panic, not a
silently-defaulted value — when given syntactically-invalid RON input. This
is the one `LoadError` variant (`Io`, `Ron`, `Validation`) with zero
existing test coverage in `tilemap_scene`'s suite: `Validation` is covered
extensively (`compile_units_test.rs`, `scene_model_test.rs`) and `Io` is
trivially guaranteed by the standard library's own file-not-found contract,
but no existing test feeds genuinely unparseable text through either
`from_ron_str` and asserts on the `Ron` arm specifically. Matters now
because the 2026-08-15 docs/layer gap audit flagged this as
`tilemap_scene`'s one remaining untested error path, and `error.rs`'s own
doc comment frames `LoadError` as the uniform error contract callers rely on
handling — an untested variant is an unverified part of that contract.
Bounded to one new test file in this one crate, zero source changes.
Testable: `cargo test -p tilemap_scene --test ron_syntax_error_test` exits
0.

## In Scope

- New `module/helper/tilemap_scene/tests/ron_syntax_error_test.rs`: feeds at
  least 2 distinct genuinely-malformed RON strings (not valid RON syntax at
  all — e.g. an unclosed paren, or a bare unquoted token where a struct is
  expected) through `RenderSpec::from_ron_str` and
  `SceneSnapshot::from_ron_str`, asserting each returns `Err( LoadError::Ron(
  _ ) )` via `matches!`.
- One additional case distinguishing `Ron` from `Validation` in the same new
  file: syntactically-valid RON that fails a SPEC §16 validation rule must
  parse successfully (`from_ron_str` alone returns `Ok`) and only fail at
  the separate `.validate()` step — pinning the boundary between the two
  variants explicitly, in one place, rather than leaving it implicit across
  the existing `Validation`-focused test files.

## Out of Scope

- `LoadError::Io` coverage — already guaranteed by the standard library's
  own file-not-found contract; not a gap this task addresses.
- Any change to `error.rs`, `load.rs`, `validate.rs`, or the
  `LoadError`/`ValidationError` type definitions — this task adds tests
  only, zero source edits.
- `SnapshotLoadError` — a distinct error type for snapshot/spec
  cross-referencing, not `LoadError`.
- Malformed RON at the `.load( path )` (file-based) entry points —
  `from_ron_str` is the parsing-only entry point both `.load()` variants
  delegate to, so testing it directly is the precise, minimal surface;
  re-testing the same parse failure through the file-reading wrapper would
  be redundant.

## Requirements

-   All work must strictly adhere to all applicable rulebooks
    (discover via `kbase .rulebooks`)

## Delivery Requirements

-   All work must strictly adhere to all applicable rulebooks
    (discover via `kbase .rulebooks`)
-   Test Matrix populated before any test code
-   Every Test Matrix case is backed by a test that failed before its
    implementing change landed
-   Minimum code to satisfy Test Matrix — no features beyond requirements
-   `verb/test` passes with zero failures and zero warnings
-   No function exceeds 50 lines; no duplication; public items have `///`
    doc comments
-   Independent verification passes per `§ Acceptance Verification :
    Procedure - Execution`
-   Task state updated to ✅ on verification pass; file moved to
    `task/completed/`

## Test Matrix

| # | Input Scenario | Config Under Test | Expected Behavior |
|---|---------------|-------------------|-------------------|
| T01 | Unclosed-paren RON string (e.g. `"RenderSpec( assets: ["`) | `RenderSpec::from_ron_str` | returns `Err( LoadError::Ron( _ ) )` |
| T02 | Distinctly-shaped malformed input (e.g. bare unquoted token where a struct is expected) | `SceneSnapshot::from_ron_str` | returns `Err( LoadError::Ron( _ ) )` |
| T03 | Syntactically-valid RON that violates a SPEC §16 rule (e.g. a duplicate id, mirroring an existing `Validation`-test fixture) | `RenderSpec::from_ron_str` then `.validate()` | `from_ron_str` alone returns `Ok`; the failure surfaces only at `.validate()` as `ValidationError`, never as `LoadError::Ron` |

## Acceptance Criteria

-   `tests/ron_syntax_error_test.rs` exists with all 3 Test Matrix cases
    passing
-   Each `Ron`-arm assertion uses `matches!( result, Err( LoadError::Ron( _
    ) ) )` (or equivalent), not a loose `is_err()` that would also accept
    `Validation`/`Io`
-   No pre-existing test in `tilemap_scene`'s suite regresses
-   Every Test Matrix row has a corresponding passing test

## Verification

**Execution:** The procedure for walking this section is defined in `§ Acceptance Verification : Procedure - Execution`. The executor does NOT self-verify — an independent verifier performs the walk after the task reaches 🔎 Accepting (via EXEC_COMPLETE, ⚙️ → 📦, followed by an acceptance CLAIM, 📦 → 🔎).

### Checklist

Desired answer for every question is YES.

**Tests**
- [ ] C1 — Does `tests/ron_syntax_error_test.rs` exist with T01/T02/T03 as distinct `#[test]` functions?
- [ ] C2 — Does each `Ron`-arm assertion specifically match `LoadError::Ron( _ )` (not a bare `is_err()`)?

**Out of Scope confirmation**
- [ ] C3 — Is `error.rs`/`load.rs`/`validate.rs` unmodified (`git diff` shows zero source changes, test file only)?
- [ ] C4 — Does the new test file avoid constructing any `LoadError::Io` case (that variant stays covered only by std's existing file-not-found contract)?
- [ ] C5 — Does the new test file avoid referencing `SnapshotLoadError` entirely?
- [ ] C6 — Do all new test cases call `from_ron_str` directly, never `.load( path )` (the file-based entry point)?

### Measurements

- [ ] M1 — new test count: `cargo test -p tilemap_scene --test ron_syntax_error_test 2>&1 | grep -c "test result: ok"` → 1 (was: file did not exist)

### Invariants

- [ ] I1 — test suite: `verb/test` → 0 failures
- [ ] I2 — compiler clean: `RUSTFLAGS="-D warnings" cargo check -p tilemap_scene --all-features` → 0 warnings

### Anti-faking checks

- [ ] AF1 — T01/T02's malformed input strings are genuinely different from each other and from any fixture already used elsewhere in the suite (not a single copy-pasted string reused across cases) — checked by reading the 3 literal string arguments in `tests/ron_syntax_error_test.rs`, not merely by the tests passing

## Verification Record

**Gate Round 1** (Tier 2 — Dual-Role Self-Check, one-shot, self-administered by user1@w002)

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Scope Coherence | — | 🟢 | — | — |
| D2 | MOST Goal Quality | — | 🟢 | — | — |
| D3 | Value / YAGNI | — | 🟢 | — | — |
| D4 | Implementation Readiness | — | 🟢 | — | — |
| D5 | Execution Scope | — | 🟢 | — | — |
| D6 | Crate Scope Unity | — | 🟢 | — | — |
| D7 | Crate Locality | — | 🟢 | — | — |
| D8 | Crate Single Responsibility | — | 🟢 | — | — |
| **Total** | | — | 🟢 | — | — |

## History

*(append-only — newest entry last; never edit or remove past entries)*

- **[2026-08-15]** `FILED` — Task filed via `/doc_tsk` Phase 2 (docs/layer gap audit): add malformed-RON-syntax `LoadError::Ron` test coverage to `tilemap_scene`.

## Related Documentation

- `module/helper/tilemap_scene/src/error.rs` — `LoadError` enum, the type under test
- `module/helper/tilemap_scene/src/load.rs` — `from_ron_str`/`load`, the functions under test
- `docs/layer/005_l4_scene_model.md` — `tilemap_scene`'s declarative-model layer entry, RON as canonical form
