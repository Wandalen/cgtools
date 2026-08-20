# Register minwebgpu TextureDescriptor default-format fix (closes BUG-300)

## Execution State

- **Executor Type:** any
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/
- **actor:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/
- **started_at:** 2026-08-19 22:45:32
- **expires_at:** 2026-08-20 00:45:32
- **round:** 1
- **state:** 🔬 (Verifying)
- **closes:** BUG-300
- **unit_type:** module
- **unit:** lib/yrd_gamedev/cgtools/module/min/minwebgpu
- **verified_by:** null
- **verification_date:** null
- **blocked_by:** null
- **in_motion:** true
- **verifying_at:** 2026-08-19 22:45:32
- **verifying_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/
- **unverified_at:** 2026-08-19 22:37:55
- **unverified_by:** system

## Goal

BUG-300 (`task/bug/verified/300_texture_descriptor_default_format_not_storage_capable.md`,
Medium severity, 🎯 Verified) found that `minwebgpu`'s `TextureDescriptor::new()`
(`module/min/minwebgpu/src/descriptor/texture.rs`) defaulted its `format` field to
`GpuTextureFormat::Rgba8unormSrgb` — valid for the builder's `TEXTURE_BINDING`/
`RENDER_ATTACHMENT`/`COPY_SRC`/`COPY_DST` usage flags, but not for `.storage_binding()`,
since no `-srgb` format supports `STORAGE_BINDING` per the WebGPU spec's texture format
capability table. A caller chaining `.storage_binding()` without an explicit `.format(..)`
override got a descriptor a real `GPUDevice.createTexture` call rejects only via an async
device error-scope event, so `texture::create()` (`src/texture.rs:20-30`, whose only error
path catches synchronous throws) silently returned `Ok` for an unusable texture. The fix
(the shared default changed from `Rgba8unormSrgb` to `Rgba8unorm`, valid across all five
usage flags this builder exposes) is already applied, with a `Fix(BUG-300)`/`Root cause`/
`Pitfall` 3-field source comment and a `default_format_supports_storage_binding_test`
(`bug_reproducer(BUG-300)`) reproducer carrying the mandatory 5-section test documentation —
both independently re-confirmed present and correct by direct file read during this task's
own filing (2026-08-18). This task performs the remaining lifecycle bookkeeping —
`tsk.rulebook.md § Core Procedures : Procedure - Promote Bug to Task` (PROC12) — to formally
register that already-complete, already-verified fix as a tracked task, closing BUG-300.
Testable: `grep -c 'let format = web_sys::GpuTextureFormat::Rgba8unorm;'
module/min/minwebgpu/src/descriptor/texture.rs` → `1`.

## In Scope

- `module/min/minwebgpu/src/descriptor/texture.rs`'s `TextureDescriptor::new()` — the
  already-applied default-format fix (`Rgba8unormSrgb` → `Rgba8unorm`) and its
  `Fix(BUG-300)`/`Root cause`/`Pitfall` source comment (verify both are present; no further
  edit expected).
- `module/min/minwebgpu/tests/texture_descriptor_tests.rs`'s
  `default_format_supports_storage_binding_test` — the already-written
  `bug_reproducer(BUG-300)` reproducer and its 5-section doc comment (verify present; no
  further edit expected).
- Formal task registration and lifecycle walk (claim, verify, attempt `tsk .verify_pass`)
  for the already-complete fix.
- Linking `task/bug/verified/300_texture_descriptor_default_format_not_storage_capable.md`'s
  header back to this task via PROC12 Step 4 (performed as a follow-up edit once this file
  is filed).

## Out of Scope

- Any further code change to `descriptor/texture.rs` or `src/texture.rs` — the fix is
  complete; `texture::create()`'s synchronous-throw-only error path (the reason the pre-fix
  defect stayed silent) was confirmed, not modified, by BUG-300's own investigation.
- Re-running BUG-300's MRE or its own VERIFY Gate — already run and recorded in the bug
  file's History (2026-08-18, real headless-Firefox run, 8/8 PASS); not re-litigated by
  this task's own Readiness Verification Gate, which checks task-file quality, not the
  underlying fix.
