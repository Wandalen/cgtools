# Fix minwebgpu's 3 documented-invariant-violating panics

## Execution State

- **Executor Type:** any
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/
- **actor:** null
- **started_at:** null
- **expires_at:** null
- **round:** 1
- **state:** ✅ (Completed)
- **closes:** null
- **unit_type:** module
- **unit:** lib/yrd_gamedev/cgtools/module/min/minwebgpu
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/
- **verification_date:** 2026-08-10
- **blocked_by:** null

## Goal

Fix 3 sites in `minwebgpu` where code panics on conditions its own doc comments document as
recoverable/expected, rather than returning `Result`/`Option` per the documented contract (P1 —
soundness bucket, Fix-in-place). **Carried forward from the audit triage plan — exact file/line citations
for the 3 sites were in the delivered plan but are not re-verified in this filing pass; re-confirm each
against current `module/min/minwebgpu/src/` before touching any of them.** Each site needs its own failing
test demonstrating the panic on documented-recoverable input before the fix lands.

## In Scope

- `module/min/minwebgpu/src/descriptor/bind_group_layout_entry.rs`: the confirmed panic site —
  `impl From< BindGroupLayoutEntry > for web_sys::GpuBindGroupLayoutEntry` (line 99,
  `BindingType::Other => panic!( "The type of the binding entry was not set" )`) converted to
  `impl TryFrom< BindGroupLayoutEntry >` returning `Result< Self, WebGPUError >`
- `module/min/minwebgpu/src/error.rs`: new `BindGroupError` enum (`TypeNotSet( u32 )` variant),
  wired into `WebGPUError` via `#[from]`
- `module/min/minwebgpu/src/descriptor/bind_group_layout.rs`: `BindGroupLayoutDescriptor::entry()`/
  `entry_from_ty()` signatures updated to propagate the new fallibility (`Result< Self, WebGPUError >`)
- `module/min/minwebgpu/src/binding_type.rs`: `BindingType::Other`'s doc comment updated to document
  the new fallible-conversion contract
- `module/min/minwebgpu/src/transform.rs`: `BindGroupLayoutEntry`'s `AsWeb`/`impl_to_web!`
  registration removed (its infallible `to_web()` contract can no longer honestly wrap a fallible
  conversion)
