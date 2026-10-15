# BUG-454: `falling_frontier` HUD's "Animate Ships Motion" toggle and Pause/Play/Fast buttons drive the same state but never repaint each other

- **Severity:** Medium (no crash, no data corruption -- both controls always write the correct
  `GridTuning` state -- but their `active`/status-text DOM rendering silently drifts out of sync
  with reality whenever a user drives both surfaces, which is confusing in a demo whose entire
  point is a live tactical HUD)
- **state:** Completed
- **Affects:** `examples/minwebgl/falling_frontier` -- the in-game HUD overlay (`src/hud.rs`) only;
  the separate dev tuning panel (`src/debug/grid_tuning_panel.rs`) is unaffected (its own
  intentional non-sync with the HUD is unrelated and already documented, see Related Bugs).
- **Component:** `examples/minwebgl/falling_frontier/src/hud.rs`
- **repo_identity:** self
- **Filed:** 2026-08-20
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verification_date:** 2026-08-20
- **Related Bugs:** None -- this module's own doc comment (lines 29-37) documents a *different*,
  intentional gap: the HUD's toggle buttons vs. the separate dev tuning panel's equivalent
  controls (two different files/surfaces) are deliberately not kept in sync, since the dev panel
  is a developer tool, not part of the in-game UI. This bug is a same-file, same-surface defect
  between two controls that are *both* part of the real HUD (`bind_tuning_toggle`'s
  "ff-toggle-animate" button and `bind_time_controls`'s Pause/Play/Fast buttons, both in
  `hud.rs`) -- not the documented cross-file gap, and not something the module doc claims is
  intentional.

## Symptom

`hud.rs` renders two independent controls over the same two `GridTuning` fields
(`animate_ships`, `speed_multiplier`):

- `bind_tuning_toggle( document, "ff-toggle-animate", ..., tuning, |t| &mut t.animate_ships )`
  (line ~211) -- a single toggle button, "Animate Ships Motion", labeled `[ACTIVE]`/`[PAUSED]`.
- `bind_time_controls` (lines ~320-373) -- three separate buttons, Pause/Play/Fast, each setting
  `animate_ships`/`speed_multiplier` and calling `set_time_control_active` to highlight itself.

Clicking "Animate Ships Motion" flips `animate_ships` and repaints only its own button -- the
Pause/Play/Fast buttons' `active` highlighting is left showing whatever was true before the click.
Clicking Pause/Play/Fast sets `animate_ships`/`speed_multiplier` and repaints only the three
time-control buttons -- "Animate Ships Motion"'s `[ACTIVE]`/`[PAUSED]` label and `active` class are
left stale.

## Impact

**Who is affected:** Any user of the `falling_frontier` demo who clicks both the "Animate Ships
Motion" toggle and any of Pause/Play/Fast during a session -- e.g. click Pause, then click
"Animate Ships Motion" to resume: the ships actually resume moving (state is correct), but the
Pause button still shows `active` and the toggle shows `[ACTIVE]` -- two buttons now visually
disagree about the same live boolean.

**What breaks:** Visual only -- `GridTuning.animate_ships`/`speed_multiplier` are always written
correctly by both surfaces; only their DOM repaint is one-button-at-a-time instead of
whole-surface.

**Magnitude:** 4 handlers affected -- `bind_tuning_toggle`'s "ff-toggle-animate" case, plus all
three of `bind_time_controls`'s closures (Pause, Play, Fast).

**Entity Scope:** None -- a code-level defect.

## How Discovered

Found during a repo-wide bug/UX sweep of the minwebgl example crates, tracing every consumer of
`GridTuning.animate_ships`/`speed_multiplier` within `hud.rs` and checking whether each write path
repaints every DOM surface that reads the same state.

## Minimum Reproducible Example

DOM-bound (`Document`/`Element` manipulation), no native construction path -- reproduced by
tracing the two handlers directly:

```rust
// bind_tuning_toggle's closure (pre-fix) -- repaints only its own button:
let el = document.get_element_by_id( id ).unwrap();
el.set_class_name( if active { "ff-toggle active" } else { "ff-toggle" } );
// ... no call into bind_time_controls's buttons

// bind_time_controls's Pause closure (pre-fix) -- repaints only Pause/Play/Fast:
tuning.borrow_mut().animate_ships = false;
set_time_control_active( &document, "ff-btn-pause" );
// ... no call into "ff-toggle-animate"'s button
```

**Verify Command:** N/A -- DOM-bound, no native harness; the context-free decision logic
(`time_control_button_classes`, already covered by 3 existing native tests) is unaffected by this
bug -- the defect is purely in which DOM elements each handler chose to repaint, not in the
pause/play/fast selection logic itself. Verified via `cargo check --target wasm32-unknown-unknown`
and `cargo test -p falling_frontier` (see Verification Record).

