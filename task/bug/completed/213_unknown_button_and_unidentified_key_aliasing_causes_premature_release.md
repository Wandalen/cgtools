# BUG-213: `MouseButton::Unknown`/`KeyboardKey::Unidentified` fallback aliasing causes premature release when two distinct unmapped inputs are held at once

- **Severity:** Medium (visible incorrect held-state for two simultaneously-held unmapped inputs;
  no crash or data loss, self-corrects once all aliased inputs are eventually released)
- **state:** Completed
- **Affects:** Every `browser_input` caller that can receive two DIFFERENT DOM `button` values
  outside `0..=4` (mouse) or two different unrecognized `code` strings (keyboard) held
  simultaneously -- e.g. an exotic input device with extra buttons, or a keyboard layout/browser
  combination producing `code` strings this crate's `KeyboardKey::from` does not yet map.
- **Component:** `module/helper/browser_input` (`src/input.rs`)
- **repo_identity:** self
- **Filed:** 2026-08-17
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verification_date:** 2026-08-17
- **Related Bugs:** Found in the same session's `browser_input` audit as BUG-212/BUG-214. The mouse
  half is co-located with, and its fix nested inside, BUG-212's `PointerButton`/`Press` cap-gate in
  the same arm -- distinct root causes (BUG-212: missing cap; this bug: missing alias-count), fixed
  independently but composably. One ID covers both the mouse and keyboard halves: identical root
  cause (a many-to-one enum fallback aliasing distinct real inputs) applied via the identical fix
  shape (a hold-count in place of a hold-bit) to two structurally analogous but otherwise unrelated
  event types, per this session's established one-ID-per-root-cause convention.

## Symptom

```rust
// pre-fix -- input.rs, events_apply_to_state, EventType::KeyboardKey( keyboard_key, action )
state.keyboard_keys[ *keyboard_key as usize ] = *action == Action::Press;   // flat overwrite
```

```rust
// pre-fix -- input.rs, events_apply_to_state, EventType::PointerButton( .., Action::Release )
if let Some( mask ) = state.held_buttons.get_mut( pointer_id )
{
  *mask &= !bit;   // clears the Unknown bit on ANY Unknown release, regardless of how many
                    // distinct real buttons are currently aliased to it
  ...
}
```

Both `MouseButton::Unknown` (catches every DOM `button` value outside `0..=4`) and
`KeyboardKey::Unidentified` (catches every unrecognized `code` string) are many-to-one fallback
variants. A flat bit/bool keyed by the collapsed discriminant cannot distinguish "one aliased input
held" from "two DIFFERENT aliased inputs held" -- releasing either one cleared the shared slot,
falsely dropping the other's still-held state.

## Impact

**Who is affected:** Any caller whose input hardware or browser/OS combination produces two
simultaneously-held inputs that both fall back to the same collapsed variant -- an input device
with buttons beyond the 5 named `MouseButton` variants, or a keyboard layout/browser pairing
emitting a `code` string `KeyboardKey::from` does not recognize.

**What breaks:** `mouse_buttons[Unknown]`/`keyboard_keys[Unidentified]` can read `false` while one
of the two aliased inputs is still physically held -- the opposite of a stuck-held bug (BUG-214):
here the state is prematurely cleared, understating what is actually held.

**Magnitude:** 2 code paths (`KeyboardKey` arm; `PointerButton` `Press`/`Release` arms), 1 shared
root cause, 1 shared fix shape (hold-count instead of hold-bit).

**Entity Scope:** None — a code-level defect.

## How Discovered

This session's audit of `browser_input`'s DOM-event-to-state translation layer, specifically
checking every enum with a fallback/catch-all variant (`MouseButton::Unknown`,
`KeyboardKey::Unidentified`) against how `events_apply_to_state` tracks its held/released state --
both were found to use a flat bit/bool with no accounting for the variant's many-to-one nature.

## Minimum Reproducible Example

```rust
// module/helper/browser_input/tests/mouse_button_state_test.rs -- pre-fix
let mut state = State::new();
events_apply_to_state( &mut state, &[ press( 1, MouseButton::Unknown, 0, 0 ), press( 1, MouseButton::Unknown, 0, 0 ) ] );
events_apply_to_state( &mut state, &[ release( 1, MouseButton::Unknown, 0, 0 ) ] );
// pre-fix: state.mouse_buttons[MouseButton::Unknown as usize] == false -- WRONG, one is still held
// post-fix: == true

// module/helper/browser_input/tests/keyboard_key_state_test.rs -- pre-fix, identical shape
let mut state = State::new();
events_apply_to_state( &mut state, &[ press( KeyboardKey::Unidentified ), press( KeyboardKey::Unidentified ) ] );
events_apply_to_state( &mut state, &[ release( KeyboardKey::Unidentified ) ] );
// pre-fix: state.keyboard_keys[KeyboardKey::Unidentified as usize] == false -- WRONG
// post-fix: == true
```

**Verify Command** (<=3 lines, standalone):
```bash
cd module/helper/browser_input && cargo nextest run --all-features -E 'test(releasing_one_aliased_unknown_button_does_not_clear_another_still_held) + test(releasing_one_aliased_unidentified_key_does_not_clear_another_still_held)'
```

