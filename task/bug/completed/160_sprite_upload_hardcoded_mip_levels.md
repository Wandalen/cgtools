# BUG-160: `sprite_upload` hardcodes `texStorage3D`'s mip level count to 8, silently failing to allocate storage for sprites smaller than 128x128

- **Severity:** High (silent failure -- `tex_storage_3d` raises `INVALID_OPERATION`, which is
  never surfaced as a JS exception or `Result::Err` by wasm-bindgen, so the texture array is
  never actually allocated and every subsequent `tex_sub_image_3d` call silently no-ops against
  it, with no error signal anywhere in the call chain)
- **state:** Completed
- **Affects:** `sprite_upload` -- any caller whose sprite sheet's per-sprite dimensions
  (`sprite_width`/`sprite_height`) are both `< 128`
- **Component:** `module/min/minwebgl` (`src/texture/d2.rs`)
- **repo_identity:** self
- **Filed:** 2026-08-16
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/verified/
- **verification_date:** 2026-08-16
- **Related Bugs:** Co-located in the same function (`sprite_upload`) as BUG-161, but a distinct
  root cause (mip-level math vs. a divide-by-zero) -- filed and fixed separately.

## Symptom

```rust
// Rust mirror of the spec constraint (see MRE -- this crate has no shader-execution test
// harness for tex_storage_3d itself, so the regression is captured via an independent
// spec-formula comparison against the pure extracted helper).
// pre-fix: `levels` was hardcoded to 8 regardless of sprite_width/sprite_height.
let levels = 8; // valid only when max(sprite_width, sprite_height) >= 128
// WebGL2 spec: levels <= floor(log2(max(width,height))) + 1
// e.g. sprite_width=32, sprite_height=32: max valid levels = floor(log2(32))+1 = 6, not 8
// tex_storage_3d(..., levels=8, width=32, height=32, ...) -> INVALID_OPERATION, no storage allocated
```

## Impact

**Who is affected:** Any caller of `sprite_upload` whose sprite sheet uses per-sprite dimensions
smaller than 128x128 in both axes. The crate's only real caller
(`examples/minwebgl/sprite_animation`) happens to use exactly 128x128 sprites -- the precise
boundary value where the hardcoded `8` is still spec-valid -- so this has never fired in this
repo, but any other caller with smaller sprites hits it immediately.

**What breaks:** `gl.tex_storage_3d(..., levels=8, ...)` raises `INVALID_OPERATION` for any
`max(sprite_width, sprite_height) < 128` and allocates no storage. WebGL errors are not surfaced
as JS exceptions or `Result::Err` by wasm-bindgen -- nothing in `sprite_upload` calls
`gl.get_error()` to detect this -- so the function returns `Ok` with a texture handle that has no
actual backing storage; every subsequent `tex_sub_image_3d` call in the per-sprite loop silently
no-ops against it.

**Magnitude:** Silent and total for affected inputs -- no error, no panic, just a texture that
renders as empty/garbage with zero diagnostic signal.

**Entity Scope:** None -- a code-level defect.

## How Discovered

Independent re-derivation: dispatched a background Explore agent to read 100% of `minwebgl`'s
`src/` (36 files) from scratch. It flagged the hardcoded `levels = 8` in `sprite_upload` as a
candidate; independently re-verified by reading `src/texture/d2.rs` directly, confirming the
hardcoded value and cross-checking the WebGL2/GLES3.0 `texStorage3D` spec constraint
(`levels <= floor(log2(max(width,height))) + 1`) and the one real call site's exact 128x128
sprite size.

## Minimum Reproducible Example

```bash
cd module/min/minwebgl && cargo test -p minwebgl --test tests sprite_upload_test::mip_levels_stays_within_the_spec_limit_for_common_sub_128_sprite_sizes 2>&1 | tail -6
```

**Expected** (post-fix):
```
test sprite_upload_test::mip_levels_stays_within_the_spec_limit_for_common_sub_128_sprite_sizes ... ok
```

**Actual** (pre-fix -- the hardcoded `8` fails the same spec-formula comparison this test makes
for every one of its 5 sub-128 cases, e.g. `mip_levels_for_dimensions(32,32)` pre-fix would
return the hardcoded `8` against a spec-max of `6`):
```
assertion `left == right` failed: mip_levels_for_dimensions(32,32) must equal the spec's own max level count
  left: 8
 right: 6
```

