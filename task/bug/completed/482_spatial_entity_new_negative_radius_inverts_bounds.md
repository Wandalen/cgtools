# BUG-482: `SpatialEntity::new` accepted a negative radius, producing inverted bounds and spurious intersection results

- **Severity:** Low (no crash -- but a negative radius silently produced geometrically invalid
  state rather than being rejected or clamped)
- **state:** Completed
- **Affects:** Any consumer of `SpatialEntity::new` that passes a caller-controlled or
  computed radius that could be negative (e.g. derived from a subtraction or an unchecked
  external input).
- **Component:** module/helper/tiles_tools (`src/spatial.rs`)
- **repo_identity:** self
- **Filed:** 2026-08-20
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/ (self)
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/ (self)
- **verification_date:** 2026-08-20
- **Related Bugs:** BUG-349 (a different crate's `Resource::new` negative-`maximum` panic --
  same general defect *class*, unrelated struct/mechanism).

## Symptom

```rust
// pre-fix -- src/spatial.rs
pub fn new(id: EntityId, position: (i32, i32), radius: i32) -> Self {
  Self { id, position, radius, .. } // radius stored as-is, no validation
}
```

A negative `radius` was stored verbatim. Any method deriving a bounding box from `radius`
(`bounds()`, computing `left = position.0 - radius`, `right = position.0 + radius`) would then
produce an *inverted* box (`left > right`) for a negative radius, since subtracting a negative
number increases the left edge past the right edge.

## Impact

**Who is affected:** Any consumer constructing a `SpatialEntity` with a radius that could be
negative -- confirmed via consumer audit (below) that no current call site in this workspace
actually passes a negative literal, but nothing prevented a computed/external radius from being
negative.

**What breaks:** `bounds()` returns an inverted box (`left > right`, `top > bottom`) for a
negative radius; any code assuming `left <= right`/`top <= bottom` (a standard invariant for
axis-aligned bounding boxes) would misbehave -- e.g. `intersects_entity` computing overlap
against an inverted box could produce spurious true/false results depending on exactly how the
overlap arithmetic composes with the inversion.

**Consumer audit:** `grep -rn 'SpatialEntity::new' --include="*.rs" .` from the repo root
confirms all current call sites (this crate's own tests and its `spatial` module's own internal
usage) pass non-negative literals or values already validated non-negative upstream -- no
current caller is affected today. This is a preventive/defensive fix, not a fix for an observed
production symptom.

**Magnitude:** Single constructor; see Fix Location.

**Entity Scope:** None -- a code-level defect.

## How Discovered

Found during a repo-wide "find and fix all bugs and UX/DX defects" sweep of
`module/helper/tiles_tools`, reading `src/spatial.rs` end to end -- `radius: i32` (a signed
type for an inherently non-negative geometric quantity) with no validation in the constructor
is a recognizable defensive-programming gap, cross-checked against `bounds()`'s subtraction to
confirm the inversion consequence concretely rather than assumed.

## Minimum Reproducible Example

```rust
// module/helper/tiles_tools/tests/spatial_test.rs
let entity = SpatialEntity::new(1, (0, 0), -7);
let bounds = entity.bounds();
assert!(bounds.left <= bounds.right && bounds.top <= bounds.bottom);
// pre-fix: fails -- radius stored as -7 verbatim, bounds() computes an inverted box
```

**Verify Command** (<=3 lines, standalone):
```bash
cd module/helper/tiles_tools && cargo nextest run -E 'binary(spatial_test) and test(new_clamps_negative_radius)'
```

## Root Cause

`SpatialEntity::new` stored its `radius: i32` parameter without validating it was non-negative
-- a signed integer type was used for a quantity (a bounding radius) that is only ever
geometrically meaningful as non-negative, and nothing at the construction boundary enforced
that invariant.

## Why Not Caught

No existing test constructed a `SpatialEntity` with a negative radius -- all prior test
fixtures used positive radii, where the defect is invisible (subtracting a positive radius from
position correctly produces `left < right`).

## Fix Location

`module/helper/tiles_tools/src/spatial.rs`: `SpatialEntity::new` now clamps via `let radius =
radius.max(0);` before constructing `Self`. Judgment call: chose to **clamp at construction**
rather than changing `radius`'s field type to an unsigned integer (`u32`) -- a type change would
be a breaking API change rippling through every internal computation currently written against
`i32` arithmetic (e.g. `position.0 - radius` mixing signed position with the field), and this
crate's scope for this sweep does not include auditing every downstream `examples/` consumer
for such a breaking change. Clamping at the single construction boundary is the minimal,
non-breaking fix that eliminates the invalid state without touching the type or any other
method's signature.

## Prevention

New test `test_spatial_entity_new_clamps_negative_radius_to_zero` in `tests/spatial_test.rs`
constructs with `radius = -7`, asserts it clamps to `0`, asserts `bounds()` is not inverted
(`left <= right`, `top <= bottom`), and asserts `intersects_entity` against a far-away entity
does not spuriously report an intersection (guarding against wraparound-style false positives
from the pre-fix inverted-box arithmetic).

## Pitfall

A signed integer type (`i32`) used for a quantity that is only ever geometrically meaningful as
non-negative (a radius) does not, by itself, prevent invalid values from being constructed --
the type system permits negative values regardless of what they mean domain-wise. Validating or
clamping at the construction boundary is required whenever the type alone cannot express the
invariant; deferring the check to every downstream consumer of the field (`bounds()`,
`intersects_entity`, etc.) means every one of them independently needs to handle or fail to
handle the invalid case, which is exactly the multiplicative-risk pattern a single
construction-time check avoids.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-20 | filed | Found during a repo-wide bug/UX-DX sweep of `module/helper/tiles_tools`, reading `src/spatial.rs` end to end and cross-checking against `bounds()`'s subtraction arithmetic. |
| 2026-08-20 | fixed | `SpatialEntity::new` now clamps `radius` to non-negative via `radius.max(0)` before construction. |
| 2026-08-20 | verified | See Verification Record below. |

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 2/2

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Regression test validity | — | 🟢 | Adversarial pass: confirmed the test's bounds-ordering assertion (`left <= right`, `top <= bottom`) genuinely fails against the pre-fix unclamped constructor for `radius = -7` (verified by direct calculation: `bounds().left = position.0 - (-7) = position.0 + 7`, `bounds().right = position.0 + (-7) = position.0 - 7`, so `left > right`) and passes against the clamped fix. | — |
| D2 | API-breaking-change avoidance confirmed | — | 🟢 | Confirmed the fix changes no method signature or field type -- `grep -rn 'SpatialEntity' --include="*.rs" .` from the repo root shows no external consumer whose call sites could be affected by a signature change, and none exists here since none was made. | — |

**Reproduced:** YES -- `test_spatial_entity_new_clamps_negative_radius_to_zero`'s bounds-ordering
assertions are false against the pre-fix unclamped constructor (verified by direct calculation,
not a temporary revert-and-rerun, since the fix was written and verified in the same pass) and
true against the fix. 2026-08-20.

## Refs: src/

| File | Change |
|------|--------|
| `module/helper/tiles_tools/src/spatial.rs` | `SpatialEntity::new` clamps `radius` to non-negative via `radius.max(0)`; `Fix(BUG-482)`/`Root cause`/`Pitfall` comment added. |

## Refs: tests/

| File | Change |
|------|--------|
| `module/helper/tiles_tools/tests/spatial_test.rs` | Added `test_spatial_entity_new_clamps_negative_radius_to_zero`, asserting clamping, non-inverted bounds, and no spurious intersection results. |
