# BUG-530: `PickBuffer::pick` passes out-of-range coordinates straight to `read_pixels`, misreporting a fresh buffer's first out-of-range pick as `Some(0)` instead of `None`

- **Severity:** Medium (no crash, no panic -- a silent correctness defect: an out-of-range pick
  returns a plausible-looking wrong answer instead of `None`, so a caller has no signal anything
  went wrong and may act on a bogus pick id)
- **state:** Completed
- **Affects:** Every consumer of `gpu_picking::PickBuffer::pick` that ever calls it with
  coordinates at or past the buffer's own edge (e.g. a pointer event fired exactly on the last
  canvas pixel, or coordinates computed from a slightly stale canvas size) -- most acutely, the
  very first `pick()` call on a freshly-constructed `PickBuffer` whose `readback` array is still
  JS-zero-initialized.
- **Component:** `module/helper/gpu_picking` (`src/lib.rs`)
- **repo_identity:** self
- **Filed:** 2026-08-21
- **filed_by:** self
- **verified_by:** self
- **verification_date:** 2026-08-21
- **Fixed:** 2026-08-21
- **Related Bugs:** Found and fixed alongside BUG-513 (`IdProgram::draw_part` accepting negative
  pick ids) and BUG-521 (`PickBuffer` leaking GL resources on drop) during one sweep of
  `gpu_picking`'s ~244-line `src/lib.rs` -- independent defects, no shared root cause, filed
  separately.

## Symptom

```rust
// pre-fix -- src/lib.rs, PickBuffer::pick
pub fn pick( &self, gl : &GL, x : i32, y : i32 ) -> Option< i32 >
{
  // no bounds check at all -- x/y go straight to read_pixels
  gl.bind_framebuffer( GL::FRAMEBUFFER, self.framebuffer.as_ref() );
  gl.read_buffer( GL::COLOR_ATTACHMENT0 );
  gl.read_pixels_with_array_buffer_view_and_dst_offset
  (
    x, y, 1, 1, GL::RED_INTEGER, GL::INT, &self.readback, 0
  ).unwrap();
  gl.bind_framebuffer( GL::FRAMEBUFFER, None );

  let id = self.readback.to_vec()[ 0 ];
  readback_to_pick_id( id )
}
```

Calling `pick(gl, width, height)` (one past the last valid pixel in both axes) -- or any other
out-of-range `(x, y)` -- was passed directly to `read_pixels` with no validation against
`self.width`/`self.height`.

## Impact

**Who is affected:** Any consumer calling `PickBuffer::pick` with coordinates that are not
strictly inside `[0, width) x [0, height)` -- easy to trigger from ordinary pointer-event
handling (e.g. a `mouseup` firing after the canvas was resized, or coordinates rounded up to
exactly the canvas edge).

