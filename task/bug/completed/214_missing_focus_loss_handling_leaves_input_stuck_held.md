# BUG-214: No `blur`/`visibilitychange` handling leaves held keys/buttons stuck when the page loses focus

- **Severity:** Medium (visible incorrect ongoing input state after a common real-world action --
  alt-tab, tab switch -- no crash or data loss, but does not self-correct until the exact same
  key/button happens to be pressed and released again)
- **state:** Completed
- **Affects:** Every `browser_input` caller with any key or pointer button held at the moment the
  OS moves focus away from the page (alt-tab, tab switch, minimizing the window) -- e.g. a game
  reading `keyboard_keys[Forward]` for continuous movement.
- **Component:** `module/helper/browser_input` (`src/input.rs`)
- **repo_identity:** self
- **Filed:** 2026-08-17
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verification_date:** 2026-08-17
- **Related Bugs:** Found in the same session's `browser_input` audit as BUG-212/BUG-213 (same
  crate, same file, same audit pass). Independent root cause and code region (a wholly new
  `EventType` variant, not a modification of the existing `PointerButton`/`KeyboardKey` arms those
  two bugs touch) -- no shared logic with either.

## Symptom

```rust
// pre-fix -- input.rs, Input::new
document.add_event_listener_with_callback( "keydown", ... )?;
document.add_event_listener_with_callback( "keyup", ... )?;
// ... pointerdown / pointerup / pointercancel / pointermove / wheel listeners ...
// no "blur" listener on window, no "visibilitychange" listener on document, anywhere
```

Nothing in `Input` ever listened for the page losing focus. A key or pointer button held at the
moment the OS delivered focus elsewhere (alt-tab, tab switch) never received its matching
`keyup`/`pointerup` -- the browser delivers that release event to whichever window or application
now has focus, not to this page -- so the corresponding `keyboard_keys`/`mouse_buttons` entry
stayed `true` indefinitely.

## Impact

**Who is affected:** Any caller whose page can lose OS-level focus while the user is actively
holding a key or pointer button -- effectively every interactive page, and especially pronounced
for anything reading held-input state continuously (e.g. WASD-style movement, drag-to-pan).

**What breaks:** `keyboard_keys[key]`/`mouse_buttons[button]` reports `true` (held) even though
the physical input was released the moment the user's attention (and the OS's focus) left the
page -- a "stuck key" bug, one of the most common and immediately noticeable classes of input-state
defect in interactive applications.

**Magnitude:** 1 missing listener pair (`blur` + `visibilitychange`), 1 new `EventType` variant, 1
new match arm.

**Entity Scope:** None — a code-level defect.

## How Discovered

This session's audit of `browser_input`'s DOM-event-to-state translation layer, checking the full
set of DOM lifecycle events `Input::new` registers listeners for against the set of ways
"currently held" state could become stale -- no listener existed for any focus-loss signal at all,
despite every held-state field being fed exclusively by matched press/release event pairs that
assume the release always arrives.

## Minimum Reproducible Example

```rust
// module/helper/browser_input/tests/focus_loss_test.rs
let mut state = State::new();
events_apply_to_state
(
  &mut state,
  &[
    ev( EventType::KeyboardKey( KeyboardKey::Space, Action::Press ) ),
    ev( EventType::PointerButton( 1, point( 5, 5 ), MouseButton::Main, Action::Press ) ),
  ]
);
// simulate the page losing focus mid-hold
events_apply_to_state( &mut state, &[ ev( EventType::FocusLost ) ] );
// pre-fix: EventType::FocusLost did not exist -- nothing could ever clear this state
// post-fix: state.keyboard_keys[Space] == false, state.mouse_buttons[Main] == false,
//           state.active_pointers.is_empty() == true
```

**Verify Command** (<=3 lines, standalone):
```bash
cd module/helper/browser_input && cargo nextest run --all-features -E 'test(focus_lost_clears_all_currently_held_state)'
```

## Hypothesis Table

