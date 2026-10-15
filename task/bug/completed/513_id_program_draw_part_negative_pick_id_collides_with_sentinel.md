# BUG-513: `IdProgram::draw_part` accepted negative `Pickable::pick_id()` values, silently colliding with `PickBuffer`'s `-1` background sentinel

- **Severity:** Medium (no crash, no visual corruption -- a silent design-invariant violation:
  a part rendered normally but became permanently unpickable, with zero diagnostic of any kind)
- **state:** Completed
- **Affects:** Every consumer implementing `gpu_picking::Pickable` that chooses a negative
  `pick_id()` for any part (e.g. reusing `-1`/`-2`/... as a natural-looking id scheme) -- the part
  renders correctly but can never be returned by `PickBuffer::pick`.
- **Component:** `module/helper/gpu_picking` (`src/lib.rs`)
- **repo_identity:** self
- **Filed:** 2026-08-21
- **filed_by:** self
- **verified_by:** self
- **verification_date:** 2026-08-21
- **Fixed:** 2026-08-21
- **Related Bugs:** Found and fixed alongside BUG-530 (`PickBuffer::pick` accepting out-of-range
  coordinates) and BUG-521 (`PickBuffer` leaking GL resources on drop) during one sweep of
  `gpu_picking`'s ~244-line `src/lib.rs` -- independent defects, no shared root cause, filed
  separately.

## Symptom

```rust
// pre-fix -- src/lib.rs, IdProgram::draw_part
fn draw_part< P : Pickable >( &self, gl : &GL, part : &P )
{
  let u = &self.uniforms;
  let id = part.pick_id();
  // no validation of `id` at all
  gl::uniform::matrix_upload( gl, u.model.clone(), part.model().to_array().as_slice(), true ).unwrap();
  gl.uniform1i( u.id.as_ref(), id );
  // ... draws normally, using `id` as-is, even if negative
}
```

`Pickable::pick_id()` returning `-1` (or any negative value) was accepted and rendered without
complaint -- but `PickBuffer::pick`'s own `readback_to_pick_id` treats `-1` as the reserved
"nothing picked" background sentinel (written via `clear_bufferiv_with_i32_array`), so a part
using that id can never be distinguished from empty space.

## Impact

**Who is affected:** Any `Pickable` implementor that assigns a negative id to any part -- a
natural mistake, since nothing in `pick_id`'s doc comment stated the `>= 0` constraint the rest
of the crate silently depends on.

**What breaks:** The part renders into the id buffer exactly like any other part (no visual
difference at all), but `PickBuffer::pick` can never return that id -- `readback_to_pick_id`
maps the sentinel value to `None` unconditionally. The part becomes permanently, silently
unpickable; nothing in the API surfaces why.

**Magnitude:** One permanently-unpickable part per negative `pick_id()` used.

**Entity Scope:** None -- a code-level defect.

## How Discovered

Found during a dedicated bug-hunting sweep of `module/helper/gpu_picking`, specifically checking
color/id round-tripping at boundary values -- `pick_id() -> i32`'s signed return type and the
crate's own `-1` background-sentinel convention (visible in `readback_to_pick_id`) meant a
negative id was representable but never validated anywhere on the write path.

## Minimum Reproducible Example

```rust
// module/helper/gpu_picking/src/lib.rs, mod tests (inline, native)
assert_pick_id_valid( 0 );   // pre-fix and post-fix: fine, the smallest valid id
assert_pick_id_valid( 42 );  // pre-fix and post-fix: fine
assert_pick_id_valid( -1 );  // pre-fix: silently accepted, rendered, never pickable again
                              // post-fix: panics immediately with a clear diagnostic
```

**Verify Command** (<=3 lines, standalone):
```bash
cd module/helper/gpu_picking && cargo test -p gpu_picking --lib -- negative_pick_id_panics
```

## Root Cause

