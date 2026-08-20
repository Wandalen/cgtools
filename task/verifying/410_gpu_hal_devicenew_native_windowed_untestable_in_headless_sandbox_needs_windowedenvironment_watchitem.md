# 410: gpu_hal Device::new_native_windowed: untestable in headless sandbox, needs windowed-environment watch-item

## Execution State

- **id:** 410
- **title:** gpu_hal Device::new_native_windowed: untestable in headless sandbox, needs windowed-environment watch-item
- **state:** 🔬 (Verifying)
- **open:** true
- **in_motion:** true
- **round:** 1
- **filed:** 2026-08-19 23:03:23
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/
- **executor_type:** any
- **unit_type:** module
- **unit:** gpu_hal
- **actor:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/
- **started_at:** 2026-08-19 23:19:54
- **expires_at:** 2026-08-20 01:19:54
- **unverified_at:** 2026-08-19 23:07:04
- **unverified_by:** unknown
- **verifying_at:** 2026-08-19 23:19:54
- **verifying_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/

## MOST Goal

Close the documented-but-unimplemented gap in `gpu_hal/readme.md`'s own "Windowed
presentation" / "Verify" sections: `Device::new_native_windowed` (the wgpu-backed
windowed constructor, sibling of `Device::new_vulkan_windowed`) has no example
exercising it, so its `Surface::NativeWindow` dispatch arm is unexercised by
anything that runs — unlike `VulkanWindow`'s, which `examples/gpu_hal/triangle_vulkan_window`
already covers. This is **not** the permanently-headless-blocked case it first
looked like: this sandbox has a real X display (`DISPLAY=:0`, a live Xorg server)
plus `wmctrl` and `import` (ImageMagick) on PATH — the exact tools
`tests/manual/readme.md` Scenario 5 already uses to verify `triangle_vulkan_window`.
The same technique applies here with no new tooling.

## In Scope