**Verify Command** (<=3 lines, standalone):
```bash
cd module/min/minwebgl && cargo test -p minwebgl --test tests sprite_upload_test::mip_levels
# 3 "ok" = fixed; assertion failure comparing computed vs spec_max_levels = bug present
```

## Hypothesis Table

| ID | Hypothesis | State | Summary | Evidence |
|----|-----------|--------|---------|----------|
| H1 | `sprite_upload`'s `tex_storage_3d` call hardcodes `levels` to `8`, which violates the WebGL2/GLES3.0 spec constraint for any sprite smaller than 128x128, and this failure is invisible because WebGL errors aren't surfaced as `Result::Err`. | ✅ Root Cause | Read `src/texture/d2.rs` directly: `gl.tex_storage_3d(GL::TEXTURE_2D_ARRAY, 8, GL::RGBA8, ...)` -- `8` is a literal, not derived from `sprite_width`/`sprite_height`. Cross-checked the spec formula and confirmed `8` is only valid when `max(width,height) >= 128`. | E1, E2 |
| H2 | The crate's real caller already exercises a range of sprite sizes, so this would have surfaced as a visible rendering bug already. | ❌ Rejected | `examples/minwebgl/sprite_animation` is the only real call site and uses exactly 128x128 sprites -- the exact boundary where `8` levels is still valid -- so no existing exercised input path ever triggers `INVALID_OPERATION`. | E3 |

## Evidence Table