## Hypothesis Table

| ID | Hypothesis | State | Summary | Evidence |
|----|-----------|--------|---------|----------|
| H1 | `MouseButton::Unknown`/`KeyboardKey::Unidentified` are many-to-one fallback variants, and both consumers track their held state as a flat bit/bool with no per-alias accounting. | ✅ Root Cause | Confirmed by direct read of `mouse.rs::from_button` (any value outside 0-4 → `Unknown`) and `keyboard.rs` (any unrecognized `code` → `Unidentified`), plus `events_apply_to_state`'s flat-bit treatment of both. | E1, E2, E3 |
| H2 | A naive hold-COUNT fix (increment on press, decrement on release) is sufficient with no further guard needed. | ❌ Falsified — refined | OS-level key auto-repeat re-fires `keydown` for an already-held key with no matching `keyup`; a naive counter would over-increment on every repeat, requiring that many releases to actually clear (a worse bug: a key that never un-sticks). Requires filtering `event.repeat()` at the DOM-listener level, upstream of the counting logic, plus `saturating_sub` for a spurious unmatched release. | E4 |

## Evidence Table

| # | Location | What it shows | Hypothesis |
|---|----------|---------------|------------|
| E1 | `module/helper/browser_input/src/mouse.rs`, `MouseButton::from_button` (direct read, unchanged) | Any `i16` outside `0..=4` maps to `Unknown` -- confirmed many-to-one. | H1 ✅ |
| E2 | `module/helper/browser_input/src/keyboard.rs`, lines ~785-786/797 (direct read, unchanged) | Any unrecognized `code` string maps to `Unidentified` via `_ => Ok(KeyboardKey::Unidentified)` / `.unwrap_or(KeyboardKey::Unidentified)` -- confirmed many-to-one. | H1 ✅ |
| E3 | `module/helper/browser_input/src/input.rs`, pre-fix `events_apply_to_state` (direct read) | `keyboard_keys[*keyboard_key as usize] = *action == Action::Press` (flat overwrite); `held_buttons`' bit-clear on any `Release` regardless of alias count. Neither distinguishes which of potentially several real inputs the event actually came from. | H1 ✅ |
| E4 | MDN `KeyboardEvent.repeat` semantics, cross-checked against this crate's `keyboard_callback` (direct read, pre-fix: no `.repeat()` check anywhere) | Confirmed OS auto-repeat re-fires `keydown` with `repeat() == true` and no intervening `keyup` -- a naive counter fix would be silently corrupted by this without an explicit filter. | H2 ❌ (refined) |

## Root Cause

Both `MouseButton::Unknown` and `KeyboardKey::Unidentified` are deliberate many-to-one fallback
variants (by design -- neither crate needs to enumerate every possible DOM button/key exhaustively).
`events_apply_to_state` tracked each one's held state as a single flat bit/bool, an encoding that
is only correct under the implicit assumption that at most one real input ever maps to that variant
at a time -- an assumption the fallback's own many-to-one nature directly contradicts.

## Why Not Caught

No existing test pressed either fallback variant twice under conditions that would alias two
distinct real inputs together -- every existing multi-input test used individually-addressable
variants (`Main`/`Secondary`, specific named keys), which by construction cannot alias. This crate
had no dedicated keyboard-state test file at all prior to this bug.

## Fix Location