- `module/min/minwebgpu/src/binding_type/storage_texture.rs` (BUG-275's own separate,
  already-completed fix for a sibling struct) and `module/min/minwebgpu/src/state/color_target.rs`
  (a different struct with its own independent `Rgba8unormSrgb` default, not named in
  BUG-300's Fix Location and not evaluated by it — confirmed a legitimate color-attachment
  default, not this task's concern).
- Introducing any new `.storage_binding()` call site anywhere in the workspace — zero exist
  today (confirmed both by BUG-300's own investigation and independently re-confirmed
  during this task's filing); adding one is unrelated future work.

## Requirements

-   All work must strictly adhere to all applicable rulebooks
    (discover via `kbase .rulebooks`)

## Delivery Requirements

-   All work must strictly adhere to all applicable rulebooks
    (discover via `kbase .rulebooks`)
-   Failing-first evidence already on record: BUG-300's MRE (revert-and-rerun of only the
    source fix) reproduced the pre-fix assertion failure (`left: Rgba8unormSrgb, right:
    Rgba8unormSrgb`) via a real headless-Firefox run (bug file MRE section, 2026-08-18) —
    this task does not re-derive that evidence
-   Fix already applied: `module/min/minwebgpu/src/descriptor/texture.rs:57` states
    `let format = web_sys::GpuTextureFormat::Rgba8unorm;`, with the 3-field
    `Fix(BUG-300)`/`Root cause`/`Pitfall` source comment immediately above `new()` —
    independently re-confirmed via direct file read during this task's filing (2026-08-18),
    not merely trusted from the bug file's own claim
-   Green state already confirmed: `default_format_supports_storage_binding_test` passes;
    full `minwebgpu` wasm32 suite (20 passed / 0 failed across 8 binaries) and clippy
    (wasm32 + native, `-D warnings`) clean per bug file History
-   No refactor needed — single default-value literal change, no structural churn
-   Fix documentation already complete at both levels: the reproducer test carries the
    5-section doc comment (Root Cause, Why Not Caught, Fix Applied, Prevention, Pitfall)
    and the bug carries the same in its own body — this task does not duplicate it, only
    cross-links via `closes: BUG-300`
-   `tests/readme.md`'s Responsibility Table already carries the `texture_descriptor_tests.rs`
    row citing BUG-300 (independently re-confirmed during this task's filing) — this task
    does not re-add it
-   Task state reaches 🎯 on this task file's own Readiness Verification Gate;
    `tsk .verify_pass`/`.acceptance_pass` then attempted per standard lifecycle (expected
    to hit this sandbox's known same-actor guard, per project convention — document rather
    than force/spoof if so)

## Test Matrix

| # | Input Scenario | Config Under Test | Expected Behavior |
|---|---------------|-------------------|-------------------|
| T01 | `grep -c 'let format = web_sys::GpuTextureFormat::Rgba8unorm;' module/min/minwebgpu/src/descriptor/texture.rs` | Fixed default-format assignment in `TextureDescriptor::new()` | `1` |
| T02 | `grep -c 'let format = web_sys::GpuTextureFormat::Rgba8unormSrgb' module/min/minwebgpu/src/descriptor/texture.rs` | Regression guard: the old sRGB default assignment must not have returned | `0` |
| T03 | `cargo check -p minwebgpu` (native) | `minwebgpu` crate compiles | 0 errors (confirmed live, 2026-08-18, 47s) |
| T04 | `cargo test -p minwebgpu --target wasm32-unknown-unknown --all-features --test texture_descriptor_tests` (already run, bug file History 2026-08-18) | Real headless-Firefox run via geckodriver | 1 passed / 0 failed |

## Acceptance Criteria

-   `module/min/minwebgpu/src/descriptor/texture.rs`'s `TextureDescriptor::new()` sets
    `format` to `web_sys::GpuTextureFormat::Rgba8unorm`, not `Rgba8unormSrgb`
-   That function's leading comment carries all 3 required fields: `Fix(BUG-300)`,
    `Root cause`, `Pitfall`
-   `default_format_supports_storage_binding_test` exists in
    `module/min/minwebgpu/tests/texture_descriptor_tests.rs`, marked
    `bug_reproducer(BUG-300)`, with a 5-section doc comment (Root Cause / Why Not Caught /
    Fix Applied / Prevention / Pitfall)
-   `task/bug/verified/300_texture_descriptor_default_format_not_storage_capable.md`'s
    header states `**Fix Task:**` pointing at this task, added by PROC12 Step 4 after filing
-   Every Test Matrix row passes

## Verification

**Execution:** The procedure for walking this section is defined in `§ Acceptance
Verification : Procedure - Execution`. The executor does NOT self-verify — an independent
verifier performs the walk after the task reaches 🔎 Accepting.

### Checklist

Desired answer for every question is YES.

**Fix correctness**
- [ ] C1 — Does `module/min/minwebgpu/src/descriptor/texture.rs`'s `TextureDescriptor::new()`
  set `format` to `web_sys::GpuTextureFormat::Rgba8unorm`?
- [ ] C2 — Does the same function's leading comment carry `Fix(BUG-300)`, `Root cause`, and
  `Pitfall` fields?
- [ ] C3 — Does `default_format_supports_storage_binding_test` exist in
  `tests/texture_descriptor_tests.rs`, marked `bug_reproducer(BUG-300)`, with all 5
  documentation sections present?
- [ ] C4 — Does `tests/readme.md`'s Responsibility Table carry a row for
  `texture_descriptor_tests.rs` citing BUG-300?

**Registration correctness**
- [ ] C5 — Does this task's `closes:` field name `BUG-300`?
- [ ] C6 — Does BUG-300's own header carry a `**Fix Task:**` line pointing back at this
  task's ID?

**Out of Scope confirmation**
- [ ] C7 — Is `module/min/minwebgpu/src/texture.rs` untouched by this task (`git diff --stat`
  empty for that path)?
- [ ] C8 — Is `module/min/minwebgpu/src/binding_type/storage_texture.rs` untouched by this
  task (`git diff --stat` empty for that path)?
- [ ] C9 — Is `module/min/minwebgpu/src/state/color_target.rs` untouched by this task
  (`git diff --stat` empty for that path)?
- [ ] C10 — Does a workspace-wide grep for `.storage_binding()` outside
  `tests/texture_descriptor_tests.rs` and `descriptor/texture.rs`'s own explanatory comment
  return empty (no new call site introduced)?

### Measurements

- [ ] M1 — `grep -c 'let format = web_sys::GpuTextureFormat::Rgba8unorm;' module/min/minwebgpu/src/descriptor/texture.rs` → 1 (was: 0, pre-fix)
- [ ] M2 — `grep -c 'let format = web_sys::GpuTextureFormat::Rgba8unormSrgb' module/min/minwebgpu/src/descriptor/texture.rs` → 0 (was: 1, pre-fix)

### Invariants

- [ ] I1 — `module/min/minwebgpu/src/texture.rs`, `src/binding_type/storage_texture.rs`,
  `src/state/color_target.rs` unaffected: `git diff --stat -- module/min/minwebgpu/src/texture.rs
  module/min/minwebgpu/src/binding_type/storage_texture.rs
  module/min/minwebgpu/src/state/color_target.rs` → empty
- [ ] I2 — crate still compiles natively: `cargo check -p minwebgpu` → 0 errors

### Anti-faking checks

- [ ] AF1 — the fix changes only the default format literal (`Rgba8unormSrgb` →
  `Rgba8unorm`) in `TextureDescriptor::new()`, not the field's type, the builder's other
  defaults, or any sibling struct's own default — checked by reading the literal current
  content at `new()` and confirming `state/color_target.rs`/`binding_type/storage_texture.rs`
  remain untouched, not just the absence of the old value in one file

## Verification Record

**Gate Round 1** (Tier 2 — Dual-Role Self-Check, one-shot, self-administered by
user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/)

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

## Journal

| Timestamp           | Actor                | Event | Note         |
|---------------------|----------------------|-------|--------------|
| 2026-08-18 16:10:27 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/ | CLAIM_VERIFY | verification claimed |
| 2026-08-18 16:10:46 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/ | VERIFY_PASS_ATTEMPTED | `tsk .verify_pass 359 "user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/"` → blocked: "self-verification forbidden (actor matches filed_by)" (exit 1) — same-actor guard, not a defect; state remains 🔬 Verifying |
| 2026-08-18 23:47:42 | system | TIMEOUT_2H | 2h exclusivity window expired |
| 2026-08-18 23:49:12 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/ | CLAIM_VERIFY | verification claimed |
| 2026-08-19 22:37:55 | system | TIMEOUT_2H | 2h exclusivity window expired |
| 2026-08-19 22:45:32 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/ | CLAIM_VERIFY | verification claimed |
| 2026-08-19 22:45:32 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/ | VERIFY_PASS_ATTEMPTED | `tsk .verify_pass 359` → exit 1, "self-verification forbidden (actor matches filed_by)" — same-actor sandbox guard; not forced/spoofed, left at 🔬 Verifying per standing project convention |

## History

*(append-only — newest entry last; never edit or remove past entries)*

- **[2026-08-18]** `FILED` — Task filed via `bug_promote` skill (PROC12) to formally
  register BUG-300's already-applied, already-verified fix
  (`module/min/minwebgpu/src/descriptor/texture.rs`'s `TextureDescriptor::new()` default
  `format` changed from `Rgba8unormSrgb` to `Rgba8unorm`) as a tracked task, closing the bug.
- **[2026-08-18]** `READINESS_GATE_PASS` — Tier 2 Dual-Role Self-Check, Round 1, 8/8 PASS; no
  findings on either pass. State transition to 🎯 is asserted only by `tsk .verify_pass`
  succeeding (see below), never by a direct hand-edit of this field.
- **[2026-08-18]** `CLAIM_VERIFY` — `tsk .claim_verify 359 "user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/"`
  succeeded (❓→🔬, moved to `verifying/`). No new code edit performed: the described fix
  (`module/min/minwebgpu/src/descriptor/texture.rs` default `format` change,
  `Fix(BUG-300)`/`Root cause`/`Pitfall` comment, `bug_reproducer(BUG-300)` test) already
  existed on disk prior to this task's filing, applied during BUG-300's own investigation
  (bug file History, 2026-08-18). This task's own contribution is the formal tracking
  registration and lifecycle walk, not the code change itself. `tsk .verify_pass 359` blocked
  by the same-actor guard (documented in `## Journal` above) — task left at 🔬 Verifying per
  this sandbox's standing, previously-documented limitation (same guard that blocked tasks
  254 and 358's own `.verify_pass`), not a quality defect in this task's own content.

## Related Documentation

- `task/bug/verified/300_texture_descriptor_default_format_not_storage_capable.md` — the
  source bug this task promotes; carries the full Root Cause/MRE/Prevention/History detail
  this task does not duplicate
- `module/min/minwebgpu/src/texture.rs` — `texture::create()`'s synchronous-throw-only error
  path (confirmed as the reason the pre-fix defect stayed silent, not modified by this task)
