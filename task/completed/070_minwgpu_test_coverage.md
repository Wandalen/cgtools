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
- **unit_type:** crate
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
