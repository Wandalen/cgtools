# BUG-156: `ObjectLayer::z_in_object` is documented as the layer draw order but is never read by the compile pipeline

- **Severity:** Medium (silent visual defect, not a crash -- any object whose layer stack relies
  on `z_in_object` to control paint order draws in raw declaration order instead, which is wrong
  whenever declaration order differs from the intended stacking order; no data loss, no panic,
  but every consumer of this documented field gets incorrect output with no error signal)
- **state:** Completed
- **Affects:** All 5 per-bucket layer-gathering sites in `compile/frame.rs` -- `vertex_pass_compile`,
  `frame_emits_gather`'s Hex-instance loop, `edge_pass_scene_compile`, `free_pass_scene_compile`,
  `viewport_pass_scene_compile` -- for any object whose state has 2+ layers in the same pipeline
  bucket with `z_in_object` values that don't already match declaration order
- **Component:** `module/helper/tilemap_scene` (`src/compile/frame.rs`)
- **repo_identity:** self
- **Filed:** 2026-08-16
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/verified/
- **verification_date:** 2026-08-16
- **Related Bugs:** None (independent of BUG-157, filed in the same review batch of task #92's
  `tilemap_scene` pass but a different code path -- this is compile-time draw order, BUG-157 is
  scene-tick event timing).

## Symptom

```rust
// One object, one state, two layers on the SAME pipeline bucket, declared with
// z_in_object REVERSED relative to declaration order.
let layers = vec!
[
  ObjectLayer { sprite_source: Static(frame "1"), z_in_object: 1, .. },  // declared FIRST
  ObjectLayer { sprite_source: Static(frame "0"), z_in_object: 0, .. },  // declared SECOND
];
// Wrong (pre-fix):   emitted Sprite order = [frame "1", frame "0"]  (declaration order)
// Correct (post-fix): emitted Sprite order = [frame "0", frame "1"]  (ascending z_in_object)
```

## Impact

**Who is affected:** Any `tilemap_scene` consumer whose object states declare 2+ layers in the
same pipeline bucket (the common case for multi-layer objects: base + overlay, body + skirt,
etc.) where the intended stacking order doesn't happen to already match declaration order in
the source data/editor.

**What breaks:** `ObjectLayer::z_in_object` (`src/layer.rs:32`) is documented -- at its own
declaration, at `src/object.rs:52`, and in `docs/format/001` -- as "Draw order within the
state's stack, ascending. Higher = later (on top)." `docs/algorithm/002_scene_rendering_pass.md`
(lines 15-30) specifies this explicitly in its own pseudocode as a per-instance pre-sort applied
before the bucket-wide `sort_mode_apply` step. But all 5 compile-pass sites that iterate an
object's layer stack (`vertex_pass_compile:198`, the Hex-instance loop in
`frame_emits_gather:693`, `edge_pass_scene_compile:1097`, `free_pass_scene_compile:1183`,
`viewport_pass_scene_compile:1298` -- pre-fix line numbers) iterated `stack` in raw declaration
order. The field was read nowhere in the compile pipeline -- confirmed via
`grep -rn "z_in_object"` across `src/` returning exactly 3 hits, all doc/declaration sites, zero
usage sites, before this fix.

**Magnitude:** Silent -- no panic, no error, no lint. Output is plausible-looking (a stack of
sprites still renders) but the relative paint order is wrong whenever declaration order and
intended `z_in_object` order diverge, which is invisible without a visual diff against the
documented contract.

**Entity Scope:** None -- a code-level defect, not an operational-entity concern.

## How Discovered

