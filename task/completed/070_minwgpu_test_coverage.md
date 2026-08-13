# Restore test-directory convention and coverage in minwgpu (decomposed from task 035)

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
- **unit:** module/min/minwgpu
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/
- **verification_date:** 2026-08-10
- **blocked_by:** null

## Goal

Census 2026-08-10 (task 035 — re-derive at pickup): **0 tests/ files; 21 inline #[test] in src/**. Zero tests/ directory; all 21 tests inline. wgpu runs native — establish whether the inline tests exercise GPU paths (need adapter) or pure logic; pure-logic tests relocate cleanly.

Per-test procedure (uniform across the 035 decomposition):
1. For each inline `#[ test ]` in `src/`: if it exercises public API only, relocate it to
   `tests/`; if it needs private access, DECIDE — expose the tested item (only when the API
   genuinely warrants it) or keep it in place as a documented exception (inline unit tests testing
   true internals are the known tension with the all-tests-in-tests/ convention; a blanket move that
   forces API widening is worse than a recorded exception). Never delete a test to satisfy the rule.
2. If the crate has no `tests/` at all, establish it with real behavior tests of the public
   surface — no mocks, loud failures.
3. Verify with `longrun .launch dir::<workspace root> -- cargo test -p minwgpu --all-features` —
   all green before and after each relocation batch.

## In Scope

- `module/min/minwgpu`: kept all 21 pre-existing inline tests (`buffer.rs` 12, `context.rs` 9) in
  place as documented exceptions — pin `pub(super)` builder-state fields with no public getters
- `tests/context_test.rs` (3 new: type-state chain error, `adapter_selector`
  invocation/propagation, `from_instance` adapter-stage config) and `tests/helper_test.rs` (2 new:
  `helper::attr` mapping, sync `request_adapter` shortcut error) — deterministic public-surface
  tests using `wgpu::Backends::empty()`
- `tests/readme.md` Responsibility Table; `wgpu` added to `[dev-dependencies]` as test
  infrastructure

## Out of Scope

- GPU-dependent surface (`build`, `finish_context`, `texture`) — stays untested natively, same
  browser/wasm runner gap recorded by tasks 064/068/069
- Exposing getters or the internal state-marker types (`AdapterBuilder`/`DeviceBuilder`) for test
  placement — rejected as API widening for zero callers
- Selector-priority-over-options-route — not fully provable without a real adapter; test pins
  invocation + error propagation only

## Verification

### Checklist

- [x] C1 — Are all 21 originally-inline tests still inline (12 in `buffer.rs`, 9 in `context.rs`), each carrying a "documented exception (task 070)" rationale comment? `grep -c "#\[ *test *\]" module/min/minwgpu/src/buffer.rs` → `12`; same for `context.rs` → `9`. Both `mod tests` blocks (`buffer.rs:315`, `context.rs:427`) are immediately preceded by a "Documented exception (task 070) to the all-tests-in-tests/ convention..." comment naming this task and the `pub( super )`-field/no-getter rationale.
- [x] C2 — Does `tests/` now exist with the 2 claimed integration test files plus a readme? `ls module/min/minwgpu/tests/` → `context_test.rs`, `helper_test.rs`, `readme.md` — all 3 present.
- [x] C3 — Does `context_test.rs` contain exactly the 3 claimed cases (full type-state chain error, `adapter_selector` invocation+propagation, `from_instance` adapter-stage config)? Read in full — `empty_backends_request_adapter_errors_without_panicking`, `adapter_selector_is_invoked_and_its_error_propagates`, `from_instance_supports_adapter_stage_configuration` — 3/3 present, matching the claimed behavior.
- [x] C4 — Does `helper_test.rs` contain exactly the 2 claimed cases (`attr` field mapping, sync `request_adapter` shortcut error)? Read in full — `attr_maps_arguments_onto_vertex_attribute_fields`, `request_adapter_shortcut_errors_on_empty_backends` — 2/2 present.
- [x] C5 — Does `tests/readme.md` carry the Responsibility Table? Read in full — present, 2 rows (`context_test.rs`, `helper_test.rs`), matching the crate's convention.
- [x] C6 — Is `wgpu` present in `[dev-dependencies]`? `Cargo.toml:14` → `wgpu.workspace = true` under `[dev-dependencies]` (crate does not re-export wgpu from `[dependencies]` alone for test use, per the claim).
- [x] C7 — Is the claimed import-path fix real — does `Context` live at `minwgpu::context::Context`, not the crate root? `tests/context_test.rs:8` → `use minwgpu::{ context::Context, Error };`; `src/lib.rs`'s `mod_interface!` block declares `layer context;` (not `own`/`exposed` at the root), consistent with `own use` not propagating to the parent module.
- [x] C8 — Is the claimed `ContextBuilder`-not-`Debug` workaround (let-else destructuring instead of `{result:?}`/`expect_err`) actually present? `tests/context_test.rs` uses `let Err( error ) = result else { panic!(..) }` at all 3 call sites; no `{result:?}` or `.expect_err(..)` appears against the builder's own `Result`.

