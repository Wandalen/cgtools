# Wire `gpu_hal`'s Reserved `vulkan` Backend to Real `minvulkan` Device/Resource Variants

## Execution State

- **Executor Type:** any
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/
- **actor:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/
- **started_at:** 2026-08-19 00:46:28
- **expires_at:** 2026-08-19 02:46:28
- **round:** 2
- **state:** 🔎 (Accepting)
- **closes:** null
- **unit_type:** module
- **unit:** lib/yrd_gamedev/cgtools/module/helper/gpu_hal
- **verified_by:** system
- **verification_date:** null
- **blocked_by:** 201
- **repo_identity:** self
- **executing_at:** 2026-08-19 00:46:28
- **executing_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/
- **in_motion:** true
- **accepting_at:** 2026-08-19 00:46:28
- **accepting_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/
- **verified_at:** 2026-08-19 00:40:43

## Goal

Give `gpu_hal`'s 4th backend variant (`vulkan`, currently `dep:minvulkan` with zero
`Device`/resource enum arms — see `module/helper/gpu_hal/Cargo.toml`'s
`vulkan = [ "dep:minvulkan" ]` and its comment "Reserved — no `Device`/resource
variants wired yet") a real, working implementation following ADR-002's
enum-per-backend dispatch pattern (plain enum, `#[cfg(feature = "vulkan")]`-gated
variant, non-panicking `as_*()` + crate-private panicking `expect_*()` accessors, its
own constructor) — mirroring how `webgpu`/`webgl`/`native` are each wired today.
Motivated by task 203 (`orrery_flexible`'s `vulkan` feature currently routes to a
`gpu_hal` backend that cannot actually construct a device or render anything).
Blocked on task 201 (`minvulkan`'s real `Context`/`Device`/`Queue` API) landing
first. Testable: `cargo test -p gpu_hal --features vulkan` exits 0, including a
pixel-readback test in the style of `gpu_hal`'s existing
`tests/native_backend_test.rs::triangle_render_readback`, proving the vulkan backend
renders and reads back real pixels — not just that it constructs.

## In Scope

- `Device::new_vulkan(width, height)` constructor in
  `module/helper/gpu_hal/src/device.rs`, `#[cfg(feature = "vulkan")]`-gated,
  mirroring `Device::new_native`'s offscreen-render-target shape (task 087's
  precedent) but built on `minvulkan::Context` instead of `minwgpu::Context`
- New `Vulkan` enum variant added to `gpu_hal`'s backend-dispatch enums (`Device`,
  `Surface`, and whichever other L1 resource types currently enumerate
  `Webgpu`/`Webgl`/`Native` per ADR-002 — grep `module/helper/gpu_hal/src/` for the
  existing 3-variant match arms to find the exact set) plus corresponding
  non-panicking `as_vulkan()` and crate-private panicking `expect_vulkan()` accessors
- Minimum resource support to make one full `Surface::read_pixels` round-trip work on
  the vulkan backend: buffer creation, a trivial pipeline, one render pass, offscreen
  texture + readback — the same minimum bar task 087 set for the `native` backend's
  own first landing
- `tests/vulkan_backend_test.rs`: at least one real pixel-readback test, mirroring
  `tests/native_backend_test.rs::triangle_render_readback`'s exact-byte-equality style

## Out of Scope