Flagged during a background review pass over `tilemap_scene` (task #92): reviewing agent noted
`z_in_object` is declared and documented but grepped as unused in the compile pipeline. Before
accepting the finding, independently re-verified via direct `grep`/`Read` of all 5 call sites'
exact shapes, the `states: HashMap<String, Vec<ObjectLayer>>` construction path (confirming no
upstream sort exists either), and located the crate's own authoritative pseudocode
(`docs/algorithm/002_scene_rendering_pass.md`, `docs/format/007`, `docs/format/001`) to resolve
whether the correct fix is a pre-sort (confirmed) or a tiebreak wired into the later bucket-wide
sort comparator (confirmed NOT what's documented) before implementing.

## Minimum Reproducible Example

```bash
cd module/helper/tilemap_scene && cargo test --test scene_model_compile_test object_layers_draw_in_ascending_z_in_object_order 2>&1 | tail -10
```

**Expected** (post-fix):
```
test object_layers_draw_in_ascending_z_in_object_order ... ok
```

**Actual** (pre-fix -- confirmed by temporarily reverting the `frame_emits_gather` call site back
to bare `for layer in stack` and re-running):
```
thread 'object_layers_draw_in_ascending_z_in_object_order' panicked at module/helper/tilemap_scene/tests/scene_model_compile_test.rs:498:3:
assertion `left == right` failed: layers must draw in ascending z_in_object order regardless of declaration order; saw [ResourceId(0), ResourceId(1)]
  left: [ResourceId(0), ResourceId(1)]
 right: [ResourceId(1), ResourceId(0)]
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 42 filtered out
```

**Verify Command** (<=3 lines, standalone):
```bash
cd module/helper/tilemap_scene && cargo test --test scene_model_compile_test object_layers_draw_in_ascending_z_in_object_order
# ok = fixed; assertion failure with reversed sprite order = bug present
```

## Hypothesis Table

| ID | Hypothesis | State | Summary | Evidence |
|----|-----------|--------|---------|----------|
| H1 | `z_in_object` is documented as the per-object layer draw order but is never read by any compile-pass call site -- all 5 sites iterate the raw `stack` slice in declaration order. | ✅ Root Cause | `grep -rn "z_in_object"` across `src/` returned exactly 3 hits pre-fix: the field declaration (`layer.rs:32`) and two doc comments (`layer.rs:17`, `object.rs:52`) -- zero usage sites. Direct read of all 5 `for layer in stack` loops confirmed none apply any ordering. | E1, E2 |
| H2 | The fix should wire `z_in_object` into the existing bucket-wide `sort_mode_apply` comparator (as a tiebreak) rather than a pre-sort of each object's own stack. | ❌ Rejected | `docs/algorithm/002_scene_rendering_pass.md:15-30`'s pseudocode explicitly specifies `for each Layer in stack ordered by z_in_object:` as a pre-sort applied BEFORE the later `if bucket.sort != None: draw_calls.sort_by(...)` step -- a pre-sort at the per-instance level, not a comparator tiebreak. | E3 |
| H3 | Some upstream construction path (e.g. spec loading/deserialization) already sorts `Vec<ObjectLayer>` by `z_in_object` before it reaches the compile pass, making the compile-pass read redundant. | ❌ Falsified | Read the `states: HashMap<String, Vec<ObjectLayer>>` construction path -- no sort is applied anywhere between spec construction and the compile-pass call sites; the vec is stored and read in whatever order the caller pushed it. | E4 |

## Evidence Table

| # | Location | What it shows | Hypothesis |
|---|----------|---------------|------------|
| E1 | `src/layer.rs:17,32` / `src/object.rs:52` (pre-fix, unedited) | `ObjectLayer.z_in_object: i32` documented "Draw order within the parent object's layer stack. Higher = later (on top)." / "Layer order is by `z_in_object`; for equal values, ..." | H1 ✅ |
| E2 | `src/compile/frame.rs` (pre-fix): lines 198, 674, 1097, 1183, 1298 | All 5 sites: `for layer in stack { ... }` -- raw declaration-order iteration, no sort call anywhere in the function bodies. | H1 ✅ |
| E3 | `docs/algorithm/002_scene_rendering_pass.md:15-30` | Pseudocode: `for each Layer in stack ordered by z_in_object:` precedes the bucket-wide `if bucket.sort != None: draw_calls.sort_by(...)` step -- confirms pre-sort, not tiebreak. | H2 ❌ |
| E4 | `src/compile/frame.rs` (all 5 sites' `object.states.get(...)` calls) | `stack` is the `&Vec<ObjectLayer>` returned directly from the spec's `HashMap` -- no intervening sort call on any path from spec construction to these reads. | H3 ❌ |

## Root Cause

```
compile/frame.rs -- all 5 layer-gathering sites (pre-fix)
  for layer in stack          // raw declaration order; z_in_object never read
  {
    ...
  }
```

`ObjectLayer::z_in_object` is documented and specified (by the crate's own algorithm doc) as a
per-instance pre-sort key, but every reader of an object's layer stack iterated it unsorted.

## Why Not Caught

Existing tests that set a non-zero `z_in_object` on a second layer
(`scene_model_compile_test.rs:1059,1162,1178`) only assert on sprite-ID *set membership*
(`.contains(...)` / `HashSet`), never on the *emission order* of the resulting `Sprite` commands
-- so a stack that happened to already declare its layers in ascending-`z_in_object` order (the
common case when authoring by hand) never exposed the missing sort. No existing test declared a
stack with `z_in_object` deliberately reversed relative to declaration order.

## Fix Location

`module/helper/tilemap_scene/src/compile/frame.rs`:

```rust
// added, before vertex_pass_compile (line ~194)
fn layers_in_z_order( stack : &[ ObjectLayer ] ) -> Vec< &ObjectLayer >
{
  let mut ordered : Vec< &ObjectLayer > = stack.iter().collect();
  ordered.sort_by_key( | layer | layer.z_in_object );
  ordered
}
```

Applied at all 5 call sites (`vertex_pass_compile`, `frame_emits_gather`, `edge_pass_scene_compile`,
`free_pass_scene_compile`, `viewport_pass_scene_compile`): `for layer in stack` → `for layer in
layers_in_z_order( stack )`. `sort_by_key` is stable, so layers sharing a `z_in_object` value
keep their relative declaration order -- matching `object.rs:52`'s "for equal values, ..."
tie-break wording.

## Prevention

Added `object_layers_draw_in_ascending_z_in_object_order` (`bug_reproducer(BUG-156)`) to
`tests/scene_model_compile_test.rs`: two layers on one object, declared with `z_in_object`
reversed relative to declaration order, `SortMode::None` on the pipeline bucket (so
`sort_mode_apply` can't mask the compile-pass order), asserts the emitted `Sprite` command order
matches ascending `z_in_object`.

## Pitfall

A documented-but-unread struct field compiles cleanly and produces plausible output --
declaration order often coincides with the intended order when data is hand-authored, so this
class of bug has no compiler signal and no obviously-wrong visual output in the common case.
Only a direct doc-vs-usage grep (does every documented field have at least one read site
matching its documented semantics?) catches it; testing only set membership instead of emission
order let 3 existing tests exercise a non-zero `z_in_object` without ever detecting it was inert.

## Generalized Version

**Broken assumption:** "if a struct field has a clear doc comment and is set correctly by every
test fixture, it's being used correctly." False here -- every fixture set `z_in_object`
faithfully, but no code path ever read it back out for its documented purpose; doc accuracy and
field *usage* are independent claims.

**Confirmed general rule:** when a field's doc comment describes an ordering/sorting contract
("draw order", "ascending", "priority"), grep for at least one `sort`/`sort_by`/`sort_by_key`
call referencing that field before trusting the contract holds -- a present-and-populated field
is not evidence it's consumed.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-16 | filed | Flagged by a background review pass over `tilemap_scene` (task #92); independently re-verified (all 5 call sites, upstream construction path, authoritative pseudocode doc) before filing. |
| 2026-08-16 | fixed | Added `layers_in_z_order` helper (stable sort by `z_in_object`) and wired it into all 5 compile-pass call sites that previously iterated `stack` in raw declaration order. |
| 2026-08-16 | verified | Added `object_layers_draw_in_ascending_z_in_object_order` (written against a temporarily-reverted call site to confirm it fails pre-fix with the predicted reversed order, then passes post-fix). Full crate suite (171 tests) + `cargo clippy -p tilemap_scene --all-targets --all-features -- -D warnings` clean. |

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 8/8

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Completeness | 🟢 | 🟢 | All FI008 sections present. | — |
| D2 | MRE Validity & Reproducibility | 🟢 | 🟢 | Confirming pass wrote the test against the fixed code and confirmed it passes; adversarial pass traced the test's placement model (`SceneSnapshot.tiles` → `Scene::from_snapshot` → `scene.spawn(Placement::Hex)`) to confirm it exercises `frame_emits_gather`'s call site specifically, then temporarily reverted that one site back to bare `for layer in stack` and re-ran, confirming a real failure with the exact predicted reversed `ResourceId` order before restoring the fix. | — |
| D3 | Cross-Reference Integrity | 🟢 | 🟢 | Independent of BUG-157 (same review batch, different code path: compile-time draw order vs. scene-tick event timing) -- no cross-dependency. | — |
| D4 | Root Cause Quality | 🟢 | 🟢 | Root cause backed by a doc-vs-usage grep (3 hits, all non-usage) plus the crate's own authoritative pseudocode doc resolving pre-sort vs. tiebreak ambiguity before implementing. | — |
| D5 | Execution Scope | 🟢 | 🟢 | Read all 5 call sites individually (not assumed identical from one) and confirmed each converted correctly via a post-edit grep showing zero remaining bare `for layer in stack`. | — |
| D6 | Crate Scope Unity | 🟢 | 🟢 | Only `tilemap_scene` src+test+bug file touched. | — |
| D7 | Crate Locality | 🟢 | 🟢 | Fix is one new private helper + 5 one-line call-site edits; no signature/field change, no public API change. | — |
| D8 | Crate Single Responsibility | 🟢 | 🟢 | No new public surface; existing documented field semantics now actually implemented, nothing else changed. | — |

**Reproduced:** YES -- `object_layers_draw_in_ascending_z_in_object_order` was confirmed to fail
with the exact predicted reversed-order assertion mismatch when the 5 call sites were temporarily
reverted to bare `for layer in stack`; restoring the fix returns the test to passing. Full crate
suite (171 tests) + `cargo clippy -p tilemap_scene --all-targets --all-features -- -D warnings`
clean, 2026-08-16.

## Refs: src/

| File | Change |
|------|--------|
| `module/helper/tilemap_scene/src/compile/frame.rs` | Added `layers_in_z_order` helper; converted all 5 `for layer in stack` call sites to `for layer in layers_in_z_order( stack )`. `Fix(BUG-156)`/`Root cause`/`Pitfall` comment on the helper; short `Fix(BUG-156)` pointers at the 4 downstream call sites. |

## Refs: tests/

| File | Change |
|------|--------|
| `module/helper/tilemap_scene/tests/scene_model_compile_test.rs` | Added `object_layers_draw_in_ascending_z_in_object_order` (`bug_reproducer(BUG-156)`, full doc comment). |
