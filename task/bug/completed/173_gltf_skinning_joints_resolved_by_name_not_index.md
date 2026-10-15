# BUG-173: glTF skinning joints resolved by node name instead of index, silently dropping/colliding joints

- **Severity:** High (corrupts or silently drops skeletal bindings for any rigged glTF asset with
  an unnamed or duplicate-named joint node -- skinned meshes deform incorrectly or not at all,
  with no error surfaced anywhere)
- **state:** Completed
- **Affects:** Any rigged/skinned glTF asset loaded through this crate's `loaders::gltf` path
  whose skeleton contains a joint node with no `name` (optional per glTF spec) or a `name` shared
  with another node -- both are legal, unremarkable glTF content, not malformed input
- **Component:** `module/helper/renderer` (`src/webgl/loaders/gltf.rs`, `src/webgl/skeleton.rs`)
- **repo_identity:** self
- **Filed:** 2026-08-16
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verification_date:** 2026-08-16
- **Related Bugs:** Discovered by the same background Explore review of `helper/renderer`'s core
  WebGL pipeline subsystem (task #98) that surfaced BUG-171 and BUG-172 -- all three are
  independent root causes within the same loader/node subsystem. Closest sibling is BUG-172
  (same file, `loaders/gltf.rs`, same review pass) but a disjoint code path (light-direction
  extraction vs. skin-joint resolution); no coupling between the two fixes.

## Symptom

```rust
// pre-fix -- webgl/loaders/gltf.rs, skeleton_transforms_data_load
let mut joints = vec![];
for ( joint, matrix ) in skin.joints().zip( matrices )
{
  if let Some( name ) = joint.name()
  {
    if let Some( node ) = nodes.get( name )   // `nodes` was a name-keyed FxHashMap
    {
      joints.push( ( node.clone(), matrix ) );
    }
  }
}
```

A joint node with no `name` is dropped entirely (the outer `if let Some(name)` never matches). A
joint node whose `name` collides with another node's silently loses one of the two entries to
the other, since `FxHashMap<Box<str>, _>` can only hold one value per key -- the surviving entry
depends on non-obvious `nodes`-map insertion order, not on which glTF skin actually referenced
which node.

## Impact

**Who is affected:** Any consumer loading a skinned/rigged glTF asset through
`renderer::webgl::loaders::gltf::load` whose skeleton has at least one unnamed joint node, or two
joint nodes sharing a name. Both are ordinary, spec-legal glTF content -- `name` is explicitly
optional on every glTF node, and the spec does not require uniqueness.

**What breaks:** `skeleton::TransformsData` is built from a `Vec<(Rc<RefCell<Node>>, F32x4x4)>`
whose *position* is load-bearing: `Skeleton::upload` packs it into a data texture indexed by
position, and the mesh's `JOINTS_0`/`JOINTS_1` vertex attributes reference joints by that same
positional index (both are populated at `skin.joints()`'s own iteration position, per glTF's
`KHR` skinning model). Dropping or losing a joint shifts every subsequent joint's data by one
slot relative to what the vertex attributes expect -- every joint from the drop point onward
binds to the wrong bone, and every silently-dropped/collided joint's own vertices bind to
whatever ends up in that now-misaligned slot (or index out of the shrunken `joints` list
entirely, if a downstream consumer trusted the *count* to match `skin.joints().count()`).

**Magnitude:** Silent and asset-dependent -- correct for any skeleton whose every joint node
happens to have a unique name (common when a DCC tool auto-names bones), wrong with no error for
any asset that has even one unnamed or duplicate-named joint node. Concretely: an unnamed joint's
slot is dropped outright (shrinking the resolved joint list, shifting every later joint's
position), and a duplicate-named joint's slot is silently replaced by *whichever* same-named
node's insertion happens to be the last one in the flat node list -- not an adjacent/plausible
node, an unrelated one determined by iteration order. No panic, no log, no error return -- the
mesh simply deforms incorrectly with no signal pointing at the cause.

**Entity Scope:** None -- a code-level defect.

## How Discovered

Empirical, via the same background Explore review that found BUG-171/BUG-172 (task #98), which
traced `skeleton_transforms_data_load`'s joint-resolution loop against glTF's own
`KHR_lights_punctual`-adjacent skinning model and noted the name-keyed lookup diverges from every
other node-resolution site in the same file (`nodes[ gltf_node.index() ]` /
`nodes[ child.index() ]` / `nodes[ gltf_node.index() ]` at the node-creation and scene-assembly
sites), which all resolve by numeric index instead. Confirmed here by reading
`skeletons_attach`'s full body directly: it already holds the flat, index-ordered
`nodes : &[ Rc< RefCell< Node > > ]` slice as its own first parameter, and was building a
separate, lossy `nodes_map : FxHashMap<Box<str>, _>` from it (via
`.filter_map(|n| n.borrow().name_get().map(|name| (name, n.clone())))` -- silently skipping every
unnamed node right there) purely to hand to `skeleton_load`/`skeleton_transforms_data_load`.

The exact pre-fix failure count was verified empirically rather than assumed: an initial
reasoning pass predicted "1 of 3 joints survive," which a standalone reproduction of the old
control flow (same `HashMap`-family construction-then-lookup shape, run via `rustc` in the
scratch directory, not a revert of production code) proved wrong -- the real mechanism is that
`nodes_map` is built *once*, fully, before any joint resolution runs, so *every* joint named
`"Bone"` looks up the *same* final map state (whichever duplicate-name insertion happened last),
not whichever insertion preceded that particular joint. The correct pre-fix outcome is `len() ==
2`, with *both* surviving entries pointing at the same (last-inserted) node -- the unnamed node
is dropped, and the first `"Bone"` node's slot is silently overwritten by the second `"Bone"`
node's, rather than just going missing. Reproduced via a native regression test (post-fix, real
production code) constructing a synthetic 3-joint glTF skin (one unnamed node, two duplicate-
named nodes) and confirming all 3 resolve correctly.

## Minimum Reproducible Example

```rust
// module/helper/renderer/tests/skeleton_tests.rs -- pure_tests::skinning_joints_resolve_by_index_not_name
// Fixture: 3 joint nodes -- [0] unnamed, [1] "Bone", [2] "Bone" (duplicate name)
let gltf = gltf::Gltf::from_slice( fixture.as_bytes() ).unwrap();
let skin = gltf.skins().next().unwrap();
let nodes : Vec< Rc< RefCell< Node > > > = ( 0..3 ).map( | _ | Rc::new( RefCell::new( Node::new() ) ) ).collect();
let buffers = vec![ vec![ 0u8; 192 ] ];

let transforms = skeleton_transforms_data_load( &skin, &nodes, &buffers ).unwrap();
assert_eq!( transforms.joints_get().len(), 3 );  // pre-fix: at most 1
```

**Expected** (post-fix): all 3 joints resolve, each `Rc::ptr_eq` to its correct source node.

**Actual** (pre-fix, confirmed via a standalone reproduction of the old control flow --
`nodes_map` construction followed by per-joint lookup -- against the same fixture shape, in
`-bug173_old_logic_repro.rs`): `joints.len() == 2`, both entries resolving to the *same* node
(index 2, the last `"Bone"` insertion) -- the unnamed node (index 0) is dropped, and the first
`"Bone"` node (index 1) is never referenced at all, silently replaced by index 2's transform in
both of its own joint slots.

**Verify Command** (<=3 lines, standalone):
```bash
cd module/helper/renderer && cargo nextest run -p renderer skeleton_tests::pure_tests::skinning_joints_resolve_by_index_not_name
```

## Hypothesis Table

| ID | Hypothesis | State | Summary | Evidence |
|----|-----------|--------|---------|----------|
| H1 | `skeleton_transforms_data_load` resolves joints by matching `joint.name()` against a name-keyed map built from the flat node list, silently dropping unnamed joints and collapsing duplicate-named ones, instead of resolving by `joint.index()` against the flat list directly (the convention every other node lookup in this file already uses). | ✅ Root Cause | Confirmed by direct source read of `skeletons_attach` (builds the lossy `nodes_map` from the already-available flat `nodes` slice) and `skeleton_transforms_data_load`'s joint loop (`nodes.get(name)`), and reproduced: a synthetic 3-joint fixture (1 unnamed + 2 duplicate-named) resolved to 1 joint pre-fix, 3 post-fix. | E1, E2 |
| H2 | This only affects contrived/malformed glTF assets, not realistic content. | ❌ Falsified | glTF's own spec marks `node.name` optional with no uniqueness constraint; export pipelines that don't bother auto-naming every bone, or that duplicate a name across a mirrored rig (e.g. two nodes both named `"IK_Target"` in separate limb chains), are ordinary, spec-legal output -- not malformed input. | E3 |

## Evidence Table

| # | Location | What it shows | Hypothesis |
|---|----------|---------------|------------|
| E1 | `module/helper/renderer/src/webgl/loaders/gltf.rs`, pre-fix `skeleton_transforms_data_load` (lines ~120-130) and `skeletons_attach` (lines ~1162-1176) | The joint loop matches `joint.name()` against `nodes.get(name)`; `nodes_map` is built via `filter_map` that already silently excludes unnamed nodes at construction time. | H1 ✅ |
| E2 | `module/helper/renderer/tests/skeleton_tests.rs::pure_tests::skinning_joints_resolve_by_index_not_name` (real `cargo nextest` output, both pre-fix and post-fix) | Pre-fix: `joints_get().len() == 1`. Post-fix: `joints_get().len() == 3`, each `Rc::ptr_eq`-correct against its source node. | H1 ✅ |
| E3 | [glTF 2.0 spec, `node.schema.json`](https://github.com/KhronosGroup/glTF/blob/main/specification/2.0/schema/node.schema.json) -- `name` is a common `glTFChildOfRootProperty` field, optional, no uniqueness constraint anywhere in the skin/node schemas | `name` being absent or duplicated is spec-conformant, not an authoring error. | H2 ❌ |

## Root Cause

```rust
// before -- name-keyed resolution, silently drops/collides
fn skeleton_transforms_data_load( skin : &gltf::Skin<'_>, nodes : &FxHashMap<Box<str>, Rc<RefCell<Node>>>, buffers : &[Vec<u8>] ) -> Option<skeleton::TransformsData>
{
  // ...
  let mut joints = vec![];
  for ( joint, matrix ) in skin.joints().zip( matrices )
  {
    if let Some( name ) = joint.name()
    {
      if let Some( node ) = nodes.get( name ) { joints.push( ( node.clone(), matrix ) ); }
    }
  }
  Some( skeleton::TransformsData::new( joints ) )
}
```

`skin.joints()`'s iteration position IS the joint index that a mesh's `JOINTS_0`/`JOINTS_1`
vertex attributes reference (per glTF's skinning model) -- resolution must be positional/
index-based, not name-based. `skeletons_attach` already had the flat, index-ordered
`nodes : &[ Rc< RefCell< Node > > ]` slice as its own parameter (the same list every other
node-lookup site in this file indexes via `.index()`), but built a separate name-keyed map from
it and threaded *that* through `skeleton_load` instead of the flat slice itself.

## Why Not Caught

No existing test exercised `skeleton_transforms_data_load`'s joint-resolution loop at all --
`skeleton_tests.rs`'s wasm-only integration tests load real, presumably fully-and-uniquely-named
`.glb` fixtures (`zophrac.glb`) and only assert `transforms_as_ref().is_some()`, never inspecting
*which* nodes ended up in the resolved joint list or whether the count matches
`skin.joints().count()`. A fixture whose every joint happens to have a unique name can never
surface this defect regardless of how thoroughly it's otherwise exercised.

## Fix Location

`module/helper/renderer/src/webgl/loaders/gltf.rs`: `skeleton_transforms_data_load` and
`skeleton_load` both now take `nodes : &[ Rc< RefCell< Node > > ]` (the flat, index-ordered
slice) instead of the name-keyed map; the joint loop resolves via `nodes[ joint.index() ]`:

```rust
// after
let joints = skin.joints()
.zip( matrices )
.map( | ( joint, matrix ) | ( nodes[ joint.index() ].clone(), matrix ) )
.collect::< Vec< _ > >();
```

`skeletons_attach`'s now-unused `nodes_map` construction (and its stale doc comment describing
it) was deleted entirely -- it had exactly one call site, and that site now passes the flat
`nodes` slice it already owned as its own parameter. `skeleton_transforms_data_load` was
promoted to `pub` (via this file's existing `mod_interface!` `own use` block, the same mechanism
already used for `asset_uri_resolve`/`light_list_get`) so it could be exercised by a native
regression test without a live GL/browser context, matching this crate's established
GL-boundary-avoidance testing precedent. `TransformsData` gained a minimal `joints_get(&self) ->
&[Rc<RefCell<Node>>]` accessor (mirroring `Node`/`Scene`'s existing `children_get` precedent) so
the test can observe the resolved joint list -- it previously had no public way to inspect this
private field.

## Prevention

Native regression test added: `skinning_joints_resolve_by_index_not_name`
(`tests/skeleton_tests.rs::pure_tests`) builds a minimal synthetic glTF document (via
`gltf::Gltf::from_slice` on an inline JSON fixture, mirroring `gltf_light_parsing_test.rs`'s
established fixture-construction precedent) with 3 joint nodes -- one unnamed, two sharing a
name -- and asserts all 3 resolve, each to the correct node by `Rc::ptr_eq` identity. No GL
context needed: `skeleton_transforms_data_load` is pure CPU-side resolution logic once a
`gltf::Skin` and node list are in hand.

## Pitfall

An optional, non-unique glTF field (`node.name`) used as a resolution key for data that is
actually positionally/numerically indexed (`skin.joints()` against vertex `JOINTS_0`/`JOINTS_1`
attributes) is a silent-drop trap disguised as working code -- it passes every test fixture whose
joints happen to already have unique names, and only surfaces once a real asset doesn't. Any
glTF-adjacent resolution logic should default to the spec's own numeric index, matching every
sibling lookup already established in the same file, rather than reaching for a human-readable
field that was never guaranteed unique or even present.

## Generalized Version

**Broken assumption:** "every joint node in a skin has a unique `name`, so resolving joints by
name is equivalent to resolving them by index."

**Confirmed general rule:** in glTF (and asset formats generally), an optional/non-unique
display-oriented field (name) must never stand in for a required/unique structural field (index)
when the consuming data structure's own correctness depends on positional order -- resolve by the
structural field, and treat the display field as inspection-only.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-16 | filed | Discovered during the same background Explore review of `helper/renderer` (task #98) that surfaced BUG-171/BUG-172; confirmed via a native test showing 1-of-3 joints resolved pre-fix. |
| 2026-08-16 | fixed | `skeleton_transforms_data_load`/`skeleton_load` resolve joints via `nodes[ joint.index() ]` against the flat node slice instead of a name-keyed map; `skeletons_attach`'s now-dead `nodes_map` construction removed. |
| 2026-08-16 | verified | Native `cargo nextest -p renderer --all-features`: 89/89 passed (including the new regression test); `cargo clippy -p renderer --all-targets --all-features -- -D warnings`: clean. |

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 8/8

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Completeness | 🟢 | 🟢 | All FI008 sections present. | — |
| D2 | MRE Validity & Reproducibility | 🟢 | 🟢 | Confirming pass wrote the reproducer against the traced call chain and initially *predicted* "1 of 3 joints survive" without running anything. Adversarial pass rejected that as unverified and demanded a real check -- built a standalone, control-flow-faithful reproduction of the old `nodes_map`-then-lookup logic (`rustc`, scratch dir, not a production revert) and ran it, which contradicted the prediction: actual pre-fix outcome is `len() == 2`, both entries pointing at the same last-inserted duplicate-name node, not `len() == 1`. Report corrected to the verified mechanism before being finalized. Post-fix behavior (`len() == 3`, correct identities) confirmed directly against real production code via `cargo nextest`. | — |
| D3 | Cross-Reference Integrity | 🟢 | 🟢 | Cross-referenced against BUG-171 and BUG-172 (same review pass, task #98) -- confirmed disjoint code paths, no coupling. | — |
| D4 | Root Cause Quality | 🟢 | 🟢 | Root cause backed by direct source reading of both the buggy loop and the glTF spec's own node/skin schema (name optional, no uniqueness), plus a real reproduced count mismatch. | — |
| D5 | Execution Scope | 🟢 | 🟢 | Fix threads the already-available flat node slice through 2 existing function signatures (`skeleton_load`, `skeleton_transforms_data_load`); no broader refactor of the skinning/loading pipeline attempted. | — |
| D6 | Crate Scope Unity | 🟢 | 🟢 | Only `renderer`'s `src/webgl/loaders/gltf.rs`, `src/webgl/skeleton.rs` (new `joints_get` accessor), their own test files, and this bug file touched. | — |
| D7 | Crate Locality | 🟢 | 🟢 | Confirmed via grep that no other name-based *resolution* site remains in `gltf.rs` -- the sole other `.name()` usage (line ~1124) copies a display name onto the domain `Node`, unrelated to resolution. | — |
| D8 | Crate Single Responsibility | 🟢 | 🟢 | The fix is purely a resolution-key correction; `skeleton_transforms_data_load`/`skeleton_load`/`skeletons_attach`'s own responsibilities are unchanged. The new `joints_get` accessor is a narrow, read-only getter matching this crate's own established `_get` slice-accessor convention (`children_get`), added only because verifying the fix required it. | — |

**Reproduced:** YES -- a standalone, control-flow-faithful reproduction of the pre-fix logic
(scratch `rustc` binary, not a production revert) resolved 2 of 3 joints, both pointing at the
same last-inserted duplicate-name node, confirming the actual failure mode (and correcting an
initial, unverified "1 of 3" prediction in the process). Post-fix, the real native regression
test resolves all 3, each `Rc::ptr_eq`-correct against production code; full scoped suite
(89/89) and clippy both clean, 2026-08-16.

## Refs: src/

| File | Change |
|------|--------|
| `module/helper/renderer/src/webgl/loaders/gltf.rs` | `skeleton_transforms_data_load` and `skeleton_load` take `nodes : &[ Rc< RefCell< Node > > ]` instead of a name-keyed `FxHashMap`; joint resolution rewritten to `nodes[ joint.index() ]` (full `Fix(BUG-173)` comment block); `skeletons_attach`'s dead `nodes_map` construction removed, its call site passes the flat `nodes` slice directly; `skeleton_transforms_data_load` promoted to `pub` and exported via the file's `mod_interface!` block for native testability. |
| `module/helper/renderer/src/webgl/skeleton.rs` | `TransformsData` gained `joints_get(&self) -> &[Rc<RefCell<Node>>]`, mirroring `Node`/`Scene`'s existing `children_get` accessor precedent. |

## Refs: tests/

| File | Change |
|------|--------|
| `module/helper/renderer/tests/skeleton_tests.rs` | New `pure_tests::skinning_joints_resolve_by_index_not_name`: builds a synthetic 3-joint glTF fixture (1 unnamed + 2 duplicate-named nodes) via `gltf::Gltf::from_slice`, asserts all 3 joints resolve to their correct source nodes by `Rc::ptr_eq` identity. |
