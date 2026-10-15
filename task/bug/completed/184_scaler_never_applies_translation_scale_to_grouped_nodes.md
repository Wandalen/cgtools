# BUG-184: `Scaler::set` never applies translation/scale to grouped nodes

- **Severity:** High (functional defect -- two of a grouped node's three transform channels
  silently never animate at all, regardless of the group's own weight configuration)
- **state:** Completed
- **Affects:** Every caller of `renderer::webgl::animation::Scaler` that groups a node whose
  animation includes a translation and/or scale channel -- e.g.
  `examples/minwebgl/animation_amplitude_change`'s GUI-driven body-part amplitude control.
- **Component:** `module/helper/renderer` (`src/webgl/animation/scaling.rs`)
- **repo_identity:** self
- **Filed:** 2026-08-16
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/ (self)
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/ (self)
- **verification_date:** 2026-08-16
- **Related Bugs:** Fixed in the same session pass as BUG-186 (same file, same function family) --
  reported separately since the two are independent root causes (missing functionality vs. a
  wrong guard condition on existing functionality). Both new functions this fix adds
  deliberately preserve BUG-185's still-open `tweens[ 0 ].start_value` stomp identically, rather
  than inventing an unreviewed fix for it under time pressure -- see Pitfall.

## Symptom

```rust
// before, AnimatableComposition::set for Scaler
used_nodes.insert( name.clone() );
self.scaled_rotation_apply( node, name, scales.y() );
// -- scales.x() ( translation weight ) and scales.z() ( scale weight ) were read by nothing
```

`scaled_nodes`'s own doc comment documents all three simple-transform weight components
(`x` - transform, `y` - rotation, `z` - scale), but only `scaled_rotation_apply` ever existed --
`set()` never called anything for translation or scale on a grouped node, so those channels
stayed frozen at the node's default transform forever, regardless of the group's `x`/`z` weights.

## Impact

**Who is affected:** Every caller grouping a node via `Scaler::add` whose underlying animation
drives that node's translation or scale channel -- the rotation channel alone continued to work.

**What breaks:** A grouped node's translation and scale never move from
`Node::new()`'s defaults, no matter what animation data is inserted under its
`.translation`/`.scale` keys or what weight the group assigns them.

**Magnitude:** Total for the two affected channels -- not a scaling-accuracy defect, a complete
absence of effect.

**Entity Scope:** None -- a code-level defect.

## How Discovered

Continuing the backlog item filed as task #136 (`Scaler::set never applies translation/scale to
grouped nodes`). Reading `AnimatableComposition::set`'s grouped-node branch confirmed only
`scaled_rotation_apply` was called; `scaled_nodes`'s own weight-vector doc comment (`x` -
transform, `z` - scale) makes clear translation/scale were intended to receive the same
treatment. Real-caller evidence in `examples/minwebgl/animation_amplitude_change/src/gui_setup.rs`
settled the fix's scope: each GUI amplitude slider splats one scalar across all four weight
components (`gl::F64x4::splat( value )`), confirming the intended design is uniform
amplitude-scaling of translation+rotation+scale together, not an unscaled passthrough for the two
missing channels.

## Minimum Reproducible Example

```rust
scaler.add( "group1", vec![ "node1".into() ], F64x4::new( 1.0, 1.0, 1.0, 1.0 ) );
scaler.set( &nodes ); // node1 has a translation/scale animation inserted
// pre-fix: node1's translation/scale stay at Node::new()'s defaults regardless of animation data
```

**Verify Command** (<=3 lines, standalone):
```bash
cd module/helper/renderer && cargo nextest run --all-features --test scaler_tests test_scaler_applies_translation_and_scale_to_grouped_nodes
```

## Hypothesis Table

| ID | Hypothesis | State | Summary | Evidence |
|----|-----------|--------|---------|----------|
| H1 | `set()`'s grouped-node branch never calls any translation/scale-applying function, so those channels are silently skipped entirely. | ✅ Root Cause | Confirmed by reading `set()`'s full body -- only `scaled_rotation_apply` is called. | E1 |
| H2 | The correct fix is to apply translation/scale *unscaled* ( ignoring the group's `x`/`z` weights ), since only rotation was ever weight-scaled in practice. | ❌ Falsified | Real caller evidence ( GUI amplitude sliders ) shows all three channels are meant to receive the SAME weighted amplitude-scaling treatment, not a plain passthrough for two of them. | E2 |

## Evidence Table

| # | Location | What it shows | Hypothesis |
|---|----------|---------------|------------|
| E1 | `module/helper/renderer/src/webgl/animation/scaling.rs`, `AnimatableComposition::set` (pre-fix) | Grouped-node loop calls only `self.scaled_rotation_apply( node, name, scales.y() )`; `scales.x()`/`scales.z()` are never read. | H1 ✅ |
| E2 | `examples/minwebgl/animation_amplitude_change/src/gui_setup.rs` | Every amplitude slider callback does `*scale = gl::F64x4::splat( f64::from( value ) );` -- one scalar drives all four weight components uniformly. | H2 ❌ |

## Root Cause

`AnimatableComposition::set` for `Scaler` was written with only a rotation-scaling function
implemented; the translation/scale weight components documented on `scaled_nodes` were never
wired to any corresponding apply function, so they were silently inert.

## Why Not Caught

`test_grouped_nodes_independence`, the one pre-existing test exercising grouped-node scaling,
only asserts on `Scaler::scale_get`'s own bookkeeping getter -- it never samples a node's actual
transform after `set()`, so a channel that does nothing at all produced no observable test
failure.

## Fix Location

`module/helper/renderer/src/webgl/animation/scaling.rs`: added `scaled_translation_apply` and
`scaled_scale_apply`, mirroring `scaled_rotation_apply`'s per-segment delta-scaling pattern for
the `F64x3` translation/scale channels (additive vector delta scaled by the weight, rather than
axis-angle decomposition), and wired both into `set()` alongside the existing rotation call.

## Prevention

New test `test_scaler_applies_translation_and_scale_to_grouped_nodes` groups a node with a
non-default translation and scale animation and asserts the resulting node transform is no
longer frozen at `Node::new()`'s defaults after `set()`.

## Pitfall

Mirroring `scaled_rotation_apply`'s existing loop structure for the two new functions would have
also copied its two co-located, independently-tracked defects (BUG-186's `scale < 1.0` guard,
BUG-185's unconditional `tweens[ 0 ]` stomp) into two more call sites verbatim. BUG-186's fix was
applied first and used in its corrected (unconditional) form from the start in both new
functions, avoiding tripling that defect's footprint; BUG-185 was deliberately left unfixed and
identically reproduced in both new functions, since a confident fix requires its own dedicated
investigation (task #137) into "has this sequence genuinely looped" semantics not otherwise
needed here -- fixing it piecemeal, once per call site, under time pressure would risk a
half-corrected inconsistency across the three sites when task #137 is eventually tackled.

## Generalized Version

**Broken assumption:** "A struct's own doc comment describing a field's intended shape (e.g. a
weight vector's four components) is automatically honored by the code that reads it."

**Confirmed general rule:** A doc comment documenting intended structure is not evidence the
structure is actually wired up -- grep for actual call sites of each documented component before
trusting that a "should do X" comment reflects "does X."

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-16 | filed | Continuing backlog task #136; confirmed via reading `set()`'s full body that translation/scale were never applied for grouped nodes. |
| 2026-08-16 | fixed | Added `scaled_translation_apply`/`scaled_scale_apply`, wired into `set()`. |
| 2026-08-16 | verified | `cargo nextest run -p renderer --test scaler_tests --all-features`: 10/10 passed. `cargo clippy -p renderer --all-targets --all-features -- -D warnings`: clean. Full workspace: `cargo nextest run --workspace --all-features --exclude object_picking`: 1911/1911 passed, doctests all `ok`, `cargo clippy --workspace --all-targets --all-features --exclude object_picking -- -D warnings`: clean. |

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 8/8

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Completeness | 🟢 | 🟢 | All FI008 sections present. | — |
| D2 | MRE Validity & Reproducibility | 🟢 | 🟢 | Confirming: new test passes against the fixed code, 10/10 in-crate. Adversarial: attempted to show the test would pass even without the fix by checking it doesn't depend on `.update()`-driven segment progression at all ( uses `current=0` only ) -- confirmed the assertion genuinely depends on the two new functions being called (pre-fix, translation/scale getters would read back `Node::new()`'s exact defaults, which the test explicitly rejects). | — |
| D3 | Cross-Reference Integrity | 🟢 | 🟢 | Cross-referenced against BUG-186 (fixed in the same pass, same file) and BUG-185 (deliberately not fixed, same file, explicitly noted). | — |
| D4 | Root Cause Quality | 🟢 | 🟢 | Backed by direct reading of `set()`'s full body and a real caller's GUI wiring, not assumed. | — |
| D5 | Execution Scope | 🟢 | 🟢 | Fix adds exactly the two missing apply functions and their call sites; no unrelated refactor. | — |
| D6 | Crate Scope Unity | 🟢 | 🟢 | Entirely within `renderer`'s own `scaling.rs`. | — |
| D7 | Crate Locality | 🟢 | 🟢 | `set()` has exactly one definition site, fixed there. | — |
| D8 | Crate Single Responsibility | 🟢 | 🟢 | Fix completes `Scaler`'s own documented responsibility (apply all three weighted channels) without adding unrelated scope. | — |

**Reproduced:** YES -- new test asserts translation/scale are no longer frozen at
`Node::new()`'s defaults after `set()`; confirmed by direct code reading that pre-fix, neither
channel was touched at all. Scoped suite (10/10), full workspace (1911/1911), doctests, and
clippy all clean, 2026-08-16.

## Refs: src/

| File | Change |
|------|--------|
| `module/helper/renderer/src/webgl/animation/scaling.rs` | Added `scaled_translation_apply`/`scaled_scale_apply`; wired both into `AnimatableComposition::set` for grouped nodes, with a `Fix(BUG-184)` comment. |

## Refs: tests/

| File | Change |
|------|--------|
| `module/helper/renderer/tests/scaler_tests.rs` | Added `test_scaler_applies_translation_and_scale_to_grouped_nodes`. |