| ID | Hypothesis | State | Summary | Evidence |
|----|-----------|--------|---------|----------|
| H1 | `Input::new` registers no listener for any focus-loss signal (`blur`, `visibilitychange`), so held state can never be externally reset once its matching release is misdelivered. | ✅ Root Cause | Confirmed by direct read: every `add_event_listener_with_callback` call in pre-fix `Input::new` is for `keydown`/`keyup`/`pointerdown`/`pointerup`/`pointercancel`/`pointermove`/`wheel` -- none for `blur` or `visibilitychange`. | E1 |
| H2 | `pointercancel` already covers the focus-loss case for pointers, making a dedicated fix redundant for the pointer half. | ❌ Falsified | `pointercancel` fires when the *browser* cancels a specific touch contact (e.g. an OS gesture takeover) -- it is not dispatched merely because the window lost focus while a mouse button or key is held; distinct trigger, distinct event. | E2 |

## Evidence Table

| # | Location | What it shows | Hypothesis |
|---|----------|---------------|------------|
| E1 | `module/helper/browser_input/src/input.rs`, pre-fix `Input::new` (direct read) | Full enumeration of registered listeners: `keydown`, `keyup`, `pointerdown`, `pointerup`, `pointercancel`, `pointermove`, `wheel` -- no `blur`, no `visibilitychange`. | H1 ✅ |
| E2 | MDN `pointercancel`/`blur`/`visibilitychange` event semantics, cross-checked against this crate's existing `Fix(BUG-130)` `PointerCancel` handling (direct read, unchanged) | `pointercancel` is scoped to a specific pointer contact being cancelled by the browser/OS (e.g. a system gesture), not the window's own focus state -- a held mouse button on a window that merely loses focus does not trigger it. | H2 ❌ |

## Root Cause

`Input`'s held-state fields (`keyboard_keys`, `mouse_buttons`, `active_pointers`, and their private
backing bookkeeping) are fed exclusively by matched press/release DOM event pairs, an implicit
assumption that every press eventually receives its matching release *through this page*. Focus
loss breaks that assumption directly: the OS reroutes the physical release event to whichever
window or application now has focus, so this page never observes it, and nothing existed to detect
"we can no longer trust our held-state bookkeeping" and reset it.

## Why Not Caught

No prior test exercised a focus-loss scenario at all -- every existing test sent only matched
press/release (or `pointercancel`) pairs, which by construction is exactly the case that never
triggers this bug. There was also no `EventType` variant capable of representing "focus was lost"
prior to this fix, so the scenario could not even be constructed in a unit test.

## Fix Location

`module/helper/browser_input/src/input.rs`:
- New `EventType::FocusLost` unit variant.
- `Input` gains a `focus_lost_closure : Closure<dyn Fn(web_sys::Event)>` field, constructed in
  `Input::new` from a shared callback that queues one `EventType::FocusLost` event, and registered
  against both `window`'s `blur` event and `document`'s `visibilitychange` event (the latter fires
  for both directions -- hidden and visible again -- but a `FocusLost` reset on the harmless
  "became visible" direction is a no-op against already-empty held state).
- `events_apply_to_state`'s new `FocusLost` arm resets `keyboard_keys`, `mouse_buttons`,
  `active_pointers`, and the private `held_buttons`/`unknown_button_counts`/
  `unidentified_key_hold_count` bookkeeping to empty/zero. `pointer_position` and `scroll` are
  deliberately left untouched (last-known-value/accumulator state, still meaningful after focus
  returns).
- `Drop for Input` gains matching `remove_event_listener_with_callback` calls for both listeners.

## Prevention

