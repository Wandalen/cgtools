# BUG-211: SVG's `cmd_set_sprite_instance`/`cmd_set_mesh_instance` silently no-op on an out-of-bounds instance index instead of erroring

- **Severity:** Medium (silently accepts and discards invalid caller input instead of surfacing a
  diagnosable error — masks caller bugs rather than corrupting data)
- **state:** Completed
- **Affects:** Every `adapter-svg` caller updating a batch instance by index (`SetSpriteInstance`
  / `SetMeshInstance` commands) with an index that is out of bounds for the currently-bound
  batch's instance array — e.g. a caller tracking instance count independently and drifting out
  of sync after a `RemoveInstance`.
- **Component:** `module/helper/tilemap_renderer` (`src/adapters/svg.rs`)
- **repo_identity:** self
- **Filed:** 2026-08-16
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verification_date:** 2026-08-17
- **Related Bugs:** Found in the same session as BUG-209/BUG-210 (same crate, same audit pass).
  Sibling of BUG-209 — both are `RenderError` contract-consistency gaps in `svg.rs`, but distinct
  functions and distinct root causes (BUG-209: asset-existence never checked; BUG-211: index
  bounds silently absorbed into an unrelated no-op branch). `webgl.rs`'s equivalent functions
  already had the correct out-of-bounds guard prior to this fix — cited as the established sibling
  contract this fix brings SVG into line with.

## Symptom

```rust
// pre-fix -- svg.rs, cmd_set_sprite_instance (cmd_set_mesh_instance had the identical shape)
fn cmd_set_sprite_instance( &mut self, c : &SetSpriteInstance ) -> Result< (), RenderError >
{
  if let Some( SvgBatch::Sprite( batch ) ) = self.batches.get_mut( &self.bound_batch? )
    && let Some( slot ) = batch.instances.get_mut( c.index as usize )
  {
    *slot = c.instance;
  }
  // else: falls through silently -- covers "no batch bound", "wrong batch variant",
  // AND "index out of bounds" with the same Ok(()) fallthrough
  Ok( () )
}
```

An out-of-bounds `c.index` silently produced `Ok(())` with no visible effect, indistinguishable
from the two legitimate no-op cases (no batch currently bound; bound batch is the wrong variant).

## Impact

**Who is affected:** Any `adapter-svg` caller issuing `SetSpriteInstance`/`SetMeshInstance` with
an index beyond the bound batch's current instance count — most concretely a caller that tracks
instance indices independently of the renderer (e.g. after a `RemoveInstance` shifts subsequent
indices via `swap_remove`) and issues a stale index.

**What breaks:** `webgl.rs`'s sibling `cmd_set_sprite_instance`/`cmd_set_mesh_instance` already
return `RenderError::BackendError` for exactly this situation — SVG silently diverged from its
own crate's established `Backend` trait contract, absorbing a caller bug into an invisible no-op
instead of surfacing it.

**Magnitude:** 2 functions (`cmd_set_sprite_instance`, `cmd_set_mesh_instance`), one shared root
cause and fix shape.

**Entity Scope:** None — a code-level defect.

## How Discovered

Same session's cross-backend `RenderError` contract-consistency audit that found BUG-209 —
having confirmed `webgl.rs`'s `cmd_set_sprite_instance`/`cmd_set_mesh_instance` already guard
against an out-of-bounds index, checking `svg.rs`'s sibling functions found the guard missing
there.

## Minimum Reproducible Example

```rust
// module/helper/tilemap_renderer/tests/svg_backend_test.rs -- pre-fix, this silently returned Ok
let mut svg = SvgBackend::new( RenderConfig::default() );
svg.assets_load( &loaded_sprite_assets() ).unwrap();
svg.submit( &[ RenderCommand::CreateSpriteBatch( CreateSpriteBatch { id : ResourceId::new( 0 ), capacity : 1, .. } ) ] ).unwrap();
svg.submit( &[ RenderCommand::BindBatch( BindBatch { id : ResourceId::new( 0 ) } ) ] ).unwrap();
let result = svg.submit( &[ RenderCommand::SetSpriteInstance( SetSpriteInstance { index : 99, .. } ) ] );
// pre-fix: Ok(()) -- index 99 silently discarded, no instance array of that size ever existed
// post-fix: Err(RenderError::BackendError(..))
```

