# 402: tilemap_renderer svg adapter — font loading and rendering

## Execution State

- **id:** 402
- **title:** tilemap_renderer svg adapter — font loading and rendering
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

**Tracking placeholder — needs scoping before becoming claimable.** `roadmap.md`'s SVG adapter section
lists font loading/rendering as remaining work: `RenderCommand::Text` today emits SVG `<text>` elements
that rely on the host viewer's/browser's system or webfont resolution by font-family name — there is no
embedded font asset loading (e.g. `@font-face`/base64-embedded webfont data) that would make text render
identically regardless of which fonts are installed on the viewing system. Too large for one-pass
implementation: needs a font-asset embedding strategy decision (inline `@font-face` with embedded font
data vs. external font-file reference) and glyph-metrics handling for layout consistency.

## In Scope

- Design and implement embedded font asset loading for the SVG adapter's text rendering path
  (`src/adapters/svg.rs`), so rendered SVG output doesn't depend on the viewer's installed fonts.

## Out of Scope

- WebGL2 adapter's text rendering (entirely separate rasterization path) — separate sibling draft task.
- Text-on-path — separate, more advanced capability.

## Requirements

- All work must strictly adhere to all applicable rulebooks (`kbase .rulebooks`)

## Delivery Requirements

- N/A while this task remains 📝 Draft — no implementation is authorized until this is fleshed out into
  a full Quality-Gate task (Test Matrix, Acceptance Criteria, Delivery Requirements re-derived against
  the actual scoped font-embedding approach at that time).

## Acceptance Criteria

- N/A while this task remains 📝 Draft — a tracking placeholder, not yet scoped for execution. Not
  intended to progress through SUBMIT/VERIFY toward 🎯 Verified/claimable state until fleshed out.

## Verification

- N/A while this task remains 📝 Draft — same rationale as Acceptance Criteria above.

## Related Documentation

- `module/helper/tilemap_renderer/roadmap.md` — SVG adapter remaining-work section

## Journal

| Timestamp           | Actor                | Event | Note         |
|---------------------|----------------------|-------|--------------|
| 2026-08-19 22:51:09 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/ | FILED | task created |
| 2026-08-19 23:30:22 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/ | NOTE | Watch-condition spot-checked, not submitted: claim that `<text>` elements rely on host font resolution with no embedded font-asset loading confirmed accurate — `src/adapters/svg.rs:245`'s own comment states emitted `<text>` elements "carry no font-family." Correctly remains 📝 Draft; not run through SUBMIT/`.claim_verify`/`.verify_pass`, mirroring precedent watch-items 056/098/291. |