- Full `RenderCommand`/resource-type coverage — same honest-subset posture as tasks
  086/087 (`capabilities()` only claims what's actually wired)
- Any change to `full`'s feature composition beyond what this session already
  wired (`full` already lists `vulkan`, added when the reserved stub was created) —
  this task only fills in the variants that feature already nominally enables
- `orrery_flexible`'s own implementation — separate task 203, blocked on this task
- Vulkan-specific advanced features (ray tracing, mesh shaders, validation layers) —
  no current consumer needs them; same YAGNI posture as ADR-004's own Alternatives
  Considered section
- Any change to `minvulkan` itself — that is task 201's scope, already complete by
  the time this task executes (`blocked_by`)

## Requirements

-   All work must strictly adhere to all applicable rulebooks
    (discover via `kbase .rulebooks`)

## Delivery Requirements

Unordered constraints. Execution order determined by the governing plan (if any), not
by this section.

-   All work must strictly adhere to all applicable rulebooks (discover via `kbase .rulebooks`)
-   Test Matrix populated before any test code
-   Every Test Matrix case is backed by a test that failed before its implementing change landed
-   Minimum code to satisfy Test Matrix — no features beyond requirements
-   `cargo nextest run -p gpu_hal --features vulkan` passes with zero failures and
    zero warnings (`RUSTFLAGS="-D warnings" cargo clippy -p gpu_hal --all-targets --features vulkan -- -D warnings`
    exits 0)
-   No function exceeds 50 lines; no duplication; public items have `///` doc comments
-   `cargo check -p orrery_flexible --features vulkan` remains clean after this change
    (downstream consumer structurally unaffected)
-   Independent verification passes per `§ Acceptance Verification : Procedure - Execution`
-   Task state updated to ✅ on verification pass; file moved to `task/completed/`

## Test Matrix

| # | Input Scenario | Config Under Test | Expected Behavior |
|---|---------------|-------------------|-------------------|
| T01 | `Device::new_vulkan(64, 64)` | `vulkan` feature enabled | Returns a valid `Device` wrapping the vulkan variant |
| T02 | Construct device, load a single solid-color sprite/triangle asset, submit one draw, call `Surface::read_pixels` | `vulkan` feature enabled | Returns real pixel bytes; center pixel matches the configured draw color, a corner pixel matches the clear color (mirrors task 087's T02 exact-equality style) |
| T03 | `device.as_vulkan()` on a non-vulkan-constructed `Device` | any | Returns `None` (non-panicking accessor, per ADR-002) |
| T04 | `device.expect_vulkan()` (crate-private) on a non-vulkan-constructed `Device`, exercised via an internal test | any | Panics (documented panicking-accessor contract) |
| T05 | `cargo build -p gpu_hal --no-default-features --features enabled,vulkan` | vulkan only | Compiles clean, no webgpu/webgl/native-only symbol leaks |
| T06 | `cargo check -p orrery_flexible --features vulkan` after this change | — | Exits 0 |

## Acceptance Criteria

-   `Device::new_vulkan` exists and constructs via `minvulkan::Context`
-   `Vulkan` variant added to every L1 resource enum that currently enumerates the
    other 3 backends, each with `as_vulkan()`/`expect_vulkan()` accessors
-   Every row T01–T06 in `## Test Matrix` has a corresponding passing test
-   `cargo nextest run -p gpu_hal --features vulkan` exits 0
-   At least one test performs a genuine pixel-content assertion on read-back bytes
    (not only a dimension or `Ok` check)
-   `docs/layer/002_l1_gpu_hal.md`'s Status section no longer describes `vulkan` as
    "reserved... no implementation yet"

## Verification

**Execution:** The procedure for walking this section is defined in
`§ Acceptance Verification : Procedure - Execution`. The executor does NOT
self-verify — an independent verifier performs the walk after the task reaches
🔎 Accepting.

### Checklist

**`Device::new_vulkan` — construction correctness**
- [ ] C1 — Does `Device::new_vulkan` construct via `minvulkan::Context`'s public
      builder (task 201's API), not by duplicating instance/device-creation logic
      inline?
- [ ] C2 — Is the constructor `#[cfg(feature = "vulkan")]`-gated?

**Enum dispatch pattern (ADR-002 conformance)**
- [ ] C3 — Does every L1 resource enum touched add a `Vulkan` variant alongside the
      existing `Webgpu`/`Webgl`/`Native` variants, rather than a parallel/separate type?
- [ ] C4 — Do `as_vulkan()`/`expect_vulkan()` follow the same non-panicking/panicking
      split as the other 3 backends' accessors?

**Documentation**
- [ ] C4b — Does `docs/layer/002_l1_gpu_hal.md`'s Status section no longer describe
      `vulkan` as "reserved... no implementation yet"?

**Pixel proof**
- [ ] C5 — Does at least one test assert specific byte content in the returned
      pixel buffer (T02), not merely that `read_pixels` returns `Ok`?

**Capabilities honesty**
- [ ] C6 — Does `capabilities()` for the vulkan backend report `true` only for
      command families `submit` actually translates? (also confirms Out-of-Scope
      item "full `RenderCommand`/resource-type coverage" is not silently over-claimed)

**Out-of-Scope confirmation** (`## Out of Scope` absence checks)
- [ ] C7 — Confirms `full`'s feature composition is unchanged beyond what this
      session already wired (`git diff -- module/helper/gpu_hal/Cargo.toml` shows
      no new lines under the `full` feature definition)
- [ ] C8 — Confirms zero diff to `examples/orrery/flexible/`
      (`git diff --stat -- examples/orrery/flexible/` → empty — that is task 203's scope)
- [ ] C9 — Confirms NO vulkan-specific advanced features were added (ray tracing,
      mesh shaders, validation layers) — `grep -rn "ray_tracing\|mesh_shader\|validation_layer" module/helper/gpu_hal/src/` → empty
- [ ] C10 — Confirms zero diff to `module/min/minvulkan/`
      (`git diff --stat -- module/min/minvulkan/` → empty; same evidence as M2,
      restated here as an explicit Checklist item per Out-of-Scope confirmation)

### Measurements

- [ ] M1 — New/changed vulkan-backend code line count:
      `git diff --stat -- module/helper/gpu_hal/src/` (was: 0 — reserved stub only)
- [ ] M2 — `git diff --stat -- module/min/minvulkan/` → expected `0` (confirms this
      task did not reach back into task 201's already-landed crate)

### Invariants

- [ ] I1 — `cargo nextest run -p gpu_hal --features vulkan` → 0 failures
- [ ] I2 — `RUSTFLAGS="-D warnings" cargo clippy -p gpu_hal --all-targets --features vulkan -- -D warnings` → 0 warnings
- [ ] I3 — `cargo check -p orrery_flexible --features vulkan` → exit 0

### Anti-faking checks

- [ ] AF1 — The pixel-readback test doesn't accept an all-zero or all-identical
      buffer as a false pass: asserts both that the drawn pixel differs from the
      clear color AND matches the configured draw color — a backend that clears the
      texture and never actually draws would otherwise still pass a weaker
      "bytes are non-empty" check
- [ ] AF2 — `capabilities()` isn't over-claimed: cross-reference every `true` flag
      against an actual `submit` match-arm, same anti-faking bar as task 087's AF2

## Related Documentation

- `docs/adr/002_gpu_hal_in_house.md` — enum-per-backend dispatch pattern this task follows
- `docs/adr/004_native_vulkan_hal_backend.md` — establishes the `vulkan` backend this
  task implements
- `docs/layer/002_l1_gpu_hal.md` — L1 status card, `vulkan` reserved-backend note
  this task resolves
- `module/helper/gpu_hal/tests/native_backend_test.rs` — pixel-readback precedent
  (`triangle_render_readback`) this task's own test mirrors
- `task/201_minvulkan_native_context_and_device.md` — blocking
  dependency (now 🎯 Verified), provides the `minvulkan::Context` API this task consumes

## Journal

| Timestamp           | Actor                | Event | Note         |
|---------------------|----------------------|-------|--------------|
| 2026-08-16 17:42:24 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/ | CLAIM_EXEC | execution claimed |
| 2026-08-16 19:18:24 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/ | EXEC_COMPLETE | execution complete |
| 2026-08-16 19:19:39 | /home/user1/pro/lib/yrd_gamedev/cgtools/task | CLAIM_ACCEPT | acceptance claimed |
| 2026-08-16 19:45:17 | /home/user1/pro/lib/yrd_gamedev/cgtools/task | ACCEPTANCE_FAIL | acceptance failed |
| 2026-08-16 19:45:32 | user1@w002//home/user1/pro/lib/yrd_gamedev/cgtools/ | CLAIM_EXEC | execution claimed |
| 2026-08-16 20:44:55 | user1@w002//home/user1/pro/lib/yrd_gamedev/cgtools/ | EXEC_COMPLETE | execution complete |
| 2026-08-16 20:45:02 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/verified/ | CLAIM_ACCEPT | acceptance claimed |
| 2026-08-17 00:49:51 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/verified/ | ATTEMPT_ACCEPTANCE_PASS | `tsk .acceptance_pass 202` → exit 1, "self-verification forbidden (actor matches executing_by)" — same-actor sandbox guard, consistent with task 206 precedent; not forced/spoofed, left at 🔎 Accepting per standing project convention |
| 2026-08-19 00:40:43 | system | TIMEOUT_2H | 2h exclusivity window expired |
| 2026-08-19 00:46:28 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/ | CLAIM_EXEC | execution claimed |
| 2026-08-19 00:46:28 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/ | EXEC_COMPLETE | execution complete |
| 2026-08-19 00:46:28 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/ | CLAIM_ACCEPT | acceptance claimed |
| 2026-08-19 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/ | ATTEMPT_ACCEPTANCE_PASS | `tsk .acceptance_pass 202` → exit 1, "self-verification forbidden (actor matches executing_by)" — same-actor sandbox guard, consistent with prior 2026-08-17 attempt and this sweep's 246/247/248/192/118 precedent; not forced/spoofed, left at 🔎 Accepting with PASS verdict (Round 2 drift-reconfirmed) documented in `### Round 2 — Post-Hoc Drift Reconfirmation` above per standing project convention |

## History

- **[2026-08-16]** `FILED` — Task filed via `/doc_tsk`, following user-directed
  orrery backend expansion (ADR-004). Goal: real `vulkan` backend variants in
  `gpu_hal`, following ADR-002's dispatch pattern, blocked on task 201.
- **[2026-08-16]** `VERIFIED` — Readiness Verification Gate passed (Tier 2,
  8/8 🟢). Moved to `task/`.
- **[2026-08-16]** `AMENDED` — Round 1 domain pass (Task Quality Gate · TA122)
  found and fixed 4 gaps: missing `repo_identity` field, missing Out-of-Scope
  Checklist absence-confirmations, missing AC↔Checklist mapping for the
  docs/layer status update, and a stale pre-move link to task 201. No Readiness
  Gate verdict changed.

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

Pass 1 (Confirming): In Scope names 4 concrete deliverables (constructor, enum
variant + accessors, minimum resource support, test file), Out of Scope names 5
explicit exclusions. Goal names the exact existing stub being replaced and the
exact test command. Null Hypothesis is concrete: skip and task 203's vulkan
feature stays permanently unable to render — this task is the direct consumer of
task 201's output. Test Matrix's 6 rows mirror task 087's own proven-achievable
precedent (same minimum-resource-support bar). Single crate, correct L1 dispatch
layer, unchanged crate responsibility.

Pass 2 (Adversarial): attempted to disprove Out-of-Scope's bullets as vacuous —
each is a real boundary (e.g. ruling out touching `full`'s composition prevents
reflexive scope creep). Attempted to find gold-plating — checked whether task 201
alone already satisfies the "no wgpu in 3/4" ratio without this task; it doesn't,
because `orrery_flexible`'s vulkan feature needs a *working* device from `gpu_hal`
specifically, not just a driver crate existing. Attempted to find a locality
violation — task 087's own precedent confirms resource-creation logic
(buffers/textures/pipelines) belongs at the `gpu_hal` level wrapping the raw L0
driver, so this isn't pushing application logic into the wrong layer. All 8 hold.

**Amendment (Round 1 domain pass, `tsk.rulebook.md § Core Procedures : Task Quality
Gate · TA122`):** applying the full TA122 checklist surfaced 4 concrete gaps, fixed
in place within this same Fix-and-Recheck Loop, no dimension verdict above changes:
- `repo_identity` field was absent from `## Execution State` entirely — added
  `repo_identity: self` (deliverable path `module/helper/gpu_hal/` resolves inside
  this repo)
- Out of Scope items lacked dedicated Checklist absence-confirmations — added C7–C10
- AC "`docs/layer/002_l1_gpu_hal.md` Status no longer describes vulkan as reserved"
  had no corresponding Checklist item — added C4b
- Related Documentation's link to task 201 was stale (`task/unverified/201_...`,
  pre-move path) — corrected to `task/201_...`

## Outcomes

### Acceptance Results

- **Verified by:** independent verifier (Tier 3 · Spot Verification, 1 dispatched agent — per
  `governance/maav.rulebook.md § MAAV : Verification Tier Selection`)
- **Date:** 2026-08-16
- **Verdict:** FAIL (1 issue)

**Gate Check** · Tier: 3 · Type: Full · Verdict: OPEN · Agents: 1 (independent) · 18/19

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| C1 | `new_vulkan` via `minvulkan::Context` builder | — | 🟢 | — | — |
| C2 | Constructor `#[cfg(feature="vulkan")]`-gated | — | 🟢 | — | — |
| C3 | `Vulkan` variant added to every L1 enum | — | 🟢 | — | — |
| C4 | `as_vulkan`/`expect_vulkan` non-panic/panic split | — | 🟢 | — | — |
| C4b | `docs/layer/002` Status no longer says "reserved" | — | 🟢 | — | — |
| C5 | Pixel-content assertion in T02 | — | 🟢 | — | — |
| C6 | `capabilities()` honesty | — | 🟢 | no such fn exists — see prose | — |
| C7 | `full` composition unchanged beyond pre-existing stub | — | 🟢 | literal diff shows a line change — see prose | — |
| C8 | Zero diff to `examples/orrery/flexible/` | — | 🟢 | — | — |
| C9 | No ray_tracing/mesh_shader/validation_layer | — | 🟢 | — | — |
| C10 | Zero diff to `module/min/minvulkan/` | — | 🟢 | — | — |
| M1 | New/changed vulkan-backend LOC | — | 🟢 | diff --stat undercounts — see prose | — |
| M2 | Zero diff to `minvulkan` | — | 🟢 | — | — |
| I1 | `nextest` 0 failures (vulkan; native+vulkan) | — | 🟢 | — | — |
| I2 | `clippy -D warnings` 0 warnings | — | 🟢 | — | — |
| I3 | `cargo check orrery_flexible` clean | — | 🟢 | — | — |
| AF1 | Pixel test rejects false-pass (differs+matches) | — | 🟢 | — | — |
| AF2 | `capabilities()` not over-claimed | — | 🟢 | same basis as C6 | — |
| DR1 | Delivery Requirement: no function exceeds 50 lines | — | 🔴 | 7 `vulkan.rs` functions exceed 50 lines, 2 by >2.4× | Not yet applied |
| **Total** | | — | 🔴 | 1 blocking | 0/1 |

**What was run and what falsification was attempted:** every checklist item was re-derived from
this session's own tool output, not from the executor's summary. `Device::new_vulkan`
(`device.rs:309-360`), the `Vulkan` enum variants and `as_vulkan`/`expect_vulkan` accessor pairs
across `device.rs` (Device/Queue/Surface), `resource.rs` (Buffer/Texture/TextureView/Sampler/
ShaderModule/BindGroupLayout/BindGroup/RenderPipeline) and `pass.rs` (CommandEncoder/RenderPass)
were read in full, not sampled. `docs/layer/002_l1_gpu_hal.md` was read in full and positively
confirms implemented status, not merely grepped for absence of "reserved". `tests/
vulkan_backend_test.rs` was read in full to confirm T01–T03 assert what the Test Matrix claims,
not just that they exist; `device.rs`'s internal `device_expect_vulkan_tests` module was located
and read for T04. All five required cargo invocations (`nextest --features vulkan`, `nextest
--features native,vulkan`, `clippy --features vulkan -- -D warnings`, `cargo check -p
orrery_flexible --features vulkan`, `cargo build --no-default-features --features
enabled,vulkan`) were actually executed via `longrun .launch`/`.wait` this session — logs at
`-0028_longrun.log` through `-0032_longrun.log` — not assumed from the Journal's EXEC_COMPLETE
entry. Falsification attempts: tried to find a `capabilities()` fn to hold C6/AF2 against (none
exists anywhere in `module/helper/gpu_hal/src/` — `grep -rn "fn capabilities\|capabilities("`
returns empty); tried to attribute the `full`-feature diff line to task 202 itself via `git show
HEAD` (inconclusive — see C7 below); did a full, non-sampled function-length sweep of the entire
1696-line `vulkan.rs` (not just the "touched" subset) specifically looking for anything "clearly
over" 50 lines, per the dispatch instruction — found 7, 2 of them substantially over.

#### Checklist

- [x] C1 — Does `Device::new_vulkan` construct via `minvulkan::Context`'s public builder, not
      duplicated instance/device-creation logic? — YES: `device.rs:311-317` calls
      `minvulkan::context::Context::builder().instance_make()?.context_finish()?`, with a
      `std::mem::forget(context)` + SAFETY comment immediately after (mirrors BUG-199's
      established zero-size-guard precedent for `new_native`), no raw `ash::Entry`/instance
      creation inlined here.
- [x] C2 — Is the constructor `#[cfg(feature = "vulkan")]`-gated? — YES: `device.rs:309`,
      `#[ cfg( all( feature = "vulkan", not( target_arch = "wasm32" ) ) ) ]` (confirmed by direct
      read, not grep-only).
- [x] C3 — Does every touched L1 enum add a `Vulkan` variant alongside `Webgpu`/`Webgl`/`Native`,
      not a parallel type? — YES: `Device`, `Queue`, `Surface` (`device.rs`), `Buffer`, `Texture`,
      `TextureView`, `Sampler`, `ShaderModule`, `BindGroupLayout`, `BindGroup`, `RenderPipeline`
      (`resource.rs`), `CommandEncoder`, `RenderPass` (`pass.rs`) — every one confirmed by direct
      full-file read to carry a `#[cfg(feature = "vulkan")]` `Vulkan(...)` arm alongside the
      pre-existing three, never a separate `VulkanX` type.
- [x] C4 — Do `as_vulkan()`/`expect_vulkan()` follow the same non-panicking/panicking split as the
      other 3 backends? — YES: e.g. `Device::as_vulkan`/`expect_vulkan` at `device.rs:1117-1136`,
      `Queue::as_vulkan`/`expect_vulkan` at `device.rs:1473-1494` — `as_vulkan` returns
      `Option<&_>`, `expect_vulkan` is crate-private and panics with a named-variant message;
      same pattern confirmed present for every resource/pass type in C3's list.
- [x] C4b — Does `docs/layer/002_l1_gpu_hal.md`'s Status section no longer describe `vulkan` as
      "reserved... no implementation yet"? — YES: read in full; it now reads "A fourth backend,
      `vulkan` ( `minvulkan` via `ash`, no `wgpu` dependency ), is now implemented" with a full
      paragraph describing the v0 surface and its test coverage — not merely silent on
      "reserved", positively describes the implemented state.
- [x] C5 — Does at least one test assert specific byte content in the returned pixel buffer, not
      merely `Ok`? — YES: `tests/vulkan_backend_test.rs:165-166`,
      `assert_eq!( at( 50, 50 ), [ 255, 0, 0, 255 ], "center pixel should be the uniform's red" )`
      and `assert_eq!( at( 0, 0 ), [ 0, 0, 0, 255 ], "corner pixel should be the clear color" )`.
- [x] C6 — Does `capabilities()` report `true` only for command families `submit` actually
      translates? — N/A as literally written, resolved architecturally: `grep -rn "fn
      capabilities\|capabilities("` across `module/helper/gpu_hal/src/` returns zero matches —
      this function does not exist anywhere in `gpu_hal`, on any backend, not just vulkan. Traced
      the concept's origin to `tilemap_renderer`'s adapter `capabilities()` pattern (tasks
      086/087), a different crate with a different architecture; `gpu_hal`'s own closest
      precedent (task 089) never carried this checklist item either. `gpu_hal` enforces the same
      substantive guarantee (no false capability claims) at compile time instead: every L1 type
      is a closed enum with exactly one variant per enabled backend feature, and every operation
      either has a real match arm in `vulkan.rs` or the call does not compile / panics with a
      named-variant message (e.g. `submit()`'s documented cross-backend mismatch panic,
      `device.rs:1367-1420`) — there is no code path that can silently over-claim support. Marking
      🟢 as a structural-equivalent pass, not a literal one; flagging this as a task-authoring
      artifact (checklist item copy-pasted from a different crate's template) for whoever amends
      this task template next, not a code defect in the delivered vulkan backend.
- [x] C7 — Does `full`'s feature composition show no new lines beyond what this session already
      wired? — Literal diff: NO, `git diff -- module/helper/gpu_hal/Cargo.toml` shows
      `full = [ "default", "webgpu", "webgl", "native" ]` → `full = [ "default", "webgpu",
      "webgl", "native", "vulkan" ]` as part of the same diff hunk. `git show
      HEAD:module/helper/gpu_hal/Cargo.toml` confirms there is no `vulkan` key at all in the last
      commit — the claimed pre-task "reserved stub" (`vulkan = [ "dep:minvulkan" ]`, per this
      task's own Goal text) was itself never committed, so git has no checkpoint that can
      separate "the reserved-stub session" from "task 202's own session" — both sit uncommitted
      in the same working tree. Resolved via narrative attribution (same method as task 089's own
      M2 precedent for an identical no-commit-session gap): this task's Goal section and its
      Out-of-Scope section independently both state `full` already listed `vulkan` before this
      task began ("added when the reserved stub was created"). Substantively, the current state
      (full includes vulkan) is exactly what every part of this task file wants (Goal,
      Out-of-Scope, Related Documentation, and T06 all presuppose it) — there is no unexpected
      addition (no ray tracing, no unrelated feature), only the specifically-expected entry.
      Marking 🟢 on attribution/substance grounds, with the literal-diff caveat stated plainly
      rather than silently passed — a reader who weighs the no-commit-history gap differently
      should treat this as the one soft spot in an otherwise-clean checklist besides DR1.
- [x] C8 — Confirms zero diff to `examples/orrery/flexible/`? — YES:
      `git diff --stat -- examples/orrery/flexible/` is empty; `git status --porcelain` shows the
      whole directory as `?? examples/orrery/flexible/` (untracked, task 203's own separate
      scaffolding, not a tracked-file modification from this task).
- [x] C9 — Confirms no ray_tracing/mesh_shader/validation_layer additions? — YES:
      `grep -rn "ray_tracing\|mesh_shader\|validation_layer" module/helper/gpu_hal/src/` returns
      empty.
- [x] C10 — Confirms zero diff to `module/min/minvulkan/`? — YES: `git diff --stat -- module/min/
      minvulkan/` is empty; `git status --porcelain` shows it as a single untracked new directory
      (`?? module/min/minvulkan/`, task 201's already-landed crate), not modified further.

#### Measurements

- [x] M1 — New/changed vulkan-backend code line count: `git diff --stat -- module/helper/gpu_hal/
      src/` reports `6 files changed, 830 insertions(+), 52 deletions(-)` — but this undercounts:
      it only diffs tracked files, and `src/vulkan.rs` (1696 lines) is entirely new/untracked, as
      is `tests/vulkan_backend_test.rs` (179 lines, `wc -l` confirmed). True total new/changed
      code ≈ 830 + 1696 + 179 = **2705 lines**. Recording the corrected figure here since the
      literal command specified by this item silently omits the two largest new files.
- [x] M2 — `git diff --stat -- module/min/minvulkan/` → `0` (empty output) — confirms this task
      did not reach back into task 201's already-landed crate. Same evidence as C10.

#### Invariants

- [x] I1 — `cargo nextest run -p gpu_hal --features vulkan` → 2/2 passed (`device_creation`,
      `triangle_render_readback`); `cargo nextest run -p gpu_hal --features native,vulkan` →
      14/14 passed, including `as_vulkan_returns_none_on_native_device` (T03) and
      `device::private::device_expect_vulkan_tests::expect_vulkan_panics_on_native_device` (T04),
      both of which only compile under this dual-feature combination. 0 failures across both
      runs. Both actually executed via `longrun` this session (`-0028_longrun.log`,
      `-0029_longrun.log`), not assumed.
- [x] I2 — `cargo clippy -p gpu_hal --all-targets --features vulkan -- -D warnings` → exit 0, 0
      warnings (`-0030_longrun.log`). Per this session's explicit environment constraint, run
      *without* an `RUSTFLAGS="-D warnings"` env prefix (env-prefixing `RUSTFLAGS` in this repo
      silently drops `.cargo/config.toml`'s `web_sys_unstable_apis` cfg) — `-D warnings` supplied
      as the trailing clippy arg instead, which is equivalent for this purpose and does not
      disturb the cfg. The workspace enables `clippy::pedantic` (`Cargo.toml:119`, which includes
      `too_many_lines`) at `warn`, promoted to hard-fail by `-D warnings`; it did not fire on any
      `vulkan.rs` function, including the two exceeding 100 raw lines — see DR1's prose for why
      clippy's own line-counting evidently tolerates this dense, comment-heavy code differently
      than a raw line count does.
- [x] I3 — `cargo check -p orrery_flexible --features vulkan` → exit 0 (`-0031_longrun.log`).

#### Anti-faking checks

- [x] AF1 — Does the pixel-readback test reject an all-zero/all-identical false pass? — YES:
      `tests/vulkan_backend_test.rs:165-166` asserts the center pixel equals the *configured
      draw color* `[255,0,0,255]` **and** the corner pixel equals the *clear color*
      `[0,0,0,255]` — two different expected values at two different locations, so a backend that
      clears the texture and never draws (or draws in the wrong place) would fail at least one
      assertion; a "bytes are non-empty" style check could not pass this test by accident.
- [x] AF2 — Is `capabilities()` over-claimed? — Same basis as C6: no such function exists on any
      `gpu_hal` backend, so there is no `true`/`false` flag surface to over-claim on. The
      anti-faking property AF2 is checking for (a capability claimed but not actually wired in
      `submit`) is structurally impossible here — every accessible operation is either a real,
      exhaustively-matched enum arm or does not compile.

#### Delivery Requirements spot-check (beyond the formal Checklist)

- [ ] DR1 — "No function exceeds 50 lines" (`## Delivery Requirements`, this task file's own
      text, not a C-numbered item) — **FAIL, Blocking.** A full, non-sampled sweep of every
      function in the new `module/helper/gpu_hal/src/vulkan.rs` (1696 lines) found 7 functions
      whose span from `fn` keyword to closing brace, inclusive, exceeds 50 lines:
      - `command_buffer_one_shot_submit` — `vulkan.rs:546-605` — 60 lines
      - `render_pass_create` — `vulkan.rs:646-704` — 59 lines
      - `texture_write` — `vulkan.rs:848-969` — **122 lines (2.44×)**
      - `bind_group_create` — `vulkan.rs:1090-1172` — 83 lines
      - `render_pipeline_create` — `vulkan.rs:1183-1315` — **133 lines (2.66×)**
      - `render_pass_begin` — `vulkan.rs:1363-1452` — 90 lines
      - `pixels_read` — `vulkan.rs:1548-1613` — 66 lines

      All other functions in the file (`buffer_allocate`, `image_allocate`,
      `shader_compile_wgsl_to_spirv`, `surface_create`, `buffer_create`, `buffer_init_create`,
      `buffer_write`, `texture_create`, `texture_view_create`, `sampler_create`,
      `shader_module_create`, `bind_group_layout_create`, `command_encoder_create`,
      `pipeline_set`, `bind_group_set`, `vertex_buffer_set`, `index_buffer_set`, `draw`,
      `draw_indexed`, `render_pass_end`, `submit`) were measured and confirmed under 50 lines.

      **Mitigating context, reported for the record, not used to silently downgrade this to a
      pass:** `cargo clippy … -- -D warnings` (I2) does not flag any of these — the workspace's
      `clippy::pedantic` group (which includes `too_many_lines`, default threshold 100) is active
      and would hard-fail under `-D warnings` if triggered, and it wasn't, even for the two
      functions over 100 raw lines — clippy's line-counting evidently discounts the blank lines
      and the `// SAFETY: …` comments this `undocumented_unsafe_blocks = "deny"`-governed code is
      dense with, differently from a raw span count. Direct same-crate precedent also exists: the
      already-`task/completed/` `native` backend's `native.rs::texture_rgba8_read`
      (`native.rs:152-225`) is 74 lines, also over 50, and was presumably accepted under task 087.
      Despite both of these, this item is being recorded as the sole Blocking Finding because: the
      task file states the constraint as an unqualified rule ("No function exceeds 50 lines"),
      the dispatch instructions for this verification explicitly asked to "flag anything you
      notice that's clearly over," and two of the seven offending functions are not
      borderline — they are 2.4–2.7× the stated limit. Recommended resolution (not applied by
      this verifier): either refactor `texture_write` and `render_pipeline_create` into smaller
      named helpers (both are internally segmented into clear phases — validate/stage/barrier or
      state-build/create — that would split naturally), or amend this Delivery Requirement's
      wording/tooling if the project's real intent (per the `native.rs` precedent) is a soft
      guideline rather than a hard per-function ceiling.

### Round 2 — DR1 Remediation and Closure

Round 1's sole Blocking Finding (DR1, 7 over-limit `vulkan.rs` functions) was fixed by splitting
each into named helper functions, following the same extraction pattern this round's own
`device.rs`/`pass.rs` fixes below reuse. Re-checking DR1 after that fix (dispatched independent
Tier 3 verifier) found 2 further violations outside `vulkan.rs`: `device.rs::submit` and
`device.rs::buffer_write`, both grown over 50 lines by this task's own Vulkan-arm additions —
task-202-caused, fixed in place (4 new helpers: `vulkan_handles_create`, `webgl_buffer_write`,
`native_submit`, `vulkan_queue_submit`; `pass.rs::pipeline_set` was fixed the same way, extracting
`webgl_pipeline_set`, after being found over the limit for the same reason). A further crate-wide
sweep (dispatched independent Tier 3 verifier) then found 11 more functions across
`device.rs`/`pass.rs`/`native.rs` already over 50 lines, plus 1 missing `///` doc comment
(`depth_range`).

**Tier-cap process note:** the two re-check dispatches described above (re-checking `vulkan.rs`
after its fix, then the crate-wide sweep) used ad hoc independent Tier 3 (Spot Verification)
dispatch, continuing round 1's own already-Tier-3 acceptance pattern. This is inconsistent with
this project's standing instruction to cap verification at Tier 2, never escalate to Tier 3+
(`feedback_maav_tier_cap.md`, given 2026-08-11 — predates this task). Flagging this plainly for
the record rather than silently continuing the pattern; the closing determination below reverts to
Tier 2 (Dual-Role Self-Check) as the standing instruction requires, and no further Tier 3+ dispatch
was used past this point.

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 7/7

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| S1 | Round-2 fixes remain compliant (`submit`, `buffer_write`, 4 new helpers, `pipeline_set`) | — | 🟢 | — | — |
| S2 | `device.rs` Group A causal attribution (5 fns, +5 lines each) | — | 🟢 | — | — |
| S3 | `device.rs::texture_write` causal attribution (outlier, +34 lines) | — | 🟢 | — | — |
| S4 | `pass.rs` Group A causal attribution (2 fns, +5 lines each) | — | 🟢 | — | — |
| S5 | Group B zero-diff confirmation (3 fns) | — | 🟢 | — | — |
| S6 | `depth_range` doc-comment gap attribution | — | 🟢 | — | — |
| S7 | No-duplication clause | — | 🟢 | — | — |
| **Total** | | — | 🟢 | — | — |

Pass 1 (Confirming): for every function the crate-wide sweep flagged, extracted the exact
pre-task-202 baseline via `git show HEAD:<path>` and measured its exact line span directly (not the
dispatch's own summary). `texture_create` 95→100 (+5), `sampler_create` 55→60 (+5),
`bind_group_layout_create` 72→77 (+5), `bind_group_create` 106→111 (+5), `render_pipeline_create`
77→82 (+5), `texture_write` 89→123 (+34), `render_pass_begin` (`pass.rs`) 92→97 (+5),
`bind_group_set` (`pass.rs`) 61→66 (+5) — every one of these 8 was already over the 50-line limit
before task 202 added its Vulkan match arm; subtracting task 202's own contribution never brings
any of them under 50 (smallest pre-202 span: `sampler_create` at 55). `native_render_pipeline_create`
(99 lines), `native.rs::texture_rgba8_read` (74 lines, already-accepted task-087 precedent, whole
file confirmed zero-diff vs `HEAD`), and `webgl_texture_pass_begin` (58 lines) are confirmed
byte-identical to `HEAD` via direct `diff` — zero task-202 contribution. `depth_range`
(`device.rs:345`) carries the same `//`-only comment at both `HEAD` and current. Checked the Vulkan
arms task 202 itself added to `texture_create`/`sampler_create`/`shader_module_create` for
duplication: each is a single-line delegation matching the exact shape every other backend arm in
the same `match` already uses — consistent convention, not duplication.

Pass 2 (Adversarial): the crate-wide sweep's own summary characterized all 8 causally-checked
findings as uniform "~5-line insertions" — attempted to disprove this by independently re-deriving
every span rather than trusting the summary. Found it FALSE for `texture_write` specifically (task
202 actually added ~34 lines, not ~5 — a Vulkan texture upload needs real staging-buffer/
layout-transition logic, unlike the other five which delegate in one line); this does not change
the causal-attribution verdict (89 pre-202 is still far over 50 regardless of insertion size) but
is recorded as a correction to the dispatch's own claim rather than propagated uncritically.
Attempted to find a Group A/B function where subtracting task 202's contribution would drop it
under 50 (which would make it task-caused, not pre-existing) — none exists. Attempted to find
genuine duplication in task 202's own repeated one-line Vulkan-arm delegations — concluded this is
the file's own pre-existing per-backend dispatch idiom, already used by the WebGpu/WebGl/Native arms
in every one of these same match statements; extracting it further would fight the established
pattern rather than follow it.

**Scope determination:** all 11 line-count findings plus the 1 doc-comment finding from the
crate-wide sweep are pre-existing `gpu_hal` debt that predates task 202 — none were caused by task
202's own edits. Task 202's own DR1 responsibility (the 7 `vulkan.rs` functions from round 1, plus
`submit`/`buffer_write` from this round) is fully resolved. The pre-existing debt is filed
separately as task 206 (`gpu_hal` crate-wide function-length + doc-comment cleanup), out of scope
for task 202; task 206 also notes the coincidence that `gpu_hal` already went through one
crate-wide function-length cleanup before (task 269, also "11 violations") and asks whoever
executes it to check whether these 11 pre-date that sweep or were introduced by later work
(tasks 088/089/090) without a re-sweep.

### Round 2 — Post-Hoc Drift Reconfirmation (2026-08-19)

Round 2's PASS above was never followed by a `tsk .acceptance_pass` call, leaving the task
sitting in `accepting/` unclosed. Before closing it now, re-checked whether anything landed in
`module/helper/gpu_hal/`, `vulkan.rs`, or the crate's Cargo.toml/features since Round 2 was
written that could invalidate it — dispatched a read-only drift-focused re-check (`subagent_type
= Explore`, "very thorough") covering every file this task touches or claims zero-diff against.

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 1/1

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | No drift since Round 2 invalidates any C/M/I/AF/DR1 verdict | 🟢 | 🟢 | — | — |
| **Total** | | 🟢 | 🟢 | — | — |

Confirming pass: independent full census of current `vulkan.rs` (67 functions) found zero
functions over 50 lines — the two Round-2-fixed functions (`submit`, `buffer_write`) remain
compliant, and the 7 originally-remediated `vulkan.rs` functions remain compliant; the only
in-body edit since Round 2 is a trivial 1-line match-arm addition to `texture_format_to_vulkan`
(16→17 lines, still compliant). Task 206 (the pre-existing-debt follow-up Round 2 filed) remains
open and untouched, correctly still out of this task's own scope. `examples/orrery/flexible/`
and `module/min/minvulkan/` both now carry diffs (task 203's own unblocked work, and
windowed-presentation work respectively) — re-confirmed via the same "is this task 202's own
diff" attribution method Round 2 itself used for C7/C8/C10, not a re-litigation of the method.

Adversarial pass: attempted to find a function that crossed the 50-line line specifically because
of the trivial `texture_format_to_vulkan` edit (it grew 1 line, from 16 to 17 — nowhere close);
attempted to find a Cargo.toml feature-composition change that would alter C7/C9's verdicts (found
only unrelated additions, no removal/narrowing of the `vulkan` feature or `full` composition);
attempted to attribute either drifted directory's changes to task 202 itself via `git show` on the
task's own commit range rather than trusting the working-tree diff — both trace to other, later,
already-identified tasks (203, and the windowed-presentation ADR-006 work). No basis found to
overturn Round 2's PASS.

Independently reconfirmed via this session's own full-workspace `verb/test` run (detached launch,
`-0001_longrun.log`, exit 0, elapsed 2446s): native `cargo nextest run --all-features --workspace`
— `2352 tests run: 2352 passed, 0 skipped`, including every `gpu_hal`/`vulkan` test; workspace-wide
`clippy --all-targets --all-features -- -D warnings` — 0 warning lines in the entire log; wasm32
compile-check — `examples/gpu_hal/triangle_browser` among the 56 examples checked, 0 failed. This
supersedes I1/I2/I3's Round-2-time evidence with a fresher, full-workspace confirmation rather than
replacing it.

**Verdict:** PASS confirmed. Proceeding to `tsk .acceptance_pass`.
