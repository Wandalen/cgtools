# BUG-245: glTF loader's material-variation cache is passed by shared reference and can never be
written to -- every primitive sharing a material + vertex-defines combination gets its own
independent clone instead of the intended shared instance

- **Severity:** Medium (not a crash/panic, but breaks the documented sharing invariant of
  `SharedMaterial = Rc< RefCell< Box< dyn Material > > > `: joint runtime mutation of a material
  no longer propagates to every primitive that was meant to share it, and every primitive using a
  distinct-but-defines-matching material clone silently wastes a GPU program/uniform-upload slot
  that should have been shared)
- **state:** Completed
- **Affects:** `gltf.rs`'s `primitive_material_resolve`, called once per primitive by
  `meshes_create`, for every glTF asset loaded through `renderer::webgl::loaders::gltf::load`
- **Component:** `module/helper/renderer` (`src/webgl/loaders/gltf.rs`)
- **repo_identity:** self
- **Filed:** 2026-08-17
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/ (self)
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/ (self)
- **verification_date:** 2026-08-17
- **Fixed:** 2026-08-17
- **Accepted By:** self -- same-session Tier 2 Dual-Role Self-Check, no separate acceptance actor
  (same-sandbox `tsk` actor-guard blocks `.acceptance_pass` in this environment)

## Symptom

`primitive_material_resolve` is supposed to reuse an existing material clone whenever another
primitive already resolved the same glTF source material with the same vertex defines --
`material_variation_map : &FxHashMap< uuid::Uuid, Vec< SharedMaterial > > ` is exactly the cache
meant to make that possible. In practice every call missed the cache: the map's per-material
entries, seeded empty by `materials_create`, were never populated, so every primitive got a freshly
`dyn_clone`d, independent `SharedMaterial` -- never the shared instance the cache lookup was
supposed to find.

## Impact

**Who is affected:** Any consumer of `renderer::webgl::loaders::gltf::load` on a glTF asset where
two or more primitives reference the same material with the same vertex-defines combination (the
overwhelmingly common case -- most glTF assets reuse materials across many primitives/meshes).

**What breaks:** `SharedMaterial`'s whole point is `Rc< RefCell< _ > > ` identity sharing -- code
that mutates one primitive's material expecting every primitive using "the same" material to see
the change (a documented, intended capability of the shared-material design) silently only affects
the one clone it holds. Independently, `GLTF.materials` (`used_materials`) ends up with many more
distinct entries than the source asset's actual material count, each a separate GPU program/
uniform-upload target instead of a shared one.

**Entity Scope:** `None` -- source-level cache-population defect, not entity directory instances.

## How Discovered

During this session's `renderer` crate scout (task #174), direct review of `gltf.rs`'s loader
functions (following BUG-243/BUG-244 in the same scouting session) traced `material_variation_map`'s
full lifecycle by hand: seeded in `materials_create` (every entry an empty `Vec::new()`), read in
`primitive_material_resolve` via `.get( &gltf_material_id )`, but the only `Vec::push` onto it
anywhere in the crate did not exist -- confirmed via a workspace-wide grep for every call site of
`materials_create`/`meshes_create`/`primitive_material_resolve` (3 sites total, all within
`gltf.rs`, all now consistent with the fix).

## Minimum Reproducible Example