## Root Cause

Each handler was written to repaint only the DOM elements it directly owns, with no awareness that
a second, independent control surface reads the same underlying state.

## Why Not Caught

`falling_frontier` carries no `tests/` requirement for DOM-bound code (`health.md`), and the only
existing native tests cover `time_control_button_classes` -- the pure pause/play/fast *selection*
logic, not which DOM elements get repainted after a click. That selection logic was never wrong;
the desync is entirely in the repaint step downstream of it, outside what those tests exercise.

## Fix Location

`examples/minwebgl/falling_frontier/src/hud.rs`:
- Added `animate_toggle_repaint( document, active )` -- repaints "ff-toggle-animate" to match a
  given `animate_ships` value (the same repaint `bind_tuning_toggle`'s own closure does for its
  button, pulled out so `bind_time_controls` can reuse it).
- Added `time_controls_repaint( document, animate_ships, speed_multiplier )` -- repaints
  Pause/Play/Fast to match given state, reusing `time_control_button_classes`'s existing threshold
  logic rather than re-deriving the pause/play/fast selection rule a second time.
- `bind_tuning_toggle`'s closure now calls `time_controls_repaint` after its own repaint, but only
  when `id == "ff-toggle-animate"` (the other 3 toggle ids it's shared with don't touch
  `animate_ships`/`speed_multiplier` at all).
- Each of `bind_time_controls`'s three closures (Pause/Play/Fast) now calls
  `animate_toggle_repaint` with the new `animate_ships` value after its existing
  `set_time_control_active` call.

## Prevention

No native regression test is practical for this DOM-repaint defect specifically (would require a
live `Document`). `time_controls_repaint` reuses `time_control_button_classes` rather than
re-deriving its threshold rule, so the 3 existing native tests on that function continue to cover
the shared selection logic both surfaces now render from -- a future change to the
pause/play/fast thresholds can't silently diverge between the two repaint paths. Verified via
`cargo check --target wasm32-unknown-unknown` (compiles) and `cargo test -p falling_frontier`
(existing 3 `time_control_button_classes` tests still pass, confirming the fix didn't disturb that
shared logic).

## Pitfall

Two DOM controls over the same underlying state need each write handler to repaint *both*
surfaces, not just the one the user clicked -- repainting only the clicked control leaves the
other one visually lying about the current state. When a second control appears later that reads
state a first control already owns, audit every existing write path for that state, not just the
new control's own handler.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-20 | filed | Found during a repo-wide bug/UX sweep of minwebgl example crates; discovery and fix landed together in one session. |
| 2026-08-20 | fixed | Added `animate_toggle_repaint`/`time_controls_repaint` helpers and wired them into both handlers. |
| 2026-08-20 | verified | See Verification Record below. |

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 3/3

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Both handlers repaint both surfaces | — | 🟢 | Adversarial pass: re-read the full post-fix `bind_tuning_toggle`/`bind_time_controls` closures -- confirmed the animate-toggle handler calls `time_controls_repaint` and all 3 time-control handlers call `animate_toggle_repaint`, with no `RefCell` double-borrow (the `borrow_mut()` guard is scoped/dropped before the new repaint calls in every closure). | — |
| D2 | No regression to existing selection logic | — | 🟢 | `cargo test -p falling_frontier` -- 9/9 native tests pass, including all 3 pre-existing `hud::tests::*` cases exercising `time_control_button_classes` (reused, not duplicated, by the new `time_controls_repaint`). | — |
| D3 | Compiles for wasm32 target | — | 🟢 | `cargo check --target wasm32-unknown-unknown -p falling_frontier` (combined with the other 7 touched crates in one invocation) -- exit 0, zero errors, zero warnings. | — |

**Reproduced:** N/A (DOM-bound; no native reproduction harness) -- pre-fix source inspected
directly at both handlers to confirm each repainted only its own surface; post-fix source
inspected directly to confirm both handlers now repaint both surfaces. 2026-08-20.

## Refs: src/

| File | Change |
|------|--------|
| `examples/minwebgl/falling_frontier/src/hud.rs` | Added `animate_toggle_repaint`, `time_controls_repaint`; wired both into `bind_tuning_toggle` (animate case) and all 3 `bind_time_controls` closures. |

## Refs: tests/

| File | Change |
|------|--------|
| — | No new test added (DOM-bound defect, no native harness); existing `hud::tests::*` (inline `#[cfg(test)]`, covering the shared `time_control_button_classes` logic the fix reuses) re-verified passing post-fix. |