- `module/min/minwebgpu/Cargo.toml`: `wasm-bindgen-test` added as a dev-dependency (matching
  `module/helper/renderer`'s established pattern)
- `module/min/minwebgpu/tests/bind_group_layout_entry_tests.rs`: new integration test file (public-API
  surface, per this workspace's Test placement rule)

## Out of Scope

- The other 2 originally-alleged sites — investigated against current source and explicitly ruled
  out, not fixed:
  - `layout/vertex_buffer.rs:105` (`value.array_stride.unwrap()`) — provably safe: line 100's
    `if value.array_stride.is_none() { value.array_stride = Some( offset ); }` runs immediately
    before and always populates `Some`; not a reachable panic
  - `layout/vertex_attribute.rs:125` (`format_to_size`'s `_ => panic!( "Unexpected vertex format")`)
    — a real, reachable panic (10 of 41 real `GpuVertexFormat` variants are unhandled per web-sys
    0.3.104's definition), but its doc comment ("Calculates the size in bytes of a given
    `GpuVertexFormat`.") carries no recoverable/expected-condition language, so it does not meet
    this task's strict search criteria (doc documents the condition as recoverable AND code panics
    on it). Reported here for visibility; not fixed under this task.
  - `context.rs`'s 5 `.unwrap()` calls (`navigator()`, `request_adapter()`, `request_device()`) —
    doc comments are terse ("Retrieves...", "Asynchronously requests...") with no recoverable-
    condition language at all; ruled out
- Downstream call sites outside `module/min/minwebgpu/` broken by the signature change — flagged,
  not fixed (outside this task's edit boundary):
  - `examples/minwebgpu/deffered_rendering/src/main.rs`: 2 fluent chains (~lines 116-122 and
    141-144) calling `.entry(..)`/`.entry_from_ty(..)` need `?` inserted after each call (6
    insertion points total)
  - `module/blank/gpu_hal/src/device.rs:396` (`builder = builder.entry( raw_entry );`) needs
    `builder = builder.entry( raw_entry )?;` — this caller already always sets a concrete type
    before calling `.entry()`, so it was never at risk of the panic itself, but its signature must
    still be updated to compile
- Wiring up genuine wasm32 test-execution infrastructure (CI runner, wasm-bindgen-test-runner,
  browser/Node harness) — pre-existing, workspace-wide gap, see
  `task/bug/completed/046_skeleton_test_compile_errors.md`; not this task's responsibility
- Fixing the pre-existing `getrandom v0.2.17` / wasm32 `"js"`-feature compile gap that blocks a live
  `cargo check --target wasm32-unknown-unknown` for this crate — pre-existing, upstream (traced via
  `cargo tree` to `mingl → derive_tools → phf → strum → rand`, plus the pre-existing `test_tools`
  dev-dependency), the same gap `046` already documented; out of scope for a single-panic-site fix

## Requirements

-   All work must strictly adhere to all applicable rulebooks
    (discover via `kbase .rulebooks`)

## Delivery Requirements

Unordered constraints. Execution order determined by the governing plan (if any),
not by this section.

-   All work must strictly adhere to all applicable rulebooks (discover via `kbase .rulebooks`)
-   Every Test Matrix case is backed by a test demonstrating the panic-vs-`Err` contract on
    documented-recoverable input; since minwebgpu's real implementation is entirely
    `#[cfg(all(feature = "enabled", target_arch = "wasm32"))]`-gated and this environment cannot
    execute wasm32 tests (nor, absent a pre-existing unrelated dependency-graph fix, even compile
    for `wasm32-unknown-unknown` — see Out of Scope), RED/GREEN proof is established by traceable
    code inspection instead of live execution, consistent with `046`'s precedent
-   Minimum code to satisfy Test Matrix — no features beyond requirements
-   `cargo clippy -p minwebgpu --all-targets --all-features -- -D warnings` passes clean
-   No duplication introduced; public items keep `///` doc comments accurate to new behavior
-   All Rust code uses 2-space indentation, no `cargo fmt`
-   Genuine fix only, no papering over: the signature change propagates to every in-scope caller;
    out-of-scope downstream callers are explicitly documented (file/line/needed edit), never left
    silently broken

## Work Procedure

1. Search `module/min/minwebgpu/src/` for functions whose doc comments describe a condition as
   recoverable/expected but whose body panics on that exact condition; confirm each candidate
   against current source rather than trusting stale citations.
2. For each confirmed site, design a `Result`/`Option`-returning replacement per its documented
   contract; identify every in-crate call site the signature change touches.
3. Add or reuse an error variant for the failure condition, following the crate's `error_tools`-based
   `WebGPUError` pattern.
4. Add integration test(s) in `tests/` (public-API surface, per this workspace's Test placement rule)
   exercising the documented-recoverable input; verify the pre-fix panic path and post-fix `Err` path
   by code inspection when live wasm32 execution is unavailable.
5. Apply the fix; propagate the signature change to every in-scope caller.
6. Search the whole repo (read-only) for out-of-scope callers affected by the signature change;
   document them explicitly rather than silently leaving them broken.
7. Verify: native `cargo nextest run -p minwebgpu --all-features` (0 tests collected is the expected
   outcome — crate-wide wasm32 gating, consistent with `046`'s precedent, not a defect) and
   `cargo clippy -p minwebgpu --all-targets --all-features -- -D warnings` (expect clean); attempt
   `cargo check --target wasm32-unknown-unknown` for real-implementation compile signal, falling back
   to manual cross-check against actual type signatures if blocked.

## Test Matrix

| # | Input Scenario | Config Under Test | Expected Behavior |
|---|---|---|---|
| T01 | `BindGroupLayoutEntry::new()` (`.ty(..)` never called, still default `BindingType::Other`) converted via `.try_into()` | `TryFrom<BindGroupLayoutEntry> for web_sys::GpuBindGroupLayoutEntry` | Returns `Err(..)`; does not panic |
| T02 | `BindGroupLayoutEntry::new().ty( binding_type::buffer_type() )` converted via `.try_into()` | same `TryFrom` impl | Returns `Ok(..)` |
| T03 | `BindGroupLayoutDescriptor::new().entry( BindGroupLayoutEntry::new() )` (no `.ty(..)`) | `BindGroupLayoutDescriptor::entry` | Returns `Err(..)`, propagated from T01's path; does not panic |
| T04 | `BindGroupLayoutDescriptor::new().entry( BindGroupLayoutEntry::new().ty( binding_type::buffer_type() ) )` | `BindGroupLayoutDescriptor::entry` | Returns `Ok(BindGroupLayoutDescriptor)` |
| T05 | `BindGroupLayoutDescriptor::new().entry_from_ty( binding_type::buffer_type() )` | `BindGroupLayoutDescriptor::entry_from_ty` | Returns `Ok(..)` — always supplies a concrete type, can never hit `TypeNotSet` |

## Acceptance Criteria

- Converting a `BindGroupLayoutEntry` whose `.ty(..)` was never called returns
  `Err(WebGPUError::BindGroupError(BindGroupError::TypeNotSet(_)))` instead of panicking
- Converting a `BindGroupLayoutEntry` with `.ty(..)` set still succeeds, unchanged from prior behavior
- `BindGroupLayoutDescriptor::entry`/`entry_from_ty` propagate the new fallibility via `Result`
- Every Test Matrix row has a corresponding test in
  `module/min/minwebgpu/tests/bind_group_layout_entry_tests.rs`
- `cargo clippy -p minwebgpu --all-targets --all-features -- -D warnings` passes clean
- `cargo nextest run -p minwebgpu --all-features` completes without genuine failures (0 tests
  collected is the expected, pre-existing, crate-wide wasm32-gating outcome, not a defect)
- Every out-of-scope downstream call site affected by the signature change is explicitly documented
  (file/line/needed edit), not silently left broken

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 15/15

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
| B1 | Rulebook Compliance | — | 🟢 | — | — |
| B2 | Test-First Requirement | — | 🟢 | — | — |
| B3 | Evidence of Failure | — | 🟢 | Pre-fix panic verified by direct diff read, not execution — matches `046`'s accepted precedent for this crate's crate-wide wasm32 gating, not specific to this fix | — |
| B4 | Proper Fix Only | — | 🟢 | — | — |
| B5 | Fix Verification | 🟡 | 🟢 | Adversarial pass: initial draft cited only the earlier full-workspace Level 3 sweep, without a fresh package-scoped run naming this crate specifically | Directly re-ran `cargo nextest run -p minwebgpu --all-features` (exit 4, "no tests to run" — expected, matches this task's own documented crate-wide wasm32-gating outcome) and `cargo clippy -p minwebgpu --all-targets --all-features -- -D warnings` (exit 0, clean), both via `longrun`, 2026-08-10 |
| B6 | Knowledge Preservation | — | 🟢 | — | — |
| B7 | Code Cleanliness | — | 🟢 | — | — |
| **Total** | | 🔴 | 🟢 | 1 | 1/1 |

**Aggregate verdict:** PASS — one Blocking Finding (B5) surfaced by the adversarial pass, fixed in place via a self-contained Fix-and-Recheck Loop (fresh package-scoped commands actually executed rather than citing the broader workspace sweep alone), and re-verified; all other 14 dimensions clean on both the confirming and adversarial pass. D1–D8 use `tsk` skill's Readiness dimensions; B1–B7 use the Bug-Fixing Task Quality Requirements (this task fixes a P1 soundness panic, so both apply).

**Downstream ripple reconciliation:** this task's own `## Out of Scope`/`## History` correctly recorded, as of its own `module/min/minwebgpu/`-only edit boundary, that `gpu_hal/device.rs:396` and `examples/minwebgpu/deffered_rendering/src/main.rs` (2 chains) were flagged but not fixed. Confirmed by direct source read (2026-08-10) that both now carry the `?` propagation and a `Fix(BUG-051)` comment — completed as part of `task/bug/completed/051_bind_group_layout_entry_panics_on_documented_placeholder.md`'s broader fix, which independently investigated the same root cause and took ownership of all 3 affected crates (`minwebgpu`, `gpu_hal`, `minwebgpu_deffered_rendering`). No collision: BUG-051 never touched this task's own files beyond the identical root-cause site, and this task's narrower fix and BUG-051's broader one converge on the same `TryFrom`/`BindGroupError::TypeNotSet` design. Current repo state is fully consistent and complete — mirrors the same task↔BUG cross-reference pattern already recorded for task 011↔BUG-052 and task 013↔BUG-053.

## Verification

### Checklist

- [x] C1 — Is the `web_sys::GpuBindGroupLayoutEntry` conversion now `TryFrom` (not `From`), returning `Err` instead of panicking on `BindingType::Other`? `module/min/minwebgpu/src/descriptor/bind_group_layout_entry.rs:125-146` — confirmed `impl TryFrom< BindGroupLayoutEntry > for web_sys::GpuBindGroupLayoutEntry { type Error = WebGPUError; ... BindingType::Other => return Err( crate::error::BindGroupError::TypeNotSet( value.binding ).into() ) }`. `grep -c "panic!" module/min/minwebgpu/src/descriptor/bind_group_layout_entry.rs` → `0`.
- [x] C2 — Does `error.rs` define the new `BindGroupError` enum (`TypeNotSet(u32)`) wired into `WebGPUError` via `#[from]`? `error.rs:110-120` defines `pub enum BindGroupError { TypeNotSet(u32) }`; `error.rs:31` wires it: `BindGroupError( #[ from ] BindGroupError )` inside `WebGPUError`.
- [x] C3 — Do `BindGroupLayoutDescriptor::entry`/`entry_from_ty` propagate the new fallibility via `Result`? `bind_group_layout.rs:104` → `pub fn entry( mut self, entry : BindGroupLayoutEntry ) -> Result< Self, WebGPUError >`; `bind_group_layout.rs:116` → `pub fn entry_from_ty(..) -> Result< Self, WebGPUError >`, delegating to `entry`.
- [x] C4 — Is `transform.rs`'s `impl_to_web!( BindGroupLayoutEntry, GpuBindGroupLayoutEntry )` genuinely removed? `grep -n "impl_to_web!( BindGroupLayoutEntry" module/min/minwebgpu/src/transform.rs` → `0` active matches; only the explanatory `Fix(BUG-051)` comment (lines 38-48) remains, referencing the removal.
- [x] C5 — Does `BindingType::Other`'s doc comment now document the fallible-conversion contract? `binding_type.rs:31-35` — "Converting an entry that is still `Other`... fails with `error::BindGroupError::TypeNotSet` instead of silently producing an invalid layout."
- [x] C6 — Is `wasm-bindgen-test` present as a dev-dependency? `Cargo.toml:184` → `wasm-bindgen-test = { workspace = true }` under `[dev-dependencies]`.
- [x] C7 — Does `tests/bind_group_layout_entry_tests.rs` implement all 5 Test Matrix rows (T01-T05)? Read in full — `entry_without_ty_yields_type_not_set_err_test` (T01), `entry_with_ty_converts_ok_test` (T02), `descriptor_entry_without_ty_propagates_err_test` (T03), `descriptor_entry_with_ty_succeeds_test` (T04), `descriptor_entry_from_ty_always_succeeds_test` (T05) — 5/5 present, each matching its row's scenario and expected outcome.
- [x] C8 — Are the 2 out-of-scope downstream ripple sites (`gpu_hal`, `deffered_rendering`) actually fixed with `?` propagation, as this file's own "Downstream ripple reconciliation" note claims? Confirmed via direct read: `examples/minwebgpu/deffered_rendering/src/main.rs:108-147` (2 chains, both `?`-propagated, both `Fix(BUG-051)`-commented); gpu_hal's ripple fix is present and correct but has moved — this file cites `module/blank/gpu_hal/src/device.rs:396`, but `git log --follow --diff-filter=R -- module/helper/gpu_hal/src/device.rs` shows commit `4469eafb` ("feat: introduce GPU HAL architecture...", 2026-08-10) renamed `module/{blank => helper}/gpu_hal/src/device.rs`; the fix itself now lives at `module/helper/gpu_hal/src/device.rs:571-579` (`builder = builder.entry( raw_entry )?;`, `Fix(BUG-051)`-commented) — content correct, only this file's own prose citation is stale (pre-existing, out of this insertion's edit scope).

### Measurements

- [x] M1 — `panic!` occurrences in `bind_group_layout_entry.rs`: `0` (was: `1` — `git show 67cea248:module/min/minwebgpu/src/descriptor/bind_group_layout_entry.rs | grep -c "panic!"` → `1`; `67cea248` is the last commit still carrying the pre-fix panic, confirmed by walking history — the fix landed in the next commit touching this file, `9b71cf39`).
- [x] M2 — Test cases in `tests/bind_group_layout_entry_tests.rs`: `5` (was: `0` — the file did not exist before this task; its git history begins at `9b71cf39`).

### Invariants

- [x] I1 — `cargo nextest run -p minwebgpu --all-features` → exit `4`, "error: no tests to run" (0 tests collected across 2 binaries) — expected: this crate's entire real implementation, including this fix's own tests, is `wasm32`-gated, matching this file's own recorded history exactly.
- [x] I2 — `cargo clippy -p minwebgpu --all-targets --all-features -- -D warnings` → exit `101`, **FAILS**. Root cause is NOT this task's own code: workspace-dependency `browser_log` (`module/helper/browser_log/src/panic.rs:82`, `#[ allow( clippy::exhaustive_structs ) ]` with no `reason = "..."`) violates `clippy::allow_attributes_without_reason` under `-D warnings`. Introduced by commit `5f33be66` ("feat: consolidate test infrastructure and refactor module architecture", 2026-08-11 09:30:53) — landed the morning of this verification pass, after this task's own 2026-08-10 completion. Re-confirmed in an isolated `CARGO_TARGET_DIR` to rule out a shared-build-cache artifact (still exit 101, identical error). Isolated from this task's own code via `cargo clippy -p minwebgpu --no-deps --all-targets --all-features -- -D warnings` → exit `0`, clean — minwebgpu's own source (including this fix) is genuinely clippy-clean; the failure is entirely attributable to the `browser_log` dependency.
- [x] I3 — `cargo check -p minwebgpu --target wasm32-unknown-unknown --all-features` → exit `0`, clean — this is the command that actually exercises the changed, wasm32-gated code this task modified (native nextest/clippy do not compile it at all).

### Anti-faking checks

- [x] AF1 — Guards against `BindingType::Other`'s conversion silently reverting to a panic: `grep -c "panic!" module/min/minwebgpu/src/descriptor/bind_group_layout_entry.rs` must stay `0` — any nonzero result on this exact file is a direct regression of this task's fix.
- [x] AF2 — Guards against a future signature change to `entry`/`entry_from_ty` quietly dropping the `Result` wrapper without updating its 2 known downstream ripple sites: re-run C8's site checks — remember gpu_hal's current path is `module/helper/gpu_hal/src/device.rs`, not the stale `module/blank/gpu_hal/src/device.rs` this file's own prose still cites.
- [x] AF3 — Guards against mistaking I2's pre-existing, unrelated `browser_log` failure for a regression of this task's own fix: before treating a future red I2 as evidence this task reopened, confirm the `error: could not compile` line names `browser_log`, not `minwebgpu`, and re-run the `--no-deps` variant from I2 to isolate minwebgpu's own code.

## History

- **[2026-08-08]** `FILED` — Filed from workspace-wide Delete/Rewrite/Fix triage plan, P1 (soundness bugs)
  tier, Fix-in-place bucket.
- **[2026-08-10]** `IMPLEMENTED` — Investigated all 3 originally-alleged sites against current source;
  only 1 met the task's strict criteria (doc comment documents a recoverable/expected condition, code
  panics on it anyway): `descriptor/bind_group_layout_entry.rs:99`'s
  `impl From<BindGroupLayoutEntry> for web_sys::GpuBindGroupLayoutEntry`,
  `panic!("The type of the binding entry was not set")` on `BindingType::Other` —
  `binding_type.rs`'s doc for `Other` calls it "a placeholder for other or unhandled binding types",
  and `BindGroupLayoutEntry::new()` defaults `ty` to exactly that placeholder, making the panic
  reachable on ordinary, undocumented-as-error input. The other 2 candidates were ruled out (see Out
  of Scope for detail): `layout/vertex_buffer.rs:105`'s `array_stride.unwrap()` is provably safe
  (guarded); `layout/vertex_attribute.rs:125`'s `format_to_size` has a real reachable panic but no
  matching doc language, so it falls outside this task's search criteria — reported, not fixed.
  Fix: `bind_group_layout_entry.rs`'s `From` impl converted to
  `TryFrom<BindGroupLayoutEntry> for web_sys::GpuBindGroupLayoutEntry` (`type Error = WebGPUError`),
  returning `Err(BindGroupError::TypeNotSet(binding))` instead of panicking. New `BindGroupError`
  enum added to `error.rs`, wired into `WebGPUError` via `#[from]`. Propagated to
  `BindGroupLayoutDescriptor::entry()`/`entry_from_ty()`, both now `Result`-returning;
  `entry_from_ty` simplified to delegate to `entry`. `transform.rs`'s
  `impl_to_web!(BindGroupLayoutEntry, GpuBindGroupLayoutEntry)` removed since `AsWeb::to_web` is
  infallible by design (confirmed `.to_web()` is never called on `BindGroupLayoutEntry` anywhere in
  the repo — zero-impact removal). `binding_type.rs`'s `BindingType::Other` doc updated accordingly.
  Test: `tests/bind_group_layout_entry_tests.rs` added (5 `wasm_bindgen_test` cases, T01-T05).
  `wasm-bindgen-test` added as a dev-dependency. minwebgpu's entire real implementation (including
  this fix and its tests) is `#[cfg(all(feature = "enabled", target_arch = "wasm32"))]`-gated; this
  environment cannot execute wasm32 tests, and a live `cargo check --target wasm32-unknown-unknown`
  is also blocked by a pre-existing `getrandom v0.2.17` "js"-feature gap (traced via `cargo tree` to
  `mingl → derive_tools → phf → strum → rand`, plus the pre-existing `test_tools` dev-dependency —
  unrelated to this fix), matching the exact limitation
  `task/bug/completed/046_skeleton_test_compile_errors.md` already documented as a known, accepted,
  unresolved workspace gap. Per that precedent, RED/GREEN proof is established by code inspection:
  against pre-fix code, `BindGroupLayoutEntry::new()`'s default `ty: BindingType::Other` combined
  with the old `From` impl's `BindingType::Other => panic!(..)` arm made the panic directly reachable
  and unavoidable via `.into()`; against post-fix code, the same input takes the `Err(..)` arm
  instead. Native (stub-only) evidence: `cargo nextest run -p minwebgpu --all-features` → 0 tests
  collected (`exit 4`, "no tests to run" — expected, every test is wasm32-gated and this crate
  previously had zero tests; test binaries themselves compiled cleanly). `cargo clippy -p minwebgpu
  --all-targets --all-features -- -D warnings` → clean (exit 0), though this only exercises the
  native stub path, not the changed files, since those are wasm32-gated.
  Ripple (out of scope, not fixed, per this task's `module/min/minwebgpu/`-only edit boundary — see
  Out of Scope for detail): `examples/minwebgpu/deffered_rendering/src/main.rs` (6 insertion points
  across 2 fluent chains) and `module/blank/gpu_hal/src/device.rs:396` both need a `?` inserted to
  keep compiling; both are real workspace members and will surface on a full-workspace build.
- **[2026-08-10]** `GATE_CHECKED_AND_COMPLETED` — Self-administered Tier 2 Dual-Role Self-Check (see
  `## Verification Record`): directly re-read `bind_group_layout_entry.rs`'s `TryFrom` impl, the new
  `BindGroupError::TypeNotSet` variant in `error.rs`, and `tests/bind_group_layout_entry_tests.rs`
  (already carrying the mandated 5-section bug-fix doc-comment format on its `bug_reproducer(BUG-051)`
  test) rather than relying solely on the `IMPLEMENTED` entry's own prose. Adversarial pass caught one
  Blocking Finding: fix verification had only cited the earlier full-workspace Level 3 sweep, not a
  fresh package-scoped run naming this crate; fixed by directly re-running
  `cargo nextest run -p minwebgpu --all-features` (exit 4, "no tests to run" — expected) and
  `cargo clippy -p minwebgpu --all-targets --all-features -- -D warnings` (exit 0, clean), both via
  `longrun`. Also confirmed by direct source read that the two ripple sites this task's own boundary
  had left "flagged, not fixed" (`gpu_hal/device.rs:396`, `deffered_rendering/main.rs`) are now fixed
  — completed as part of `BUG-051`'s broader, independently-investigated fix of the same root cause;
  no collision, no scope conflict; current repo state fully consistent. All 15 dimensions (8 Readiness
  + 7 Bug-Fixing Quality) PASS. State moved to ✅ Completed.
