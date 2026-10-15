# BUG-212: `held_buttons` grows without bound, bypassing the cap `active_pointers` already enforces

- **Severity:** Medium (unbounded internal memory growth under a sustained flood of distinct
  pointer ids — the exact scenario this crate's own `MAX_ACTIVE_POINTERS` cap and "DoS Protection"
  manual test scenario already exist to prevent, just for a sibling collection the cap never
  reached)
- **state:** Completed
- **Affects:** Every `browser_input` caller receiving `PointerButton` events under a sustained
  stream of distinct, never-released `pointer_id` values (e.g. a hostile or buggy embedding page
  synthesizing `PointerEvent`s with ever-incrementing `pointerId`, since `pointerId` carries no
  authenticity guarantee and is freely constructible by any script on the page).
- **Component:** `module/helper/browser_input` (`src/input.rs`)
- **repo_identity:** self
- **Filed:** 2026-08-17
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verification_date:** 2026-08-17
- **Related Bugs:** Found in the same session's `browser_input` audit as BUG-213/BUG-214 (same
  crate, same file, same audit pass). Builds directly on top of `Fix(BUG-130)`'s existing
  per-pointer `held_buttons : HashMap<i32,u32>` bitmask (introduced to fix cross-pointer/
  cross-button aggregation) without altering that fix's own logic. Sibling of BUG-213 — both touch
  the `PointerButton` `Press`/`Release` arms in the same audit pass, but distinct root causes (this
  bug: a missing cap; BUG-213: a missing alias-count) and distinct, independently-composable fixes.

## Symptom

```rust
// pre-fix -- input.rs, events_apply_to_state, EventType::PointerButton( .., Action::Press )
Action::Press =>
{
  *state.held_buttons.entry( *pointer_id ).or_insert( 0 ) |= bit;   // <- unconditional insert
  if !state.active_pointers.iter().any( | ( id, _ ) | *id == *pointer_id )
    && state.active_pointers.len() < MAX_ACTIVE_POINTERS             // <- active_pointers IS capped
  {
    state.active_pointers.push( ( *pointer_id, *pos ) );
  }
}
```

`held_buttons` inserted a new entry for every distinct `pointer_id` ever pressed, with no cap —
while `active_pointers`, tracking the same conceptual "which pointer ids are currently active" set,
was already correctly capped at `MAX_ACTIVE_POINTERS` (32).

## Impact

**Who is affected:** Any caller whose page can receive a sustained flood of `pointerdown` events
carrying distinct, never-repeating `pointer_id` values — most concretely a hostile or misbehaving
script on the same page synthesizing `PointerEvent`s directly (`pointerId` is a plain JS-settable
field with no browser-enforced uniqueness-to-a-real-contact guarantee).

**What breaks:** `held_buttons` (a private `HashMap<i32,u32>`) grows by one entry per distinct
flooded `pointer_id`, unboundedly — unbounded server-side-equivalent (in-page) memory growth is
exactly the class of issue `MAX_ACTIVE_POINTERS` and `active_pointers`' own cap already exist to
prevent, per this crate's own `tests/manual/readme.md` "Excessive Pointer Flood (DoS Protection)"
scenario. That scenario only ever checked the already-capped `active_pointers`, so the sibling
collection's missing cap went unnoticed.

**Additionally** (found while designing the fix, see Evidence Table E3): once more than
`MAX_ACTIVE_POINTERS` distinct pointer ids have pressed the *same* button, `mouse_buttons[button]`
can get stuck `true` forever after every `active_pointers`-visible pointer releases it — because
`held_buttons` (unbounded) kept a live entry for a pointer id `active_pointers` (capped) never
admitted and thus no caller could ever discover needs releasing.

**Magnitude:** 1 code path (`PointerButton` → `Action::Press`), 1 missing cap, 1 downstream
observable staleness consequence.

**Entity Scope:** None — a code-level defect.

## How Discovered

This session's audit of `browser_input`'s DOM-event-to-state translation layer
(`events_apply_to_state`/`Input::new`), cross-checking each event path's edge-case handling
(flood volume, enum-fallback aliasing, focus lifecycle) against what its existing tests actually
covered — comparing `held_buttons`' own insertion logic directly against `active_pointers`' already
established cap in the same match arm surfaced the asymmetry.

## Minimum Reproducible Example

