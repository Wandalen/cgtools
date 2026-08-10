# Establish a runnable test story for minwebgl (decomposed from task 035)

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
- **unit:** module/min/minwebgl
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/
- **verification_date:** 2026-08-10
- **blocked_by:** null

## Goal

Census 2026-08-10 (task 035 — re-derive at pickup): **0 tests/ files; 4 inline `#[ test ]` in
src/**. minwebgl is a wasm-first crate (web-sys WebGL2 bindings), so the FIRST deliverable is the
runnability story, not relocation: establish which of the 4 inline tests (and which future tests)
can run natively vs need `wasm-bindgen-test` in a browser/headless runner, and document the chosen
invocation in the crate readme (its sibling mingl runs 34 native test markers across 5 tests/
files — the pure-logic layer is testable; the GL-context layer is not without a browser).

Then apply the 035 uniform procedure: relocate public-API-only tests to `tests/` under whatever
target/gating the runnability story prescribes; private-access tests get an expose-or-exception
decision each; establish real coverage for natively testable pure-logic modules currently untested.
Never set bare `RUSTFLAGS` on wasm32 builds (clobbers `.cargo/config.toml`'s
`--cfg web_sys_unstable_apis`).

Verify with the invocation the runnability story establishes (native subset via
`longrun .launch dir::<workspace root> -- cargo test -p minwebgl` at minimum — confirm it compiles
and runs the native-safe subset, exit 0).

## History

- **[2026-08-10]** `FILED` — Decomposed from task 035's workspace test-coverage census per Crate
  Scope Unity (PROC17). Runnability-first shape chosen because the crate is wasm-targeted: a
  tests/ directory that can't execute anywhere is worse than none.
- **[2026-08-10]** `IMPLEMENTED` — All three deliverables landed:
  - **Runnability story (readme.md § Testing, under Building and Deployment):** the crate
    compiles natively, so the pure-logic layer runs via `cargo test -p minwebgl --all-features`
    — no browser. The GL-context/DOM layer (context, shaders, VAOs, textures, uniforms,
    file/fetch) is NOT natively testable and has no runner yet (workspace-level
    `wasm-bindgen-test` infrastructure gap, same as recorded by tasks 064/068). Includes the
    self-verification recipe (grep the two inline sites, ls tests/) and the standing wasm32
    RUSTFLAGS hazard (bare value clobbers `--cfg web_sys_unstable_apis`).
  - **Relocation decision for the 4 inline tests:** census re-derived and confirmed — 2 in
    `geometry.rs` (BUG-052 `validate_natoms` pair) + 2 in `clean.rs` (TASK-011
    `convert_attachment_id` bug_reproducer pair). ALL FOUR test private helpers whose
    extraction-into-testable-form WAS the respective bug fix; neither helper is in the
    `mod_interface` exports, and publishing either for test placement is API widening for zero
    callers. Both modules kept inline as documented exceptions with rationale comments naming
    the task (068 precedent); RED-state notes and 5-section docs untouched.
  - **New native coverage for the untested public pure-logic surface:**
    `tests/data_type_test.rs` pins the `DataType` → `Const< DataType >` mapping to the seven
    WebGL2 scalar constants (0x1400–0x1406) and the roundtrip back — `Const` is externally
    constructible only via `try_from`, so the roundtrip covers both public conversion
    directions. The `Err` arm is NOT tested: `DataType` currently has exactly the 7 convertible
    variants, so the arm is defensive dead space for future `non_exhaustive` additions —
    untestable externally without inventing API. `tests/readme.md` carries the Responsibility
    Table. Other pure-logic candidates (`error.rs`, `math.rs`, `mem.rs`) are re-export shims
    with no logic to pin — no tests invented for them (YAGNI).
- **[2026-08-10]** `GATE_CHECKED_AND_COMPLETED` — Suite green: log `-0035` exit 0 — unit 4/4
  (the documented-exception inline modules), `tests/data_type_test.rs` 2/2 NEW, doc-tests
  1 passed + 7 ignored (pre-existing `ignore`-marked wasm-context examples), 0 failed. In-loop
  adversarial catches: (1) the planned Err-path test for `NoCorrespndingType` turned out to be
  UNWRITABLE — every current `DataType` variant converts, the `_ =>` arm exists only for future
  variants; test dropped rather than faked. (2) The draft's suggested minimal invocation
  (`cargo test -p minwebgl`, default features) was upgraded to `--all-features` in the
  documented story after confirming all features are additive dep-gates that compile natively —
  matching the invocation every other crate task uses.

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 15/15

| Gate | Name | Prev | Now | Issues | Fixes |
| ---- | ---- | ---- | --- | ------ | ----- |
| D1 | Scope Coherence | 🟢 | 🟢 | — | — |
| D2 | MOST Goal Quality | 🟢 | 🟢 | — | — |
| D3 | Value/YAGNI | 🟡 | 🟢 | Err-path test for `NoCorrespndingType` would require inventing a non-convertible variant — the arm is defensive dead space today | Test dropped, reasoning recorded; only the 7 real mappings pinned |
| D4 | Implementation Readiness | 🟢 | 🟢 | Census re-derived: exactly 4 inline tests in 2 modules, both private-helper pairs from prior bug fixes | — |
| D5 | Execution Scope | 🟢 | 🟢 | — | — |
| D6 | Crate Scope Unity | 🟢 | 🟢 | — | — |
| D7 | Crate Locality | 🟢 | 🟢 | — | — |
| D8 | Crate Single Responsibility | 🟢 | 🟢 | — | — |
| B1 | Rulebook Compliance | 🟢 | 🟢 | Exceptions follow the convention's own escape hatch; RED-state notes + 5-section reproducer docs untouched | — |
| B2 | Test-First | 🟢 | 🟢 | — | — |
| B3 | Evidence of Failure | 🟢 | 🟢 | — | — |
| B4 | Proper Fix Only | 🟢 | 🟢 | Exposing `validate_natoms`/`convert_attachment_id` rejected — extraction-into-private WAS each fix; publishing reverses the design | — |
| B5 | Fix Verification | 🟢 | 🟢 | Log `-0035` exit 0: unit 4/4, new native suite 2/2, doc-tests 1 ok + 7 pre-existing ignored | — |
| B6 | Knowledge Preservation | 🟡 | 🟢 | Runnability story only in the task record would be invisible to the next contributor | readme.md § Testing carries the invocation, the split, the self-verification recipe, and the RUSTFLAGS hazard |
| B7 | Code Cleanliness | 🟢 | 🟢 | — | — |
| **Total** | | 🔴 | 🟢 | 2 findings resolved in-loop | 15/15 |
