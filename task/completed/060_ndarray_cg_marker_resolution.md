# Resolve ndarray_cg's 7 task markers (decomposed from task 038)

## Execution State

- **Executor Type:** any
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/
- **actor:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/
- **started_at:** 2026-08-10
- **expires_at:** null
- **round:** 1
- **state:** ✅ (Completed)
- **closes:** null
- **unit_type:** crate
- **unit:** module/math/ndarray_cg
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/
- **verification_date:** 2026-08-10
- **blocked_by:** null

## Goal

Resolve the 7 live task markers in `module/math/ndarray_cg` (census 2026-08-10, task 038 — re-derive
at pickup). Grouped by kind:

**API hygiene (`src/vector/general.rs`):**
- `:5` — `// qqq : xxx : document please` (undocumented public surface).
- `:152` — `// qqq : xxx : use typed error` (stringly/ad-hoc error at a fallible boundary).
- `:189` — `// xxx : test cover`.
- `:224` — `// // xxx : enable and test cover, maybe` (commented-out code + hedged marker — decide:
  enable and cover, or delete the dead block outright; "maybe" markers must not survive triage).

**Duplication:**
- `src/vector/arithmetics.rs:109` — `// xxx : reuse` (logic that should delegate to an existing
  implementation instead of duplicating it — identify the reuse target before rewriting).

**Lint policy (cross-reference task 058 — do not resolve independently):**
- `Cargo.toml:82` and `:84` — `# missing_docs = "warn" # qqq : uncomment please` and same for
  `missing_debug_implementations`. ndarray_cg does NOT inherit `[workspace.lints]` (task 058's
  inheritance map); resolve these two as part of 058's ndarray_cg pass. This task only confirms
  they were handled when it closes.

Note: `tests/inc/d2_test/arithmetic_test/mul_test.rs:80` matches marker grep but is a fix-doc comment
QUOTING a resolved marker (TASK-014's record) — documentation, not backlog; leave it.

Per-marker outcomes follow task 038's triage contract: fix in code, or file evidence why the marker
stays, or delete if stale. Verify with `cargo test -p ndarray_cg --all-features` (via
`longrun .launch`) — note the alias crate `ndarray_tools` now includes this suite by path (task 038
enabled it), so donor-suite edits must keep both green:
`cargo test -p ndarray_tools --all-features`.

## History

- **[2026-08-10]** `FILED` — Decomposed from task 038's workspace marker census (80 lines → per-crate
  tasks per Crate Scope Unity).
- **[2026-08-10]** `IMPLEMENTED` — All 5 in-code markers resolved; the 2 Cargo.toml lint markers
  confirmed still live and still owned by task 058 (census now returns exactly those two plus the
  mul_test.rs:80 doc-quote the Goal explicitly excludes). Per-marker outcomes:
  - **`document please` (general.rs:5)** — stale, deleted with evidence: every public item in the
    file already carries a doc comment (`splat`/`to_array`/`from_array`/`from_slice`, the
    `IntoVector`/`VectorSpace`/`VectorSpaceMut` traits and their methods); the marker predates the
    docs. Drive-by: "lenght" typo in `from_slice`'s panic doc fixed.
  - **`use typed error` (general.rs:152)** — implemented: new `VectorLengthMismatch { expected,
    actual }` struct (Debug/Clone/Copy/PartialEq/Eq + Display naming both lengths +
    `std::error::Error`), replacing `&'static str` as `TryFrom< &[ E ] >`'s error type; exposed via
    `mod_interface!`. No error crate is adopted by ndarray_cg (no error_tools/thiserror/anyhow —
    verified by grep), so a std-style dedicated type is the consistent choice. Breaking change
    verified safe: zero external `TryFrom`/`try_from` callers on `Vector` workspace-wide.
  - **`test cover` (general.rs:189, on `IntoVector`)** — implemented: new
    `tests/inc/vector_conversion_test.rs` (4 tests: tuple/array `into_vector`, `as_vector`
    non-consumption, `try_from` happy path, typed-error field/Display/boxability contract).
  - **`enable and test cover, maybe` (general.rs:224)** — deleted outright per the Goal's own
    triage rule ("maybe markers must not survive triage"): the commented-out `FromVector` trait had
    zero consumers and no concrete need (YAGNI). Same-file consistency: the unmarked commented-out
    `From< E > for Vector` block and the `// AsVector,`/`// FromVector,` commented exposure lines
    deleted too (house rule: no commented-out code blocks — git history preserves them).
  - **`reuse` (arithmetics.rs:109)** — stale, deleted with evidence: the marker sits directly above
    `reuse ::mdmath_core::vector::arithmetics;` — the exact directive it requests, already present;
    the file's `mag`/`distance`/`min`/`max` methods are thin delegates to the reused free functions,
    so no duplicated logic remains to consolidate.
  Verification: `cargo test -p ndarray_cg -p ndarray_tools --all-features` — exit 0: ndarray_cg 261
  passed (was 257; +4 conversion tests) + 5 doc-tests, ndarray_tools 261 (donor-suite coupling
  intact — the alias runs the same suite by path-include and picked up the new file automatically).
  Log `-0019` (in module/math/mdmath_core/, the invocation cwd).
