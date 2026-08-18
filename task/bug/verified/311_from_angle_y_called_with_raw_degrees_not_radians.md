# BUG-311: `Quat::from_angle_y( 90.0 )` called with a raw degree literal instead of radians at 3 sibling example call sites, producing a ~116.62° rotation instead of 90°

- **Severity:** Medium (active, visually-wrong behavior -- not latent -- but confined to 3
  non-critical example/demo crates, not library code; the library API itself, `ndarray_cg`'s
  `Quat::from_angle_y`, is correct and correctly documented)
- **state:** Verified
- **Affects:** the "clouds" mesh orientation in 3 sibling `minwebgl` example crates
- **Component:** `examples/minwebgl/curve_surface_rendering`,
  `examples/minwebgl/lottie_surface_rendering`, `examples/minwebgl/animation_surface_rendering`
  (fix); `module/math/ndarray_cg` (reproducer test only -- the library API itself is not buggy)
- **repo_identity:** self
- **Filed:** 2026-08-18
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/module/helper/
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/module/helper/ (self)
- **verification_date:** 2026-08-18

## Symptom

`ndarray_cg::Quat::from_angle_y( y : E )` takes its angle **in radians** -- its own doc comment
states "The rotation angle in radians" explicitly, and its implementation applies the half-angle
formula `(angle / two).sin_cos()`, which is only correct for a radians input. All 3 sibling
"surface rendering" example crates build a "clouds" mesh via a byte-identical copy-pasted setup
block that calls `gl::Quat::from_angle_y( 90.0 )`, clearly intending a 90-degree rotation about Y
but passing the raw degree value directly instead of `90.0_f32.to_radians()`.

## Impact

**Who is affected:** anyone running (or visually inspecting the rendered output of)
`curve_surface_rendering`, `lottie_surface_rendering`, or `animation_surface_rendering`.

**What breaks:** `90.0` radians is `90.0 * ( 180 / π ) ≈ 5156.62` degrees; reduced modulo 360° that
is `≈ 116.62°` -- not 90°, and not even close to it (a ~26.62° error, not a rounding-scale
difference). The "clouds" mesh in all 3 examples is therefore rendered rotated to the wrong
orientation about the Y axis. This is a purely cosmetic/example-content defect: nothing panics,
no data is corrupted, and it does not affect any library API consumed elsewhere.

**Entity Scope:** `None` -- source-level call-site defect, not entity directory instances.

## How Discovered

Found during this session's workspace-wide bug-hunt pass, `examples/` review stage, by grepping
for `from_angle_y`/`from_angle_x`/`from_angle_z` calls across all `examples/minwebgl/*` crates
(the same "grep for a known-bad pattern across sibling examples" technique that surfaced BUG-097
and BUG-114 in this same repo). Exactly one hit per crate, all three `gl::Quat::from_angle_y(
90.0 )`, at `curve_surface_rendering/src/main.rs:173`, `lottie_surface_rendering/src/main.rs:177`,
and `animation_surface_rendering/src/main.rs:234`. Checking the surrounding context in all 3 files
confirmed a byte-identical "clouds" mesh setup block (clone from earth, translate, scale, rotate,
`local_matrix_update()`) -- a single copy-pasted defect, not 3 independent mistakes.

## Minimum Reproducible Example

**Verify Command** (library-level reproducer -- the 3 example crates are `fn main()`-only binaries
with no test harness of their own; see Why Not Caught):
```bash
cd /home/user1/pro/lib/yrd_gamedev/cgtools
cargo nextest run -p ndarray_cg -E 'test(test_from_angle_y_rejects_raw_degrees)' --all-features
```
**Expected** (fixed -- this test is not itself a regression test on `ndarray_cg`, since
`from_angle_y` was never buggy; it locks in the correct-vs-wrong boundary the 3 example call sites
crossed): 1 passed / 0 failed.