The defect lives in mutation/aliasing logic, not GPU-dependent math, and glTF material construction
(`PbrMaterial::new`) needs a live `WebGl2RenderingContext` this crate has no native (non-browser)
path to obtain (confirmed via `gltf_loader_tests.rs`'s own scope note) -- so the lookup-or-insert
pairing itself was extracted into its own function, `material_variation_resolve`, taking the
`Material` trait (already fully public) rather than the GL-bound `PbrMaterial` concretely, making it
directly unit-testable with a minimal zero-GL-call `Material` stand-in. See
`tests/gltf_material_variation_test.rs`.

**Verify Command:**
```bash
cd /home/user1/pro/lib/yrd_gamedev/cgtools
cargo test -p renderer --test gltf_material_variation_test
```
**Expected** (fixed): all 4 tests pass. **Actual** (pre-fix, confirmed via temporary direct-edit
revert-and-rerun of the map-insert line): all 4 fail -- every test's map-length assertion sees `0`
entries instead of the expected `1`/`2`, since the insert never happens.

## Root Cause

`primitive_material_resolve` (pre-fix):
```rust
fn primitive_material_resolve
(
  gltf_primitive : &gltf::Primitive< '_ >,
  materials : &[ SharedMaterial ],
  material_variation_map : &FxHashMap< uuid::Uuid, Vec< SharedMaterial > >,  // shared reference
  used_materials : &mut Vec< SharedMaterial >,
  dummy_material : &PbrMaterial
)
-> SharedMaterial
{
  // ... lookup via material_variation_map.get(...) ...
  if let Some( material ) = variation { material }
  else
  {
    let material = /* dyn_clone + apply vertex defines */;
    used_materials.push( material.clone() );
    // material_variation_map was never written to here -- couldn't be; it's `&`, not `&mut`.
    material
  }
}
```
The map parameter's type, `&FxHashMap< ... > `, made writing to it a compile error waiting to not
happen -- the `else` branch's author evidently intended `material_variation_map` to gain the new
entry (mirroring the identical push onto `used_materials` two lines above) but the type never
allowed it, and no compiler warning flags an immutable-reference parameter that is only ever read
from -- that is exactly what "read-only cache lookup" looks like from the type system's point of
view, indistinguishable from "cache that nobody happens to populate yet."

## Why Not Caught

No test exercised `material_variation_map` across two sequential resolutions of the same
`( material_id, vertex_defines )` pair -- the closest existing coverage,
`gltf_light_parsing_test.rs`/`gltf_loader_tests.rs`, targets unrelated pure sub-logic
(`light_list_get`/`asset_uri_resolve`) in the same file. The bug produces no crash, no visibly wrong
render (each clone is a materially-identical copy at load time, differing only in future-mutation
sharing semantics and clone count), and no compiler warning -- a `&FxHashMap` parameter that's only
read from is indistinguishable, by type alone, from an intentionally read-only cache.

## Fix Applied (2026-08-17)

**`src/webgl/loaders/gltf.rs`:** extracted the lookup-or-insert pairing out of
`primitive_material_resolve` into its own function, `material_variation_resolve`, taking
`material_variation_map` as `&mut` and performing the insert immediately after constructing a new
variation -- in the same function, so the lookup and its matching insert can't drift apart again:
```rust
pub fn material_variation_resolve
(
  material_variation_map : &mut FxHashMap< uuid::Uuid, Vec< SharedMaterial > >,
  material_id : uuid::Uuid,
  vertex_defines_str : &str,
  new_material : impl FnOnce() -> SharedMaterial
)
-> SharedMaterial
{
  let variation = material_variation_map
  .get( &material_id )
  .and_then( | m | m.iter().find( | m | m.borrow().vertex_defines_str() == vertex_defines_str ) )
  .cloned();

  if let Some( existing ) = variation { existing }
  else
  {
    let material = new_material();
    material_variation_map.entry( material_id ).or_default().push( material.clone() );
    material
  }
}
```
`primitive_material_resolve` now delegates to it, passing the GPU-dependent clone-and-configure step
( `dyn_clone` + `vertex_define_add` + `used_materials.push` ) as a lazily-invoked closure, so a
cache hit still never touches GL state. `SharedMaterial`'s `type` alias and the new function were
both promoted to `pub` ( exported via `mod_interface!` ) specifically so the cache logic could be
unit-tested directly, independent of the glTF-parsing/GPU-material-construction context around it --
mirroring this same file's existing `light_list_get`/`light_get`/`asset_uri_resolve` precedent of
promoting a pure sub-surface of the loader for direct testing.

**`tests/gltf_material_variation_test.rs`** (new file): a minimal zero-GL-call `TestMaterial`
stand-in implementing the (already fully public) `Material` trait, plus 4 native `#[ test ]`
functions covering: cache-miss creates and records a new variation; cache-hit on the same
`( material_id, vertex_defines )` returns the exact prior `Rc` without invoking `new_material` again
(the core regression check); distinct vertex-defines under the same material id create two
independent, unshared variations; distinct material ids are cached independently.

Verified no downstream consumer depends on `GLTF.materials`'s pre-fix length/ordering: a
workspace-wide grep found zero callers of `GLTF::material_get` (the only positional-index accessor)
anywhere in the workspace, and the 4 example crates that touch `gltf.materials` at all only ever
`.push()` an additional material -- never index into or assume a specific pre-fix count.

## Verification

`longrun`-detached, from repo root:
- `cargo test -p renderer --test gltf_material_variation_test` -- pre-fix (temporary direct-source-
  edit revert of the map-insert line only): 4 passed, 0 failed **before revert**; after commenting
  out the insert line to simulate the original defect: 0 passed, 4 failed (every map-length
  assertion saw the empty `Vec::new()` seed instead of a populated entry). Post-fix (line restored):
  4 passed, 0 failed again.
- `verb/test_only pkg::renderer` (full scoped suite, post-fix): **147 tests run: 147 passed, 0
  skipped** -- up from 143 (this bug's 4 new tests), including the real GPU-backed
  `native_render_test.rs::opaque_path_renders_lit_quad`.
- `cargo clippy -p renderer --all-features --all-targets -- -D warnings`: exit 0, clean (required
  fixing 3 lints triggered specifically by promoting `material_variation_resolve`/`SharedMaterial`
  to `pub` -- `clippy::implicit_hasher` (`#[ expect(...) ]`'d: this loader uses `FxHashMap`
  exclusively, everywhere, so genericizing over `BuildHasher` here would be unexercised
  generality), `clippy::type_complexity` (resolved by using the existing `SharedMaterial` alias
  instead of the fully-spelled-out nested type), and `clippy::single_match_else` (resolved by using
  `if let`/`else` instead of `match`, which is also what the pre-extraction code already did)).

## Generalized Version

**Broken assumption:** a cache/memoization map parameter typed `&Map< K, V >` (shared reference)
looks, and type-checks, identical to a genuinely read-only lookup table -- there is no signal at the
call site or in the function signature that distinguishes "this cache is never populated" from
"this cache doesn't need populating here." The bug is only visible by tracing every write site for
a given map across the whole call graph and finding none, or by testing that a *second* lookup for
a key just inserted by the *first* lookup actually returns the cached value. Whenever a function
takes a `Vec`/`Map` by shared reference and internally reasons about "cache hit vs. cache miss," audit
whether the miss branch is ever expected to grow that same collection -- if so, the reference must be
`&mut`, and a lookup-or-insert pairing is safest implemented as a single function owning both halves
end-to-end (mirroring this fix's extraction), not two call sites relying on the caller to keep them
in sync.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-17 | filed + fixed + verified | Found by direct review of `gltf.rs`'s loader functions during task #174's `renderer` crate scout, following the BUG-243/BUG-244 fixes in the same session. Root cause: `material_variation_map` was passed by shared reference (`&FxHashMap`) into `primitive_material_resolve`, so its cache-miss branch could never write the newly-created variation back into it -- every lookup permanently missed, and every primitive sharing a glTF material + vertex-defines combination got its own independent clone instead of sharing the intended `SharedMaterial` instance. Fixed by extracting the lookup-or-insert pairing into its own `pub` function, `material_variation_resolve`, taking the map as `&mut` and inserting immediately after construction. Verified via 4 new native unit tests against a minimal zero-GL-call `Material` stand-in (all 4 confirmed to fail pre-fix / pass post-fix via temporary revert-and-rerun of the insert line), the full 147/147 scoped suite, and clean clippy (after resolving 3 lints newly triggered by exposing the extracted function publicly). Closed same-session (Tier 2 Dual-Role Self-Check). |