**Verify Command** (<=3 lines, standalone):
```bash
cd module/helper/tilemap_renderer && cargo nextest run --features adapter-svg --test svg_backend_test -E 'test(set_sprite_instance_out_of_bounds_returns_error) + test(set_mesh_instance_out_of_bounds_returns_error)'
```

## Hypothesis Table

| ID | Hypothesis | State | Summary | Evidence |
|----|-----------|--------|---------|----------|
| H1 | `cmd_set_sprite_instance`/`cmd_set_mesh_instance`'s chained `if let ... && ...` conflates 3 distinct conditions ("no batch bound", "wrong batch variant bound", "index out of bounds") into one silent `Ok(())` fallthrough. | ✅ Root Cause | Confirmed by direct read: `batch.instances.get_mut(c.index as usize)` returning `None` (out-of-bounds) is structurally indistinguishable, in the chained condition, from `self.batches.get_mut(...)` returning `None` (no/wrong batch). | E1 |
| H2 | `webgl.rs`'s sibling functions already correctly distinguish these cases, so the fix shape should mirror them rather than invent a new one. | ✅ Confirmed | Direct read of `webgl.rs`'s `cmd_set_sprite_instance`/`cmd_set_mesh_instance` shows an explicit `let Some(..) = .. else { return Ok(()) }` for the two legitimate no-ops, followed by an explicit bounds check returning `Err(RenderError::BackendError(..))`. | E2 |

## Evidence Table

| # | Location | What it shows | Hypothesis |
|---|----------|---------------|------------|
| E1 | `module/helper/tilemap_renderer/src/adapters/svg.rs`, pre-fix `cmd_set_sprite_instance`/`cmd_set_mesh_instance` (direct read) | Single chained `if let ... && ...` with one `else`-equivalent fallthrough covering 3 semantically distinct conditions. | H1 ✅ |
| E2 | `module/helper/tilemap_renderer/src/adapters/webgl.rs`, `cmd_set_sprite_instance`/`cmd_set_mesh_instance` (direct read, unchanged) | Already splits the two legitimate no-ops from an explicit out-of-bounds `Err(RenderError::BackendError(..))` — the exact shape this fix brings SVG into line with. | H2 ✅ |

## Root Cause

The chained `if let Some(batch) = ... && let Some(slot) = batch.instances.get_mut(...)` pattern
treats "batch not found / wrong variant" and "index out of bounds within a correctly-found
batch" as the same failure mode, because both paths reach the same implicit `else` fallthrough.
The first two are legitimate no-ops (a caller may reasonably query/set against a currently-unbound
or differently-typed batch during normal command-stream construction); the third is a genuine
caller bug that `webgl.rs`'s sibling implementation already surfaces as an error.

## Why Not Caught

No test previously exercised an out-of-bounds index against a *correctly bound, correctly typed*
batch — existing tests covered "no batch bound" and "wrong batch variant" (both correctly
asserting `Ok`), but none constructed the specific combination of "batch bound, correct variant,
index beyond capacity."

## Fix Location

`module/helper/tilemap_renderer/src/adapters/svg.rs`: both `cmd_set_sprite_instance` and
`cmd_set_mesh_instance` split the single chained condition into an explicit `let Some(batch) =
self.batches.get_mut(&self.bound_batch?) else { return Ok(()) }` (no batch bound — legitimate
no-op) followed by an explicit variant match returning `Ok(())` on a mismatch (legitimate no-op),
then an explicit `if c.index as usize >= batch.instances.len() { return
Err(RenderError::BackendError(...)); }` before the mutation — matching `webgl.rs`'s already-correct
sibling shape verbatim.

## Prevention

2 new tests added, `module/helper/tilemap_renderer/tests/svg_backend_test.rs`:
`set_sprite_instance_out_of_bounds_returns_error`, `set_mesh_instance_out_of_bounds_returns_error`
— each binds a real batch of a known small capacity, then issues a `SetSpriteInstance`/
`SetMeshInstance` with an index past that capacity and asserts `Err(RenderError::BackendError(..))`.
2 pre-existing tests (covering "no batch bound" and "wrong batch variant") re-verified to still
assert `Ok(())` post-fix, confirming the two legitimate no-op paths were preserved exactly.

