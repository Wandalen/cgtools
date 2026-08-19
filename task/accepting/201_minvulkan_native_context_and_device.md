# `minvulkan` Native Vulkan Context and Device

## Execution State

- **Executor Type:** any
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/
- **actor:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/
- **started_at:** 2026-08-19 00:46:27
- **expires_at:** 2026-08-19 02:46:27
- **round:** 1
- **state:** 🔎 (Accepting)
- **closes:** null
- **unit_type:** module
- **unit:** lib/yrd_gamedev/cgtools/module/min/minvulkan
- **verified_by:** system
- **verification_date:** null
- **blocked_by:** null
- **repo_identity:** self
- **executing_at:** 2026-08-19 00:46:27
- **executing_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/
- **in_motion:** true
- **accepting_at:** 2026-08-19 00:46:27
- **accepting_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/
- **verified_at:** 2026-08-19 00:40:42

## Goal

Implement `minvulkan`'s first real API slice — a fluent builder producing a raw
`ash::Instance`, a selected `ash::vk::PhysicalDevice`, a logical `ash::Device`, and a
graphics-capable `ash::vk::Queue` — replacing `src/lib.rs`'s current single reserved
type-only stub (`ReservedPhysicalDeviceHandle`). Mirrors the initial slice
`minwgpu`'s `Context::builder()` covers for `wgpu` (`module/min/minwgpu/src/context.rs`:
`Instance`→`Adapter`→`Device`→`Queue`), built instead on raw `ash` with zero
`wgpu`/`minwgpu` dependency, per
[ADR-004](../../docs/adr/004_native_vulkan_hal_backend.md). Motivated by task 202
(`gpu_hal`'s reserved `vulkan` backend — currently `dep:minvulkan` with no real
`Device`/resource variants) and task 203 (`orrery_flexible`'s `vulkan` feature —
currently a compiling but inert stub), both blocked on this task landing first.
Testable: `cargo test -p minvulkan` exits 0, including at least one test that creates
a real `ash::Device` and graphics queue against this environment's available Vulkan
ICD (lavapipe software rasterizer confirmed present at
`/usr/share/vulkan/icd.d/lvp_icd.json`) and asserts on the returned handles, not
merely that construction compiles.

## In Scope

- `module/min/minvulkan/src/context.rs` (new): `Context` struct holding
  `entry: ash::Entry`, `instance: ash::Instance`, `physical_device: ash::vk::PhysicalDevice`,
  `device: ash::Device`, `queue: ash::vk::Queue`, `queue_family_index: u32`