| # | Location | What it shows | Hypothesis |
|---|----------|---------------|------------|
| E1 | `src/texture/d2.rs` (pre-fix, unedited) | `gl.tex_storage_3d(GL::TEXTURE_2D_ARRAY, 8, GL::RGBA8, dim_as_i32(sprite_sheet.sprite_width), dim_as_i32(sprite_sheet.sprite_height), dim_as_i32(sprite_sheet.amount))` -- levels is a bare literal `8`. | H1 ✅ |
| E2 | WebGL2/GLES3.0 spec (`texStorage3D`) | `levels <= floor(log2(max(width,height))) + 1` -- for `width=height=32`: `floor(log2(32))+1 = 5+1 = 6 < 8`, violating the constraint. | H1 ✅ |
| E3 | `examples/minwebgl/sprite_animation` (grep for the crate's only `sprite_upload` call site) | Sprite sheet configured with 128x128 per-sprite dimensions -- `floor(log2(128))+1 = 7+1 = 8`, exactly matching the hardcoded value; no other call site exists in this repo. | H2 ❌ |

## Root Cause

```rust
// before
gl.tex_storage_3d
(
  GL::TEXTURE_2D_ARRAY,
  8, // hardcoded, valid only when max(sprite_width, sprite_height) >= 128
  GL::RGBA8,
  dim_as_i32( sprite_sheet.sprite_width ),
  dim_as_i32( sprite_sheet.sprite_height ),
  dim_as_i32( sprite_sheet.amount )
);
```

A hardcoded constant that happens to match the spec-derived formula's value at the one input
actually exercised in this repo (128 -> 8).

## Why Not Caught

The only real caller's exact sprite size (128x128) is the precise boundary value where 8 levels
is still valid, coincidentally masking the bug for every dimension actually exercised. WebGL
errors are also not surfaced as JS exceptions/`Result::Err` by wasm-bindgen, so nothing short of
an explicit `gl.get_error()` call (never made in this function) could have caught this live
either.

## Fix Location

`module/min/minwebgl/src/texture/d2.rs`.

```rust
// after: extracted into a standalone pure fn, computed from the real dimensions
pub fn mip_levels_for_dimensions( width : u32, height : u32 ) -> u32
{
  width.max( height ).max( 1 ).ilog2() + 1
}

// call site
let levels = dim_as_i32( mip_levels_for_dimensions( sprite_sheet.sprite_width, sprite_sheet.sprite_height ) );
gl.tex_storage_3d( GL::TEXTURE_2D_ARRAY, levels, GL::RGBA8, ... );
```

## Prevention

Added `tests/sprite_upload_test.rs` (new file, includes `bug_reproducer(BUG-160)`): a boundary
happy-path case (128x128 still yields 8), a regression test independently computing the
spec-mandated max level count for 5 common sub-128 sprite sizes and asserting the extracted
function agrees (asserting each result also differs from the pre-fix hardcoded 8, so the test is
a genuine spec check and would have failed pre-fix), and a zero-dimension panic-safety case.

## Pitfall

A hardcoded constant that happens to match a spec-derived formula's value at one particular
input (128 -> 8) reads as correct until a different, equally ordinary input is tried.

## Generalized Version

**Broken assumption:** "the one caller I have today exercises the general case." False when that
caller's input happens to sit exactly on a boundary value where a hardcoded shortcut coincides
with the correct spec-derived answer -- the shortcut looks universally correct until a different,
equally valid input is tried.

**Confirmed general rule:** when a GL parameter has a spec-mandated formula relating it to other
call parameters (here, `levels` to `width`/`height`), never hardcode it even when the current
caller's dimensions make the hardcoded value coincidentally correct -- compute it from the real
inputs, and check whether the single existing caller's values happen to sit on a boundary before
trusting that they've exercised the general case.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-16 | filed | Flagged by an independent Explore-agent re-derivation of minwebgl's `src/` (36 files, task #93); independently re-verified by reading `src/texture/d2.rs` directly and cross-checking the WebGL2 spec formula plus the one real caller's exact sprite size. |
| 2026-08-16 | fixed | Extracted `mip_levels_for_dimensions`, replacing the hardcoded `8` with a call computed from the real `sprite_width`/`sprite_height`. |
| 2026-08-16 | verified | Added `tests/sprite_upload_test.rs` (3 tests covering this bug), independently computing the spec formula rather than calling the function under test for the regression case. Scoped crate suite (13 tests) + `cargo clippy -p minwebgl --all-targets --all-features -- -D warnings` clean. |

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 8/8

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Completeness | 🟢 | 🟢 | All FI008 sections present. | — |
| D2 | MRE Validity & Reproducibility | 🟢 | 🟢 | Confirming pass wrote the test's spec-formula comparison independently of the function under test; adversarial pass hand-recomputed the spec formula for all 5 test cases plus the 128x128 boundary case and confirmed each against the extracted function's actual output. | — |
| D3 | Cross-Reference Integrity | 🟢 | 🟢 | Independent of BUG-159/BUG-161 (same investigation batch; BUG-161 shares this function but a distinct root cause) -- no cross-dependency. | — |
| D4 | Root Cause Quality | 🟢 | 🟢 | Root cause backed by direct source reading, the WebGL2 spec formula, and a grep confirming the one real caller's exact 128x128 boundary-masking dimensions. | — |
| D5 | Execution Scope | 🟢 | 🟢 | Only the `levels` computation in `sprite_upload`'s `tex_storage_3d` call site changed; rest of the function untouched. | — |
| D6 | Crate Scope Unity | 🟢 | 🟢 | Only `minwebgl` src + test + bug file touched. | — |
| D7 | Crate Locality | 🟢 | 🟢 | Fix is a function extraction plus a one-line call-site substitution; no signature/API change to `sprite_upload`. | — |
| D8 | Crate Single Responsibility | 🟢 | 🟢 | New public fn (`mip_levels_for_dimensions`) has one responsibility (spec-formula computation), exported as a plain top-level `pub fn` matching `texture/d2.rs`'s existing non-`mod_interface` convention. | — |

**Reproduced:** YES -- the regression test's independently-computed spec formula was confirmed to
disagree with the pre-fix hardcoded `8` for all 5 sub-128 test cases (e.g. 32x32: spec max 6 vs.
hardcoded 8); the extracted, dimension-derived function now agrees with the spec formula in
every case. Scoped crate suite (13 tests) + `cargo clippy -p minwebgl --all-targets --all-features
-- -D warnings` clean, 2026-08-16.

## Refs: src/

| File | Change |
|------|--------|
| `module/min/minwebgl/src/texture/d2.rs` | Extracted `mip_levels_for_dimensions`; `sprite_upload`'s `tex_storage_3d` call site now computes `levels` from it instead of a hardcoded `8` (full `Fix(BUG-160)` root cause/pitfall comment). |

## Refs: tests/

| File | Change |
|------|--------|
| `module/min/minwebgl/tests/sprite_upload_test.rs` | New file: includes the boundary happy-path case and the `bug_reproducer(BUG-160)` sub-128 regression test (file shared with BUG-161's tests). |
| `module/min/minwebgl/tests/readme.md` | Added Responsibility Table row for `sprite_upload_test.rs`. |