`Pickable::pick_id`'s doc comment never stated the `>= 0` constraint implied by
`readback_to_pick_id` treating `-1` as the reserved background sentinel -- no code path in
`IdProgram::draw_part` (the only place a `pick_id()` value is actually consumed) validated the
id before using it, so the constraint existed only implicitly, in the relationship between two
functions that never checked each other's assumptions.

## Why Not Caught

`gpu_picking` had zero test coverage of any kind before this sweep, and the failure mode is
completely silent -- no panic, no error, no visual difference. The only symptom is a part that
simply never responds to picks, which looks identical to an unrelated bug in the caller's own
event handling, making it very hard to trace back to the id choice.

## Fix Location

`module/helper/gpu_picking/src/lib.rs`: added a pure `assert_pick_id_valid(id: i32)` helper and
called it at the top of `IdProgram::draw_part`, for every part drawn -- panics loudly, with a
diagnostic explaining the `-1` sentinel reservation, the moment a negative `pick_id` is used,
instead of silently producing an unpickable part.

## Prevention

New inline tests in `src/lib.rs`'s `#[cfg(test)] mod tests` block (placed inline because
`assert_pick_id_valid` is a private free function -- see `rulebook.md § Test placement`):
`negative_pick_id_panics` (`#[should_panic(expected = "pick ids must be >= 0")]`, exercising the
exact reserved boundary `-1`) and `zero_and_positive_pick_ids_are_accepted` (confirming `0` and
representative positive ids remain valid, so a regression tightening the check to `id > 0` would
also be caught).

## Pitfall

`id >= 0` (not `id > 0`) is the correct check -- `0` is a valid, pickable id; only negative
values are reserved. Silently clamping or re-mapping a negative id instead of asserting would
hide the caller's bug behind a part that renders but stays permanently unpickable, with no
diagnostic at all -- the loud panic is a deliberate choice over a silent "fix."

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-21 | filed | Found during a dedicated bug-hunting sweep of `module/helper/gpu_picking`. |
| 2026-08-21 | fixed | Added `assert_pick_id_valid`, called from `IdProgram::draw_part`; added `Fix(BUG-513)`/`Root cause`/`Pitfall` source comment and inline reproducer tests. |
| 2026-08-21 | verified | See Verification Record below. |

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 3/3

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Regression test validity | — | 🟢 | Confirming pass: `cargo test -p gpu_picking --lib -- negative_pick_id_panics zero_and_positive_pick_ids_are_accepted` passes (5/5 native tests green). Adversarial pass: temporarily loosened `assert_pick_id_valid` to `id >= -1` (reintroducing the exact sentinel collision for `-1`) and confirmed `negative_pick_id_panics` failed (no panic raised), then reverted and reconfirmed green. | — |
| D2 | Fix documentation compliance | — | 🟢 | `Fix(BUG-513)`/`Root cause`/`Pitfall` 3-field format applied to the source comment at `src/lib.rs`; 5-section test doc comments applied to both reproducer tests. | — |
| D3 | Scope containment | — | 🟢 | Fix confined to `IdProgram::draw_part` plus the new `assert_pick_id_valid` helper and its own inline tests; no other file touched for this specific item. | — |

**Reproduced:** YES -- adversarial pass reintroduced the exact sentinel-collision defect (loosened
the assert to accept `-1`) and confirmed `negative_pick_id_panics` failed precisely because no
panic was raised, then reverted and reconfirmed the fix passes. 2026-08-21.

## Refs: src/

| File | Change |
|------|--------|
| `module/helper/gpu_picking/src/lib.rs` | Added `assert_pick_id_valid` helper and a call to it at the top of `IdProgram::draw_part`, with `Fix(BUG-513)`/`Root cause`/`Pitfall` comment. |

## Refs: tests/

| File | Change |
|------|--------|
| `module/helper/gpu_picking/src/lib.rs` | Added inline `mod tests::negative_pick_id_panics` and `mod tests::zero_and_positive_pick_ids_are_accepted` (native, needs private-function access per `rulebook.md § Test placement`). |
