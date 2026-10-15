# BUG-175: Spot-light shadow softness scaling is dead arithmetic

- **Severity:** Medium (visual-fidelity only -- shadows still render correctly, but every spot
  light bakes identically tight/hard soft-shadow penumbrae regardless of its actual cone angle;
  no crash, no incorrect-but-plausible geometry)
- **state:** Completed
- **Affects:** Any scene using `ShadowBaker::soft_shadow_render` with a `Light` built via
  `From<SpotLight>` -- i.e. every spot light's baked shadow softness.
- **Component:** `module/helper/renderer` (`src/webgl/shadow.rs`)
- **repo_identity:** self
- **Filed:** 2026-08-16
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verification_date:** 2026-08-16
- **Related Bugs:** Independent discovery from `task/readme.md`'s task #98 review pass (not found
  by investigating another bug this session).

## Symptom

```rust
// pre-fix -- webgl/shadow.rs, impl From<SpotLight> for Light
// Light size affects shadow softness - derive from cone angle
// Smaller angles = tighter beam = smaller physical size
let radius = spot.outer_cone_angle * 2.0;
let max_radius = 135.0_f32.to_radians();

let light_size = ( ( radius / max_radius ).min( 1.0 ) * 1.7 ).min( 0.01 );
```

The final `.min( 0.01 )` is a *ceiling*, not the floor its position (last operation, small
constant) suggests. The preceding term `( radius / max_radius ).min( 1.0 ) * 1.7` is `>= 0.01`
for every `outer_cone_angle` above roughly 0.4 degrees -- i.e. every physically realistic spot
light -- so `.min` always discards it and returns the constant `0.01`, unconditionally.

## Impact

**Who is affected:** Every spot light shadow baked through `ShadowBaker::soft_shadow_render` --
`Light::size()` feeds the `u_light_size` uniform (`shadow.rs:263-264`, `light_upload`), which
`bake.frag`'s PCSS pass reads directly as `light_world_size`, driving the penumbra-size formula
`penumbra = (receiver - blocker) * lightSize / blocker` (`bake.frag:165,202`).

**What breaks:** With `light_size` pinned at `0.01` for every spot light regardless of cone angle,
every spot light in a scene bakes an identically tight, barely-soft shadow penumbra -- a spot
light with a wide 80-degree cone (expected: a large, diffuse light source, soft penumbra) renders
indistinguishably from one with a narrow 5-degree cone (expected: a near-point source, hard
penumbra). No crash, no NaN, no visibly "broken" geometry -- purely a silently-flattened visual
parameter that never does what its own doc comment says it does.

**Magnitude:** Universal across every spot light in the workspace -- the defect is in the shared
`From<SpotLight> for Light` conversion itself, not any one caller.

**Entity Scope:** None -- a code-level defect.

## How Discovered

