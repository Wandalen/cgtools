# 399: tilemap_renderer webgl adapter — group commands

## Execution State

- **id:** 399
- **title:** tilemap_renderer webgl adapter — group commands
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

**Tracking placeholder — needs scoping before becoming claimable.** `roadmap.md`'s WebGL2 adapter section
lists group commands as remaining work: nested transform/clip group stacking (the SVG adapter tracks
this today via its `group_depth` nesting counter and native `<g>` element nesting, per
`docs/invariant/003_z_layer_draw_ordering.md`'s Enforcement Mechanism section) has no WebGL2 equivalent —
group-level transform composition and group-scoped clipping aren't honored by the adapter. Too large for
one-pass implementation: needs a transform-stack design (matrix composition on group enter/exit) and a
clip-scoping mechanism (stencil buffer or scissor-rect based) before any code lands.

## In Scope

- Design and implement group-command handling (transform-stack composition, group-scoped clipping) in
  `src/adapters/webgl.rs`, matching the nesting semantics the SVG adapter already exhibits.

## Out of Scope

- Clip-mask *asset* support (image-based masks) — separate sibling draft task (gradient/pattern/clip-mask
  asset loading); this task is specifically about group-level transform/clip nesting, not mask assets.
- Terminal adapter's own group clip/effects gap — separate sibling draft task.

## Requirements

- All work must strictly adhere to all applicable rulebooks (`kbase .rulebooks`)

## Delivery Requirements

- N/A while this task remains 📝 Draft — no implementation is authorized until this is fleshed out into
  a full Quality-Gate task (Test Matrix, Acceptance Criteria, Delivery Requirements re-derived against
  the actual scoped transform-stack/clip design at that time).

## Acceptance Criteria

- N/A while this task remains 📝 Draft — a tracking placeholder, not yet scoped for execution. Not
  intended to progress through SUBMIT/VERIFY toward 🎯 Verified/claimable state until fleshed out.

## Verification

- N/A while this task remains 📝 Draft — same rationale as Acceptance Criteria above.

## Related Documentation

- `module/helper/tilemap_renderer/roadmap.md` — WebGL2 adapter remaining-work section
- `module/helper/tilemap_renderer/docs/invariant/003_z_layer_draw_ordering.md` — SVG adapter's existing
  `group_depth` nesting precedent

## Journal

| Timestamp           | Actor                | Event | Note         |
|---------------------|----------------------|-------|--------------|
| 2026-08-19 22:51:09 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/ | FILED | task created |
| 2026-08-19 23:30:22 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/ | NOTE | Watch-condition spot-checked, not submitted: claim that the SVG adapter already tracks group-level transform/clip nesting via a `group_depth` counter (this task's WebGL2 precedent target) confirmed accurate — `src/adapters/svg.rs:259,285,1675,1683,1686,1721` show real `group_depth` field/logic, and `docs/invariant/003_z_layer_draw_ordering.md:30,67` reference it. No WebGL2 equivalent found in `src/adapters/webgl.rs` (grep for group/clip-stack logic returns nothing beyond the flat capability flags) — the gap is real. Correctly remains 📝 Draft; not run through SUBMIT/`.claim_verify`/`.verify_pass`, mirroring precedent watch-items 056/098/291. |