```rust
// module/helper/browser_input/tests/mouse_button_state_test.rs -- pre-fix
let mut state = State::new();
let presses : Vec< Event > = ( 1_i32 ..= 33 ).map( | id | press( id, MouseButton::Main, id, id ) ).collect();
events_apply_to_state( &mut state, &presses );
let releases : Vec< Event > = ( 1_i32 ..= 32 ).map( | id | release( id, MouseButton::Main, id, id ) ).collect();
events_apply_to_state( &mut state, &releases );
// pre-fix: state.mouse_buttons[MouseButton::Main as usize] == true  -- stuck, pointer 33's
//          untracked-by-active_pointers press is still silently counted by held_buttons
// post-fix: state.mouse_buttons[MouseButton::Main as usize] == false
```

**Verify Command** (<=3 lines, standalone):
```bash
cd module/helper/browser_input && cargo nextest run --all-features -E 'test(held_buttons_respects_the_same_cap_as_active_pointers)'
```

## Hypothesis Table

| ID | Hypothesis | State | Summary | Evidence |
|----|-----------|--------|---------|----------|
| H1 | `held_buttons`' `Press`-arm insertion has no admission cap, unlike `active_pointers`' identical-purpose insertion in the same arm. | ✅ Root Cause | Confirmed by direct read: `held_buttons.entry(*pointer_id).or_insert(0)` runs unconditionally; `active_pointers.push(...)` is gated by `.len() < MAX_ACTIVE_POINTERS`. | E1 |
| H2 | The missing cap is purely a memory-growth concern with no observable behavioral effect. | ❌ Falsified | An untracked-by-`active_pointers`-but-tracked-by-`held_buttons` pointer id can keep `mouse_buttons[button]` stuck `true` after every admitted pointer releases that button — a real, externally observable divergence, not just unbounded memory. | E2, E3 |

## Evidence Table

| # | Location | What it shows | Hypothesis |
|---|----------|---------------|------------|
| E1 | `module/helper/browser_input/src/input.rs`, pre-fix `events_apply_to_state`'s `PointerButton`/`Action::Press` arm (direct read) | `held_buttons` insertion unconditional; `active_pointers` insertion gated by `MAX_ACTIVE_POINTERS`, in the very same match arm. | H1 ✅ |
| E2 | `module/helper/browser_input/tests/manual/readme.md`, "Excessive Pointer Flood (DoS Protection)" scenario (direct read) | Only asserts `active_pointers().length` caps at 32 -- never inspects `mouse_buttons` after a flood, so the staleness in E3 has no existing coverage of any kind. | H2 ❌ |
| E3 | Traced by hand against the pre-fix Press/Release arms for the exact MRE sequence above | Pointer 33's Press adds an untracked `held_buttons` entry; releasing pointers 1-32 clears their own entries but never pointer 33's, leaving `mouse_buttons[Main]` derived as `true` from that one orphaned entry indefinitely. | H2 ❌ |

## Root Cause

`held_buttons` and `active_pointers` both exist to track the same conceptual set --  "which pointer
ids are currently contributing held-button state" -- but only `active_pointers` enforced the
`MAX_ACTIVE_POINTERS` admission cap. `held_buttons`' own insertion had no equivalent guard, so once
a flood of distinct pointer ids exceeded the cap, the two collections silently disagreed on
membership: `active_pointers` correctly stopped admitting new ids, while `held_buttons` kept
growing and kept influencing the public `mouse_buttons` derived view for ids no caller could ever
learn about (since `active_pointers` is the only externally-visible enumeration of "current"
pointers).

## Why Not Caught

`tests/manual/readme.md`'s pre-existing "Excessive Pointer Flood (DoS Protection)" scenario only
checks `active_pointers().length` capping -- it never inspects `held_buttons` (a private field
with no direct external accessor) or the derived `mouse_buttons` view after a flood, so the
sibling collection's missing cap, and its downstream stuck-button consequence, had no test
surface at all.

## Fix Location

`module/helper/browser_input/src/input.rs`: `events_apply_to_state`'s `PointerButton`/
`Action::Press` arm now computes `already_tracked = state.held_buttons.contains_key(pointer_id)`
and gates both the `held_buttons` insertion and the `unknown_button_counts` increment (BUG-213,
same arm) behind `already_tracked || state.held_buttons.len() < MAX_ACTIVE_POINTERS` -- the
identical admission check `active_pointers`' own insertion already used, so a pointer id beyond the
cap is now consistently invisible to every collection, not just one of them.

