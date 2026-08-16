# BUG-188: A glTF animation channel with exactly one keyframe is silently dropped

- **Severity:** High (entire animation channels vanish with no diagnostic anywhere -- a valid,
  spec-conformant glTF input produces a silently incomplete animation)
- **state:** Completed (fix landed, empirically validated both directions via a dispatched
  background agent's wasm32/browser test run; native scoped verification clean)
- **Affects:** Every glTF animation channel authored with exactly one keyframe (a legitimate,
  spec-conformant authoring pattern for "hold this value for the whole clip" -- not malformed
  input) -- translation, rotation, and morph-target weight channels alike.
- **Component:** `module/helper/renderer` (`src/webgl/animation/loaders/gltf.rs`)
- **repo_identity:** self
- **Filed:** 2026-08-16
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/ (self)
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/ (self, via a
  dispatched background agent for the wasm32/browser-executed empirical verification)
- **verification_date:** 2026-08-16
- **Related Bugs:** None -- independent defect in the glTF loader, unrelated to the `Scaler`-side
  BUG-185/BUG-186/BUG-198 cluster in the same crate.

## Symptom

```rust
// before, at the end of quat_sequence / vec3_sequence / weights_sequence
Sequence::new( tweens ).ok()
```

`Sequence::new` requires at least 2 players and returns `Err` otherwise; the `.ok()` conversion
silently discards that `Err` (and any diagnostic it might have carried) and the caller stores
`None` for the whole channel with no warning logged anywhere.

## Impact

**Who is affected:** Any glTF asset with a channel authored using exactly one keyframe -- a
normal, spec-conformant way to express "this transform/weight is constant for the entire clip,"
not an authoring mistake. Affects all three channel kinds: translation/scale (`vec3_sequence`),
rotation (`quat_sequence`), and morph-target weights (`weights_sequence`).

**What breaks:** The channel is dropped entirely -- no key for it exists in the loaded
`Sequencer` at all, so the affected node/property silently stays at whatever default the engine
falls back to, with zero indication in logs or return values that anything was lost.

**Magnitude:** Deterministic, not an edge case in the sense of rare input -- any single-keyframe
channel hits this every time, and single-keyframe channels are a normal authoring pattern (e.g.
exported from tools that always emit at least a hold-keyframe for unanimated-but-tracked
properties).

**Entity Scope:** None -- a code-level defect.

## How Discovered

Continuing backlog task #140. Traced `Sequence::new`'s minimum-player-count contract in
`module/helper/animation/src/sequencer.rs` against all 3 tween-collecting functions in
`loaders/gltf.rs` (`quat_sequence`, `vec3_sequence`, `weights_sequence`), each of which builds
`tweens` directly from the glTF's own keyframe count with no lower-bound guard before handing the
`Vec` to `Sequence::new(tweens).ok()`.

## Minimum Reproducible Example

```rust
// glTF animation channel with exactly 1 keyframe (see fixture below): translation held at
// (1.5, 2.5, -3.5) for the whole clip.
let gltf = load( &document, "single_keyframe_translation.gltf", &gl ).await.unwrap();
// pre-fix: gltf.animations[0]'s Sequencer has NO key ending in TRANSLATION_PREFIX at all --
//          the channel silently doesn't exist.
// post-fix: the key exists, its Sequence has 2 players (the lone tween duplicated), and
//           current_get().value_get() == F64x3::from_array([1.5, 2.5, -3.5]).
```

**Verify Command** (browser-only, wasm32 target -- see Pitfall):
```bash
cd module/helper/renderer && wasm-pack test --headless --chrome -- --test animation_tests --features animation -- test_single_keyframe_translation_not_dropped
```

## Hypothesis Table

| ID | Hypothesis | State | Summary | Evidence |
|----|-----------|--------|---------|----------|
| H1 | `Sequence::new`'s minimum-2-players requirement silently rejects a lone tween built from a single-keyframe channel, and the `.ok()` conversion swallows the resulting error with no diagnostic. | ✅ Root Cause | Confirmed: all 3 tween-collecting functions build `tweens` with exactly as many entries as the channel has keyframes, then pass straight to `Sequence::new(tweens).ok()` with no length guard. | E1 |
| H2 | A single-keyframe channel is malformed/out-of-spec input that's reasonable to reject. | ❌ Falsified | glTF's own animation sampler spec permits a single input/output pair; tools commonly emit exactly this for a property that's constant across a clip but still explicitly tracked. Rejecting it silently (vs. erroring loudly, if rejection were truly intended) is inconsistent with treating it as invalid input. | E2 |

## Evidence Table

| # | Location | What it shows | Hypothesis |
|---|----------|---------------|------------|
| E1 | `module/helper/renderer/src/webgl/animation/loaders/gltf.rs`, `quat_sequence`/`vec3_sequence`/`weights_sequence` (pre-fix) | Each ends with `Sequence::new( tweens ).ok()` -- no length check on `tweens` beforehand, and `.ok()` discards `Result::Err` entirely. | H1 ✅ |
| E2 | `assets/gltf/animated/single_keyframe_translation.gltf` (this bug's fixture) | A hand-authored, spec-conformant glTF: one sampler, one accessor pair with `count: 1`, holding translation `(1.5, 2.5, -3.5)` -- valid input, not malformed. | H2 ❌ |

## Root Cause

`quat_sequence`, `vec3_sequence`, and `weights_sequence` each collect one `Tween` per glTF
keyframe pair and hand the resulting `Vec` straight to `Sequence::new(tweens).ok()`.
`Sequence::new` enforces a minimum of 2 players (a structural requirement of the sequencer's own
segment model, which always interpolates *between* two players), but a legitimately-authored
single-keyframe channel produces exactly 1 tween -- `Sequence::new` correctly returns `Err` for
that case, but the `.ok()` conversion silently discards it, and the caller's `if let Some(seq) = ...`
pattern (or equivalent) simply never inserts a key for that channel, with no error path reaching
the loader's own caller at all.

## Why Not Caught

No pre-existing test asset ever authored a channel with exactly one keyframe -- every prior test
fixture used 2+ keyframes (the common case for genuinely time-varying animation), so
`Sequence::new`'s minimum-2 rejection path was never exercised by a real load.

## Fix Applied

Added an identical 3-line guard immediately before each `Sequence::new(tweens).ok()` call, in all
3 tween-collecting functions: if `tweens.len() == 1`, push a clone of that single tween so the
`Vec` satisfies `Sequence::new`'s minimum-2 requirement while preserving the exact authored
meaning (holding one constant value, now expressed as two identical back-to-back tweens rather
than one that gets rejected outright).

## Prevention

New test `test_single_keyframe_translation_not_dropped` (in `animation_tests.rs`, browser/wasm32
-gated like its siblings in the same file) loads a hand-authored single-keyframe-translation
fixture and asserts: (1) the channel's Sequencer key exists at all (`.expect(...)` with a
BUG-188-labeled diagnostic message, so a regression fails loudly with the bug ID rather than a
generic `unwrap` panic), (2) the resulting `Sequence` has exactly 2 players (the post-fix
duplicated shape), and (3) the sampled value matches the fixture's single authored value exactly.

## Pitfall

A `.ok()` conversion on a `Result` returned from a validating constructor (`Sequence::new`'s
own minimum-player-count check) silently converts a real, informative error into an
indistinguishable-from-"nothing to load here" `None` -- the caller has no way to tell "this
channel doesn't exist in the source data" apart from "this channel exists but failed a validation
the caller never sees." Any `.ok()` immediately downstream of a constructor with documented
invariants is worth checking for legitimate inputs that could trip those invariants.

This test suite is browser-only by design (`#[cfg(target_arch = "wasm32")]` +
`wasm_bindgen_test_configure!(run_in_browser)`, shared by every test in `animation_tests.rs`,
since loading a real glTF exercises a real WebGL2 context) -- it cannot be run via a plain native
`cargo test`/`cargo nextest` invocation.

## Generalized Version

**Broken assumption:** "A channel with fewer keyframes than the sequencer's structural minimum is
rare/malformed enough that silently dropping it is acceptable."

**Confirmed general rule:** When a loader's tween/player count is derived directly from
input-data cardinality (one entry per keyframe, per vertex, per whatever the source format
provides), any downstream structural minimum (here, `Sequence::new`'s "at least 2 players") needs
an explicit guard at the point the count is known to be too low -- silently discarding the
`Result` via `.ok()` turns a legitimate, low-but-valid-cardinality input into an invisible full
data loss instead of either handling it (as this fix does) or surfacing it loudly.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-16 | filed | Continuing backlog task #140; traced `Sequence::new`'s minimum-player contract against all 3 tween-collecting functions in `loaders/gltf.rs`. |
| 2026-08-16 | fixed | Dispatched a background agent to add the single-keyframe duplication guard to `quat_sequence`/`vec3_sequence`/`weights_sequence`, author a hand-crafted single-keyframe glTF fixture, and add a browser/wasm32 regression test. |
| 2026-08-16 | scoped-verified | Background agent empirically confirmed both directions via the wasm32/browser test harness: fails pre-fix (channel silently absent, `.expect(...)` panics with the BUG-188-labeled diagnostic), passes post-fix (key present, 2 players, correct sampled value). Native scoped compilation/lint of the surrounding crate confirmed clean. |
| 2026-08-16 | committed | Landed in commit `3843aef7` ("feat: add comprehensive examples and test coverage expansion") alongside a large batch of other accumulated session work; independently re-confirmed post-commit via `git show 3843aef7 -- .../loaders/gltf.rs` (exact 3-guard diff matches this report) and direct reads of the fixture (`assets/gltf/animated/single_keyframe_translation.gltf`) and test file (`animation_tests.rs`'s `test_single_keyframe_translation_not_dropped`). |

## Verification Record

**Tier 2 Dual-Role Self-Check.** Confirming pass: the fix is a minimal, structurally-obvious
guard (duplicate the lone tween to satisfy a documented minimum) whose correctness follows
directly from `Sequence::new`'s own contract; the regression test's 3 assertions (key exists,
player count, sampled value) directly target the exact failure mode described in Symptom/Root
Cause. Adversarial pass: re-derived the diff from `git show` rather than trusting the background
agent's report alone, confirmed the fixture file's keyframe count (`"count": 1`) and its
translation `min`/`max` both equal `[1.5, 2.5, -3.5]` (matching the test's asserted value
independently, not just re-stating the test's own claim), and confirmed all 3 call sites
(`quat_sequence`, `vec3_sequence` used for both translation and scale, `weights_sequence`) carry
the identical guard rather than only a subset. No gaps found.

**Reproduced:** YES, via the dispatched background agent's wasm32/browser test execution (both
directions) -- independently cross-checked post-hoc against the actual committed diff and fixture
content, 2026-08-16.

## Refs: src/

| File | Change |
|------|--------|
| `module/helper/renderer/src/webgl/animation/loaders/gltf.rs` | Added a `if tweens.len() == 1 { tweens.push( tweens[ 0 ].clone() ); }` guard immediately before `Sequence::new( tweens ).ok()` in `quat_sequence`, `vec3_sequence`, and `weights_sequence`, each with a `Fix(BUG-188)` comment. |

## Refs: tests/

| File | Change |
|------|--------|
| `module/helper/renderer/tests/animation_tests.rs` | Added `test_single_keyframe_translation_not_dropped` (wasm32/browser-gated, matching the file's existing 2 tests). |
| `assets/gltf/animated/single_keyframe_translation.gltf` | New hand-authored fixture: single translation keyframe holding `(1.5, 2.5, -3.5)`. |