Pre-identified by an earlier review pass (task #98, this session) as "spot-light shadow softness
scaling is dead arithmetic"; this bug's work was to confirm, precisely characterize, and fix it.
Confirmed empirically: hand-derived the crossover point at which the `.min(1.0)*1.7` term drops
below `0.01` (`outer_cone_angle < ~0.007 radians`, `~0.4 degrees`) and confirmed via direct source
read that `light_size` (`Light::size()`) feeds a real, visually meaningful PCSS penumbra
calculation in `bake.frag`, not a cosmetic or already-clamped-elsewhere value.

## Minimum Reproducible Example

```text
radius = outer_cone_angle * 2.0          max_radius = 135deg in radians = 2.356

outer_cone_angle =  5deg (0.0873 rad): radius=0.1745, ratio=0.0741, *1.7=0.1259 -> min(0.01)=0.01
outer_cone_angle = 80deg (1.3963 rad): radius=2.7925, ratio=1.0*   , *1.7=1.7   -> min(0.01)=0.01
  ( * ratio clamped to 1.0 by the inner .min(1.0), since 2.7925/2.356 > 1.0 )

Both a near-point-source narrow cone and a wide, diffuse-source cone produce the exact same
light_size = 0.01 -- the entire angle-dependent computation between them never has any effect.
```

**Expected** (post-fix): the two cases above produce visibly different `light_size` values --
narrow cone floors at `0.01`, wide cone reaches `1.7`.

**Actual** (pre-fix): both produce exactly `0.01`.

**Verify Command** (<=3 lines, standalone):
```bash
cd module/helper/renderer && cargo nextest run -p renderer webgl::shadow::wide_cone_produces_a_larger_light_size_than_narrow_cone
```

## Hypothesis Table

| ID | Hypothesis | State | Summary | Evidence |
|----|-----------|--------|---------|----------|
| H1 | `.min( 0.01 )` was written where `.max( 0.01 )` (a lower-bound floor) was intended, making the entire preceding angle-dependent scaling term dead for all realistic inputs. | ✅ Root Cause | Confirmed by hand-deriving the crossover point (~0.4 degrees) below which `.min` even changes behavior at all, and cross-checking against the doc comment's own explicit "smaller angles = tighter beam = smaller physical size" scaling intent, which only makes sense if the term is allowed to vary. | E1, E2, E3 |
| H2 | `0.01` is an intentional fixed constant (e.g. a deliberate design choice to keep all spot-light shadows equally hard), and the preceding scaling computation is intentionally-inert legacy code. | ❌ Falsified | No code author writes a 3-line normalize/clamp/scale computation immediately before unconditionally discarding it -- the doc comment directly above it explicitly describes a scaling relationship ("Smaller angles = tighter beam = smaller physical size"), which is meaningless if the output can never vary. | E1, E4 |

## Evidence Table

| # | Location | What it shows | Hypothesis |
|---|----------|---------------|------------|
| E1 | `module/helper/renderer/src/webgl/shadow.rs` (pre-fix, `impl From<SpotLight> for Light`) | The exact clamp expression and its doc comment stating an intended scaling relationship. | H1 ✅, H2 ❌ |
| E2 | `module/helper/renderer/src/webgl/shadow.rs:263-264` (`light_upload`) + `src/webgl/shaders/bake.frag:11,165,202` | `light.size()` feeds `u_light_size` -> `light_world_size` -> a real PCSS penumbra formula, confirming this is a visually meaningful parameter, not dead/unused state. | H1 ✅ |
| E3 | Hand-derivation (see Minimum Reproducible Example) | The `.min(1.0)*1.7` term exceeds `0.01` for any `outer_cone_angle` above ~0.4 degrees -- effectively all realistic spot lights. | H1 ✅ |
| E4 | Workspace-wide search, zero pre-existing tests of `From<SpotLight> for Light` or `Light::size()` | No caller or test ever compared two different cone angles' resulting `light_size`, so the constant-output behavior was never observed against a second data point. | H2 ❌ |

## Root Cause

```rust
// before -- .min acts as a ceiling that always wins over the preceding scaling term
let light_size = ( ( radius / max_radius ).min( 1.0 ) * 1.7 ).min( 0.01 );
```

The trailing clamp's direction is inverted relative to its evident intent: positioned last, with a
small constant, it reads as a lower-bound safety floor (avoiding a degenerate near-zero size at a
near-zero cone angle) -- but `.min` instead makes it an upper bound that is, for all practical
purposes, always tighter than the value it's supposed to be bounding.

## Why Not Caught

`From<SpotLight> for Light` had zero test coverage of any kind prior to this bug -- no test
anywhere constructed more than one `SpotLight` and compared the resulting `Light::size()` values,
so the "constant regardless of input" behavior was never exercised against a second data point
that would have exposed it.

## Fix Location

`module/helper/renderer/src/webgl/shadow.rs`, `impl From<SpotLight> for Light`: changed the
trailing `.min( 0.01 )` to `.max( 0.01 )`, restoring it as a lower-bound floor (still guarding
against a degenerate near-zero size at a near-zero cone angle) while letting the angle-dependent
scaling term -- unchanged -- actually reach the caller.

## Prevention

3 new tests added, `module/helper/renderer/tests/webgl/shadow.rs`:
`wide_cone_produces_a_larger_light_size_than_narrow_cone` (the primary regression test -- pre-fix,
both sides of this assertion were exactly `0.01`), `near_zero_cone_angle_floors_at_a_sane_minimum_size`
(confirms the floor still engages for a genuinely degenerate input), and
`wide_cone_light_size_is_well_above_the_floor` (confirms the wide-cone case reaches a meaningfully
different value, not just barely-greater by floating-point noise).

## Pitfall

A `.min( FLOOR )`/`.max( FLOOR )` mixup still compiles cleanly and always returns *a* value within
a plausible range -- there is no type error, no panic, no NaN to reveal it. The only way to expose
a clamp silently discarding the computation that feeds it is a test that compares outputs across
at least two distinct inputs; a single-input smoke test (or none at all, as here) cannot
distinguish "correctly scaled" from "accidentally constant."

## Generalized Version

**Broken assumption:** "a clamp positioned last, using a small constant, is self-evidently a
safety floor -- its correctness doesn't need a dedicated test."

**Confirmed general rule:** `.min`/`.max` are easy to transpose since both compile and both return
an in-range value for any input; a clamp's *direction* (floor vs. ceiling) is a semantic fact
about the surrounding formula that only a differential test (comparing outputs across varied
inputs) can verify -- position and constant size alone are not evidence of correctness.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-16 | filed | Pre-identified by task #98's review pass; confirmed and precisely characterized this session via hand-derivation of the crossover point and direct source read of the downstream PCSS consumer. |
| 2026-08-16 | fixed | Changed `.min( 0.01 )` to `.max( 0.01 )` in `impl From<SpotLight> for Light`, restoring it as a lower-bound floor instead of an always-winning ceiling. |
| 2026-08-16 | verified | Workspace `cargo check --workspace --all-targets --all-features`: clean. `cargo nextest run -p renderer --all-features`: 104/104 passed (3 new, individually confirmed PASS against the correct, freshly-verified log file after an initial stale-log mixup was caught and corrected mid-verification). `cargo test --doc -p renderer --all-features`: 3/3 passed. `cargo clippy -p renderer --all-targets --all-features -- -D warnings`: clean. |

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 8/8

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Completeness | 🟢 | 🟢 | All FI008 sections present. | — |
| D2 | MRE Validity & Reproducibility | 🟢 | 🟢 | Confirming pass derived the crossover point and wrote the primary differential test. Adversarial pass re-checked the verification log itself: the first `grep` against a hardcoded log filename (`-0006_longrun.log`) silently matched a stale file from earlier in this long session, returning a spurious "1793 tests run" figure -- caught by cross-checking `ls -t` timestamps against the `.wait` call's own reported time before trusting it, then re-verified against the correct, freshly-written log (`-0088_longrun.log`, 104/104, all 3 new tests individually PASS). | — |
| D3 | Cross-Reference Integrity | 🟢 | 🟢 | Cross-referenced against BUG-171/172/173/174/189 (same review-pass area); no overlap -- disjoint code path (`shadow.rs`'s `From<SpotLight>`, untouched by any prior bug this session). | — |
| D4 | Root Cause Quality | 🟢 | 🟢 | Backed by direct source read plus a hand-derived numeric crossover point, not inferred from the diff's shape alone. | — |
| D5 | Execution Scope | 🟢 | 🟢 | Fix is a single clamp-direction correction; no unrelated refactor attempted. | — |
| D6 | Crate Scope Unity | 🟢 | 🟢 | Entirely within `renderer`'s own `shadow.rs` + its own new test file; no downstream call sites needed updating (unlike BUG-174 -- this is a value-level fix, not a signature change). | — |
| D7 | Crate Locality | 🟢 | 🟢 | Confirmed via grep that `From<SpotLight> for Light` has exactly one definition site, already fixed. | — |
| D8 | Crate Single Responsibility | 🟢 | 🟢 | The fix corrects the conversion's existing responsibility (derive light size from cone angle); no responsibility added or removed. | — |

**Reproduced:** YES (by direct mathematical derivation against the real formula, cross-checked
against the real downstream PCSS consumer in `bake.frag`) -- the primary regression test
(`wide_cone_produces_a_larger_light_size_than_narrow_cone`) fails against the pre-fix `.min(0.01)`
clamp (both sides evaluate to exactly `0.01`) and passes post-fix. Full scoped suite (104/104,
+3 new), doctest (3/3), and clippy all clean, 2026-08-16.

## Refs: src/

| File | Change |
|------|--------|
| `module/helper/renderer/src/webgl/shadow.rs` | `impl From<SpotLight> for Light`: `.min( 0.01 )` -> `.max( 0.01 )` on the final `light_size` clamp (full `Fix(BUG-175)` comment block). |

## Refs: tests/

| File | Change |
|------|--------|
| `module/helper/renderer/tests/webgl/shadow.rs` | New file, 3 tests: primary differential regression test (`wide_cone_produces_a_larger_light_size_than_narrow_cone`), floor-still-engages test, and wide-cone-well-above-floor test. Registered via `mod shadow;` in `tests/webgl/mod.rs`. |
| `module/helper/renderer/tests/readme.md` | Added `webgl/shadow.rs` Responsibility Table row. |