**What breaks:** On a freshly-constructed `PickBuffer`, `self.readback` is a JS `Int32Array`
that always starts zero-filled (per the ECMAScript `TypedArray` spec) -- an out-of-range
`read_pixels` call that leaves the buffer untouched (driver-dependent, but plausible for a
read outside the framebuffer's bounds) reads back as `id == 0`, which `readback_to_pick_id`
happily converts to `Some(0)` instead of the caller-expected `None`. A caller has no way to
distinguish "pixel 0 was genuinely picked" from "the read never actually happened."

**Magnitude:** One misreported pick per out-of-range `pick()` call; worst on the very first call
after construction/resize, before any in-bounds pick has ever overwritten `self.readback`.

**Entity Scope:** None -- a code-level defect.

## How Discovered

Found during a dedicated bug-hunting sweep of `module/helper/gpu_picking` (a small WebGL2
GPU-based object-picking crate), specifically checking `PickBuffer::pick` (the one function in
the crate that turns raw pointer coordinates into a GPU read) for bounds validation against
`self.width`/`self.height`. None existed.

## Minimum Reproducible Example

```rust
// module/helper/gpu_picking/src/lib.rs, mod tests (inline, native)
assert!( pick_in_bounds( 0, 0, 4, 4 ) );      // first valid pixel
assert!( pick_in_bounds( 3, 3, 4, 4 ) );      // last valid pixel of a 4x4 buffer
assert!( !pick_in_bounds( 4, 0, 4, 4 ) );     // pre-fix: PickBuffer::pick would still read this
assert!( !pick_in_bounds( 0, 4, 4, 4 ) );     // pre-fix: PickBuffer::pick would still read this
```

**Verify Command** (<=3 lines, standalone):
```bash
cd module/helper/gpu_picking && cargo test -p gpu_picking --lib -- pick_in_bounds_rejects_out_of_range_coordinates
```

## Root Cause

`PickBuffer::pick` had no bounds check of any kind -- `(x, y)` were handed straight to
`gl.read_pixels_with_array_buffer_view_and_dst_offset` regardless of whether they actually fell
inside the id texture's own `[0, width) x [0, height)` extent. Combined with `self.readback`
being a reused, JS-zero-initialized `Int32Array` (never reallocated between picks), an
out-of-range read that the driver leaves untouched silently reads back as the same value as a
genuine pick of id `0`.

## Why Not Caught

`gpu_picking` had zero test coverage of any kind before this sweep -- nothing exercised
`PickBuffer::pick` with any coordinates, in-bounds or otherwise, so a missing bounds check
produced no observable failure in CI or manual testing. The failure mode is also silent by
nature (a wrong *value*, not a panic or compile error), so it would only surface as a confusing,
hard-to-reproduce "wrong object got picked at the edge of the canvas" report from a real user.

## Fix Location

`module/helper/gpu_picking/src/lib.rs`: added a pure `pick_in_bounds(x, y, width, height) -> bool`
helper and called it at the top of `PickBuffer::pick`, returning `None` immediately for any
out-of-range coordinate before ever touching the GPU.

## Prevention

New inline test `pick_in_bounds_rejects_out_of_range_coordinates` in `src/lib.rs`'s
`#[cfg(test)] mod tests` block (placed inline because `pick_in_bounds` is a private free function
-- see `rulebook.md § Test placement`). Covers both valid boundary pixels (`(0,0)` and
`(width-1, height-1)` on a 4x4 buffer), all four invalid boundary edges (`x`/`y` each at `-1` and
at exactly `width`/`height` -- the classic off-by-one), and the degenerate `0x0` buffer case.

## Pitfall

`x < width` (not `x <= width`) is the correct upper-bound check -- valid columns are `0..width`,
so `x == width` is one column past the last valid one and must be rejected, not accepted. Adding
a new `pick`-adjacent convenience wrapper in the future without routing it through
`pick_in_bounds` first would silently reopen this exact defect.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-21 | filed | Found during a dedicated bug-hunting sweep of `module/helper/gpu_picking`. |
| 2026-08-21 | fixed | Added `pick_in_bounds`, called at the top of `PickBuffer::pick`; added `Fix(BUG-530)`/`Root cause`/`Pitfall` source comment and inline reproducer test. |
| 2026-08-21 | verified | See Verification Record below. |

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 3/3

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Regression test validity | — | 🟢 | Confirming pass: `cargo test -p gpu_picking --lib -- pick_in_bounds_rejects_out_of_range_coordinates` passes (5/5 native tests green). Adversarial pass: temporarily changed the upper-bound check from `x < width` to `x <= width` (reintroducing the exact off-by-one) and confirmed the test failed precisely on the `(4,0,4,4)`/`(0,4,4,4)` boundary assertions, then reverted and reconfirmed green. | — |
| D2 | Fix documentation compliance | — | 🟢 | `Fix(BUG-530)`/`Root cause`/`Pitfall` 3-field format applied to the source comment at `src/lib.rs`; 5-section test doc comment applied to the reproducer. | — |
| D3 | Scope containment | — | 🟢 | Fix confined to `PickBuffer::pick` plus the new `pick_in_bounds` helper and its own inline test; no other file touched for this specific item. | — |

**Reproduced:** YES -- adversarial pass reintroduced the exact off-by-one bounds bug and confirmed
`pick_in_bounds_rejects_out_of_range_coordinates` failed on precisely the boundary cases the fix
is meant to guard, then reverted and reconfirmed the fix passes. 2026-08-21.

## Refs: src/

| File | Change |
|------|--------|
| `module/helper/gpu_picking/src/lib.rs` | Added `pick_in_bounds` helper and a bounds check at the top of `PickBuffer::pick`, with `Fix(BUG-530)`/`Root cause`/`Pitfall` comment. |

## Refs: tests/

| File | Change |
|------|--------|
| `module/helper/gpu_picking/src/lib.rs` | Added inline `mod tests::pick_in_bounds_rejects_out_of_range_coordinates` (native, needs private-function access per `rulebook.md § Test placement`). |
