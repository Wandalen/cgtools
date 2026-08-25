# BUG-130: pointer/button state tracking assumes one button per pointer at a time

- **Severity:** Medium (silent, reachable state-corruption across 3 related symptoms — not a
  panic, but wrong `is_button_down`/`active_pointers` results under ordinary multi-touch or
  multi-button-mouse usage)
- **state:** Completed
- **Affects:** Any caller of `Input::is_button_down`, `Input::active_pointers`, or the underlying
  `pub fn events_apply_to_state` whenever more than one pointer is simultaneously active, or one
  pointer holds more than one button at once
- **Component:** `module/helper/browser_input` (`src/input.rs::events_apply_to_state`,
  `PointerButton`/`PointerCancel` arms)
- **repo_identity:** self
- **Filed:** 2026-08-15
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/verified/
- **verification_date:** 2026-08-16
- **Related Bugs:** None — first bug filed for this crate under task #64's targeted
  `browser_input` review; independent of BUG-127/128/129 (different crate)

## Symptom

```rust
// Two simultaneous touches both report button = Main (per the Pointer Events spec):
events_apply_to_state( &mut state, &[ press( 1, Main, .. ), press( 2, Main, .. ) ] );
events_apply_to_state( &mut state, &[ release( 1, Main, .. ) ] );

// Wrong (pre-fix): pointer 2 is still touching, but:
state.mouse_buttons[ Main as usize ] == false   // touch 2 is invisible to is_button_down

// Correct (post-fix):
state.mouse_buttons[ Main as usize ] == true    // touch 2 still held, correctly reflected
```

## Impact

