# BUG-536: Without vertex tangents, the TBN bitangent pointed down the image

- **Severity:** Medium (every normal map on a mesh without tangents had its green channel
  inverted, so relief was lit from the wrong vertical side; no crash)
- **state:** Completed
- **Affects:** Every normal-mapped mesh without a TANGENT attribute, and, since PR #209, the
  clearcoat normal map and the anisotropy direction on such meshes.
- **Component:** `module/helper/renderer` (`src/webgl/shaders/main.frag`)
- **repo_identity:** self
- **Filed:** 2026-10-06
- **filed_by:** self
- **verified_by:** self
- **verification_date:** 2026-10-06
- **Related Bugs:** BUG-534 and BUG-535 were fixed in the same PR (#209).

## Symptom

`getTBN` reconstructs the tangent frame from screen-space derivatives when the mesh has no
tangents. Its bitangent column was the gradient of v. glTF puts the UV origin at the image's
upper-left corner, and the loader uploads images unflipped, so v increases down the image, while
glTF's tangent space has +Y up. A normal-map sample's green channel was therefore applied upside
down, and an anisotropy direction authored at angle θ rendered at −θ.

## Impact

**Who is affected:** Any glTF asset with a normal map and no TANGENT attribute, which is common:
glTF lets a client derive the frame itself.

**What breaks:** Relief is lit from the wrong side vertically; anisotropic highlights are mirrored.

**Entity Scope:** None -- a code-level defect.

## How Discovered

Round 2 of the PR #209 review (2026-10-01). Unchanged from master.

## Minimum Reproducible Example

Render column 1 of `getTBN( +Z, pos, uv )` on a full-screen quad with u = ( 1 + x ) / 2 and
v = ( 1 - y ) / 2, an upright image: pre-fix the bitangent read back as ( 0, -1, 0 ) instead of
( 0, 1, 0 ).

**Verify Command:**
```bash
cd module/helper/renderer && wasm-pack test --headless --chrome -- --features test_internals --test tangent_frame_test
```

## Root Cause

"v increases" was taken for "up". In glTF it is the opposite, given the upper-left UV origin and
unflipped image uploads.

## Why Not Caught

Nothing read the frame back, and an inverted green channel still looks like plausible relief.

## Fix Location

- `module/helper/renderer/src/webgl/shaders/main.frag`: `getTBN` orients its bitangent up the
  image, the direction in which v decreases (commit `be32788f`), and, since `2cf7afef`, builds an
  orthonormal frame from the surface direction in which u increases. `Fix(BUG-536)` comment there.

## Prevention

`derivative_frame_bitangent_points_up_the_image` in
`module/helper/renderer/tests/tangent_frame_test.rs` cuts `getTBN` out of the shipped `main.frag`
and reads every column of the frame back from an upright quad. The file's other cases pin mirrored,
stretched, sheared and degenerate UVs and the back face.

## Pitfall

In glTF, "v increases" is not "up"; check a tangent frame against an upright image.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-10-01 | fixed | Raised by round 2 of the PR #209 review; fixed in `be32788f` without a record. |
| 2026-10-06 | filed | Round 3 of the review asked for a record. |
| 2026-10-06 | verified | See Verification Record below. |

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 3/3

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Regression test validity | — | 🟢 | Confirming pass: `tangent_frame_test` passes 6/6. Adversarial pass: with the bitangent oriented along increasing v again, the reproducer fails, with four other cases of the file (1/6). | — |
| D2 | Fix documentation compliance | — | 🟢 | `Fix(BUG-536)` / `Root cause` / `Pitfall` comment above `getTBN`; 5-section doc comment and `bug_reproducer(BUG-536)` marker on the reproducer. | — |
| D3 | Scope containment | — | 🟢 | Fix confined to `getTBN`'s bitangent orientation. | — |

**Reproduced:** YES -- the reverted-orientation run above. 2026-10-06.

## Refs: src/

| File | Change |
|------|--------|
| `module/helper/renderer/src/webgl/shaders/main.frag` | `getTBN` bitangent up the image; `Fix(BUG-536)` comment. |

## Refs: tests/

| File | Change |
|------|--------|
| `module/helper/renderer/tests/tangent_frame_test.rs` | `derivative_frame_bitangent_points_up_the_image`, marked `bug_reproducer(BUG-536)`. |
