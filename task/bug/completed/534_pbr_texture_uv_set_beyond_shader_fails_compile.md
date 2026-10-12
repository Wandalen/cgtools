# BUG-534: A PBR texture on UV set 5 or above failed the program's compile on every frame

- **Severity:** Medium (the primitive draws nothing, and its program is compiled again every
  frame; only assets with a `texCoord` of 5 or more are affected)
- **state:** Completed
- **Affects:** Every `PbrMaterial` texture, core and extension alike, whose `TextureInfo` names a
  UV set above 4: a glTF `texCoord` of 5 or more, or a caller-built `TextureInfo`.
- **Component:** `module/helper/renderer` (`src/webgl/material/pbr.rs`)
- **repo_identity:** self
- **Filed:** 2026-10-06
- **filed_by:** self
- **verified_by:** self
- **verification_date:** 2026-10-06
- **Related Bugs:** BUG-535 and BUG-536 were fixed in the same PR (#209).

## Symptom

`texture_define_push` wrote `#define v<Name>Uv vUv_<n>` with the texture's UV set index. For a
set of 5 or above that names a varying `main.frag` doesn't declare (`vUv_0` to `vUv_4`), so the
fragment shader failed to compile. A failed program is never cached, so every `render()` compiled
it again and returned the error before drawing.

## Impact

**Who is affected:** Any asset with a texture on `texCoord` 5 or above. The extension textures
added by PR #209 (clearcoat, anisotropy) are read from raw JSON, so nothing upstream bounds
them either.

**What breaks:** The primitive never draws, and the failed compile repeats every frame.

**Entity Scope:** None -- a code-level defect.

## How Discovered

Round 2 of the PR #209 review (2026-10-01). Present on master for core textures.

## Minimum Reproducible Example

```rust
let mut info = texture_info(); // any texture, as in the reproducer
info.uv_position = 5;
material.base_color_texture_set( Some( info ) );
// pre-fix: the fragment defines contain "#define vBaseColorUv vUv_5"
assert!( !material.fragment_defines_str().contains( "vUv_5" ) );
```

**Verify Command:**
```bash
cd module/helper/renderer && wasm-pack test --headless --chrome -- --features test_internals --test pbr_material_live_test
```

## Root Cause

The UV set index went from the asset into the define unchecked. `gltf-json` doesn't bound
`texCoord`, and nothing between the loader and `texture_define_push` compared it with the sets
the shader declares.

## Why Not Caught

No test put a texture above UV set 4.

## Fix Location

- `module/helper/renderer/src/webgl/material/pbr.rs`: `texture_define_push`, which writes every
  texture's UV define, falls back to UV set 0 with a warning when the set is `UV_SET_COUNT` (5)
  or above. `Fix(BUG-534)` comment there. (Commit `10d1fdea`.)
- `module/helper/renderer/src/webgl/loaders/gltf_extensions.rs`: the extension reader logs an
  invalid `texCoord`, a textureInfo without a valid index and an index outside the asset's
  textures, which it used to drop silently.

## Prevention

`uv_set_beyond_the_shader_falls_back_to_set_zero` in
`module/helper/renderer/tests/pbr_material_live_test.rs` puts a clearcoat and a base color
texture on UV set 5 and checks that no define names `vUv_5`.

## Pitfall

An asset index that names a shader symbol must be bounded by what the shader declares;
`UV_SET_COUNT` has to stay in step with `main.vert` / `main.frag`.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-10-01 | fixed | Raised by round 2 of the PR #209 review; fixed in `10d1fdea` without a record. |
| 2026-10-06 | filed | Round 3 of the review asked for a record. |
| 2026-10-06 | verified | See Verification Record below. |

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 3/3

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Regression test validity | — | 🟢 | Confirming pass: `pbr_material_live_test` passes 16/16. Adversarial pass: with the fallback disabled in `texture_define_push`, the reproducer fails (15/16). | — |
| D2 | Fix documentation compliance | — | 🟢 | `Fix(BUG-534)` / `Root cause` / `Pitfall` comment on `texture_define_push`; 5-section doc comment and `bug_reproducer(BUG-534)` marker on the reproducer. | — |
| D3 | Scope containment | — | 🟢 | Fix confined to `texture_define_push` and the extension reader's logging. | — |

**Reproduced:** YES -- the disabled-fallback run above. 2026-10-06.

## Refs: src/

| File | Change |
|------|--------|
| `module/helper/renderer/src/webgl/material/pbr.rs` | `UV_SET_COUNT`; `texture_define_push` falls back to UV set 0; `Fix(BUG-534)` comment. |
| `module/helper/renderer/src/webgl/loaders/gltf_extensions.rs` | Logs invalid texture references instead of dropping them silently. |

## Refs: tests/

| File | Change |
|------|--------|
| `module/helper/renderer/tests/pbr_material_live_test.rs` | `uv_set_beyond_the_shader_falls_back_to_set_zero`, marked `bug_reproducer(BUG-534)`. |
| `module/helper/renderer/tests/gltf_material_extensions_test.rs` | Pins how `texCoord` 5, -1 and 1.5 parse. |
