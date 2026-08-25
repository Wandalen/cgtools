# 409: tilemap_renderer svg adapter: Source::Path async fetch redesign for wasm32

## Execution State

- **id:** 409
- **title:** tilemap_renderer svg adapter: Source::Path async fetch redesign for wasm32
- **state:** 📝 (Draft)
- **open:** true
- **in_motion:** false
- **round:** 1
- **filed:** 2026-08-19 23:03:02
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/
- **executor_type:** any
- **unit_type:** module
- **unit:** lib/yrd_gamedev/cgtools/module/helper/tilemap_renderer
- **actor:** null
- **started_at:** null
- **expires_at:** null

## MOST Goal

**Tracking placeholder — needs scoping before becoming claimable.** `roadmap.md`'s "svg adapter
gaps" section notes `Source::Path` geometry loading is blocking `std::fs` only — works natively,
but on wasm32 the read fails at runtime and the geometry is skipped loudly (stderr warning +
diagnostic SVG comment). An async `fetch()` path would need a redesign of the `Backend` trait's
currently-synchronous `assets_load` contract, shared across every adapter (WebGL's own async
image/geometry loading is a separate, adapter-internal code path, not a trait-level async
contract). Real design work, not a mechanical patch — needs an evaluation of whether the shared
trait signature can change without breaking the WebGL/native/none adapters' own synchronous
implementations.

## In Scope

- Investigate whether `Backend::assets_load`'s synchronous signature can accommodate an async
  loading path without breaking the WebGL/native/none adapters' existing (synchronous)
  implementations.
- Design and implement an async `fetch()`-based `Source::Path` resolution path for `SvgBackend`
  on wasm32, replacing today's loud-skip-on-wasm32 behavior.
- Update `roadmap.md`'s "svg adapter gaps" section to remove this item once delivered.

## Out of Scope

- Native `std::fs` loading path — already works correctly, not touched.
- Any other svg adapter gap (font loading/rendering, interactive JS event hooks — tracked in
  sibling tasks 402/403).
- Any other adapter's asset-loading contract, beyond what's needed to keep them unaffected by
  this SVG-side change.

## Requirements

- All work must strictly adhere to all applicable rulebooks (`kbase .rulebooks`)

## Delivery Requirements

- N/A while this task remains 📝 Draft — no implementation is authorized until this is fleshed
  out into a full Quality-Gate task (Test Matrix, Acceptance Criteria, Delivery Requirements
  re-derived against the actual scoped async-loading approach at that time).

## Acceptance Criteria

- N/A while this task remains 📝 Draft — a tracking placeholder, not yet scoped for execution.
  Not intended to progress through SUBMIT/VERIFY toward 🎯 Verified/claimable state until
  fleshed out.

## Verification

- N/A while this task remains 📝 Draft — same rationale as Acceptance Criteria above.

## Related Documentation

- `module/helper/tilemap_renderer/roadmap.md` — "svg adapter gaps" section
- `task/draft/402_tilemap_renderer_svg_adapter_font_loading_and_rendering.md` and
  `task/draft/403_tilemap_renderer_svg_adapter_interactive_js_event_hooks.md` — sibling svg
  adapter gap tracking tasks filed the same session

## Journal

| Timestamp           | Actor                | Event | Note         |
|---------------------|----------------------|-------|--------------|
| 2026-08-19 23:03:02 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/ | FILED | task created |
| 2026-08-19 23:30:22 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/ | NOTE | Watch-condition spot-checked, not submitted: claim that `Backend::assets_load`'s signature is currently synchronous and shared across every adapter confirmed accurate — the trait method (`src/backend.rs:166`) and all 6 adapter implementations (`terminal.rs:889`, `webgpu.rs:253`, `native.rs:154`, `none.rs:37`, `webgl.rs:1152`, `svg.rs:1698`) are all synchronous, no `async` anywhere in the set. Sibling tasks 402/403 confirmed to exist as filed the same session. Correctly remains 📝 Draft; not run through SUBMIT/`.claim_verify`/`.verify_pass`, mirroring precedent watch-items 056/098/291. |