## Pitfall

A chained `if let ... && let ... = ...` condition reads as one logical gate but frequently hides
multiple, semantically distinct failure reasons behind a single fallthrough — exactly the shape
that makes "no batch bound" and "index out of bounds" indistinguishable at the call site despite
being wildly different in caller-actionability (one is routine, the other signals a caller bug).
Splitting into named, individually-`return`-ing guards makes each condition's intended
distinctness visible in the code itself, not just in a comment.

## Generalized Version

**Broken assumption:** "a chained `if let ... && ...` that reaches the same fallthrough for
multiple sub-conditions treats them all with equally-correct severity."

**Confirmed general rule:** When a multi-condition guard's sub-conditions have different correct
outcomes (some legitimately silent, one that should error), the guard must be structurally split
so each sub-condition's outcome is independently visible and independently testable — a shared
fallthrough silently downgrades every sub-condition to the most lenient one's behavior.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-16 | filed | Found via this session's cross-backend `RenderError` contract-consistency audit; `webgl.rs`'s already-correct sibling functions used as the reference contract. |
| 2026-08-16 | fixed | Both `cmd_set_sprite_instance`/`cmd_set_mesh_instance` split into explicit `let Some(..) = .. else { return Ok(()) }` guards for the two legitimate no-ops, followed by an explicit out-of-bounds `Err(RenderError::BackendError(..))` check, matching `webgl.rs` verbatim. 2 new regression tests added. |
| 2026-08-17 | verified | `cargo nextest run -p tilemap_renderer --all-features --no-fail-fast`: 144/144 passed, 0 skipped, including both new tests and the 2 pre-existing legitimate-no-op tests re-confirmed unchanged. `cargo clippy -p tilemap_renderer --all-targets --all-features -- -D warnings`: clean. |

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 6/6

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Completeness | — | 🟢 | All FI008 sections present. | — |
| D2 | MRE Validity & Reproducibility | — | 🟢 | MRE constructs the exact 3-way condition split (bound + correct variant + out-of-bounds index) the fix targets, distinct from the two pre-existing no-op cases it must not disturb. | — |
| D3 | Cross-Reference Integrity | 🟠 | 🟢 | Confirming pass initially considered folding this into BUG-209 (same file, same audit pass). Adversarial pass checked root-cause identity against BUG-209's actual defect (asset-existence never checked at all) versus this bug's (index bounds absorbed into an unrelated no-op branch) and confirmed they are distinct defects in distinct functions with distinct fix shapes — correctly kept as separate IDs, cross-referenced rather than merged. | Filed as a distinct ID (BUG-211) instead of folding into BUG-209. |
| D4 | Root Cause Quality | — | 🟢 | Backed by direct read of both the pre-fix chained condition and `webgl.rs`'s already-correct sibling shape — the fix is a direct structural match, not an invented alternative. | — |
| D5 | Execution Scope | — | 🟢 | Fix confined to the 2 affected functions; the two legitimate no-op branches (no batch bound, wrong variant) deliberately preserved unchanged and re-verified via their pre-existing tests, not collapsed into the new error path. | — |
| D6 | Crate Scope Unity | — | 🟢 | Fix lives entirely in `tilemap_renderer`; no downstream crate changes needed. | — |

**Reproduced:** YES — pre-fix, `set_sprite_instance_out_of_bounds_returns_error` /
`set_mesh_instance_out_of_bounds_returns_error` fail (`Ok` returned for an out-of-bounds index);
post-fix, both pass (`Err(BackendError(..))`). 2026-08-16/17.

## Refs: src/

| File | Change |
|------|--------|
| `module/helper/tilemap_renderer/src/adapters/svg.rs` | `cmd_set_sprite_instance`/`cmd_set_mesh_instance`: chained `if let ... && ...` split into explicit no-op guards plus an explicit out-of-bounds `Err(RenderError::BackendError(..))` check, matching `webgl.rs` (full `Fix(BUG-211)` comment blocks). |

## Refs: tests/

| File | Change |
|------|--------|
| `module/helper/tilemap_renderer/tests/svg_backend_test.rs` | Added `set_sprite_instance_out_of_bounds_returns_error`, `set_mesh_instance_out_of_bounds_returns_error`. |