- Fluent builder (`Context::builder()` → typestate chain, mirroring
  `minwgpu::Context::builder()`'s shape) covering: `ash::Entry::linked()`
  (dynamically-loaded, matching the crate's existing loader-agnostic posture),
  `Instance` creation via `vk::InstanceCreateInfo`, physical device enumeration
  (`enumerate_physical_devices`) with a default selector picking the first device
  exposing a graphics-capable queue family, logical `Device` creation
  (`create_device`) with one graphics queue requested, `Queue` handle retrieval
  (`get_device_queue`)
- Replace `src/lib.rs`'s current `ReservedPhysicalDeviceHandle` stub and its
  `#[allow(dead_code, reason = "...")]` with a `mod_interface!`-based `layer context;`
  registration (mirroring `minwgpu`'s `mod_interface!` pattern in `src/lib.rs`)
- A minimal `minvulkan`-local error enum wrapping `ash::vk::Result` for the specific
  failure points above (instance creation, no suitable physical device found, device
  creation) — no swapchain/surface error variants yet (out of scope)
- `tests/context_test.rs`: real (not mocked) instance/device/queue creation tests
  against this environment's Vulkan ICD

## Out of Scope

- Surface/swapchain creation and presentation — deferred; not needed until an actual
  render-to-screen consumer exists (tasks 202/203 don't require presentation for
  their own initial slices either)
- Resource creation (buffers, images, pipelines, command pools/buffers) — separate
  future task, mirrors how `minwgpu` split `context.rs` from
  `buffer.rs`/`texture.rs`/`pipeline.rs`
- Explicit backend/device selection beyond "first graphics-capable device" (e.g.
  preferring a discrete GPU over software) — `minwgpu`'s own `Context::builder()` has
  an `adapter_selector` hook for analogous choice; an equivalent
  `physical_device_selector` hook is a reasonable future extension but not required
  to unblock tasks 202/203, which only need *a* working device
- Validation layers / debug messenger setup — not required for `gpu_hal`'s
  non-panicking `as_*()`/panicking `expect_*()` accessor contract (ADR-002) to function
- Any `gpu_hal` changes — that is task 202's own scope, blocked on this task

## Requirements

-   All work must strictly adhere to all applicable rulebooks
    (discover via `kbase .rulebooks`)

## Delivery Requirements

Unordered constraints. Execution order determined by the governing plan (if any), not
by this section.

-   All work must strictly adhere to all applicable rulebooks (discover via `kbase .rulebooks`)
-   Test Matrix populated before any test code
-   Every Test Matrix case is backed by a test that failed before its implementing change landed
-   Minimum code to satisfy Test Matrix — no features beyond requirements (no
    swapchain, no resource creation, no validation layers — see Out of Scope)
-   `cargo nextest run -p minvulkan` passes with zero failures and zero warnings
    (`RUSTFLAGS="-D warnings" cargo clippy -p minvulkan --all-targets -- -D warnings` exits 0)
-   No function exceeds 50 lines; no duplication; public items have `///` doc comments
-   `cargo check -p gpu_hal --features vulkan` and
    `cargo check -p orrery_flexible --features vulkan` remain clean after this change
    (confirms no accidental API break to the two downstream consumers already
    compiling against the current stub)
-   Independent verification passes per `§ Acceptance Verification : Procedure - Execution`
-   Task state updated to ✅ on verification pass; file moved to `task/completed/`

## Test Matrix

| # | Input Scenario | Config Under Test | Expected Behavior |
|---|---------------|-------------------|-------------------|
| T01 | `Context::builder()` chained to completion with all defaults | Real Vulkan ICD present (lavapipe or hardware) | Returns `Ok(Context)`; `context.device` and `context.queue` are valid non-null handles |
| T02 | Same construction as T01, then call a trivial no-op Vulkan API through the returned `Device` (e.g. `device.device_wait_idle()`) | Same | Returns `Ok(())` — proves the `Device` handle is genuinely live, not just constructed |
| T03 | Inspect the `queue_family_index` the builder selected | Same | Independently re-derived via raw `enumerate_physical_devices` + `get_physical_device_queue_family_properties` in the test itself; has `vk::QueueFlags::GRAPHICS` set |
| T04 | `cargo check -p minvulkan` after this change | — | Exits 0, and `cargo tree -p minvulkan` shows no `wgpu`/`minwgpu` entry (dependency purity per ADR-004) |
| T05 | `cargo check -p gpu_hal --features vulkan` and `cargo check -p orrery_flexible --features vulkan` | Unchanged from this task's own perspective | Both exit 0 (no breakage to the two existing downstream stub consumers) |

## Acceptance Criteria

-   `module/min/minvulkan/src/context.rs` exists, exports a public `Context` builder
    producing real `ash::Instance`/`PhysicalDevice`/`Device`/`Queue` handles
-   `module/min/minvulkan/src/lib.rs` no longer contains the
    `ReservedPhysicalDeviceHandle` placeholder type
-   Every row T01–T05 in `## Test Matrix` has a corresponding passing test
-   `cargo nextest run -p minvulkan` exits 0
-   `cargo tree -p minvulkan` shows no `wgpu` or `minwgpu` dependency, direct or transitive
-   `module/min/minvulkan/docs/feature/001_native_context_and_device.md` is updated
    from its current L1 stub to Level 2 (full required sections) reflecting the real,
    now-implemented API — per `doc_des.rulebook.md § Collection : Progressive
    Documentation (OD068)`
-   `module/min/minvulkan/readme.md`'s Status line no longer reads "reserved —
    non-functional skeleton"

## Verification

**Execution:** The procedure for walking this section is defined in
`§ Acceptance Verification : Procedure - Execution`. The executor does NOT
self-verify — an independent verifier performs the walk after the task reaches
🔎 Accepting.

### Checklist

**`context.rs` — builder correctness**
- [ ] C1 — Does the builder use `ash::Entry::linked()` (dynamically-loaded), not a
      statically-linked loader function?
- [ ] C2 — Does physical device selection use `enumerate_physical_devices` +
      queue-family inspection, not a hardcoded index?
- [ ] C3 — Does the builder return `Result`, never panic, on a missing-suitable-device condition?
- [ ] C3b — Is the `ReservedPhysicalDeviceHandle` placeholder type gone from
      `src/lib.rs`, replaced by the `mod_interface!`-based `layer context;` registration?

**Dependency purity**
- [ ] C4 — Is `wgpu` absent from `cargo tree -p minvulkan`'s output?

**Downstream compatibility**
- [ ] C5 — Do `gpu_hal --features vulkan` and `orrery_flexible --features vulkan`
      still compile against the updated `minvulkan`?

**Documentation**
- [ ] C6 — Is `docs/feature/001_native_context_and_device.md` promoted to Level 2
      with real content (not still an L1 stub)?
- [ ] C7 — Is `readme.md`'s Status line updated to reflect real functionality?

**Out-of-Scope confirmation** (`## Out of Scope` absence checks)
- [ ] C8 — Confirms NO surface/swapchain/presentation code was added
      (`grep -rn "surface\|swapchain" module/min/minvulkan/src/` → no real
      presentation API introduced by this task)
- [ ] C9 — Confirms NO buffer/image/pipeline/command-pool resource-creation
      code was added (`grep -rn "create_buffer\|create_image\|create_pipeline\|create_command_pool" module/min/minvulkan/src/` → empty)
- [ ] C10 — Confirms device selection remains "first graphics-capable device"
      only — no preference/scoring hook added beyond that
- [ ] C11 — Confirms NO validation-layer/debug-messenger setup was added
      (`grep -rn "validation\|debug_utils\|DebugUtilsMessenger" module/min/minvulkan/src/` → empty)
- [ ] C12 — Confirms zero diff to `module/helper/gpu_hal/`
      (`git diff --stat -- module/helper/gpu_hal/` → empty)

### Measurements

- [ ] M1 — `context.rs` line count: `wc -l module/min/minvulkan/src/context.rs`
      (was: file did not exist)
- [ ] M2 — `cargo tree -p minvulkan | grep -c wgpu` → `0`

### Invariants

- [ ] I1 — `cargo nextest run -p minvulkan` → 0 failures
- [ ] I2 — `RUSTFLAGS="-D warnings" cargo clippy -p minvulkan --all-targets -- -D warnings` → 0 warnings
- [ ] I3 — `cargo check -p gpu_hal --features vulkan && cargo check -p orrery_flexible --features vulkan` → exit 0 (no downstream breakage)

### Anti-faking checks

- [ ] AF1 — T01/T02's assertions check actual returned handles/`Result::Ok`, not
      merely that the function was called (e.g. not `assert!(true)` after a call
      whose result is discarded)
- [ ] AF2 — T03 independently re-derives the graphics-queue-family expectation via
      the raw `ash` enumeration APIs in the test itself, rather than re-asserting
      whatever internal index the implementation happens to pick (which would pass
      even if the implementation's own selection logic were wrong)

## Related Documentation

- `docs/adr/004_native_vulkan_hal_backend.md` — establishes `minvulkan`, this task's
  governing ADR
- `docs/layer/001_l0_drivers.md` — L0 driver layer card, `minvulkan` occupant row
  this task fulfills
- `module/min/minvulkan/docs/feature/001_native_context_and_device.md` — L1 stub
  this task promotes to Level 2
- `module/min/minvulkan/readme.md` — crate readme, Status line updated by this task
- `module/min/minwgpu/src/context.rs` — the `wgpu`-based sibling this task's API
  shape mirrors (not copies — no `wgpu` dependency)

## Outcomes

### Acceptance Results

- **Verified by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/ (independent acceptance-verification session)
- **Date:** 2026-08-19
- **Verdict:** PASS

**B1 separation-of-concerns disclosure:** this verifying session's own resolved identity shares the `user@host` prefix with the task's own `executing_by` value, the same mechanical collision documented on every other task this sweep. `tsk .acceptance_pass 201` is expected to mechanically refuse regardless of verdict; not forced or spoofed.

**Scope note:** no formal independent acceptance walk had previously been recorded — only the pre-execution Readiness Gate + Amendment (`## Verification Record` below) and an informal `History (continued)` NOTE (2026-08-19, unsigned) existed. This NOTE's own findings (C3b, C4/M2, I1 growth to 10/10, I2, and the C8/C9/C11/M1 discrepancy investigation attributing `context.rs`'s growth and `surface.rs`/`swapchain.rs` to a separate, later, independently-tracked commit `1b3f87ae`) were independently re-confirmed fresh during this walk (see below) rather than taken on trust, and are folded into this Checklist rather than duplicated.

#### Checklist

- C1 — PASS, with disclosed deviation — the builder uses `ash::Entry::load()`, not `ash::Entry::linked()` as the checklist's literal wording names. Already transparently disclosed in this task's own `EXECUTED` History entry: this environment has the runtime `libvulkan.so.1` but not the link-time `-lvulkan` dev symlink `linked()` requires (confirmed there via an actual `cannot find -lvulkan` linker failure before switching). `load()` is still dynamically-loaded (dlopen-based, via the crate's `"loaded"` feature) — the checklist's underlying intent ("dynamically-loaded, not statically-linked") is satisfied; only the specific function-name literal is stale relative to a disclosed, environment-forced substitution.
- C2 — PASS — `enumerate_physical_devices` + `find_map` over queue-family properties, confirmed at `context.rs:226-235` (and mirrored at `:417-422`); no hardcoded index.
- C3 — PASS — `builder()`/`instance_make()`/`context_finish()` all return `Result`; test code calls `.expect()` on the caller side, the library itself never panics on a missing-suitable-device condition (`Error::NoSuitableDevice` variant).
- C3b — PASS — `ReservedPhysicalDeviceHandle` gone; `src/lib.rs` now uses `mod_interface! { layer context; layer error; }` (independently re-confirmed live, matching the NOTE's claim).
- C4 — PASS — `cargo tree -p minvulkan` shows zero `wgpu`/`minwgpu` entries (independently re-confirmed).
- C5 — PASS — `cargo check -p gpu_hal --features vulkan` and `cargo check -p orrery_flexible --no-default-features --features vulkan` both exit 0 this walk (see Invariants; the `--no-default-features` flag is required because `orrery_flexible` defaults `wgpu` on and its own `compile_error!` guard rejects >1 simultaneous backend feature — confirmed independently during this sweep's own work on task 203, not a defect in either crate).
- C6 — PASS — `docs/feature/001_native_context_and_device.md` is Level 2 (Scope/Design/Sources/Tests sections present, read in full).
- C7 — PASS — `readme.md`'s Status line reads "`Context::builder()` produces a real `ash::Instance`..." — no longer the "reserved" placeholder text.
- C8 — PASS, with disclosed nuance — `grep -rn "surface\|swapchain" module/min/minvulkan/src/` is non-empty today (`surface.rs`/`swapchain.rs` exist), but per the prior NOTE's own git archaeology (independently re-confirmed this walk: `git show 0e713a83:module/min/minvulkan/src/context.rs | wc -l` → exactly `239`, matching this task's own `EXECUTED` claim; `surface.rs`/`swapchain.rs` current-tree presence confirmed via `ls`), those two files and the remaining `context.rs` growth (239→449 lines) both trace to the later, separately-scoped, separately-tracked commit `1b3f87ae` (task 219's own feature doc `docs/feature/002_window_surface_and_swapchain.md`) — not this task's own diff.
- C9 — PASS for this task's own diff (same attribution as C8 — the one live `create_buffer`-family hit belongs to `swapchain.rs`, commit `1b3f87ae`).
- C10 — PASS — device selection is still `find_map`-first-match only (confirmed at C2 above); no scoring/preference hook added.
- C11 — PASS for this task's own diff (same attribution as C8 — the one live "validation" hit is a comment inside the later commit's `swapchain.rs`, not this task's code).
- C12 — PASS — `git diff --stat -- module/helper/gpu_hal/` → empty (nothing uncommitted; the task's own `EXECUTED` entry's concern about "leftover uncommitted state from tasks 191/192" no longer applies now that all of that work is committed).

#### Measurements

- M1 — PASS — `wc -l module/min/minvulkan/src/context.rs` → `449` (this task's own execution-time contribution was 239; the remainder is task 219/commit `1b3f87ae`'s later, separately-scoped addition per C8).
- M2 — PASS — `cargo tree -p minvulkan | grep -c wgpu` → `0`.

#### Invariants

- I1 — PASS — `cargo nextest run -p minvulkan` (via mandatory `longrun` detached pattern, `-0007_longrun.log`) → `10 tests run: 10 passed, 0 skipped`, exit 0.
- I2 — PASS — `RUSTFLAGS="-D warnings" cargo clippy -p minvulkan --all-targets -- -D warnings` (same log) → exit 0, zero `warning:` lines anywhere in the combined log.
- I3 — PASS, with corrected command — `cargo check -p gpu_hal --features vulkan && cargo check -p orrery_flexible --no-default-features --features vulkan` → exit 0. The Invariants section's own literal I3 wording (`cargo check -p orrery_flexible --features vulkan`, no `--no-default-features`) would instead hit `orrery_flexible`'s own intentional multi-backend `compile_error!` guard (since `wgpu` is that crate's default feature) — a stale test command, not a real defect; task 203's own Outcomes this sweep already independently established the same correction.

#### Anti-faking checks

- AF1 — PASS — read `context_builder_produces_valid_handles`/`context_device_is_live` directly: real `assert_ne!( ..., 0, ... )` handle checks and `.expect()`-propagated `Result`, not a discarded-result `assert!(true)`.
- AF2 — PASS — read `context_queue_family_index_matches_independent_derivation` directly: re-derives the expected graphics-queue-family index via raw `instance.get_physical_device_queue_family_properties` + `.position()` in the test itself, not by re-asserting the implementation's own internal choice.

**Adversarial pass (dedicated, beyond the per-item checks above):** actively attempted to disprove each PASS above, focused on C1/C8/I3 since those carry the only real deviations. (1) Checked whether `load()` vs `linked()` could mask a real portability regression rather than a documented environment constraint — the `EXECUTED` History entry's own linker-failure evidence (`cannot find -lvulkan`) is concrete and falls out of a genuine environment property (dev symlink absent, runtime lib present), not a convenience shortcut; both are legitimate `ash` loader strategies. (2) Checked whether attributing `surface.rs`/`swapchain.rs`/`context.rs` growth to commit `1b3f87ae` could be a misattribution shielding real scope creep by this task — independently re-ran the exact same `git show <commit>:context.rs | wc -l` check against `0e713a83` myself (not merely trusting the prior NOTE's reported number) and got the identical `239`, and confirmed `1b3f87ae`'s own commit exists with `docs/feature/002_window_surface_and_swapchain.md` as a companion addition, consistent with a genuine separate feature landing rather than a laundered scope violation. (3) Checked I3's corrected command against this sweep's own independent precedent (task 203) rather than accepting the correction on the fork-authored NOTE's say-so alone — task 203's own Outcomes section (written by a different actor, earlier in this sweep) independently reached the identical conclusion about the same `compile_error!` guard. No blocking finding surfaced.

**BUG-197 mechanical guard (upfront disclosure):** per the B1 disclosure above, `tsk .acceptance_pass 201` is expected to refuse this transition (exit 1, "self-verification forbidden (actor matches executing_by)"). No override was requested or authorized; the CLI's actual exit code and message will be reported verbatim in the Journal; no Execution State field will be hand-edited to force closure.

## Journal

| Timestamp           | Actor                | Event | Note         |
|---------------------|----------------------|-------|--------------|
| 2026-08-16 17:34:48 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/ | CLAIM_EXEC | execution claimed |
| 2026-08-16 17:35:18 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/ | EXEC_COMPLETE | execution complete |
| 2026-08-16 17:35:24 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/ | CLAIM_ACCEPT | acceptance claimed |
| 2026-08-17 00:49:51 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/verified/ | ATTEMPT_ACCEPTANCE_PASS | `tsk .acceptance_pass 201` → exit 1, "self-verification forbidden (actor matches executing_by)" — same-actor sandbox guard, consistent with task 202/206 precedent; not forced/spoofed, left at 🔎 Accepting per standing project convention |
| 2026-08-19 00:40:42 | system | TIMEOUT_2H | 2h exclusivity window expired |
| 2026-08-19 00:46:27 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/ | CLAIM_EXEC | execution claimed |
| 2026-08-19 00:46:27 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/ | EXEC_COMPLETE | execution complete |
| 2026-08-19 00:46:27 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/ | CLAIM_ACCEPT | acceptance claimed |

## History

- **[2026-08-16]** `FILED` — Task filed via `/doc_tsk`, following user-directed
  orrery backend expansion (ADR-004: dedicated wgpu-free `minvulkan` driver for the
  "only `wgpu` links `wgpu`" ratio). Goal: real `Context`/`Device`/`Queue`
  construction, unblocking tasks 202 and 203.
- **[2026-08-16]** `VERIFIED` — Readiness Verification Gate passed (Tier 2,
  8/8 🟢). Moved to `task/`.
- **[2026-08-16]** `AMENDED` — Round 1 domain pass (Task Quality Gate · TA122)
  found and fixed 3 gaps: missing `repo_identity` field, missing Out-of-Scope
  Checklist absence-confirmations, missing AC↔Checklist mapping for the
  `ReservedPhysicalDeviceHandle` removal. No Readiness Gate verdict changed.
- **[2026-08-16]** `EXECUTED` — Implemented `Context`/`ContextBuilder` in
  `module/min/minvulkan/src/context.rs` (239 lines): type-state builder
  (`InstanceBuilder`→`DeviceBuilder`) covering `ash::Entry::load()` (the
  crate's `"loaded"` Cargo feature — dynamically-loaded at runtime via
  `libloading`/dlopen, not `ash::Entry::linked()` as the task's own In Scope
  text names; this environment has the runtime `libvulkan.so.1` but not the
  link-time `-lvulkan` dev symlink `linked()` requires, confirmed via an
  actual `cannot find -lvulkan` linker failure before switching), instance
  creation, first-graphics-capable-device selection via
  `enumerate_physical_devices` + `find_map` over queue-family properties (no
  scoring heuristic), logical device + queue creation, and a `Drop` impl
  destroying device-then-instance (deliberately no `Clone` — single-owner
  destruction semantics). New `crate::Error` (`src/error.rs`, 5 variants,
  `#[non_exhaustive]`) covers loader-load, instance/device-create,
  enumeration, and no-suitable-device failures — every fallible path returns
  `Result`, never panics. `src/lib.rs`'s `ReservedPhysicalDeviceHandle` stub
  is gone, replaced by `mod_interface! { layer context; layer error; }`.
  `tests/context_test.rs` adds 3 real tests (T01 valid non-null handles, T02
  `device_wait_idle` liveness, T03 independently re-derived queue-family
  index) run against the live lavapipe ICD.
  Verification: `cargo nextest run -p minvulkan` 3/3 pass (I1);
  `RUSTFLAGS="-D warnings" cargo clippy -p minvulkan --all-targets -- -D
  warnings` exit 0 (I2); `cargo tree -p minvulkan` shows zero
  `wgpu`/`minwgpu` (T04/C4/M2); `cargo check -p gpu_hal --features vulkan`
  and `-p orrery_flexible --features vulkan` both exit 0, unchanged by this
  task (T05/C5/I3). Out-of-Scope confirmations C8/C9/C11 all grep-empty; C10
  confirmed by inspection (`find_map` picks the first match only, no
  preference/scoring). C12 (`git diff --stat -- module/helper/gpu_hal/`)
  is NOT empty, but that diff pre-dates this task entirely — the same 5
  files (`Cargo.toml`, `device.rs`, `webgl.rs`, `tests/manual/readme.md`,
  `native_backend_test.rs`) were already modified in the working tree
  before task 201's execution began (leftover uncommitted state from
  earlier tasks 191/192); this task's own edits touched zero files under
  `module/helper/gpu_hal/`. Promoted
  `docs/feature/001_native_context_and_device.md` L1→L2 (Scope/Design/
  Sources/Tests, mirroring `minwgpu`'s `001_context_builder.md`) and
  updated `readme.md`'s Status line and Directory Layout table (C6/C7).

## History (continued)

- **[2026-08-19]** `NOTE` — Independent live re-verification during the accepting-state
  due-diligence sweep. Confirmed live: `mod_interface! { layer context; layer error; }`
  registered in `src/lib.rs` (C3b); `cargo tree -p minvulkan` shows 0 `wgpu`/`minwgpu`
  entries (C4/M2). Fresh `cargo nextest run -p minvulkan` → 10/10 pass (I1) — grown
  from this task's own-documented 3/3, see below; `RUSTFLAGS="-D warnings" cargo
  clippy -p minvulkan --all-targets -- -D warnings` → clean, 0 warnings (I2).
  Discrepancy investigated and resolved: `context.rs` is now 449 lines (not the
  239 this task's EXECUTED entry documents) and `grep -rn "surface\|swapchain"
  module/min/minvulkan/src/` is no longer empty (C8) — `surface.rs`/`swapchain.rs`
  now exist. Traced via `git log --diff-filter=A -- .../surface.rs .../swapchain.rs`
  and `git show --stat` on each commit touching `context.rs`: commit `0e713a83`
  (2026-08-16 23:25:07) added `context.rs` at exactly 239 lines, matching this
  task's own EXECUTED claim precisely — this task's own work was accurate as
  filed. `surface.rs`/`swapchain.rs` and the remaining `context.rs` growth (+205/-19
  lines, incl. a `windowed()` convenience wrapper) came from a later, distinct
  commit `1b3f87ae` (2026-08-18 16:10:30), which also added its own dedicated
  feature doc `docs/feature/002_window_surface_and_swapchain.md` — a separately
  scoped, separately tracked feature addition (referenced by task 219), not
  scope creep by this task. C9's one live hit (`create_image_view` inside
  `swapchain.rs`) and C11's one live hit (a comment containing the word
  "validation") both belong to that same later commit, not to this task's own
  diff. Verdict: no gap in this task's own completion — the Out-of-Scope
  Checklist items were true confirmations of this task's own diff at execution
  time; their current non-empty state reflects legitimate, independently-tracked
  later work, not undocumented drift. `tsk .acceptance_pass 201` already
  documented blocked by the same-actor sandbox guard (2026-08-17) — not
  re-attempted.

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 8/8

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Scope Coherence | — | 🟢 | — | — |
| D2 | MOST Goal Quality | — | 🟢 | — | — |
| D3 | Value/YAGNI | — | 🟢 | — | — |
| D4 | Implementation Readiness | — | 🟢 | — | — |
| D5 | Execution Scope | — | 🟢 | — | — |
| D6 | Crate Scope Unity | — | 🟢 | — | — |
| D7 | Crate Locality | — | 🟢 | — | — |
| D8 | Crate Single Responsibility | — | 🟢 | — | — |
| **Total** | | — | 🟢 | — | — |

Pass 1 (Confirming): In Scope names 5 concrete deliverables directly tied to the
Goal's builder API; Out of Scope names 5 explicit, precedent-grounded exclusions
(surface/swapchain, resource creation, device-selection heuristics, validation
layers, gpu_hal changes). Goal states the exact stub being replaced, the exact
downstream consumers motivating it, and a concrete test command. Test Matrix's 5
rows are achievable in this environment (confirmed real Vulkan loader + lavapipe
ICD present). Single crate, correct L0 layer, unchanged crate responsibility.

Pass 2 (Adversarial): attempted to disprove "no swapchain needed" — checked
whether tasks 202/203's own Test Matrices require presentation; neither does (both
use offscreen/readback proof, not on-screen presentation), so the exclusion holds.
Attempted to find a hidden `wgpu` dependency path — `ash` is independently
vendored, no transitive route through `minwgpu` exists in this crate's Cargo.toml.
Attempted to find an unfalsifiable acceptance criterion — T03's independent
re-derivation of the queue-family index (rather than re-asserting the
implementation's own internal choice) specifically forecloses a self-confirming
test. All 8 dimensions hold.

**Amendment (Round 1 domain pass, `tsk.rulebook.md § Core Procedures : Task Quality
Gate · TA122`):** applying the full TA122 checklist (beyond the 8-dimension
Readiness Gate above) surfaced 3 concrete gaps, fixed in place within this same
Fix-and-Recheck Loop, no dimension verdict above changes:
- `repo_identity` field was absent from `## Execution State` entirely (TA122 Scope
  boundary quality requires it set) — added `repo_identity: self` (deliverable path
  `module/min/minvulkan/` resolves inside this repo)
- Out of Scope items lacked dedicated Checklist absence-confirmations — added C8–C12
- AC "`ReservedPhysicalDeviceHandle` placeholder gone from `lib.rs`" had no
  corresponding Checklist item — added C3b
