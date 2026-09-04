# 401: tilemap_renderer webgl adapter — post-processing effects + BlendMode::Overlay

## Execution State

- **id:** 401
- **title:** tilemap_renderer webgl adapter — post-processing effects + BlendMode::Overlay
- **state:** 📝 (Draft)
- **open:** true
- **in_motion:** false
- **round:** 1
- **filed:** 2026-08-19 22:51:09
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/
- **executor_type:** any
- **unit_type:** module
- **unit:** lib/yrd_gamedev/cgtools/module/helper/tilemap_renderer
- **actor:** null
- **started_at:** null
- **expires_at:** null

## MOST Goal

**Tracking placeholder — needs scoping before becoming claimable.** Two related WebGL2-adapter gaps from
`roadmap.md`'s remaining-work section, grouped here as both are post-processing/blend richness items:
(1) `capabilities().effects == false` — blur/drop-shadow post-processing isn't implemented (needs an
offscreen-framebuffer render-to-texture pass plus a blur kernel or shadow-offset shader); (2)
`Capabilities::supported_blend_modes` is `[Normal, Add, Multiply, Screen]` — `BlendMode::Overlay` (a
defined variant per `src/types.rs`) is absent from the list despite the enum already carrying it. Too
large for one-pass implementation: effects need a new render-target/pass-composition mechanism;
Overlay's GLSL blend-equation math itself is well-known but the surrounding capability-declaration and
test-matrix work still needs scoping.

## In Scope

- Design and implement `BlendMode::Overlay`'s WebGL2 blend-equation/shader-side implementation; add it
  to `supported_blend_modes`.
- Design and implement blur and drop-shadow post-processing effects (render-to-texture composition) in
  `src/adapters/webgl.rs`; update `declared_capabilities()` to report `effects: true`.

## Out of Scope

- Any other adapter's blend-mode or effects support.
- Gradient/pattern/clip-mask asset loading — separate sibling draft task.

## Requirements

- All work must strictly adhere to all applicable rulebooks (`kbase .rulebooks`)

## Delivery Requirements

- N/A while this task remains 📝 Draft — no implementation is authorized until this is fleshed out into
  a full Quality-Gate task (Test Matrix, Acceptance Criteria, Delivery Requirements re-derived against
  the actual scoped blend-equation/render-pass design at that time).

## Acceptance Criteria

- N/A while this task remains 📝 Draft — a tracking placeholder, not yet scoped for execution. Not
  intended to progress through SUBMIT/VERIFY toward 🎯 Verified/claimable state until fleshed out.

## Verification

- N/A while this task remains 📝 Draft — same rationale as Acceptance Criteria above.

## Related Documentation

- `module/helper/tilemap_renderer/roadmap.md` — WebGL2 adapter remaining-work section
- `task/accepting/246_tilemap_renderer_webgl_adapter_test_coverage.md` — pins today's honest
  `effects: false` capability and 4-entry `supported_blend_modes` baseline this task would extend

## Journal

| Timestamp           | Actor                | Event | Note         |
|---------------------|----------------------|-------|--------------|
| 2026-08-19 22:51:09 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/ | FILED | task created |
| 2026-08-19 23:30:22 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/ | NOTE | Watch-condition spot-checked, not submitted: both claims confirmed accurate — `src/adapters/webgl.rs:1131-1139` shows `effects: false` and `supported_blend_modes` with exactly 4 entries (Normal/Add/Multiply/Screen); `src/types.rs:455` confirms `Overlay` is a real, already-defined enum variant absent from that list. Task 246 cross-reference confirmed to exist. Correctly remains 📝 Draft; not run through SUBMIT/`.claim_verify`/`.verify_pass`, mirroring precedent watch-items 056/098/291. |
