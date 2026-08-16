# BUG-153: `mesh_def_generate`'s `TriangleList` arm panics on an index buffer whose length isn't a multiple of 3

- **Severity:** High (a panic reachable from caller-supplied asset data, not just a crash from
  internal-only trusted input -- any `GeometryAsset` with a malformed index buffer and
  `Topology::TriangleList` brings down the whole render call)
- **state:** Completed
- **Affects:** `SvgBackend::submit` (via `cmd_mesh`/`cmd_draw_batch` → `mesh_def_generate`) for
  any `GeometryAsset` whose `indices` buffer, once loaded, has a length not divisible by 3 and
  is rendered with `Topology::TriangleList`
- **Component:** `module/helper/tilemap_renderer` (`src/adapters/svg.rs`)
- **repo_identity:** self
- **Filed:** 2026-08-16
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/verified/
- **verification_date:** 2026-08-16
- **Related Bugs:** None (independent crate/code path from BUG-150/151/152's
  `embroidery_tools` work in the same review batch, task #91's `tilemap_renderer` pass).

## Symptom

```rust
use tilemap_renderer::assets::*;
use tilemap_renderer::commands::*;

let positions : &[ f32 ] = &[ 0.0, 0.0, 100.0, 0.0, 50.0, 100.0 ]; // 3 vertices
// 4 indices: one full triangle (0,1,2) plus a trailing partial triangle (just index 0).
let indices : Vec< u32 > = vec![ 0, 1, 2, 0 ];
let assets = Assets
{
  geometries : vec![ GeometryAsset
  {
    id : ResourceId::new( 0 ),
    positions : Source::Bytes( bytemuck::cast_slice( positions ).to_vec() ),
    uvs : None,
    indices : Some( Source::Bytes( bytemuck::cast_slice( &indices ).to_vec() ) ),
    data_type : DataType::U32,
  }],
  ..empty_assets()
};
svg.assets_load( &assets ).unwrap();
svg.submit( &[ RenderCommand::Mesh( Mesh { topology : Topology::TriangleList, /* .. */ }) ]).unwrap();
// Wrong (pre-fix):   panics -- "index out of bounds: the len is 4 but the index is 4"
// Correct (post-fix): does not panic; the leading full triangle still renders, the trailing
//                      partial triangle is silently skipped
```

## Impact

**Who is affected:** Any caller of `SvgBackend::submit` rendering a `Topology::TriangleList`
mesh whose `GeometryAsset.indices` buffer -- loaded via the fully public `Backend::assets_load`
API from arbitrary bytes (`geometries_load` performs no length/divisibility validation on the
raw index bytes) -- ends up with a length not divisible by 3.

**What breaks:** `mesh_def_generate`'s `TriangleList` arm chunks the index buffer with
`( 0..count ).step_by( 3 )` but never rounds `count` down to a multiple of 3 first. On the
final chunk of a buffer whose length isn't a multiple of 3, the inner loop's `v[ i + j ]` --
a direct, unchecked slice index into the index buffer itself -- runs past the buffer's end and
panics. This is distinct from (and upstream of) the *value*-validation the function already
performs two lines below (`geom.positions.get( v_idx * 2 )`, bounds-checked): that guards
against an in-range index pointing to an out-of-range *vertex*, not against the index-buffer
*lookup itself* running out of bounds.

**Magnitude:** A hard process panic (not silent corruption), reachable with caller/asset-data
control and no internal trust boundary in between -- `GeometryAsset`'s fields are `pub` and
explicitly documented as constructed via struct-literal syntax from outside the crate.

**Entity Scope:** None -- a code-level defect, not an operational-entity concern.

## How Discovered

Flagged during a background review pass over the `tilemap_renderer` crate (task #91). Deferred
as task #102 pending the `embroidery_tools` batch (BUG-150/151/152); confirmed and precisely
located via a dedicated read-only investigation (grep for `TriangleList`/`mesh_def_generate`,
full read of the function, its callers, and the existing malformed-index test coverage) before
filing.

## Minimum Reproducible Example

```bash
cd module/helper/tilemap_renderer && cargo test --test svg_backend_test geometry_index_count_not_multiple_of_three_no_panic 2>&1 | tail -10
```

**Expected** (post-fix):
```
test geometry_index_count_not_multiple_of_three_no_panic ... ok
```

**Actual** (pre-fix -- confirmed via in-place revert-test-restore against the real unfixed
code):
```
thread 'geometry_index_count_not_multiple_of_three_no_panic' panicked at src/adapters/svg.rs:1150:52:
index out of bounds: the len is 4 but the index is 4
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 132 filtered out
```

**Verify Command** (<=3 lines, standalone):
```bash
cd module/helper/tilemap_renderer && cargo test --test svg_backend_test geometry_index_count_not_multiple_of_three_no_panic
# ok = fixed; "index out of bounds" panic = bug present
```

## Hypothesis Table

| ID | Hypothesis | State | Summary | Evidence |
|----|-----------|--------|---------|----------|
| H1 | `TriangleList`'s inner loop indexes the index buffer directly (`v[ i + j ]`) with no bounds check, and `count` isn't rounded down to a multiple of 3, so a trailing partial triangle overruns the buffer. | ✅ Root Cause | Direct code reading confirmed the unchecked index; the MRE's captured failure (`the len is 4 but the index is 4`, exactly matching a 4-length buffer's trailing partial-triangle access) matches precisely. | E1, E2 |
| H2 | The bug is in the two position lookups (`geom.positions.get(...)`), not the index-buffer access above them. | ❌ Rejected | Those two lookups are already bounds-checked via `.get()` with a `valid = false; break;` fallback -- the panic site (line 1149, confirmed by the panic's own reported column) is strictly the `v[ i + j ]` expression that runs *before* either position lookup executes. | E2, E3 |
| H3 | `TriangleStrip`/`LineList`/`LineStrip` (the sibling arms in the same `match`) share the same defect. | ❌ Falsified | `TriangleStrip` has an explicit `if count < 3 { return None; }` guard and its sliding-window bound (`0..(count-2)`) keeps `i+j <= count-1` by construction; `LineList`/`LineStrip` only ever index `v[i]` for `i < count`, which is always in-bounds by the loop's own range. Read all three sibling arms directly to confirm. | E4 |

## Evidence Table

| # | Location | What it shows | Hypothesis |
|---|----------|---------------|------------|
| E1 | `src/adapters/svg.rs:1149` (pre-fix) | `let v_idx = idx.map_or( i + j, \| v \| v[ i + j ] as usize );` -- direct, unchecked slice index visible at the call site. | H1 ✅ |
| E2 | `-0018_longrun.log` (in-place revert-test-restore run against the real unfixed code) | Captured exact pre-fix panic: `index out of bounds: the len is 4 but the index is 4` at `svg.rs:1150:52` -- matches the predicted mechanism (a 4-index buffer's second chunk reaching index 4). | H1 ✅, H2 ❌ |
| E3 | `src/adapters/svg.rs:1150-1151` (unedited) | `geom.positions.get( v_idx * 2 )` / `.get( v_idx * 2 + 1 )`, both already bounds-checked with a `valid = false; break;` fallback -- confirms the pre-existing position-value validation is sound; the defect is strictly upstream of it. | H2 ❌ |
| E4 | `src/adapters/svg.rs:1157-1201` (unedited, `TriangleStrip`/`LineList`/`LineStrip` arms) | `TriangleStrip`'s `if count < 3 { return None; }` + `0..(count-2)` bound, and `LineList`/`LineStrip`'s `0..count` bound with only `v[i]` access, both structurally prevent the same overrun. | H3 ❌ |

## Root Cause

```
mesh_def_generate() -> TriangleList arm   (pre-fix)
  let count = idx.map_or( positions.len() / 2, <[u32]>::len );   // NOT rounded to a multiple of 3
  for i in ( 0..count ).step_by( 3 )
  {
    for j in 0..3
    {
      let v_idx = idx.map_or( i + j, | v | v[ i + j ] as usize );   // <-- unchecked; panics
                                                                     //     when i+j >= v.len()
      ...
    }
  }
```

On the final `step_by(3)` chunk, if `count % 3 != 0`, `i + j` can reach or exceed `v.len()`
before the loop body's own bounds-checked position lookups ever run.

## Why Not Caught

The one existing malformed-index test (`geometry_oob_index_no_panic`) used an index buffer
whose *length* was already a multiple of 3 (6 indices, 2 full triangles) with an out-of-*range*
*value* inside it (`99`) -- a different failure mode, already caught by the position lookups'
existing `.get()` guards. No existing test supplied an index buffer whose length itself isn't a
multiple of 3.

## Fix Location

`module/helper/tilemap_renderer/src/adapters/svg.rs`, `mesh_def_generate`'s `TriangleList` arm:

```rust
// before
let v_idx = idx.map_or( i + j, | v | v[ i + j ] as usize );
let Some( &x ) = geom.positions.get( v_idx * 2 )     else { valid = false; break; };
let Some( &y ) = geom.positions.get( v_idx * 2 + 1 ) else { valid = false; break; };

// after
let Some( v_idx ) = idx.map_or( Some( i + j ), | v | v.get( i + j ).map( | &v | v as usize ) )
else { valid = false; break; };
let Some( &x ) = geom.positions.get( v_idx * 2 )     else { valid = false; break; };
let Some( &y ) = geom.positions.get( v_idx * 2 + 1 ) else { valid = false; break; };
```

Changed the direct slice index to `.get()`, mapping a miss to the same `valid = false; break;`
fallback the two lines below already use -- so a short trailing chunk is skipped exactly like
an out-of-range vertex index already is, not introduced as a new error variant (this function
already models bad data via `Option`/silent-skip, e.g. `geometry_skip`, matching existing
convention rather than adding a `Result`/`RenderError`).

## Prevention

Added `geometry_index_count_not_multiple_of_three_no_panic` (`bug_reproducer(BUG-153)`) to
`tests/svg_backend_test.rs`: supplies a 4-index buffer (one full triangle plus a trailing
single index) and asserts `submit` does not panic and the valid leading triangle still renders.

## Pitfall

A function can have some of its malformed-input handling already correct (here, the two
position lookups) while an *earlier* step feeding into it (the index-buffer lookup itself)
remains unchecked -- reading only the parts of a function that already look defensive can miss
that the defensiveness doesn't extend all the way back to the actual external input boundary.
Every value that ultimately derives from caller-supplied bytes needs bounds-checking at the
point it's first extracted, not just at its eventual use.

## Generalized Version

**Broken assumption:** "if the values produced by an index lookup are validated downstream,
the lookup itself doesn't also need validating." False here -- `v_idx`'s *value* being
plausible (a `u32` cast to `usize`) says nothing about whether the *lookup that produced it*
(`v[ i + j ]`) was itself in-bounds; the two are independent bounds-checking obligations.

**Confirmed general rule:** when chunking a caller-controlled buffer into fixed-size groups
(`step_by(N)`), either round the iteration bound down to a multiple of `N` up front, or bounds-
check every access inside the inner loop -- a fixed chunk size does not guarantee the buffer's
length is a multiple of it.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-16 | filed | Flagged by a background review pass over `tilemap_renderer` (task #91); deferred as task #102, then located and confirmed via a dedicated read-only investigation before filing. |
| 2026-08-16 | fixed | Changed the `TriangleList` arm's direct `v[ i + j ]` index-buffer access to bounds-checked `.get()`, matching the fallback convention of the two position lookups immediately below it. |
| 2026-08-16 | verified | Added `geometry_index_count_not_multiple_of_three_no_panic` (written test-first against the unfixed code); confirmed it panics pre-fix with the exact predicted `index out of bounds: the len is 4 but the index is 4` and passes post-fix. Full crate suite (133 tests) + `cargo clippy --all-targets --all-features -- -D warnings` clean. |
| 2026-08-16 | completed | Independently re-verified in a later session: fix and regression test still present exactly as documented. `bug/readme.md`'s Closed Bugs table already declared this bug closed (same interrupted-closure pattern seen with BUG-150/151/152) but the file had not yet been moved out of `verified/` -- corrected here. State → Completed. |

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 8/8

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Completeness | 🟢 | 🟢 | All 12 FI008 sections present. | — |
| D2 | MRE Validity & Reproducibility | 🟢 | 🟢 | Confirming pass wrote the test-first against unfixed code, captured the exact panic message and location; adversarial pass specifically checked whether the fix's `.get()` guard could itself change output for the already-passing `geometry_oob_index_no_panic`/`mesh_triangle_list`/`geometry_u8_indices_loaded` tests (all length-clean buffers, so the new guard is a no-op for them) -- confirmed via the full 133-test pass, not assumed. | — |
| D3 | Cross-Reference Integrity | 🟢 | 🟢 | Independent of BUG-150/151/152 (different crate, different code path); no cross-dependency to note. | — |
| D4 | Root Cause Quality | 🟢 | 🟢 | Root cause backed by direct code reading (unchecked slice index) plus a captured real panic matching the predicted mechanism and location exactly. | — |
| D5 | Execution Scope | 🟢 | 🟢 | Read all 3 sibling `match` arms (`TriangleStrip`, `LineList`/`LineStrip`) to confirm they don't share the defect (H3), rather than assuming from one arm's fix that siblings are also safe or also broken. | — |
| D6 | Crate Scope Unity | 🟢 | 🟢 | Only `tilemap_renderer` src+test+bug file touched. | — |
| D7 | Crate Locality | 🟢 | 🟢 | Fix local to one expression inside one match arm; no signature/field change. | — |
| D8 | Crate Single Responsibility | 🟢 | 🟢 | No new public surface; existing function's "skip malformed data, don't panic" contract (already followed by its own position-lookup guards) extended to cover the index-buffer lookup too. | — |

**Reproduced:** YES -- `geometry_index_count_not_multiple_of_three_no_panic` was written and run
against the unfixed function first (test-first), producing the exact predicted panic
(`index out of bounds: the len is 4 but the index is 4`); applying the fix and re-running
returned the test to passing, and the full crate suite (133 tests) + `cargo clippy
--all-targets --all-features -- -D warnings` remained clean, 2026-08-16.

## Refs: src/

| File | Change |
|------|--------|
| `module/helper/tilemap_renderer/src/adapters/svg.rs` | `mesh_def_generate`'s `TriangleList` arm: changed the direct `v[ i + j ]` index-buffer access to bounds-checked `.get()`. `Fix(BUG-153)`/`Root cause`/`Pitfall` comment added. |

## Refs: tests/

| File | Change |
|------|--------|
| `module/helper/tilemap_renderer/tests/svg_backend_test.rs` | Appended `geometry_index_count_not_multiple_of_three_no_panic` (`bug_reproducer(BUG-153)`, 5-section doc comment). |
