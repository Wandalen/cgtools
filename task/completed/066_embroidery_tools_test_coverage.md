# Restore test-directory convention and coverage in embroidery_tools (decomposed from task 035)

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
- **unit:** module/helper/embroidery_tools
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/
- **verification_date:** 2026-08-10
- **blocked_by:** null

## Goal

Census 2026-08-10 (task 035 — re-derive at pickup): **0 tests/ files; 8 inline #[test] in src/**. Zero tests/ directory; all 8 tests live inline in src/ (format readers/writers). Native crate — no wasm barrier.

Per-test procedure (uniform across the 035 decomposition):
1. For each inline `#[ test ]` in `src/`: if it exercises public API only, relocate it to
   `tests/`; if it needs private access, DECIDE — expose the tested item (only when the API
   genuinely warrants it) or keep it in place as a documented exception (inline unit tests testing
   true internals are the known tension with the all-tests-in-tests/ convention; a blanket move that
   forces API widening is worse than a recorded exception). Never delete a test to satisfy the rule.
2. If the crate has no `tests/` at all, establish it with real behavior tests of the public
   surface — no mocks, loud failures.
3. Verify with `longrun .launch dir::<workspace root> -- cargo test -p embroidery_tools --all-features` —
   all green before and after each relocation batch.

## History

- **[2026-08-10]** `FILED` — Decomposed from task 035's workspace test-coverage census per Crate
  Scope Unity (PROC17). Claim-vs-reality dimension of 035 dissolved workspace-wide (zero readme
  coverage claims found); this crate carries the tests-location/coverage remainder.
- **[2026-08-10]** `IMPLEMENTED` — Census re-derived at pickup and confirmed: 8 inline `#[ test ]`
  across 5 src files, no `tests/`. All 8 are public-API-only (verified against `mod_interface`
  exports — every touched item is `own`/`orphan`-exported; all struct fields `pub`; only
  `Instruction`/`PESVersion` are `non_exhaustive`, used variant-wise only) — so ALL relocate, no
  documented exceptions needed:
  - `tests/embroidery_file_test.rs` — the 2 stitch-accumulation tests (relative accumulates,
    absolute keeps), plus 2 NEW pins of untested public methods: `bounds()` (min/max quadruple)
    and `as_command_blocks()` (splits at instruction changes — expected blocks derived from the
    algorithm: `[ Stitch, Stitch ], [ Jump ], [ Stitch ]`).
  - `tests/pes_test.rs` — writer-vs-fixture byte-prefix tests for V1 (192 bytes) and V6
    (361 bytes), with the duplicated 14-line stitch program consolidated into one
    `fixture_program()` helper; plus the V6 write→read roundtrip preserving 4 metadata texts and
    4 thread fields (relocated from `pes/reader.rs`; debug `println!` dropped, intermediate
    bindings folded into direct asserts).
  - `tests/pec_test.rs` — the 2 fixture-decode tests (14 stitch asserts matching pyembroidery,
    2 default-palette thread resolutions) and the encoding roundtrip (9 stitch asserts + thread
    selection). One dead commented-out assert (`threads()[ 1 ]`) deliberately NOT relocated —
    commented-out code is banned; noted here instead.
  - `tests/readme.md` — Responsibility Table for the 3 test files; documents the relative
    `test_files/` fixture-path contract (test binaries run with crate root as cwd).
  - 5 src files cleaned via boundary-asserted line-range cuts (blank line + `#[ cfg( test ) ]`
    block + `mod test(s)` close verified before each cut); `Cargo.toml` already shipped
    `tests/**/*` in its include list — no packaging change needed.
- **[2026-08-10]** `GATE_CHECKED_AND_COMPLETED` — Suite green: log `-0032` exit 0 — unit 0
  (relocation complete), integration 4 + 3 + 3 = 10 passed (8 relocated + 2 new), doc-tests 0,
  0 failed; fixture-relative paths proven by the passing from-disk tests. Post-cut grep: zero
  `cfg( test )`/`#[ test ]`/`mod tests` remain anywhere in src (exit 1). Two genuine in-loop
  adversarial catches: (1) the two `test_version6` names from `pes/writer.rs` and `pes/reader.rs`
  would COLLIDE when merged into one `pes_test.rs` — all tests renamed descriptively;
  (2) relocation moved tests from inside `mod private` (private access) to external linkage —
  every touched item was re-verified externally reachable BEFORE cutting, catching that this
  cannot be assumed from "tests pass inline" (e.g. `Thread`'s FRU literal requires the struct to
  not be `non_exhaustive` externally).

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 15/15

| Gate | Name | Prev | Now | Issues | Fixes |
| ---- | ---- | ---- | --- | ------ | ----- |
| D1 | Scope Coherence | 🟢 | 🟢 | — | — |
| D2 | MOST Goal Quality | 🟢 | 🟢 | — | — |
| D3 | Value/YAGNI | 🟢 | 🟢 | Only 2 tests added beyond relocation, both pinning documented-but-untested public methods (`bounds`, `as_command_blocks`) | — |
| D4 | Implementation Readiness | 🟢 | 🟢 | Census re-derived: 8 inline tests confirmed, none hidden (cfg-test grep + `#[ test ]` grep both clean after cut) | — |
| D5 | Execution Scope | 🟢 | 🟢 | — | — |
| D6 | Crate Scope Unity | 🟢 | 🟢 | — | — |
| D7 | Crate Locality | 🟢 | 🟢 | — | — |
| D8 | Crate Single Responsibility | 🟢 | 🟢 | — | — |
| B1 | Rulebook Compliance | 🟡 | 🟢 | Relocating `pec/writer.rs`'s commented-out assert would plant banned dead code in a new file | Dropped; recorded in History instead |
| B2 | Test-First | 🟢 | 🟢 | Relocation task — the tests ARE the change; green run is the evidence | — |
| B3 | Evidence of Failure | 🟢 | 🟢 | — | — |
| B4 | Proper Fix Only | 🟡 | 🟢 | Two `test_version6` fns from different src modules collide when merged into one `pes_test.rs` | All tests renamed descriptively; collision impossible |
| B5 | Fix Verification | 🟢 | 🟢 | Log `-0032` exit 0: unit 0, integration 10/10 (8 relocated + 2 new), 0 failed | — |
| B6 | Knowledge Preservation | 🟢 | 🟢 | tests/readme.md documents fixture-path cwd contract + Responsibility Table | — |
| B7 | Code Cleanliness | 🟡 | 🟢 | Inline tests lived inside `mod private` — external reachability of every touched item cannot be assumed from "passes inline" (FRU literals additionally require non-`non_exhaustive`) | Verified every item `own`/`orphan`-exported + all fields `pub` BEFORE cutting |
| **Total** | | 🔴 | 🟢 | 3 findings resolved in-loop | 15/15 |