## Prevention

New test `held_buttons_respects_the_same_cap_as_active_pointers`,
`module/helper/browser_input/tests/mouse_button_state_test.rs`: floods 33 distinct pointer ids
pressing the same button, confirms `active_pointers` still caps at 32 (pre-existing behavior,
unchanged), then releases the 32 admitted ids and asserts `mouse_buttons[Main]` correctly reads
`false` afterward -- which fails pre-fix (the 33rd id's orphaned `held_buttons` entry keeps it
`true`). `module/helper/browser_input/src/input.rs`'s `held_buttons` field doc comment now states
the shared-cap invariant explicitly, next to `MAX_ACTIVE_POINTERS`'s own doc comment, so a future
sibling collection added to this per-pointer bookkeeping group is more likely to be checked against
it.

## Pitfall

Two collections meant to track the same conceptual membership set do not automatically share an
admission policy just because they are updated in the same code path -- a cap added to one is not
inherited by a sibling collection unless the sibling's own insertion is explicitly gated by the
same check. Reviewing "does this event handler have a cap" per-collection, not per-arm, would have
caught this at `Fix(BUG-130)` time.

## Generalized Version

**Broken assumption:** "two collections updated together in the same match arm, for the same
conceptual purpose, share the same admission invariants."

**Confirmed general rule:** Every collection that participates in a shared "currently active set"
concept must have its own admission check independently verified against that concept's cap --
co-location in the same code path is not evidence of consistent enforcement.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-17 | filed | Found via this session's `browser_input` audit, comparing `held_buttons`' insertion directly against `active_pointers`' already-capped insertion in the same match arm. |
| 2026-08-17 | fixed | `held_buttons` (and the co-located BUG-213 `unknown_button_counts` increment) now gated behind the identical `already_tracked || len() < MAX_ACTIVE_POINTERS` check `active_pointers` already used. 1 new regression test added. |
| 2026-08-17 | verified | `cargo nextest run -p browser_input --all-features --no-fail-fast`: 24/24 passed, 0 skipped. `cargo clippy -p browser_input --all-targets --all-features -- -D warnings`: clean. |

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 6/6

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Completeness | — | 🟢 | All FI008 sections present. | — |
| D2 | MRE Validity & Reproducibility | — | 🟢 | MRE traced by hand against the exact pre-fix arm logic (E3) before being encoded as the actual regression test; confirmed the test fails pre-fix and passes post-fix by direct code-path tracing rather than assumption. | — |
| D3 | Cross-Reference Integrity | — | 🟢 | Correctly distinguished from BUG-213 (co-located in the same arm, but a missing-cap defect versus a missing-alias-count defect, independently composable fixes) and correctly attributed as building on `Fix(BUG-130)`'s existing per-pointer map without altering its logic. | — |
| D4 | Root Cause Quality | — | 🟢 | Backed by direct comparison of `held_buttons`' and `active_pointers`' insertion logic in the same arm -- not assumed from the "DoS Protection" test name alone. | — |
| D5 | Execution Scope | — | 🟢 | Fix confined to gating the existing insertion; adversarial pass specifically re-checked that the co-located `unknown_button_counts` increment (BUG-213) is nested inside the SAME outer `if`, so it inherits the identical cap rather than needing a second, separately-added guard. | — |
| D6 | Crate Scope Unity | — | 🟢 | Fix lives entirely in `browser_input`; no downstream crate changes needed. | — |

**Reproduced:** YES — pre-fix, `held_buttons_respects_the_same_cap_as_active_pointers` fails
(`mouse_buttons[Main]` reads `true` after all admitted pointers release); post-fix, it passes.
2026-08-17.

## Refs: src/

| File | Change |
|------|--------|
| `module/helper/browser_input/src/input.rs` | `events_apply_to_state`'s `PointerButton`/`Action::Press` arm: `held_buttons` insertion gated behind `already_tracked \|\| len() < MAX_ACTIVE_POINTERS`, matching `active_pointers`' own cap (full `Fix(BUG-212)` comment block). `held_buttons` field doc comment updated to state the shared-cap invariant. |

## Refs: tests/

| File | Change |
|------|--------|
| `module/helper/browser_input/tests/mouse_button_state_test.rs` | Added `held_buttons_respects_the_same_cap_as_active_pointers`; module doc comment extended to note BUG-212/BUG-213 coverage. |
