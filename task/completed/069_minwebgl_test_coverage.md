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
- **unit_type:** module
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

## In Scope

- `module/min/minwebgl`: establish the runnability story (readme.md § Testing) documenting which
  layers run natively (`cargo test -p minwebgl --all-features`) versus require a browser
- `tests/data_type_test.rs` (+ `tests/readme.md` Responsibility Table) — new coverage pinning the
  `DataType`↔`Const<DataType>` mapping for all 7 WebGL2 scalar constants and the roundtrip
- The 4 pre-existing inline tests (`geometry.rs` BUG-052 `validate_natoms` pair; `clean.rs`
  TASK-011 `convert_attachment_id` pair) kept inline as documented exceptions, not relocated

## Out of Scope

- GL-context/DOM layer (context, shaders, VAOs, textures, uniforms, file/fetch) — not natively
  testable, no browser/`wasm-bindgen-test` runner yet (workspace-level gap)
- `Err` arm of the `DataType`→`Const` conversion — untestable externally; no non-convertible
  variant exists today
- `error.rs`, `math.rs`, `mem.rs` — re-export shims with no logic; no tests invented (YAGNI)

## Verification

### Checklist

- [x] C1 — Does `tests/data_type_test.rs` exist with 2 tests pinning the 7 `DataType`↔`Const` mappings? Direct read confirms an `EXPECTED` array of exactly 7 `(DataType, u32)` pairs and 2 `#[test]` fns (`data_type_to_const_pins_webgl_constants`, `const_to_data_type_roundtrips`).
- [x] C2 — Does `tests/readme.md` exist with a Responsibility Table? Direct read confirms both, listing `data_type_test.rs`'s responsibility.
- [x] C3 — Does the crate readme's `### Testing` section document the exact invocation, the inline-exception rationale, and the wasm32 `RUSTFLAGS` hazard? Direct read of `readme.md:154-164` confirms all 3 elements present verbatim.
- [x] C4 — Does the documented invocation reproduce the exact claimed breakdown live? `cargo test -p minwebgl --all-features` → 4 unit (`ok`) + 2 integration (`ok`) + 1 doc passed + 7 doc ignored, 0 failed — matches the task's own claimed counts exactly.
- [x] C5 — Is the "Err arm untestable" reasoning (D3/YAGNI) still accurate — does `DataType` still have exactly 7 convertible variants, no more? `data_type.rs`'s `TryFrom` match arms (both directions) list exactly 7 mappings each; no 8th variant added since.

### Measurements

- [x] M1 — Native test count for `minwebgl` (`cargo test -p minwebgl --all-features`): now `7` passing (4 unit + 2 integration + 1 doc; 7 doc ignored) (was: `4`, all inline unit, `0` `tests/` files — cite `git ls-tree 9b71cf39 -- module/min/minwebgl/tests/`, empty output confirming the directory did not exist before this task).

### Invariants

- [x] I1 — Documented invocation, re-run live (not merely re-read): `cargo test -p minwebgl --all-features` → exit `0`.
- [ ] I2 — Lint cleanliness, literal historically-cited command: `cargo clippy -p minwebgl --all-targets --all-features -- -D warnings` → exit `101` today, blocked at the unrelated `browser_log` dependency — same root cause as task 011's I2 (commit `5f33be66`, dated after this task completed), unrelated to `tests/`.
- [x] I3 — Lint cleanliness, isolated: `cargo clippy -p minwebgl --no-deps --all-targets --all-features -- -D warnings` (still exits 101 crate-wide, see task 011's I3) reports zero findings in `tests/data_type_test.rs` or `tests/readme.md`.

### Anti-faking checks

- [x] AF1 — Guards against the readme's documented invocation silently drifting out of sync with what actually runs: re-run C4's live command and diff its summary line counts against `readme.md`'s § Testing description whenever a test is added, removed, or a feature gate changes.
- [x] AF2 — Guards against a genuinely-testable Err-path being added to `DataType` later without a corresponding test: if `DataType` ever gains an 8th, non-convertible-by-design variant, `data_type_test.rs`'s `EXPECTED` array (fixed at 7) will silently stop covering 100% of variants — re-run C5's match-arm count check whenever `DataType` itself changes.

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