**Who is affected:** Any caller reading `Input::is_button_down` or `Input::active_pointers` on a
touch-capable device with 2+ simultaneous contacts, or on a multi-button mouse with more than one
button held at once — both are ordinary, expected usage this crate's own API and doc comments
explicitly anticipate (`active_pointers`'s doc: "Useful for multi-touch gestures such as
pinch-to-zoom or two-finger pan"; `MouseButton` enumerates 5 real buttons, not just one).

**What breaks (3 related symptoms, one shared root cause):**
1. `mouse_buttons[button]` was a flat, last-writer-wins toggle keyed only by `button` — releasing
   *any* pointer holding a button cleared it, even while a *different* pointer still held the
   identical button (touch's `button` is always `Main` per spec, so any 2-finger touch triggers
   this).
2. `active_pointers` evicted a pointer's entire entry on *any* button release for that
   `pointer_id` — wrong for one physical mouse holding 2 buttons at once, since both buttons'
   press/release events share the mouse's one stable `pointer_id`; releasing the first evicted the
   pointer even though the second button was still held.
3. `PointerCancel` only cleared `mouse_buttons` when `active_pointers` became fully empty — if a
   *different* pointer was still active, the cancelled pointer's own button state stayed stuck
   `true` forever, since the coarse `is_empty()` guard could never fire again for it.

**Magnitude:** Not a crash — silent, incorrect boolean/collection state read by real application
logic (`hexagonal_map`'s tile-painting reads both `Main` and `Secondary` simultaneously from the
same `Input`; `touch_input_test` is built specifically around 2-finger pinch gestures via
`active_pointers()`). Wrong values here mean dropped/stuck button state or a vanished multi-touch
contact, silently, with no error signal.

**Entity Scope:** None — a code-level defect, not an operational-entity concern.

## How Discovered

Task #64, a targeted code review of `browser_input` dispatched under the standing bug-hunt
mandate. The reviewing agent flagged that `mouse_buttons`/`active_pointers` updates are keyed
without regard to whether multiple pointers can share a button or one pointer can hold multiple
buttons. Independently re-verified before filing by direct source reads and reachability tracing:

```bash
$ sed -n '564,612p' module/helper/browser_input/src/input.rs
# (pre-fix) confirms mouse_buttons[*mouse_button as usize] = *action == Action::Press --
# a flat overwrite with no pointer_id in the key at all

$ grep -n "Pinch\|active_pointers" examples/minwebgl/touch_input_test/src/main.rs
# confirms touch_input_test's own doc comment: "Pinch with two fingers to zoom in/out",
# and that it reads active_pointers() directly (line 78)

$ grep -n "is_button_down" examples/minwebgl/hexagonal_map/src/main.rs
# confirms hexagonal_map reads BOTH MouseButton::Main and MouseButton::Secondary from the
# same Input on the same frame (lines 478-479), the multi-button-one-pointer scenario
```

## Minimum Reproducible Example

```bash
cd module/helper/browser_input && cargo test --test mouse_button_state_test 2>&1 | tail -20
```

**Expected** (post-fix — all 3 pass):
```
releasing_one_pointer_does_not_clear_a_button_another_pointer_still_holds ... ok
releasing_one_button_does_not_evict_a_pointer_still_holding_another ... ok
cancel_only_clears_the_cancelled_pointers_own_buttons ... ok
```

**Actual** (pre-fix — confirmed by temporarily reverting `events_apply_to_state`'s
`PointerButton`/`PointerCancel` arms to their exact pre-fix bodies in the real crate source, then
restoring the fix immediately after capturing the failure — not a separate scratch crate, since
this function has zero external dependencies and is already the crate's own direct unit-test
target):
```
thread 'releasing_one_button_does_not_evict_a_pointer_still_holding_another' panicked at
module/helper/browser_input/tests/mouse_button_state_test.rs:120:3:
assertion `left == right` failed: Secondary is still held -- releasing Main must not evict
pointer 1 (buggy code would have removed this entry)
  left: []
 right: [(1, Vector([5, 5]))]
3 failed: cancel_only_clears_the_cancelled_pointers_own_buttons,
releasing_one_pointer_does_not_clear_a_button_another_pointer_still_holds,
releasing_one_button_does_not_evict_a_pointer_still_holding_another
```

**Verify Command** (≤3 lines, standalone):
```bash
cd module/helper/browser_input && cargo test --test mouse_button_state_test
# 3 passed = fixed; 3 failed (assertion mismatches as above) = bug present
```

**Known MRE limitation (check 205):** none — `events_apply_to_state` and `State` have no
wasm/browser/file-I/O dependency at all, so this MRE runs as an ordinary native `cargo test`
against the real crate directly, no scratch crate or isolated reproduction needed.

## Hypothesis Table

| ID | Hypothesis | State | Summary | Evidence |
|----|-----------|--------|---------|----------|
| H1 | `mouse_buttons[button]` is a flat overwrite keyed only by button, so two pointers sharing a button value (e.g. two touches, always `Main`) clobber each other's state on release. | ✅ Root Cause (symptom 1) | Reverting to the pre-fix single-line overwrite reproduces exactly this: releasing pointer 1 (of 2, both holding `Main`) sets `mouse_buttons[Main]` false despite pointer 2 still holding it. | E1, E2 |
| H2 | `active_pointers.retain(...)` on any release assumes one button per pointer's whole lifecycle, breaking a multi-button mouse sharing one `pointer_id` across buttons. | ✅ Root Cause (symptom 2) | Reverted code evicts pointer 1's entry from `active_pointers` on releasing just `Main`, even though `Secondary` (same pointer_id) is still held — confirmed via the reverted test run's exact assertion failure (`left: []`, `right: [(1, ...)]`). | E1, E2 |
| H3 | `PointerCancel`'s `is_empty()`-gated `mouse_buttons.fill(false)` only clears state when it happens to be the last active pointer, not scoped to the cancelled pointer's own buttons. | ✅ Root Cause (symptom 3) | Reverted code leaves `mouse_buttons[Main]` stuck `true` after cancelling the only pointer that ever held `Main`, because a second, unrelated pointer (holding `Secondary`) keeps `active_pointers` non-empty. | E1, E2 |
| H4 | These are 3 independent, unrelated defects that happen to be in the same function. | ❌ Falsified | All 3 trace to the identical missing capability: no per-pointer record of which buttons it holds. The fix (`held_buttons : HashMap<i32, u32>`) resolves all 3 with one shared mechanism — a coherent single root cause, not 3 coincidental bugs. | E3 |

## Evidence Table

| # | Location | What it shows | Hypothesis |
|---|----------|---------------|------------|
| E1 | Pre-fix code, reverted in place and immediately restored | `mouse_buttons[*mouse_button as usize] = *action == Action::Press` (flat overwrite) and `active_pointers.retain(\|(id,_)\| *id != *pointer_id)` (unconditional on any release) and `if state.active_pointers.is_empty() { state.mouse_buttons.fill(false) }` (coarse cancel heuristic) — all 3 confirmed present verbatim before this fix. | H1 ✅, H2 ✅, H3 ✅ |
| E2 | `cargo test --test mouse_button_state_test` run against the reverted code | 3/3 new tests fail with the exact predicted wrong values (`mouse_buttons[Main]` false when it should be true; `active_pointers` empty when it should retain an entry; `mouse_buttons[Main]` stuck true when it should be false) — all captured in the pre-fix test log before restoring the fix. | H1 ✅, H2 ✅, H3 ✅ |
| E3 | `src/input.rs`'s fix: `held_buttons : std::collections::HashMap<i32, u32>` bitmask per pointer id | One shared data structure and derivation rule (`mouse_buttons[b]` = "does any pointer's mask have bit b set"; `active_pointers` evicts only when a pointer's mask reaches zero; `PointerCancel` removes only its own bits and re-derives only the affected buttons) resolves all 3 symptoms without 3 separate patches. | H4 ❌ |
| E4 | `examples/minwebgl/touch_input_test/src/main.rs` (doc comment line 4: "Pinch with two fingers to zoom in/out"; line 78: `input.active_pointers().to_vec()`) | Real example built specifically around simultaneous 2-pointer touch, reading `active_pointers()` directly — the exact scenario symptom 1/2 corrupt. | Reachability |
| E5 | `examples/minwebgl/hexagonal_map/src/main.rs:478-479` | Real example reads `is_button_down(Main)` and `is_button_down(Secondary)` from the same `Input` on the same frame — the exact scenario symptom 2/3 corrupt if both are ever held at once. | Reachability |

## Root Cause

```
events_apply_to_state( state, events )
  PointerButton(pointer_id, pos, mouse_button, action):
    mouse_buttons[mouse_button] = (action == Press)     <- keyed only by button, not pointer_id
    Press:   active_pointers.push_if_absent(pointer_id, pos)
    Release: active_pointers.retain(id != pointer_id)   <- evicts on ANY release, any button
  PointerCancel(pointer_id):
    active_pointers.retain(id != pointer_id)
    if active_pointers.is_empty(): mouse_buttons.fill(false)   <- all-or-nothing heuristic
```

All three lines assume a false invariant: that exactly one button is ever "in play" for exactly
one pointer at a time. That invariant holds for the single most common case (one mouse, one
button, sequential clicks) but is false the moment either axis of concurrency the crate's own API
explicitly supports actually occurs — multiple simultaneous pointers (multi-touch), or multiple
simultaneous buttons on one pointer (a multi-button mouse).

## Why Not Caught

No existing test exercised either concurrency axis: `active_pointers_test.rs`'s multi-pointer
cases always used a single button (`Main`) per distinct pointer id, and its single-pointer cases
never used more than one button. The two axes — "which pointer" and "which button" — were each
tested independently but never in combination, which is exactly where the bug lives.

## Fix Location

`module/helper/browser_input/src/input.rs`. Added a private `held_buttons : HashMap<i32, u32>`
field to `State` (a per-pointer button-bitmask, not exposed publicly — `mouse_buttons` and
`active_pointers` remain the public derived view with unchanged public types) and rewrote the
`PointerButton`/`PointerCancel` arms of `events_apply_to_state` to derive both public fields from
it:

```rust
// before (PointerButton)
state.mouse_buttons[ *mouse_button as usize ] = *action == Action::Press;
match action {
  Action::Press => { /* push if absent */ }
  Action::Release => { state.active_pointers.retain( |(id,_)| *id != *pointer_id ); }
}

// after (PointerButton)
let bit = 1u32 << ( *mouse_button as u32 );
match action {
  Action::Press => {
    *state.held_buttons.entry( *pointer_id ).or_insert( 0 ) |= bit;
    /* push if absent, unchanged */
  }
  Action::Release => {
    if let Some( mask ) = state.held_buttons.get_mut( pointer_id ) {
      *mask &= !bit;
      if *mask == 0 {
        state.held_buttons.remove( pointer_id );
        state.active_pointers.retain( |(id,_)| *id != *pointer_id );
      }
    } else {
      state.active_pointers.retain( |(id,_)| *id != *pointer_id );
    }
  }
}
state.mouse_buttons[ *mouse_button as usize ] =
  state.held_buttons.values().any( |mask| mask & bit != 0 );

// before (PointerCancel)
state.active_pointers.retain( |(id,_)| *id != *pointer_id );
if state.active_pointers.is_empty() { state.mouse_buttons.fill( false ); }

// after (PointerCancel)
state.active_pointers.retain( |(id,_)| *id != *pointer_id );
if let Some( mask ) = state.held_buttons.remove( pointer_id ) {
  for i in 0 .. MouseButton::COUNT {
    if mask & ( 1u32 << i ) != 0 {
      state.mouse_buttons[ i ] = state.held_buttons.values().any( |m| m & ( 1u32 << i ) != 0 );
    }
  }
}
```

`PointerMove`, `Wheel`, and `KeyboardKey` arms are untouched — only pointer-button bookkeeping was
affected. `State::new()`/`Default` initialize the new field to an empty map.

## Prevention

Added `tests/mouse_button_state_test.rs` with 3 tests, each targeting one of the 3 symptoms:
`releasing_one_pointer_does_not_clear_a_button_another_pointer_still_holds`,
`releasing_one_button_does_not_evict_a_pointer_still_holding_another`, and
`cancel_only_clears_the_cancelled_pointers_own_buttons`.

**Pitfall:** global input state that is "set" per event instead of "derived from the union of all
current sources" silently breaks the instant two sources can overlap — verify against the
*simultaneous* case explicitly (two pointers live at once, or one pointer holding two buttons at
once), not just sequential press/release pairs, which is all the pre-existing test suite covered.

## Generalized Version

**Broken assumption:** "there is at most one active button-holder at a time for `mouse_buttons`,
and at most one active button per pointer lifecycle for `active_pointers`" — silently false for
any input-tracking code whose own domain (pointer/touch events) explicitly supports concurrency
the tracking logic itself does not account for.

**Confirmed general rule:** when state is aggregated from multiple independent sources (multiple
pointers, multiple input devices, multiple concurrent requests), a plain overwrite-on-event or
evict-on-any-terminal-event pattern is only correct if the domain guarantees exactly one source is
ever active — verify that guarantee explicitly (here: it does not; touch and multi-button mice are
first-class, documented use cases of this very crate) before using it, or track state per-source
and derive the aggregate.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-15 | filed | Discovered via task #64's targeted code review of `browser_input`; confirmed via in-place revert-and-restore of the real crate source (no scratch crate needed — the function is a pure, dependency-free unit). |
| 2026-08-15 | fixed | Added a per-pointer `held_buttons` bitmask; `mouse_buttons`/`active_pointers` are now derived from it in all 3 affected arms. |
| 2026-08-15 | verified | Added 3 tests to `tests/mouse_button_state_test.rs`; confirmed all 3 fail against the reverted pre-fix code and pass against the fix; scoped test run (`cargo nextest run --all-features` via `longrun`) passed 16/16 alongside the pre-existing suite; `cargo clippy --all-targets --all-features -- -D warnings` clean. |
| 2026-08-16 | completed | Acceptance verification by a distinct session (filer/fixer/self-verifier 2026-08-15, this verifier 2026-08-16). Independently re-read `events_apply_to_state`'s `PointerButton`/`PointerCancel` arms (confirmed the `held_buttons : HashMap<i32,u32>` bitmask genuinely present, both arms deriving `mouse_buttons`/`active_pointers` from it as described, 3-field comments intact in both) and `releasing_one_pointer_does_not_clear_a_button_another_pointer_still_holds` (non-tautological: two real pointers press/release the same button, asserts `mouse_buttons[Main]` stays true after only one releases). Fresh `cargo nextest run -p browser_input --all-features` via `longrun`: 16/16 passed. `cargo clippy -p browser_input --all-features --all-targets -- -D warnings`: clean. `**Related Bugs:** None` confirmed accurate. MAAV Tier 2 Dual-Role Self-Check (`governance/maav.rulebook.md`). State → Completed. |

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 8/8

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Completeness | 🟢 | 🟢 | All 12 FI008 sections + `Refs: src/`/`Refs: tests/` present. | — |
| D2 | MRE Validity & Reproducibility | 🟡 | 🟢 | Confirming pass initially treated the passing post-fix test suite as sufficient; adversarial pass required actually observing the tests FAIL against the exact pre-fix code, not just assuming they would — closed by reverting `events_apply_to_state`'s 2 arms in place, capturing the 3 real failures with exact assertion output, then restoring the fix and re-confirming 16/16 pass plus a clean `cargo clippy -D warnings`. | — |
| D3 | Cross-Reference Integrity | 🟢 | 🟢 | Adversarial pass confirmed this is the first bug filed for `browser_input` this session — no `**Related Bugs:**` cross-references needed. | — |
| D4 | Root Cause Quality | 🟢 | 🟢 | Adversarial pass challenged whether the 3 symptoms were genuinely one root cause or 3 coincidental bugs bundled together (H4) — falsified by confirming one shared fix (`held_buttons`) resolves all 3 without symptom-specific patches. | — |
| D5 | Execution Scope | 🟢 | 🟢 | Adversarial pass checked `PointerMove`/`Wheel`/`KeyboardKey` arms for the same button/pointer-conflation pattern — none apply (no button/pointer-lifecycle state involved in those arms). | — |
| D6 | Crate Scope Unity | 🟢 | 🟢 | Only `browser_input`'s own `src/`/`tests/` and this bug-tracking file touched. | — |
| D7 | Crate Locality | 🟢 | 🟢 | Fix is local to `events_apply_to_state` and one new private `State` field; `Input`'s public methods (`is_button_down`, `active_pointers`) and all real callers are unmodified — same public types, corrected derivation. | — |
| D8 | Crate Single Responsibility | 🟢 | 🟢 | `held_buttons` is a private bookkeeping field, not a new public responsibility; `mouse_buttons`/`active_pointers` remain the crate's sole public state-query surface, now correctly derived. | — |

**Reproduced:** YES — reverting `events_apply_to_state`'s `PointerButton`/`PointerCancel` arms to
their exact pre-fix bodies in the real crate source and running `cargo test --test
mouse_button_state_test` produced 3/3 failures with the exact predicted wrong values; restoring
the fix immediately after returned the suite to 16/16 passing plus a clean `cargo clippy -D
warnings`, 2026-08-15

## Refs: src/

| File | Change |
|------|--------|
| `module/helper/browser_input/src/input.rs` | `State`: added private `held_buttons : HashMap<i32, u32>` field. `events_apply_to_state`: `PointerButton` and `PointerCancel` arms rewritten to derive `mouse_buttons`/`active_pointers` from `held_buttons` instead of flat overwrite/unconditional-evict/is_empty-heuristic logic. `Fix(BUG-130)`/`Root cause`/`Pitfall` comments added to both arms. |

## Refs: tests/

| File | Change |
|------|--------|
| `module/helper/browser_input/tests/mouse_button_state_test.rs` | New file: 3 tests (`bug_reproducer(BUG-130)`, 5-section doc comments each) — `releasing_one_pointer_does_not_clear_a_button_another_pointer_still_holds`, `releasing_one_button_does_not_evict_a_pointer_still_holding_another`, `cancel_only_clears_the_cancelled_pointers_own_buttons`. |