`module/helper/browser_input/src/input.rs`:
- **Mouse half:** `State` gains `unknown_button_counts : HashMap<i32,u32>` (per-pointer). The
  `PointerButton`/`Press` arm increments it (nested inside BUG-212's cap-gate) when
  `*mouse_button == MouseButton::Unknown`; the `Release` arm checks and decrements it first, only
  falling through to the normal bit-clear once the count reaches zero. `PointerCancel` removes the
  pointer's entry to avoid a leak.
- **Keyboard half:** `State` gains `unidentified_key_hold_count : u32` (global -- keyboard events
  carry no pointer id). The `KeyboardKey` arm branches on `*keyboard_key == KeyboardKey::Unidentified`,
  incrementing/`saturating_sub`-decrementing the count and deriving `keyboard_keys[Unidentified]`
  from `count > 0`, instead of the flat overwrite used for every other (non-aliasing) key.
- **Repeat guard (prerequisite for the keyboard half):** `Input::new`'s `keyboard_callback` now
  returns immediately when `event.repeat()` is `true`, before constructing any `EventType` --
  every mapped key already treats "held" as a level, not an edge, so repeat events carry no
  information the counting fix needs, and admitting them would silently corrupt the count.

## Prevention

3 new regression tests: `releasing_one_aliased_unknown_button_does_not_clear_another_still_held`
(`tests/mouse_button_state_test.rs`); `releasing_one_aliased_unidentified_key_does_not_clear_another_still_held`
and `a_spurious_unidentified_release_with_no_prior_press_does_not_panic`
(new `tests/keyboard_key_state_test.rs`, this crate's first dedicated keyboard-state test file).
The `event.repeat()` interaction is DOM-listener-level behavior with no live `KeyboardEvent` in
`cargo test` -- documented instead in `tests/manual/readme.md` as a new manual scenario (see Refs).

## Pitfall

Any many-to-one fallback/catch-all enum variant breaks a flat hold-bit the instant two distinct
real inputs alias to it simultaneously -- invisible for every one of the many individually-mapped
variants, which need no counting, and easy to miss precisely because the fallback variant is the
one place a reviewer might assume "it's just one more case" rather than "this is structurally
different from every other case." A hold-count fix for such a variant must also be checked against
platform-level event semantics (auto-repeat) that a normal, non-aliasing key's simple level-toggle
never had to worry about.

## Generalized Version

**Broken assumption:** "every variant of an enum can track its held/pressed state the same way,
via one flat bit/bool."

**Confirmed general rule:** A many-to-one fallback variant requires a hold-COUNT, not a hold-bit,
because it can be the true target of more than one concurrently-held real input at once -- and any
count-based fix for a repeatable input event must independently verify whether the platform can
re-fire that event for an already-held input without a matching release (auto-repeat), or the count
will desynchronize.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-17 | filed | Found via this session's `browser_input` audit, checking both fallback-variant enums against their flat-bit state tracking. |
| 2026-08-17 | fixed | Added per-pointer `unknown_button_counts` (mouse) and global `unidentified_key_hold_count` (keyboard) hold-counts; added an `event.repeat()` filter to the keyboard DOM listener as a prerequisite. 3 new regression tests added. |
| 2026-08-17 | documented | `tests/manual/readme.md`: new manual scenario added for the `event.repeat()` DOM-listener behavior, which has no live-browser equivalent in `cargo test`. |
| 2026-08-17 | verified | `cargo nextest run -p browser_input --all-features --no-fail-fast`: 24/24 passed, 0 skipped. `cargo clippy -p browser_input --all-targets --all-features -- -D warnings`: clean. |

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 6/6

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Completeness | — | 🟢 | All FI008 sections present. | — |
| D2 | MRE Validity & Reproducibility | — | 🟢 | Both halves' MREs constructed to alias two distinct conceptual real inputs onto the one fallback variant -- confirmed this is exactly what the fallback's own mapping function (E1/E2) collapses, not an artificial scenario. | — |
| D3 | Cross-Reference Integrity | — | 🟢 | Correctly scoped as one ID for both halves (identical root cause and fix shape per this session's established convention) rather than split into two; correctly distinguished from BUG-212 (co-located mouse-half code, distinct root cause). | — |
| D4 | Root Cause Quality | 🟠 | 🟢 | Confirming pass initially considered a naive hold-count fix sufficient. Adversarial pass specifically checked platform event semantics for repeatable inputs and found `KeyboardEvent.repeat()` would corrupt a naive counter -- the repeat-guard was added as a documented prerequisite before any code was written, not discovered as a defect after the fact. | Added the `event.repeat()` filter to `keyboard_callback` as part of the initial fix design. |
| D5 | Execution Scope | — | 🟢 | Fix confined to the two aliasing variants; every individually-mapped variant's existing flat-bit behavior deliberately left untouched and re-verified via the pre-existing BUG-130 tests and the new sanity test for `KeyboardKey::Space`. | — |
| D6 | Crate Scope Unity | — | 🟢 | Fix lives entirely in `browser_input`; no downstream crate changes needed. | — |

**Reproduced:** YES — pre-fix, both `releasing_one_aliased_unknown_button_does_not_clear_another_still_held`
and `releasing_one_aliased_unidentified_key_does_not_clear_another_still_held` fail (shared slot
clears after the first of two aliased releases); post-fix, both pass. 2026-08-17.

## Refs: src/

| File | Change |
|------|--------|
| `module/helper/browser_input/src/input.rs` | `State`: added `unknown_button_counts`, `unidentified_key_hold_count` fields. `events_apply_to_state`: `KeyboardKey` arm branches on `Unidentified` for count-based tracking; `PointerButton` `Press`/`Release`/`PointerCancel` arms gain matching count-based tracking for `MouseButton::Unknown`. `Input::new`'s `keyboard_callback` gains an `event.repeat()` early-return guard (full `Fix(BUG-213)` comment blocks throughout). |

## Refs: tests/

| File | Change |
|------|--------|
| `module/helper/browser_input/tests/mouse_button_state_test.rs` | Added `releasing_one_aliased_unknown_button_does_not_clear_another_still_held`. |
| `module/helper/browser_input/tests/keyboard_key_state_test.rs` | New file: added `releasing_one_aliased_unidentified_key_does_not_clear_another_still_held`, `a_normally_mapped_key_is_unaffected_by_the_unidentified_counting_fix`, `a_spurious_unidentified_release_with_no_prior_press_does_not_panic`. |

## Refs: docs/

| File | Change |
|------|--------|
| `module/helper/browser_input/tests/manual/readme.md` | New manual scenario documenting the `event.repeat()` DOM-listener filter, which has no live-`KeyboardEvent` equivalent testable via `cargo test`. |