### Measurements

- [x] M1 — Total test count for `minwgpu`: `26` (`21` inline + `5` integration) (was: `21` inline / `0` integration — `git ls-tree -r 4469eafb^ -- module/min/minwgpu/` lists no `tests/` entries at all, confirming the pre-fix "0 tests/ files" baseline; `4469eafb` is the commit that added `tests/context_test.rs`/`tests/helper_test.rs`).

### Invariants

- [x] I1 — `cargo nextest run -p minwgpu --all-features` → exit `0` — `26 tests run: 26 passed, 0 skipped`, exactly matching M1 (21 inline + 5 integration).
- [x] I2 — `cargo clippy -p minwgpu --all-targets --all-features -- -D warnings` → exit `0`, clean.
  (Both I1 and I2 were run under an isolated `CARGO_TARGET_DIR` after two initial attempts in the shared workspace `target/` directory failed with transient `.fingerprint`/dep-info "No such file or directory" errors on unrelated crates — confirmed, via `ps aux`, to be caused by several other concurrently-running sibling sessions building/clippy-ing other packages in the same shared `target/` directory at the same wall-clock time, not a `minwgpu` defect.)

### Anti-faking checks

- [x] AF1 — Guards against a future edit silently deleting one of the 21 documented-exception inline tests instead of relocating it: re-run C1's exact counts (`12`/`9`) — a drop below either with no corresponding rationale-comment update is the exact regression this task's own Work Procedure forbade ("Never delete a test to satisfy the rule").
- [x] AF2 — Guards against the 5 new integration tests becoming vacuous: 3 of the 5 assert the specific `Error::RequestAdapterError` variant via `matches!`, not merely `.is_err()` — a future change to `wgpu`'s empty-backends behavior would surface as a loud, specific test failure, not a silently-passing weaker assertion.
- [x] AF3 — Guards against total test count drifting unnoticed: re-run I1 and compare its "N tests run" line against this file's own M1 (`26`) — any deviation not explained by a deliberate, documented addition/removal is drift.

## History

- **[2026-08-10]** `FILED` — Decomposed from task 035's workspace test-coverage census per Crate
  Scope Unity (PROC17). Claim-vs-reality dimension of 035 dissolved workspace-wide (zero readme
  coverage claims found); this crate carries the tests-location/coverage remainder.