- New example crate `examples/gpu_hal/triangle_native_window`, mirroring
  `examples/gpu_hal/triangle_vulkan_window/src/main.rs` structurally:
  - `Device::new_native_windowed( window, size )` in place of `new_vulkan_windowed`
  - `gpu_hal = { workspace = true, features = [ "native" ] }` in place of `vulkan`
    (confirm exact feature name against `Cargo.toml` — `new_native_windowed` is
    gated `#[cfg(all(feature = "native", not(target_arch = "wasm32")))]` in
    `device.rs`)
  - Window passed via `Arc<Window>` (per `examples/minwgpu/flecs_bouncing_circles`'s
    established pattern for satisfying `impl Into<wgpu::SurfaceTarget<'static>>`),
    not the bare `&Window` the Vulkan example uses (Vulkan takes handle traits by
    reference; wgpu's `SurfaceTarget` needs an owned/`Arc`'d handle)
  - **No `Surface::as_native_windowed()` accessor exists** (checked — only
    `as_vulkan_windowed()` does). The Vulkan example's `size()` helper reads the
    surface back for its `SurfaceNotReady` recovery path; this example must
    instead track the window's current size itself (store last-known size from
    `WindowEvent::Resized`, or call `window.inner_size()` directly on the
    retained `Arc<Window>`) — do not add a new gpu_hal API to work around this,
    the window already knows its own size
  - `readme.md` + `verb/run` siblings, following the Vulkan example's shape
  - `[package.metadata.action] tags = ["runtime:native", "api:wgpu"]` (or
    whatever tag vocabulary `action/readme.md` currently documents for
    native-wgpu examples — verify against existing tags, don't invent new ones)
- Register in workspace `Cargo.toml` members + the same 4 gallery-tracking files
  every prior example addition has touched (`examples/readme.md`, `examples/index.md`,
  `examples/index.html`, `demo_completeness.md` — confirm exact set via how
  `triangle_browser`/`hello_triangle_quickstart` were registered)
- Add a Scenario to `gpu_hal/tests/manual/readme.md`, mirroring existing Scenario 5
  (Vulkan windowed: `wmctrl` + `import` against a live window), for this example
- Update `gpu_hal/readme.md`'s "Windowed presentation" / "Verify" sections: replace
  "`new_native_windowed` has no such example... unexercised by anything that runs"
  with an accurate statement once the example exists
- Actually run and manually verify the example renders (drag/resize the window,
  screenshot via `import`, confirm a triangle paints and the swapchain survives
  a resize) — this is the whole point, not just code that compiles

## Out of Scope

- Any automated `cargo test` coverage — same structural constraint as the Vulkan
  windowed path: no crate under `module/` can produce a window handle without a
  windowing dependency, so this stays manually-verified-by-example, never
  `cargo nextest`-automated
- Changing `new_native_windowed`'s own signature or contract
- The WebGPU/WebGL browser-canvas paths — already covered by `triangle_browser`,
  unrelated to this gap

## Requirements

- All work must strictly adhere to all applicable rulebooks (`kbase .rulebooks`)

## Delivery Requirements

- `examples/gpu_hal/triangle_native_window` builds (`cargo build -p gpu_hal_triangle_native_window`)
  and renders a real, visually-verified triangle in a real window in this sandbox
- All 4 gallery-tracking files + workspace `Cargo.toml` updated consistently with
  every other example's registration
- `gpu_hal/tests/manual/readme.md` gains a Scenario for this example
- `gpu_hal/readme.md`'s stale "has no such example" callout is corrected
- Full-workspace native verification (`cargo nextest run --all-features` equivalent
  scope) still passes after the addition — new example crate must not break
  workspace resolution

## Acceptance Criteria

- AC1: `examples/gpu_hal/triangle_native_window` exists with Cargo.toml, readme.md,
  src/main.rs, verb/run, mirroring `triangle_vulkan_window`'s shape
- AC2: `cargo build -p gpu_hal_triangle_native_window` succeeds
- AC3: The example was actually run in this sandbox and visually confirmed —
  screenshot evidence (via `import`) showing a rendered triangle, not just a
  clean compile
- AC4: Window resize survives — the swapchain rebuilds rather than panicking or
  leaving a stale frame (matching the Vulkan example's `SurfaceNotReady` handling)
- AC5: Registered in workspace `Cargo.toml` + all 4 gallery-tracking files
  consistent with every prior example's registration
- AC6: `gpu_hal/tests/manual/readme.md` has a new Scenario documenting how to
  reverify this example manually
- AC7: `gpu_hal/readme.md`'s stale "has no such example" line is corrected
- AC8: Full-workspace native verification (`cargo nextest run --all-features`
  scope, or narrower package-scoped equivalent per longrun.rulebook.md breadth
  selection) still passes — new crate doesn't break workspace resolution

## Verification

- T01: `cargo build -p gpu_hal_triangle_native_window` — clean build
- T02: `cargo clippy -p gpu_hal_triangle_native_window --all-targets -- -D warnings`
  — zero warnings
- T03: Manual run + screenshot — triangle renders, colors cycle across frames
  (same technique as the Vulkan example's own verification)
- T04: Manual resize test — drag the window edge, confirm the swapchain
  rebuilds instead of erroring
- T05: `cargo nextest run -p gpu_hal --all-features` — confirm the addition
  doesn't regress gpu_hal's own existing test suite
- T06: Gallery file consistency spot-check — new example's entry present and
  correctly formatted in all 4 tracking files

## Verification Record

Fresh Tier 2 Dual-Role Self-Check (8-dimension Readiness Gate), self-administered, no subagent dispatch.
Despite the filename's "watchitem" label, the body content resolves its own watch condition (confirms a
real X display + `wmctrl`/`import` exist in this sandbox) and is fully scoped/actionable — treated as a
genuine candidate for 🎯 Verified, not parked like task 291.

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 8/8

| Gate | Name | Prev | Now | Issues | Fixes |
| ---- | ---- | ---- | --- | ------ | ----- |
| D1 | Scope Coherence | — | 🟢 | In/Out Scope well-formed; matching in-flight uncommitted work already exists at the exact In-Scope path (see below) | — |
| D2 | MOST Goal Quality | — | 🟢 | Motivated/Observable/Scoped/Testable all concrete (AC1-AC8, T01-T06) | — |
| D3 | Value/YAGNI | — | 🟢 | Real gap, not speculative; central premise independently re-verified (below) | — |
| D4 | Implementation Readiness | — | 🟢 | Delivery Requirements/Acceptance Criteria/Verification all concrete and executable | — |
| D5 | Execution Scope | — | 🟢 | All paths resolve inside repo | — |
| D6 | Crate Scope Unity | — | 🟢 | New example crate + gpu_hal's own docs + workspace registration/gallery files — the established, precedented pattern for every prior example-crate-addition task this session | — |
| D7 | Crate Locality | — | 🟢 | Targets leaf crates directly | — |
| D8 | Crate Single Responsibility | — | 🟢 | Both `gpu_hal` and the new example crate stay one-sentence-statable | — |
| **Total** | | — | 🟢 | 2 non-blocking (D1, D6) | 0/0 |

**Adversarial pass — independently re-verified rather than trusting the file's self-report:**
- `DISPLAY=:0` confirmed live; `wmctrl` and `import` both confirmed on PATH (`/usr/bin/wmctrl`, `/usr/bin/import`) — the task's central "not actually headless-blocked" claim holds.
- `module/helper/gpu_hal/readme.md:134-137` still literally reads "`new_native_windowed` has no such example ... unexercised by anything that runs" — claim of a currently-stale doc gap confirmed accurate as of this check.
- `module/helper/gpu_hal/tests/manual/readme.md` Scenario 5 precedent (wmctrl+import windowed-verification technique, lines 174-178/192/240) confirmed present — the technique this task proposes reusing genuinely already exists.
- **Real finding, not reflexive**: `examples/gpu_hal/triangle_native_window/` already exists on disk (`Cargo.toml`, `readme.md`, `src/main.rs` 8752 bytes, `verb/run`) — `git status --short` shows it fully untracked (`??`), all files timestamped minutes before this check. The `Cargo.toml` package name (`gpu_hal_triangle_native_window`), `features = ["native"]`, and `tags = ["runtime:native", "api:wgpu"]` match this task's own In Scope description exactly — this is very likely the same concurrent actor who filed this task already executing it in parallel, without a `CLAIM_EXEC` state transition (task never left ❓/🔬 in this system). Per the project's task-098 precedent (`task/cancelled/098_obj_viewer_example_proposal_watch_item.md`), flagging rather than silently ignoring: **whoever next claims 410 for execution should inspect this existing uncommitted directory first** rather than assume a clean slate — it may already satisfy some or all of AC1/AC2, or may need reconciliation/replacement. Judged Non-Blocking for *Readiness* (the task description itself remains accurate and well-scoped regardless of who has started building against it), not Acceptance — no acceptance-style judgment of that code's correctness was performed here.

`tsk .verify_pass 410` attempted next; same-actor sandbox guard is expected to block (see Journal).

## Journal

| Timestamp           | Actor                | Event | Note         |
|---------------------|----------------------|-------|--------------|
| 2026-08-19 23:03:23 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/ | FILED | task created |
| 2026-08-19 23:07:04 | unknown | SUBMIT | structural completeness gate passed |
| 2026-08-19 23:22:31 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/ | VERIFY_PASS_ATTEMPTED | Fresh 8-dimension Readiness Gate walk (8/8 PASS, 2 non-blocking: D1 matching in-flight uncommitted work already found at the exact In-Scope path — flagged for whoever claims execution next, D6 standard example-crate registration touches). Central "not headless-blocked" claim independently re-verified (DISPLAY/wmctrl/import all confirmed live). `## Verification Record` appended. `tsk .verify_pass 410` → exit 1, "self-verification forbidden (actor matches filed_by)" — same-actor sandbox guard; not forced/spoofed, left at 🔬 Verifying per standing project convention |
| 2026-08-19 23:19:54 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/ | CLAIM_VERIFY | verification claimed |
