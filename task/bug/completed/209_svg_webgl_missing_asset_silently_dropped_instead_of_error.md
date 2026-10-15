# BUG-209: `cmd_mesh`/`cmd_sprite` silently drop draws referencing a never-loaded asset instead of erroring (SVG + WebGL)

- **Severity:** Medium (silently wrong output -- a dropped draw call, not a crash or data
  corruption -- but violates the `Backend` trait's own established error contract)
- **state:** Completed
- **Affects:** Every caller of `tilemap_renderer`'s `adapter-svg` and `adapter-webgl` backends
  that submits a `Mesh`/`Sprite` command referencing an asset id `assets_load` was never asked to
  load, or (SVG only) one whose load was skipped -- e.g. a stale scene-compile output, a caller
  typo in an asset id, or a race between `assets_load` and `submit`.
- **Component:** `module/helper/tilemap_renderer` (`src/adapters/svg.rs`, `src/adapters/webgl.rs`)
- **repo_identity:** self
- **Filed:** 2026-08-16
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verification_date:** 2026-08-17
- **Related Bugs:** Same defect class independently occurring at 2 backend sites within one
  crate, filed under one ID per this repo's established one-defect-multiple-sites convention
  (BUG-181/BUG-193, BUG-165/BUG-207/BUG-208 precedent). Sibling of BUG-211 (same crate, same
  session, a related but distinct "silently accepts bad caller input instead of erroring" gap in
  SVG's batch-instance commands).

## Symptom

```rust
// pre-fix -- svg.rs, cmd_sprite
fn cmd_sprite( &mut self, s : &Sprite ) -> Result< (), RenderError >
{
  // no check that `s.sprite` was ever loaded
  let sprite = format!( "<use href=\"#sprite_{}\" ... />", s.sprite.inner(), ... );
  self.content.body_push( &sprite ); // dangling <use> reference; SVG viewer just fails to resolve it
  Ok( () )
}

// pre-fix -- webgl.rs, cmd_mesh
fn cmd_mesh( &self, m : &Mesh, viewport : [ f32; 2 ] ) -> Result< (), RenderError >
{
  let res = self.resources.borrow();
  if let Some( geom ) = res.geometry( m.geometry )
  {
    /* ... draw ... */
  }
  // else: falls through, nothing drawn, no error
  Ok( () )
}
```

Both backends' `Result`-returning command handlers had a path where a missing-asset lookup
silently produced `Ok(())` with no visible effect, instead of surfacing the miss.

## Impact

**Who is affected:** Every `adapter-svg`/`adapter-webgl` caller submitting a `Mesh`/`Sprite`
command whose referenced asset id was never loaded -- most concretely `tilemap_renderer`'s own
scene-compile consumers, where a stale/desynced asset id (e.g. after a scene edit removes an
asset but a cached render command stream still references it) silently renders nothing instead
of surfacing a diagnosable error, unlike the crate's own `native.rs`/`webgpu.rs` backends, which
already return `RenderError::MissingAsset` for the identical situation.

**What breaks:** `RenderError::MissingAsset` exists specifically for this case and is already the
established contract on 2 of the crate's 4 real backends -- SVG and WebGL silently diverged from
it. On SVG specifically, `cmd_sprite`'s pre-fix behavior was worse than a no-op: it *always*
emitted a `<use href="#sprite_N">` reference regardless of whether sprite `N` was ever defined,
producing a dangling reference the SVG viewer silently fails to resolve -- visually
indistinguishable from "nothing was ever there."

**Magnitude:** 2 functions in `svg.rs` (`cmd_mesh`, `cmd_sprite`) + 2 functions in `webgl.rs`
(`cmd_mesh`, `cmd_sprite`) -- 4 call sites, one shared root cause per backend.

**Entity Scope:** None -- a code-level defect.

## How Discovered

This session's systematic cross-backend audit of `tilemap_renderer`'s 5 `Backend` implementations
for `RenderError::MissingAsset` contract consistency, after confirming `native.rs`/`webgpu.rs`
already return it correctly for the same situation -- SVG and WebGL were the two backends found
diverging.

## Minimum Reproducible Example

```rust
// module/helper/tilemap_renderer/tests/svg_backend_test.rs -- pre-fix, this silently returned Ok
let mut svg = SvgBackend::new( RenderConfig::default() );
svg.assets_load( &empty_assets() ).unwrap(); // sprite id 0 never loaded
let result = svg.submit( &[ RenderCommand::Sprite( Sprite { sprite : ResourceId::new( 0 ), .. } ) ] );
// pre-fix: Ok(()) with a dangling <use href="#sprite_0"> written to output
// post-fix: Err(RenderError::MissingAsset(0))
```

**Verify Command** (<=3 lines, standalone):
```bash
cd module/helper/tilemap_renderer && cargo nextest run --features adapter-svg --test svg_backend_test -E 'test(sprite_command_missing_asset_returns_error) + test(mesh_command_missing_asset_returns_error)'
```

## Hypothesis Table

| ID | Hypothesis | State | Summary | Evidence |
|----|-----------|--------|---------|----------|
| H1 | SVG's `cmd_sprite`/`cmd_mesh` never checked whether the referenced asset was actually loaded before emitting output, unlike `native.rs`/`webgpu.rs`. | ✅ Root Cause (SVG) | `sprites_load` writes SVG `<symbol>` text directly with no queryable success/failure record; `cmd_sprite` unconditionally trusted every `s.sprite` id. Confirmed by direct read of both functions. | E1 |
| H2 | WebGL's `cmd_mesh`/`cmd_sprite` already have queryable resource maps (`res.geometry`/`res.sprite`) but never propagated a lookup miss as `Err`. | ✅ Root Cause (WebGL) | Both functions predate `submit`'s command loop propagating `?` and were never updated once every other fallible handler in the file adopted `Result` (confirmed via the functions' own `Fix(BUG-209)` comment, written after reading the surrounding file's history). | E2 |
| H3 | A `Mesh` command whose geometry was declared-but-unreadable, or declared-but-topologically-degenerate, should ALSO now error, since it also currently reaches `mesh_def_generate`'s `None` return. | ❌ Falsified | `geometry_on_missing_path_is_skipped_with_comment` and `mesh_triangle_strip_degenerate_no_output` (both pre-existing, both still passing) establish these two cases as an intentionally-designed silent no-op, not a caller-facing error -- conflating them with the "never declared at all" case would have been a regression, caught before it shipped by reading these two tests first. | E3 |

## Evidence Table

| # | Location | What it shows | Hypothesis |
|---|----------|---------------|------------|
| E1 | `module/helper/tilemap_renderer/src/adapters/svg.rs`, pre-fix `cmd_sprite`/`cmd_mesh` (direct read) | Neither function consulted any loaded-state bookkeeping before emitting SVG output referencing the asset id. | H1 ✅ |
| E2 | `module/helper/tilemap_renderer/src/adapters/webgl.rs`, pre-fix `cmd_mesh`/`cmd_sprite` (direct read) | `if let Some(geom) = res.geometry(...) { ... }` / equivalent for sprite, with no `else` arm returning `Err` -- the resource map already exists and is already queried, just not propagated. | H2 ✅ |
| E3 | `module/helper/tilemap_renderer/tests/svg_backend_test.rs`, `geometry_on_missing_path_is_skipped_with_comment` / `mesh_triangle_strip_degenerate_no_output` (pre-existing, read before writing the fix) | Both assert `Ok(())` for a geometry that was declared but failed to load / produced a degenerate mesh -- the intentional graceful-skip contract this fix must NOT break. | H3 ❌ |

## Root Cause

**SVG**: no bookkeeping existed to distinguish a loaded asset id from an unloaded one at draw
time. Fixed by adding `sprite_defs : IntSet<ResourceId<asset::Sprite>>` (populated only on a
successful `<symbol>` def) and `geometries_known : IntSet<ResourceId<asset::Geometry>>`
(populated unconditionally at `geometries_load` time, regardless of subsequent success) -- the
asymmetry between the two sets is deliberate, see Hypothesis H3.

**WebGL**: the resource lookup already existed and was already queried; the `if let Some(..) =
.. { .. }` shape (no `else`) simply never routed the `None` case to `Err`. Both functions predate
`submit`'s command loop adopting `?`-based `Result` propagation crate-wide and were never
retrofitted.

## Why Not Caught

**SVG**: `command_consistency_test.rs`'s own `svg_backend::sprite_command_returns_ok` test
asserted `Ok` for what was, at the time, an unloaded sprite -- the pre-fix behavior was locked in
by a test that never itself loaded a real sprite asset (see Refs: tests/ for how this was
corrected). 4 more pre-existing tests in `svg_backend_test.rs` (`sprite_white_tint_no_filter`,
`screen_space_sprite_renders_through_sprite_path`, `sprite_colored_tint_creates_filter`,
`two_tinted_sprites_get_distinct_filter_ids`) used a malformed 4-byte bitmap fixture for a
declared 16×16 Rgba8 image (needing 1024 bytes) -- `bitmap_to_png` silently rejected it,
`images_load` never stored the image, and `sprites_load` never registered the sprite, so these
tests were unknowingly exercising the exact dangling-reference bug this fix closes, invisible to
their own weak assertions (checking only `filter=` presence, not resolvability).

**WebGL**: no test existed for either function at all prior to this session --
`webgl_backend_test.rs` is capability-flag-only (no live `WebGl2RenderingContext`, see this
crate's own established no-live-GL-context testing boundary).

## Fix Location

`module/helper/tilemap_renderer/src/adapters/svg.rs`: new `geometries_known` field + accessor,
`sprite_defs` converted from `IntMap<_,()>` to `IntSet<_>`; `cmd_sprite` and `cmd_mesh` both check
membership before emitting output, returning `RenderError::MissingAsset` on a miss. `cmd_mesh`'s
check is scoped to "never declared at all" only -- it does not reuse `mesh_def_generate`'s `None`
return, which also legitimately covers the two graceful-skip cases named in Hypothesis H3.

`module/helper/tilemap_renderer/src/adapters/webgl.rs`: `cmd_mesh`/`cmd_sprite` both converted
from `if let Some(..) = .. { .. }` (implicit `Ok(())` fallthrough) to `let Some(..) = .. else {
return Err(RenderError::MissingAsset(..)) }`, matching `native.rs`/`webgpu.rs`'s existing pattern
verbatim.

## Prevention

**SVG** (4 new tests, `svg_backend_test.rs`): `sprite_command_missing_asset_returns_error`,
`mesh_command_missing_asset_returns_error`, plus `command_consistency_test.rs`'s
`svg_backend::sprite_command_returns_ok` corrected to load a real sprite first (previously
asserted `Ok` for an unloaded sprite -- correct under the old contract, wrong under the fixed
one), plus 4 pre-existing tests' malformed bitmap fixtures corrected from `vec![0u8;4]` to
`vec![255u8; 16*16*4]` so they exercise real, resolvable sprites as originally intended.

**WebGL**: no automated regression test -- `webgl_backend_test.rs` has zero live
`WebGl2RenderingContext` tests by this crate's own established design (a real GL context is
needed to construct `WebGlBackend`, which this workspace's test infrastructure cannot provide
natively; browser-based verification is this crate's documented alternative for such cases, e.g.
BUG-200's history). The WebGL-side fix is a direct textual match to the already-tested,
already-correct `native.rs`/`webgpu.rs` sibling pattern (same `let Some(..) = .. else { return
Err(..) }` shape), reducing but not eliminating the residual risk left by the missing live test.
Flagged here rather than silently left undocumented.

## Pitfall

A function whose fallback path collapses multiple distinct failure modes into one signal (here,
`mesh_def_generate`'s `None`, which legitimately covers "never declared", "declared but
unreadable", and "declared but degenerate") is dangerous to harden by turning that one signal into
a hard error -- the correct fix must first identify which sub-case actually needs to change,
using the existing test suite as ground truth for which sub-cases must remain non-error. Caught
here by reading `geometry_on_missing_path_is_skipped_with_comment` and
`mesh_triangle_strip_degenerate_no_output` BEFORE writing the fix, not after a red test surprised
it into existence.

## Generalized Version

**Broken assumption:** "every backend that returns `Result` from its command handlers already
propagates every failure through it."

**Confirmed general rule:** A `Result`-returning function signature is not proof every internal
lookup miss is actually routed to `Err` -- an `if let Some(..) = .. { .. }` with no `else`, or an
`else { return }` that discards the miss, silently degrades to `Ok(())` regardless of the
function's own signature. Auditing sibling implementations of the same trait method for
contract consistency (here: do all `Backend` impls return `MissingAsset` for the same situation?)
surfaces this class of drift that reading any one implementation in isolation cannot.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-16 | filed | Found via this session's cross-backend `MissingAsset` contract-consistency audit of `tilemap_renderer`'s 5 backends; SVG and WebGL both found diverging from the already-correct `native.rs`/`webgpu.rs` pattern. |
| 2026-08-16 | fixed | SVG: added `geometries_known`/`sprite_defs` bookkeeping, both `cmd_mesh`/`cmd_sprite` now check membership before emitting. WebGL: both functions converted to propagate the existing lookup miss as `Err`. A conflation regression in the SVG `cmd_mesh` fix (treating "declared but load-skipped/degenerate" as an error too) was caught by reading `geometry_on_missing_path_is_skipped_with_comment`/`mesh_triangle_strip_degenerate_no_output` before running the suite, and corrected before it ever reached a failing test. |
| 2026-08-16 | fixed (follow-on) | While validating the SVG fix, found `command_consistency_test.rs`'s `svg_backend::sprite_command_returns_ok` asserted the pre-fix contract (`Ok` for an unloaded sprite) -- corrected to load a real sprite first, mirroring `native_backend`'s own already-correct `loaded_sprite_assets()` pattern. Also found 4 pre-existing tests with malformed bitmap fixtures (`vec![0u8;4]` for a 16×16 Rgba8 image needing 1024 bytes) that were silently masking this exact bug via `bitmap_to_png`'s validation rejecting the malformed bytes; corrected to properly-sized fixtures. | 
| 2026-08-16 | fixed (clippy) | `sprite_defs`/`geometries_known` both flagged by `clippy::zero_sized_map_values` as `IntMap<_,()>`; converted both to `IntSet<_>` (already an established pattern in `src/assets.rs`), updating all call sites. |
| 2026-08-17 | verified | `cargo nextest run -p tilemap_renderer --all-features --no-fail-fast`: 144/144 passed, 0 skipped. `cargo clippy -p tilemap_renderer --all-targets --all-features -- -D warnings`: clean. WebGL half verified by direct source inspection and textual match to `native.rs`/`webgpu.rs`'s already-tested pattern only -- no live-`WebGl2RenderingContext` test exists in this crate to run (see Prevention section). |

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 6/6

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Completeness | — | 🟢 | All FI008 sections present, including an explicit Prevention note on the WebGL side's untested status rather than silently omitting it. | — |
| D2 | MRE Validity & Reproducibility | 🟠 | 🟢 | Confirming pass initially assumed `mesh_def_generate`'s `None` return could be reused directly for the "never declared" check. Adversarial pass specifically re-read `geometry_on_missing_path_is_skipped_with_comment`/`mesh_triangle_strip_degenerate_no_output` before trusting that assumption and found it would have broken both -- caught and corrected before any test ran red, not after. | Added scoped `geometries_known` check instead of reusing `mesh_def_generate`'s `None`. |
| D3 | Cross-Reference Integrity | — | 🟢 | Correctly filed as one ID spanning 2 backends (shared defect class, per BUG-181/193 precedent), correctly distinguished from BUG-211 (a related but distinct SVG-only "bad input silently accepted" gap in different functions). | — |
| D4 | Root Cause Quality | — | 🟢 | SVG and WebGL root causes independently confirmed via direct read of both files, not assumed to be identical -- WebGL's is a missing `else`, SVG's is genuinely missing bookkeeping; the fixes are shaped differently because the causes are different. | — |
| D5 | Execution Scope | — | 🟢 | Fix confined to the 4 affected functions plus the bookkeeping they needed; deliberately did NOT extend to `sprite_batch_create_draw`/`mesh_batch_create_draw`'s same malformed-fixture pattern (separate, unvalidated `cmd_draw_batch` code path) -- flagged as out of scope, not silently fixed or silently ignored. | — |
| D6 | Crate Scope Unity | — | 🟢 | Fix lives entirely in `tilemap_renderer`; no downstream crate changes needed. | — |

**Reproduced:** YES (SVG) -- pre-fix, `sprite_command_missing_asset_returns_error` /
`mesh_command_missing_asset_returns_error` fail (`Ok` returned); post-fix, both pass
(`Err(MissingAsset(0))`). WebGL confirmed via direct source inspection only (see Prevention). 2026-08-16/17.

## Refs: src/

| File | Change |
|------|--------|
| `module/helper/tilemap_renderer/src/adapters/svg.rs` | New `geometries_known : IntSet<ResourceId<asset::Geometry>>` field + `geometry_known()` accessor; `sprite_defs` converted `IntMap<_,()>` → `IntSet<_>`; `geometries_load` inserts into `geometries_known` unconditionally; `cmd_mesh`/`cmd_sprite` both check membership, returning `RenderError::MissingAsset` on a miss (full `Fix(BUG-209)` comment blocks). |
| `module/helper/tilemap_renderer/src/adapters/webgl.rs` | `cmd_mesh`/`cmd_sprite`: converted silent-fallthrough lookups to `let Some(..) = .. else { return Err(RenderError::MissingAsset(..)) }`, matching `native.rs`/`webgpu.rs` (full `Fix(BUG-209)` comment block). |

## Refs: tests/

| File | Change |
|------|--------|
| `module/helper/tilemap_renderer/tests/svg_backend_test.rs` | Added `sprite_command_missing_asset_returns_error`, `mesh_command_missing_asset_returns_error`; corrected 4 pre-existing tests' malformed bitmap fixtures (`vec![0u8;4]` → `vec![255u8; 16*16*4]`). |
| `module/helper/tilemap_renderer/tests/command_consistency_test.rs` | `svg_backend::sprite_command_returns_ok` corrected to load a real sprite via new `loaded_sprite_assets()` helper (mirrors `native_backend`'s existing one), instead of asserting `Ok` for an unloaded sprite. |