- **[2026-08-10]** `GATE_CHECKED_AND_COMPLETED` — Dual-Role Self-Check passed 15/15. Adversarial
  catches: (1) the confirming pass initially read `arithmetics.rs:109` as "duplication to fix" per
  the draft's framing; the adversarial pass read the next line and found the reuse directive already
  wired, flipping the outcome from rewrite to stale-deletion; (2) the typed-error change was
  gated on a caller sweep before editing — grep proved zero external callers, converting a
  potentially breaking API change into a verified-safe one; (3) `IntoVector`'s marker deletion
  landed in the same change as its tests, keeping the test-cover obligation and its discharge in
  one verified increment.

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 15/15

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Scope Coherence | 🟢 | 🟢 | — | — |
| D2 | MOST Goal Quality | 🟢 | 🟢 | — | — |
| D3 | Value/YAGNI | 🟢 | 🟢 | `FromVector` maybe-block and `From< E >` dead block deleted, not enabled — zero consumers, no concrete need | — |
| D4 | Implementation Readiness | 🟢 | 🟢 | — | — |
| D5 | Execution Scope | 🟢 | 🟢 | 2 Cargo.toml lint markers left live by design — owned by 058; mul_test.rs doc-quote left per Goal | — |
| D6 | Crate Scope Unity | 🟢 | 🟢 | — | — |
| D7 | Crate Locality | 🟢 | 🟢 | — | — |
| D8 | Crate Single Responsibility | 🟢 | 🟢 | vector_conversion_test.rs responsibility distinct from vec4_test.rs (accessors) — one-second test passed | — |
| B1 | Rulebook Compliance | 🟢 | 🟢 | error_tools not adopted by crate — std-style error consistent with local convention (Framework Consistency rule) | — |
| B2 | Test-First | 🟢 | 🟢 | Typed error + IntoVector landed with their 4 tests in the same increment | — |
| B3 | Evidence of Failure | 🟢 | 🟢 | Census grep before/after; stale-marker evidence recorded per marker | — |
| B4 | Proper Fix Only | 🟡 | 🟢 | Draft framed :109 as duplication needing rewrite; evidence showed reuse already wired | Outcome corrected to stale-deletion with evidence, no code churn |
| B5 | Fix Verification | 🟢 | 🟢 | `cargo test -p ndarray_cg -p ndarray_tools --all-features` exit 0, 261+261 passed, new tests confirmed in BOTH suites (log `-0019`) | — |
| B6 | Knowledge Preservation | 🟢 | 🟢 | Typed-error contract captured in test doc comment; stale-marker rationales in History | — |
| B7 | Code Cleanliness | 🟡 | 🟢 | 3 commented-out code artifacts found in general.rs during the pass | All deleted (git history preserves); census clean |
| **Total** | | 🔴 | 🟢 | 2 findings resolved in-loop | 15/15 |
