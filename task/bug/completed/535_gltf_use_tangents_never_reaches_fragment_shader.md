# BUG-535: glTF vertex tangents never reached the fragment shader

- **Severity:** Medium (every asset that ships TANGENT data was shaded with the derivative frame
  instead of its authored tangents; no crash, plausible-looking but wrong normal-map shading)
- **state:** Completed
- **Affects:** Every glTF primitive with a TANGENT attribute and a tangent-space texture or
  anisotropy.
- **Component:** `module/helper/renderer` (`src/webgl/loaders/gltf.rs`, `src/webgl/shaders/main.vert`)
- **repo_identity:** self
- **Filed:** 2026-10-06
- **filed_by:** self
- **verified_by:** self
- **verification_date:** 2026-10-06
- **Related Bugs:** BUG-534 and BUG-536 were fixed in the same PR (#209).

## Symptom

`main.frag` builds its tangent frame from `vTangent` only under `#ifdef USE_TANGENTS`. The
loader records a TANGENT attribute as `USE_TANGENTS` on a scratch material for both stages, but
copied only that material's vertex defines onto each primitive's material. The fragment stage is
compiled from the fragment defines alone, so the vertex-tangent frame never compiled for a loaded
asset, and `#ifndef USE_TANGENTS` always picked the screen-space derivative frame.

## Impact

**Who is affected:** Assets that ship tangents, typically because their normal maps were baked
against them.

**What breaks:** Normal maps, the clearcoat normal map and the anisotropy direction are applied in
the derivative frame, which differs from the authored MikkTSpace frame wherever UVs are sheared,
mirrored per island or unevenly stretched.

**Entity Scope:** None -- a code-level defect.

## How Discovered

Round 2 of the PR #209 review (2026-10-01). Master never copied the fragment defines; the branch
had added the loop and lost it in the merge 7a37bd43.

## Minimum Reproducible Example

```rust
let gltf = gltf::load( &document, &two_primitive_gltf_uri( &[] ), &gl ).await?;
let primitive = gltf.meshes[ 0 ].borrow().primitives[ 0 ].clone(); // has TANGENT
// pre-fix: false
assert!( primitive.borrow().material.borrow().fragment_defines_str().contains( "#define USE_TANGENTS" ) );
```

**Verify Command:**
```bash
cd module/helper/renderer && wasm-pack test --headless --chrome -- --features test_internals --test gltf_tangent_defines_test
```

## Root Cause

The two shader stages compile from separate define sets, and the loader copied attribute defines
into the vertex set only.

## Why Not Caught

No test loaded an asset with TANGENT data, and the derivative frame still shades plausibly.

## Fix Location

- `module/helper/renderer/src/webgl/loaders/gltf.rs`: the scratch material's fragment defines are
  copied onto each primitive's material as well. `Fix(BUG-535)` comment there. (Commit `8118a9a9`.)
- `module/helper/renderer/src/webgl/shaders/main.vert`: with the tangent path live, `vTangent` had
  to move into world space with the world matrix, to pair with the world-space `vNormal`; it was
  passed through in object space. `Fix(BUG-535)` comment there.

## Prevention

`tangent_attribute_reaches_the_fragment_defines` in
`module/helper/renderer/tests/gltf_tangent_defines_test.rs` loads a two-primitive `data:` URI
asset through `gltf::load` and checks that only the primitive with TANGENT gets `USE_TANGENTS` in
its fragment defines.

## Pitfall

A define recorded for both stages must be copied to both; the fragment stage never sees the
vertex defines.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-10-01 | fixed | Raised by round 2 of the PR #209 review; fixed in `8118a9a9` without a record. |
| 2026-10-06 | filed | Round 3 of the review asked for a record. |
| 2026-10-06 | verified | See Verification Record below. |

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 3/3

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Regression test validity | — | 🟢 | Confirming pass: `gltf_tangent_defines_test` passes 3/3. Adversarial pass: with the fragment-define copy removed from `gltf.rs`, the reproducer fails (2/3). | — |
| D2 | Fix documentation compliance | — | 🟢 | `Fix(BUG-535)` / `Root cause` / `Pitfall` comment on the copy loop, `Fix(BUG-535)` note in `main.vert`; 5-section doc comment and `bug_reproducer(BUG-535)` marker on the reproducer. | — |
| D3 | Scope containment | — | 🟢 | Fix confined to the copy loop and the tangent transform in `main.vert`. | — |

**Reproduced:** YES -- the removed-loop run above. 2026-10-06.

## Refs: src/

| File | Change |
|------|--------|
| `module/helper/renderer/src/webgl/loaders/gltf.rs` | Copies the fragment defines; `Fix(BUG-535)` comment. |
| `module/helper/renderer/src/webgl/shaders/main.vert` | `vTangent` moved into world space; `Fix(BUG-535)` comment. |

## Refs: tests/

| File | Change |
|------|--------|
| `module/helper/renderer/tests/gltf_tangent_defines_test.rs` | `tangent_attribute_reaches_the_fragment_defines`, marked `bug_reproducer(BUG-535)`. |
