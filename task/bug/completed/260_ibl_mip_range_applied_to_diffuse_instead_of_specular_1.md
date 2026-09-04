# BUG-260: `ibl::load`'s `mip_range` clamp is applied to `diffuse_texture` instead of
`specular_1_texture`, the only IBL texture with a real mip chain

- **Severity:** Medium (latent -- no observed wrong pixels at any of the 10 current real call
  sites, 9 of which pass `mip_range: None` and 1 of which coincidentally passes `Some( 0..0 )`
  which happens to visually match "no clamp" for a caller that also always samples mip 0 -- but
  any future caller passing a genuine non-degenerate range silently gets no effect on the texture
  that actually has multiple mips, and a meaningless clamp on the one that doesn't)
- **state:** Completed
- **Affects:** `webgl::loaders::ibl::load` (`src/webgl/loaders/ibl.rs`)
- **Component:** `module/helper/renderer` (`src/webgl/loaders/ibl.rs`)
- **repo_identity:** self
- **Filed:** 2026-08-17
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/ (self)
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/ (self)
- **verification_date:** 2026-08-17
- **Fixed:** 2026-08-17
- **Accepted By:** self -- same-session Tier 2 Dual-Role Self-Check, no separate acceptance actor
  (same-sandbox `tsk` actor-guard blocks `.acceptance_pass` in this environment)

## Symptom

`ibl::load`'s texture-parameter setup block bound 3 different textures to the single global
`TEXTURE_CUBE_MAP` binding point in sequence: first `specular_1_texture` (setting its
min/mag filter), then -- via an intervening `TEXTURE_2D` bind for `specular_2_texture` -- rebound
to `diffuse_texture` (setting its min/mag filter too). The caller-supplied `mip_range`
(`TEXTURE_BASE_LEVEL`/`TEXTURE_MAX_LEVEL`) was applied only after this *last* `TEXTURE_CUBE_MAP`
rebind -- landing on `diffuse_texture`, which has exactly one mip level (clamping its range is
meaningless), instead of `specular_1_texture`, the texture actually carrying the 10-level chain
`IBL::num_mips` documents.

## Impact

**Who is affected:** any caller of `ibl::load` (or, post-fix, `ibl_texture_parameters_apply`
directly) passing a non-trivial `mip_range` intending to restrict which mip levels of the
pre-filtered specular environment map get sampled (e.g. for roughness-based specular IBL
lookups). Of the 10 real call sites across `examples/`, 9 pass `mip_range: None` (the `if let
Some(..)` block never ran, so the bug was unreachable there), and the 1 real non-`None` caller
(`examples/minwebgl/pbr_lighting/src/main.rs`) happens to pass `Some( 0..0 )` -- `TEXTURE_BASE_LEVEL`
written as `0` coincidentally matches the spec default, and `TEXTURE_MAX_LEVEL` written as `0`
instead of the spec default `1000` still visually clamps to mip 0 for a caller that also always
samples mip 0, so the misapplication produced no visibly wrong output there either.

**What breaks:** any future caller passing a genuine non-degenerate range (e.g. `Some( 2..5 )`) to
restrict specular sampling would find the restriction silently applied to the wrong texture --
`specular_1_texture` keeps sampling its full mip chain unrestricted, while `diffuse_texture`
(irrelevant to this parameter) gets a meaningless clamp.

**Entity Scope:** `None` -- source-level GL state-binding-order defect, not entity directory
instances.

## How Discovered

During this session's Group H review of `module/helper/renderer/src/webgl/loaders/*`, direct
trace of `ibl::load`'s texture-parameter block against `IBL`'s own doc comment ("Number of mip
levels in specular_1_texture") revealed the `mip_range` `if let` block sat textually after the
*third* `bind_texture( TEXTURE_CUBE_MAP, .. )` call in the function (to `diffuse_texture`), not
the *first* (to `specular_1_texture`, the texture the parameter is actually documented to apply
to) -- confirmed by tracing which texture is bound to `TEXTURE_CUBE_MAP` at the point the `if let
Some( mip_range )` block executes.

## Minimum Reproducible Example

Requires a real WebGL2 context (texture-parameter state cannot be observed any other way) --
headless-browser test, not a native unit test. Call `ibl_texture_parameters_apply` directly
against 3 freshly-created real textures with a non-degenerate `mip_range`, then read back
`TEXTURE_BASE_LEVEL`/`TEXTURE_MAX_LEVEL` via `get_tex_parameter` for `specular_1_texture` and
`diffuse_texture`. See
`tests/webgl/ibl.rs::ibl_texture_parameters_apply_targets_mip_range_at_specular_1_not_diffuse`.

**Verify Command:**
```bash
cd /home/user1/pro/lib/yrd_gamedev/cgtools
cargo test --target wasm32-unknown-unknown -p renderer --test tests ibl_texture_parameters_apply
```
**Expected** (fixed): 1 passed. **Actual** (pre-fix, confirmed via temporary direct-source-edit
revert of the extraction back to the original inline ordering, real Firefox headless run): 1
failed -- `assertion left == right failed: mip_range.start must land on specular_1_texture (..),
got TEXTURE_BASE_LEVEL=0` (`left: 0, right: 2`), i.e. `specular_1_texture` never received the
clamp at all.

## Root Cause

`ibl::load` (pre-fix), abbreviated:
```rust
gl.bind_texture( TEXTURE_CUBE_MAP, specular_1_texture );
gl.tex_parameteri( TEXTURE_CUBE_MAP, TEXTURE_MIN_FILTER, LINEAR_MIPMAP_LINEAR );
gl.tex_parameteri( TEXTURE_CUBE_MAP, TEXTURE_MAG_FILTER, LINEAR );

gl.bind_texture( TEXTURE_2D, specular_2_texture );
gl.tex_parameteri( TEXTURE_2D, TEXTURE_MIN_FILTER, LINEAR );
gl.tex_parameteri( TEXTURE_2D, TEXTURE_MAG_FILTER, LINEAR );

gl.bind_texture( TEXTURE_CUBE_MAP, diffuse_texture );
gl.tex_parameteri( TEXTURE_CUBE_MAP, TEXTURE_MIN_FILTER, LINEAR );
gl.tex_parameteri( TEXTURE_CUBE_MAP, TEXTURE_MAG_FILTER, LINEAR );
if let Some( mip_range ) = mip_range
{
  gl.tex_parameteri( TEXTURE_CUBE_MAP, TEXTURE_BASE_LEVEL, mip_range.start as i32 );
  gl.tex_parameteri( TEXTURE_CUBE_MAP, TEXTURE_MAX_LEVEL, mip_range.end as i32 );
}
```
WebGL's `bind_texture`/`tex_parameteri` pair operates on whichever texture is *currently* bound to
the target -- by the time the `mip_range` block runs, `TEXTURE_CUBE_MAP` is bound to
`diffuse_texture` (the last rebind), not `specular_1_texture` (the first). The filter-setup block
bound 3 different textures to the single global `TEXTURE_CUBE_MAP`/`TEXTURE_2D` binding points in
sequence, and the `mip_range` block was textually adjacent to the *wrong* bind call.

## Why Not Caught

No test exercised `ibl::load`'s texture-parameter wiring prior to this bug -- texture-parameter
state can only be observed via a real WebGL2 context (`get_tex_parameter`), and no existing test
in this crate did so for the IBL loader specifically. The bug produces no panic and no compiler
warning; every current real call site's `mip_range` argument happens to make the misapplication
visually unobservable (see Impact).

## Fix Applied (2026-08-17)

**`src/webgl/loaders/ibl.rs`:** extracted the filter/mip-range block into its own `pub fn
ibl_texture_parameters_apply`, moving the `mip_range` application to sit immediately after
`specular_1_texture`'s own filter `tex_parameteri` calls -- while `specular_1_texture` is still
the texture bound to `TEXTURE_CUBE_MAP` -- instead of after the later rebind to
`diffuse_texture`. Pulling this block into its own function also makes the filter/mip-range wiring
unit-testable independent of any HDR file I/O. `mod_interface!`'s `own use` list extended to
export `ibl_texture_parameters_apply` alongside `load`.

**`tests/webgl/ibl.rs`** (new file): 1 new wasm32/browser `#[ wasm_bindgen_test( async ) ]`
function, `ibl_texture_parameters_apply_targets_mip_range_at_specular_1_not_diffuse`, calling
`ibl_texture_parameters_apply` directly against 3 freshly-created real textures with
`Some( 2..5 )` (deliberately distinct from both endpoints' spec defaults so a misapplication
cannot hide behind a coincidental match), then reading back
`TEXTURE_BASE_LEVEL`/`TEXTURE_MAX_LEVEL` via `get_tex_parameter` for both `specular_1_texture`
(expected `2`/`5`) and `diffuse_texture` (expected to stay at the WebGL2/ES3.0 spec defaults
`0`/`1000`).

## Verification

`longrun`-detached, from repo root:
- `cargo test --target wasm32-unknown-unknown -p renderer --test tests
  ibl_texture_parameters_apply` -- pre-fix (temporary direct-source-edit revert of the extraction
  back to the original inline ordering, real Firefox headless run): 1 failed, panicking at
  `assertion left == right failed: mip_range.start must land on specular_1_texture (..), got
  TEXTURE_BASE_LEVEL=0` (`left: 0, right: 2`). Post-fix (extraction restored): 1 passed, 0 failed
  (0.11s).
- `cargo clippy -p renderer --all-targets --all-features -- -D warnings`: clean (see final
  workspace-scoped verification run below).

## Generalized Version

**Broken assumption:** a sequence of `tex_parameteri` calls following a `bind_texture` call stays
correctly scoped to that call's texture for the entire remainder of a function, even after later
code rebinds the same target to a different texture. WebGL's `bind_texture`/`tex_parameteri` pair
always operates on whichever texture is *currently* bound to the target -- any `tex_parameteri`
call must stay textually adjacent to the `bind_texture` call for the texture it is meant to
configure, especially once more than one texture shares the same binding point within one
function. When a function configures multiple textures sharing a binding point, verify each
parameter-setting block sits directly after its own texture's bind call, not after whichever bind
call happens to be textually last.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-17 | filed + fixed + verified | Found during Group H review of `renderer::webgl::loaders::ibl`. Root cause: `ibl::load`'s `mip_range` clamp block sat after the *last* `TEXTURE_CUBE_MAP` rebind (to `diffuse_texture`) instead of staying adjacent to the *first* rebind (to `specular_1_texture`, the texture the parameter is documented to apply to and the only one with a real multi-level mip chain) -- silently misapplying the clamp to the wrong texture for any caller passing a non-degenerate range. Fixed by extracting the filter/mip-range block into its own `pub fn ibl_texture_parameters_apply` with the `mip_range` application moved immediately after `specular_1_texture`'s own filter calls. Verified via 1 new wasm32/browser unit test (confirmed fail pre-fix / pass post-fix via temporary revert-and-rerun against a real headless Firefox WebGL2 context), the full scoped suite, and clean clippy. Filed as BUG-260 after a fresh on-disk + repo-wide-grep re-scan confirmed no collision at time of filing (BUG-258 and BUG-259 both independently claimed by concurrent session actors during this session). Closed same-session (Tier 2 Dual-Role Self-Check). |