- **[2026-08-10]** `IMPLEMENTED` — Census re-derived and confirmed: 21 inline tests (12 in
  `buffer.rs`, 9 in `context.rs`), 0 tests/ files. All three deliverables landed:
  - **Expose-or-exception decision — all 21 stay inline as documented exceptions:** every test
    pins builder-state accumulation through `pub( super )` fields (`inner.*`,
    `instance_descriptor`, `device_descriptor`, …); the context tests additionally construct
    mid-state builders by struct literal, impossible externally (private fields, unexported
    state markers `AdapterBuilder`/`DeviceBuilder`). No getters exist, and the only public
    observables (`build`, `finish_context`) need a live `wgpu::Device`/adapter —
    environment-dependent, so not usable for deterministic tests. Publishing getters or the
    state markers solely for test placement is API widening for zero callers (068/069
    precedent). Rationale comments naming the task added on both `mod tests`.
  - **tests/ established with real, deterministic public-surface behavior tests:** an instance
    created with `wgpu::Backends::empty()` can never yield an adapter on any host, making the
    adapter-request error surface natively testable. `tests/context_test.rs` (3): full
    type-state chain errors with `Error::RequestAdapterError` instead of panicking; custom
    `adapter_selector` is genuinely invoked (Cell flag) and its error propagates — the real
    `Error` value is harvested from a prior empty-backends request since
    `wgpu::RequestAdapterError` is not constructible; `Context::from_instance` accepts
    adapter-stage configuration end-to-end. `tests/helper_test.rs` (2): `helper::attr` maps
    arguments onto `wgpu::VertexAttribute` fields; sync `helper::adapter::request_adapter`
    shortcut surfaces the same typed error. `tests/readme.md` carries the Responsibility
    Table. GPU-dependent surface (`build`, `finish_context`, `texture`) stays untested
    natively — same runner gap recorded by 064/068/069.
  - **`wgpu` added to `[dev-dependencies]`:** the crate does not re-export wgpu, and the
    integration tests need `Backends`/`InstanceDescriptor`/`PowerPreference`/`VertexFormat`.
    Test infrastructure, not API surface.
- **[2026-08-10]** `GATE_CHECKED_AND_COMPLETED` — Suite green: log `-0038` exit 0 — unit
  21/21 (documented-exception inline), `tests/context_test.rs` 3/3 NEW, `tests/helper_test.rs`
  2/2 NEW, doc-tests 0. In-loop adversarial catches: (1) first compile failed on
  `use minwgpu::Context` — `own use` places `Context` in `minwgpu::context::Context`, not the
  crate root (mod_interface `own` does not propagate to parent); import corrected. (2)
  `ContextBuilder` holds a boxed selector closure so it is not `Debug` — `{result:?}` assert
  messages and `expect_err` cannot format the Ok arm; rewritten with `let Err( error ) = …
  else { panic!( … ) }` destructuring so failure output still prints the error value. (3)
  Selector PRIORITY over the options route is not fully provable without a real adapter (both
  routes error on empty backends) — the test honestly pins invocation + error propagation
  only, recorded in its doc comment.

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 15/15

| Gate | Name | Prev | Now | Issues | Fixes |
| ---- | ---- | ---- | --- | ------ | ----- |
| D1 | Scope Coherence | 🟢 | 🟢 | — | — |
| D2 | MOST Goal Quality | 🟢 | 🟢 | — | — |
| D3 | Value/YAGNI | 🟢 | 🟢 | Selector-priority not fully provable without an adapter — test pins invocation + propagation only, limitation recorded in its doc | — |
| D4 | Implementation Readiness | 🟡 | 🟢 | `use minwgpu::Context` failed: `own use` lands in `minwgpu::context::Context`, not crate root | Import corrected; export-path verified against mod_interface semantics |
| D5 | Execution Scope | 🟢 | 🟢 | — | — |
| D6 | Crate Scope Unity | 🟢 | 🟢 | — | — |
| D7 | Crate Locality | 🟢 | 🟢 | — | — |
| D8 | Crate Single Responsibility | 🟢 | 🟢 | — | — |
| B1 | Rulebook Compliance | 🟢 | 🟢 | No mocks — real wgpu instances; empty-backends determinism replaces any fake adapter | — |
| B2 | Test-First | 🟢 | 🟢 | — | — |
| B3 | Evidence of Failure | 🟢 | 🟢 | — | — |
| B4 | Proper Fix Only | 🟢 | 🟢 | Getters/state-marker exports rejected — API widening for zero callers; dev-dependency is test infrastructure, not surface | — |
| B5 | Fix Verification | 🟢 | 🟢 | Log `-0038` exit 0: unit 21/21, context_test 3/3, helper_test 2/2 | — |
| B6 | Knowledge Preservation | 🟢 | 🟢 | Exception comments name the task + rejected options; tests/readme documents the empty-backends determinism argument and the GPU gap | — |
| B7 | Code Cleanliness | 🟡 | 🟢 | `{result:?}`/`expect_err` unusable — `ContextBuilder` not `Debug` (boxed closure) | let-else destructuring keeps loud, informative failures |
| **Total** | | 🔴 | 🟢 | 2 findings resolved in-loop | 15/15 |