**Actual** (the raw call-site expression's real behavior): the same test's second assertion
constructs `QuatF64::from_angle_y( 90.0 )` -- the exact expression all 3 example call sites used
pre-fix -- and asserts (via `assert_ne!`) it does NOT equal the correct 90-degree-about-Y
quaternion. This assertion executed and passed, empirically confirming the divergence, not merely
computing it by hand. The magnitude of that divergence, independently derived:
```
90.0 radians = 90.0 * (180 / π) = 5156.620156177409 degrees
5156.620156177409 mod 360       = 116.62015617740872 degrees
```
`from_angle_y( 90.0 )` (the pre-fix call) therefore produces a quaternion representing a
`~116.62°` rotation about Y, not `90°`.

## Hypothesis Table

| ID | Hypothesis | State | Summary | Evidence |
|----|-----------|--------|---------|----------|
| H1 | `Quat::from_angle_y` takes radians, documented explicitly | ✅ Root Cause | `arithmetics.rs:261` doc comment: "The rotation angle in radians."; implementation halves the input directly, no degree conversion anywhere | E1, E2 |
| H2 | All 3 call sites are one copy-pasted defect, not 3 independent bugs | ✅ Verified | Identical surrounding "clouds" mesh setup block (clone/translate/scale/rotate/`local_matrix_update`) confirmed byte-for-byte across all 3 files | E3 |
| H3 | The resulting misrotation is large and unambiguous, not a subtle near-miss | ✅ Verified | `90.0` rad `≈ 116.62°` mod 360°, a `~26.62°` error from the intended `90°` | E4 |
| H4 | Nothing in these 3 crates could have caught this mechanically -- no test harness exists | ✅ Verified | `find examples/minwebgl/{curve,lottie,animation}_surface_rendering -iname '*test*'` returns nothing in any of the 3 crates | E5 |

## Evidence Table

| # | Location | What it shows | Hypothesis |
|---|----------|---------------|------------|
| E1 | `module/math/ndarray_cg/src/quaternion/arithmetics.rs:261` | `/// * \`y\` - The rotation angle in radians.` | H1 |
| E2 | `module/math/ndarray_cg/src/quaternion/arithmetics.rs:263-268` | `from_angle_y` implementation: `let (s,c) = (y/two).sin_cos();` -- no degree conversion | H1 |
| E3 | `examples/minwebgl/{curve,lottie,animation}_surface_rendering/src/main.rs` (pre-fix lines 173/177/234 respectively) | All 3: `clouds.borrow_mut().rotation_set( gl::Quat::from_angle_y( 90.0 ) );`, in identical surrounding context | H1, H2 |
| E4 | Terminal output (MRE section above) | `90.0` rad `= 5156.620156177409°`, `mod 360 = 116.62015617740872°` | H3 |
| E5 | Terminal output (this section, `find` commands) | Empty output for all 3 crates -- no `tests/` directory or test file anywhere in any of them | H4 |

## Root Cause

```
ndarray_cg::Quat::from_angle_y( y : E )  -- documented + implemented as taking RADIANS
  |
  |  examples/minwebgl/curve_surface_rendering/src/main.rs:173     \
  |  examples/minwebgl/lottie_surface_rendering/src/main.rs:177     } -- identical copy-pasted
  |  examples/minwebgl/animation_surface_rendering/src/main.rs:234 /     "clouds" setup block
  |
  +-- gl::Quat::from_angle_y( 90.0 )
         intended: 90 DEGREES about Y
         actual:   90 RADIANS about Y  ==  ~116.62 degrees about Y (mod 360)
```
The library API (`ndarray_cg::Quat::from_angle_y`) is correct and its radians contract is clearly
documented. The defect is entirely at the 3 call sites: a human-readable degree constant (`90`,
clearly meant as "90 degrees") was written directly as the argument without the required
`.to_radians()` conversion, then copy-pasted identically into 2 sibling example crates.

## Why Not Caught

None of the 3 affected crates (`curve_surface_rendering`, `lottie_surface_rendering`,
`animation_surface_rendering`) has a `tests/` directory or any test file (confirmed via `find`,
E5) -- they are `fn main()`-only WebGL demo binaries, verified only by running them in a browser
and looking at the rendered output. `from_angle_y` itself has no way to distinguish a
degrees-shaped caller mistake from a genuine (small) radians value -- `90.0` radians is a
perfectly valid input to the function, just not the value the caller actually intended -- so
nothing at the API boundary could have rejected it either.

## Fix Location

`examples/minwebgl/curve_surface_rendering/src/main.rs:173-178`,
`examples/minwebgl/lottie_surface_rendering/src/main.rs:177-182`,
`examples/minwebgl/animation_surface_rendering/src/main.rs:234-239` (identical change at all 3):

```rust
// Before:
clouds.borrow_mut().rotation_set( gl::Quat::from_angle_y( 90.0 ) );

// After:
clouds.borrow_mut().rotation_set( gl::Quat::from_angle_y( 90.0_f32.to_radians() ) );
```
Source comment (`Fix(BUG-311)`/`Root cause`/`Pitfall`) added immediately above the call at all 3
sites.

**`module/math/ndarray_cg/tests/inc/quat_test/arithmetic.rs`** (new test, added to the existing
crate rather than to the 3 untestable example binaries -- see Why Not Caught and this repo's own
`rulebook.md` § Test placement): `test_from_angle_y_rejects_raw_degrees` asserts
`QuatF64::from_angle_y( 90.0_f64.to_radians() )` matches the closed-form 90-degree-about-Y
quaternion `[0, FRAC_1_SQRT_2, 0, FRAC_1_SQRT_2]` (mirroring the existing BUG-272 closed-form
precedent for `from_angle_y( -90 deg )` in the same file), and separately asserts the raw literal
`90.0` -- what all 3 call sites passed pre-fix -- does NOT produce that same quaternion. This
locks in the correct/incorrect boundary the call sites crossed; it does not itself test the 3
example crates (which remain untestable without a live browser), only the library contract they
misused.

## Prevention

Detection command for the general pattern (a literal argument passed directly to a radians-only
rotation constructor -- coarse by construction, a starting point for human review, not a
zero-false-positive check: it also matches already-correct `N.to_radians()` calls and harmless
`0.0` calls, since a leading digit is all it looks for):
```bash
grep -rn "from_angle_[xyz]( *[0-9]" examples/ --include=*.rs
```
Re-run after this fix (confirmed): still matches 4 pre-existing, already-correct call sites
(`diamond`, `make_cube_map` x2, `character_control`'s `from_angle_y( 0.0 )` -- zero is unambiguous
in any unit -- and `obj_load`/`minimize_wasm`'s `from_angle_y( 180.0f32.to_radians() )`, already
wrapped) plus this fix's own 3 now-corrected `.to_radians()`-wrapped sites -- every remaining
bare-degree-shaped literal (the actual defect pattern) is gone. Each hit still needs a human
glance to tell "already correct" from "still bare," same as before this fix.

**Pitfall:** a rotation constructor documented as taking radians gives no compile-time or run-time
signal when a caller passes a degrees-shaped value instead -- `to_radians()` must be applied
explicitly at every call site that starts from a human-readable degree constant, and a copy-pasted
setup block silently propagates the same mistake to every sibling that reuses it.

## Generalized Version

**Broken assumption:** a human-readable degree constant written directly as the argument to a
radians-only rotation constructor is implicitly converted, or "close enough" to matter.

Fails whenever:
1. A call site passes a literal that reads naturally as degrees (e.g. `90`, `180`, `45`) directly
   to a function whose contract is radians, AND
2. No `.to_radians()` (or equivalent) conversion wraps that literal

**Detection invariant:**
```
for every call site of from_angle_x/from_angle_y/from_angle_z/from_axis_angle:
  the argument expression must be either a computed value, OR explicitly wrapped in .to_radians()
```
Third confirmed instance of an angle-unit-contract violation in this workspace's quaternion code
this session/history, after BUG-120 (`from_axis_angle` missing an internal half-angle conversion)
and BUG-272 (`to_euler_xyz`'s own sign/doubling defect) -- distinct root causes, but all three
concern a rotation API's angle-unit or angle-scale contract being violated somewhere in the
pipeline. Unlike those two, this one is entirely a *caller-side* misuse, not a library defect.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-18 | filed | Found during this session's workspace-wide bug-hunt task, `examples/` review stage, by grepping `from_angle_[xyz]` across all `minwebgl` examples |
| 2026-08-18 | fix_applied | All 3 call sites: `from_angle_y( 90.0 )` -> `from_angle_y( 90.0_f32.to_radians() )` |
| 2026-08-18 | verified | `test_from_angle_y_rejects_raw_degrees` (bug_reproducer) passes; full `ndarray_cg` suite (282 tests) and clippy (native + wasm32 for the 3 example crates, `-D warnings`) clean |

## Refs: src/

- `examples/minwebgl/curve_surface_rendering/src/main.rs` — `from_angle_y( 90.0 )` -> `from_angle_y( 90.0_f32.to_radians() )`
- `examples/minwebgl/lottie_surface_rendering/src/main.rs` — same fix
- `examples/minwebgl/animation_surface_rendering/src/main.rs` — same fix

## Refs: tests/

- `module/math/ndarray_cg/tests/inc/quat_test/arithmetic.rs` — added `test_from_angle_y_rejects_raw_degrees` (bug_reproducer)

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 8/8

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Completeness | — | 🟢 | Adversarial pass hunted for stub sections by re-reading each of the 12+2 headers (mechanically confirmed present via `grep -n "^## "`) for substantive, non-generic content -- none found thin | — |
| D2 | MRE Validity & Reproducibility | 🟠 | 🟢 | Confirming pass accepted the original "Actual" framing as a hand computation; adversarial pass caught that the test's own `assert_ne!` assertion already executes and empirically proves the exact pre-fix call-site expression diverges -- stronger than a manual calculation, and the report undersold it | Reworded MRE's "Actual" paragraph to lead with the executed assertion, keeping the magnitude calculation as corroborating detail |
| D3 | Cross-Reference Integrity | — | 🟢 | `grep -rln "BUG-311" --include=*.rs .` re-run after the ID-collision rename: exactly 4 files, 8 lines, matching `## Refs:` sections exactly in both directions | — |
| D4 | Root Cause Quality | 🟠 | 🟢 | Adversarial pass re-verified the cited `arithmetics.rs:261` line number against current file content (unchanged despite BUG-298's earlier edit landing elsewhere in the same file) -- accurate; separately caught Prevention's detection command claim ("only non-degree-shaped values remain") was false -- the coarse regex also matches 4 pre-existing already-correct call sites | Reworded Prevention to state the command's real (coarse, human-review-assisted) precision instead of an inaccurate zero-false-positive claim |
| D5 | Execution Scope | — | 🟢 | `git diff --stat` on all 5 touched files re-checked: each shows only the intended rotation-fix comment block + one-line change (3 example files) or the one new test function (arithmetic.rs) -- no scope creep | — |
| D6 | Crate Scope Unity | — | 🟢 | One root cause (bare degree literal at a radians-only API), 3 byte-identical copy-pasted instances across 3 sibling crates -- consolidated as one report per FI008's own Generalized Version design intent, not 3 near-duplicate reports | — |
| D7 | Crate Locality | 🟠 | 🟢 | `git status --porcelain` on all touched paths re-confirmed clean of concurrent drift immediately before this gate. Separately, and more significantly: this gate's own re-check exposed that this session's "live highest ID" command (`find task -maxdepth 2 ...`) was blind to `task/bug/{state}/NNN_*.md` paths (one directory deeper than maxdepth 2 reaches) -- the true highest ID had climbed to 310 (a concurrent actor filed BUG-301..310 during this bug's own investigation), silently colliding with this report's originally-drafted "BUG-301" | Corrected the live-ID command to unbounded depth (`find task -type f -name '*.md' \| grep -oE '/[0-9]+_' \| grep -oE '[0-9]+' \| sort -n \| uniq \| tail`), re-derived the true next-free ID (311), and renamed every "BUG-301" reference across all 5 touched files to "BUG-311" before filing -- caught before any real collision landed on disk |
| D8 | Crate Single Responsibility | — | 🟢 | Re-scanned the 3 example files' surrounding "clouds" setup block for any other latent defect noticed-but-deferred during this investigation -- none found; fix stayed scoped to exactly the reported rotation-unit defect | — |
| **Total** | | — | 🟢 | 0 open | 2/2 |

**Reproduced:** YES — `test_from_angle_y_rejects_raw_degrees` exit 0 (1 passed), both assertions
(correct-radians match, raw-degrees mismatch) executed for real. Full `ndarray_cg` suite (282
tests) and `cargo clippy -p ndarray_cg --all-targets --all-features -- -D warnings` clean. All 3
fixed example crates (`curve_surface_rendering`, `lottie_surface_rendering`,
`animation_surface_rendering`) re-checked via `cargo clippy --target wasm32-unknown-unknown
--all-targets --all-features -- -D warnings`, exit 0.
