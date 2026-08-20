# 403: tilemap_renderer svg adapter — interactive JS event hooks

## Execution State

- **id:** 403
- **title:** tilemap_renderer svg adapter — interactive JS event hooks
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
lists interactive JS events as remaining work: rendered SVG elements carry no mechanism today for
attaching DOM event hooks (click/hover/etc.) back to scene entities — the adapter is a pure
render-to-markup path with no interaction/event-binding layer. Too large for one-pass implementation:
needs an entity-to-DOM-element identity/addressing scheme (e.g. `data-*` attributes or `id` conventions)
and a decision on where the event-binding/dispatch logic lives (adapter-emitted inline handlers vs. a
separate JS-side binding layer consuming addressable output).

## In Scope

- Design an entity-addressing convention for SVG output elements (stable IDs/data-attributes) in
  `src/adapters/svg.rs`.
- Design and implement a JS-side (or inline-handler) event-binding mechanism connecting DOM interaction
  events back to scene/entity identity.

## Out of Scope

- Any other adapter's interactivity — WebGL2/terminal have no DOM presence to hook into the same way.

## Requirements

- All work must strictly adhere to all applicable rulebooks (`kbase .rulebooks`)

## Delivery Requirements

- N/A while this task remains 📝 Draft — no implementation is authorized until this is fleshed out into
  a full Quality-Gate task (Test Matrix, Acceptance Criteria, Delivery Requirements re-derived against
  the actual scoped addressing/event-binding design at that time).

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
| 2026-08-19 23:30:22 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/ | NOTE | Watch-condition spot-checked, not submitted: claim that no DOM event-hook mechanism exists confirmed accurate — grep across `src/adapters/svg.rs` for `addEventListener`/`onclick`/`data-entity`/`data-id`/`EventTarget`/`dispatch_event` returns zero matches; the adapter is genuinely a pure render-to-markup path with no interaction-binding layer. Correctly remains 📝 Draft; not run through SUBMIT/`.claim_verify`/`.verify_pass`, mirroring precedent watch-items 056/098/291. |
