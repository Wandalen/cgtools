# 400: tilemap_renderer webgl adapter — gradient/pattern/clip-mask asset loading

## Execution State

- **id:** 400
- **title:** tilemap_renderer webgl adapter — gradient/pattern/clip-mask asset loading
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

**Tracking placeholder — needs scoping before becoming claimable.** `WebGlBackend::capabilities()`
declares `gradients: false`, `patterns: false`, `clip_masks: false` (task 246's honest-subset pin).
`roadmap.md`'s WebGL2 adapter section lists gradient/pattern/clip-mask asset loading as remaining work —
these need image/asset-backed shader support (gradient LUT textures, tiled pattern textures, alpha/
stencil clip-mask textures) that the current pipeline doesn't build or bind. Too large for one-pass
implementation: three distinct asset-loading paths, each with its own shader sampling logic.

## In Scope

- Design and implement gradient fill support (LUT-texture-based) in `src/adapters/webgl.rs`.
- Design and implement pattern fill support (tiled texture sampling).
- Design and implement clip-mask support (alpha/stencil-texture-based masking).
- Update `WebGlBackend::declared_capabilities()` to report each capability `true` as it lands —
  independently shippable, not required as one atomic change.

## Out of Scope

- Group-level (non-asset-backed) clipping — separate sibling draft task (webgl group commands).
- Gradient/pattern support for other adapters — SVG already supports both natively via SVG defs.

## Requirements

- All work must strictly adhere to all applicable rulebooks (`kbase .rulebooks`)

## Delivery Requirements

- N/A while this task remains 📝 Draft — no implementation is authorized until this is fleshed out into
  a full Quality-Gate task (or split into 3 per-capability tasks) with Test Matrix/Acceptance Criteria
  re-derived against the actual scoped asset-loading design at that time.

## Acceptance Criteria

- N/A while this task remains 📝 Draft — a tracking placeholder, not yet scoped for execution. Not
  intended to progress through SUBMIT/VERIFY toward 🎯 Verified/claimable state until fleshed out.

## Verification

- N/A while this task remains 📝 Draft — same rationale as Acceptance Criteria above.

## Related Documentation

- `module/helper/tilemap_renderer/roadmap.md` — WebGL2 adapter remaining-work section
- `task/accepting/246_tilemap_renderer_webgl_adapter_test_coverage.md` — pins today's honest
  `gradients/patterns/clip_masks: false` capability baseline this task would flip to `true`

## Journal

| Timestamp           | Actor                | Event | Note         |
|---------------------|----------------------|-------|--------------|
| 2026-08-19 22:51:09 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/ | FILED | task created |
| 2026-08-19 23:30:22 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/ | NOTE | Watch-condition spot-checked, not submitted: claim that `WebGlBackend::capabilities()` declares `gradients: false, patterns: false, clip_masks: false` confirmed accurate — `src/adapters/webgl.rs:1131-1139` shows exactly that. Task 246 (`task/accepting/246_tilemap_renderer_webgl_adapter_test_coverage.md`) confirmed to exist, supporting the "honest-subset pin" cross-reference. Correctly remains 📝 Draft; not run through SUBMIT/`.claim_verify`/`.verify_pass`, mirroring precedent watch-items 056/098/291. |