3 new tests, new file `module/helper/browser_input/tests/focus_loss_test.rs`:
`focus_lost_clears_all_currently_held_state` (the BUG-214 reproducer),
`focus_lost_does_not_reset_last_known_position_or_accumulated_scroll` (confirms the deliberate
non-reset of accumulator fields), `focus_lost_with_nothing_held_is_a_harmless_no_op` (confirms no
panic/underflow when nothing was held). These pin the *state-reset* half of the fix, which is
directly constructible via `EventType::FocusLost`. The *DOM-listener-wiring* half -- that a real
browser `blur`/`visibilitychange` event actually gets queued as `EventType::FocusLost` -- has no
live browser context in `cargo test`; documented instead as a new manual scenario in
`tests/manual/readme.md` (see Refs), consistent with this crate's established handling of
DOM-only behavior (e.g. BUG-210's WebGL upload path).

## Pitfall

Any global "currently held" state fed exclusively by matched press/release event pairs carries an
implicit, easy-to-miss assumption that the release always arrives through the same channel as the
press -- focus loss is one of several platform-level events (window minimize, tab switch, OS-level
alt-tab) that can silently break that assumption without the application ever being notified via
the input events it already listens for.

## Generalized Version

**Broken assumption:** "every press event will eventually be followed by its own matching release
event, delivered through the same page."

**Confirmed general rule:** Any input-state tracker relying on matched press/release pairs needs an
explicit external reset path for the platform-level conditions (focus loss being the most common)
under which the OS can misdeliver or withhold the matching release entirely -- the tracker cannot
derive this from the press/release events themselves, since by definition the release never comes.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-17 | filed | Found via this session's `browser_input` audit, enumerating every DOM listener `Input::new` registers and checking for focus-loss coverage. |
| 2026-08-17 | fixed | Added `EventType::FocusLost`, `blur`/`visibilitychange` listeners in `Input::new`, a full-reset match arm in `events_apply_to_state`, and matching listener removal in `Drop`. 3 new regression tests added. |
| 2026-08-17 | verified | `cargo nextest run -p browser_input --all-features --no-fail-fast`: 24/24 passed, 0 skipped. `cargo clippy -p browser_input --all-targets --all-features -- -D warnings`: clean. |

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 6/6

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Completeness | 🟠 | 🟢 | Confirming pass initially wrote the `FocusLost` match arm's own comment claiming `last_pointer_type` is a `State` field left deliberately untouched. Adversarial pass re-checked `State`'s actual field list and found `last_pointer_type` lives on `Input`, not `State` -- the comment was factually wrong about which struct it described. | Corrected the comment to state `last_pointer_type` is untouched because `events_apply_to_state` has no access to it at all (it lives on `Input`), not because it was considered and spared. |
| D2 | MRE Validity & Reproducibility | — | 🟢 | MRE directly constructs `EventType::FocusLost` (the only externally-constructible way to invoke the fix's state-reset half) and asserts against public `State` fields only. | — |
| D3 | Cross-Reference Integrity | — | 🟢 | Correctly scoped as independent of BUG-212/BUG-213 (new code region, no shared logic); correctly distinguished `pointercancel` (H2) as a different trigger rather than redundant prior coverage. | — |
| D4 | Root Cause Quality | — | 🟢 | Backed by direct enumeration of every registered listener in pre-fix `Input::new` (E1) and direct comparison against `pointercancel`'s actual trigger semantics (E2), not assumed. | — |
| D5 | Execution Scope | — | 🟢 | Fix confined to the new `FocusLost` variant/arm/listeners; explicitly verified `pointer_position`/`scroll` are left untouched by both a dedicated test and by design (accumulator semantics), not silently swept into the reset. | — |
| D6 | Crate Scope Unity | — | 🟢 | Fix lives entirely in `browser_input`; no downstream crate changes needed. | — |

**Reproduced:** YES — pre-fix, `EventType::FocusLost` did not exist at all (the reproducer could
not even be constructed); post-fix, `focus_lost_clears_all_currently_held_state` passes. 2026-08-17.

## Refs: src/

| File | Change |
|------|--------|
| `module/helper/browser_input/src/input.rs` | New `EventType::FocusLost` variant; `Input` gains `focus_lost_closure` field; `Input::new` constructs the closure and registers `blur`(window)/`visibilitychange`(document) listeners; `events_apply_to_state` gains a `FocusLost` full-reset match arm; `Drop for Input` removes both new listeners (full `Fix(BUG-214)` comment blocks). |

## Refs: tests/

| File | Change |
|------|--------|
| `module/helper/browser_input/tests/focus_loss_test.rs` | New file: added `focus_lost_clears_all_currently_held_state`, `focus_lost_does_not_reset_last_known_position_or_accumulated_scroll`, `focus_lost_with_nothing_held_is_a_harmless_no_op`. |

## Refs: docs/

| File | Change |
|------|--------|
| `module/helper/browser_input/tests/manual/readme.md` | New manual scenario documenting `blur`/`visibilitychange` DOM-listener wiring, which has no live-browser equivalent testable via `cargo test`. |
