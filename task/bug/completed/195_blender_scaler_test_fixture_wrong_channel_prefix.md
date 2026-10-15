# BUG-195: `blender_tests.rs`/`scaler_tests.rs` locally redefine wrong channel-name prefixes

- **Severity:** Medium (test-only defect -- no production impact by itself, but it silently
  disabled all value-level test coverage for `Blender`/`Scaler`'s core blend/apply behavior)
- **state:** Completed
- **Affects:** `module/helper/renderer`'s own test suite only -- `blender_tests.rs` and
  `scaler_tests.rs`.
- **Component:** `module/helper/renderer` (`tests/blender_tests.rs`, `tests/scaler_tests.rs`)
- **repo_identity:** self
- **Filed:** 2026-08-16
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/ (self)
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/ (self)
- **verification_date:** 2026-08-16
- **Related Bugs:** Blocked BUG-183's own regression test from exercising real code until fixed
  first (the new test's `Sequencer::insert` calls used the wrong key, so `Blender::rotation_blend`'s
  lookup silently found nothing). Discovering and fixing this is what allowed BUG-196 to surface
  (the newly-reachable code path exposed a second, independent defect). Note: `scaler_tests.rs`'s
  fixture is now fixed, but this does NOT touch BUG-184 (task #136), a separate, still-open
  production defect in `Scaler::set`/`scaled_rotation_apply` unrelated to channel-name prefixes.

## Symptom

```rust
// before, blender_tests.rs
const TRANSLATION_PREFIX: &str = "_translation";
const ROTATION_PREFIX: &str = "_rotation";
const SCALE_PREFIX: &str = "_scale";
```

```rust
// production, base.rs (real values)
pub const TRANSLATION_PREFIX: &str = ".translation";
pub const ROTATION_PREFIX: &str = ".rotation";
pub const SCALE_PREFIX: &str = ".scale";
```

Both test files locally redefined these three constants with an underscore separator
(`_translation`) instead of importing the real, dot-separated production constants
(`.translation`). Every test using `format!("{node_name}{PREFIX}")` to build a `Sequencer::insert`
key therefore inserted under a key that `Blender`/`Scaler`'s own internal lookups
(`animation.get::<...>(&format!("{name}{ROTATION_PREFIX}"))`, using the REAL production constant)
could never match -- every such lookup silently returned `None`, and the corresponding transform
channel was silently skipped rather than blended/applied.

## Impact

**Who is affected:** Nobody in production -- this defect lives entirely in test fixtures. The
affected population is future maintainers relying on `blender_tests.rs`/`scaler_tests.rs`'s green
status as evidence that `Blender`/`Scaler`'s blend/apply arithmetic is correct.

**What breaks:** Every existing test in both files that constructs a `Sequencer` and inserts a
tween sequence under a `format!("{name}{PREFIX}")` key, then calls `.set()`/`.update()` expecting
that value to actually be applied to a node, silently exercises a no-op lookup instead. The tests
still pass (they don't assert on the resulting node transform value in most cases), but they
provide zero actual coverage of the blend/apply arithmetic.

**Magnitude:** Systemic across both files -- every test using the wrong local constants, which
was all of them (both files defined the constants once at file scope and every test used them).

**Entity Scope:** None -- a code-level test-fixture defect.

## How Discovered

Writing BUG-183's new regression test (the first value-asserting test for `Blender::rotation_blend`)
failed with the blended rotation coming back as pure identity `[0,0,0,1]` -- inconsistent with
either the pre-fix (long-path) or post-fix (short-path) expected values. Traced to the
`Sequencer::insert` key built via `format!("{node_name}{ROTATION_PREFIX}")` using the file's local
`ROTATION_PREFIX = "_rotation"` constant, while `Blender::rotation_blend`'s own lookup used the
real production `ROTATION_PREFIX = ".rotation"` -- confirmed by reading `base.rs` directly for the
real constant values.

## Minimum Reproducible Example

```rust
// test file's local (wrong) constant:
const ROTATION_PREFIX: &str = "_rotation";
sequencer.insert( format!( "node1{ROTATION_PREFIX}" ).as_str(), seq );  // inserts "node1_rotation"

// Blender::rotation_blend's internal lookup (real production constant):
animation.get::<...>( &format!( "{name}{ROTATION_PREFIX}" ) )  // looks up "node1.rotation" -- MISS
```

**Verify Command** (<=3 lines, standalone):
```bash
cd module/helper/renderer && cargo nextest run --all-features --test blender_tests --test scaler_tests
```

## Hypothesis Table

| ID | Hypothesis | State | Summary | Evidence |
|----|-----------|--------|---------|----------|
| H1 | Both test files locally redefine the three channel-name prefix constants with the wrong separator, instead of importing the real production constants from `base.rs`. | ✅ Root Cause | Confirmed by reading both files' constant definitions and diffing against `base.rs`'s real values. | E1 |
| H2 | This silently changed the behavior of pre-existing tests that DO assert on blended/applied transform values, i.e. this fix is not safe to apply without also updating expected values. | ❌ Falsified | Read every pre-existing test in both files; none assert on a blended/applied node-transform VALUE -- only on `Blender`/`Scaler`'s own weight/group bookkeeping getters (`weights_get`, `scale_get`, `group_get`) or panic-freedom (`update` "should not panic"). The fix is behaviorally inert for every pre-existing test. | E2 |

## Evidence Table

| # | Location | What it shows | Hypothesis |
|---|----------|---------------|------------|
| E1 | `module/helper/renderer/src/webgl/animation/base.rs` vs. pre-fix `tests/blender_tests.rs`/`tests/scaler_tests.rs` | Production: `.translation`/`.rotation`/`.scale` (dot). Tests (pre-fix): `_translation`/`_rotation`/`_scale` (underscore). | H1 ✅ |
| E2 | `module/helper/renderer/tests/blender_tests.rs`, `tests/scaler_tests.rs` (all pre-existing tests, read in full) | Every existing assertion targets `weights_get`/`scale_get`/`group_get`/`is_completed`/panic-freedom -- zero assertions on a blended/applied node-transform value. | H2 ❌ |

## Root Cause

Both test files independently defined their own local copies of the channel-name prefix constants
instead of importing the real ones from `renderer::webgl::animation::base`, and the locally
redefined values used the wrong separator character, making every `format!`-built lookup key a
guaranteed miss against production's own internal lookups.

## Why Not Caught

No pre-existing test asserted on the actual result of a blend/apply operation, so the silent
lookup miss produced no observable test failure -- every test that touched this path either
checked bookkeeping unrelated to the lookup, or checked only "does not panic," both of which
remain true whether the lookup hits or silently misses.

## Fix Location

- `module/helper/renderer/tests/blender_tests.rs`: removed the local wrong constants; added
  `base::{ TRANSLATION_PREFIX, ROTATION_PREFIX, SCALE_PREFIX }` to the existing
  `renderer::webgl::animation::{...}` import.
- `module/helper/renderer/tests/scaler_tests.rs`: removed the local wrong constants; added
  `base::{ TRANSLATION_PREFIX, ROTATION_PREFIX }` to the existing
  `renderer::webgl::animation::{...}` import (this file has no scale-channel test, so `SCALE_PREFIX`
  was not needed).

## Prevention

Both fixes were verified safe (not merely assumed) by reading every pre-existing test in both
files before applying the fix, confirming none depend on the wrong constants' values for any
assertion -- then re-running both files' full test suites post-fix (20/20 in `blender_tests.rs`,
8/8 in `scaler_tests.rs`) to empirically confirm zero regressions. BUG-183's new test is itself
additional prevention: it is the first test in either file to assert on an actual blended
transform value, so a future reintroduction of this class of defect would now be caught.

## Pitfall

A test file that locally redefines a constant "for convenience" instead of importing the real one
creates a silent drift risk: if production's value ever changes, or (as here) was simply
transcribed wrong at authoring time, nothing forces the two to stay in sync, and a mismatch
produces no compile error -- only a silently-skipped code path. Prefer importing the real constant
directly over redefining an equivalent local copy, even for test-only convenience wrappers.

## Generalized Version

**Broken assumption:** "a test file's own local constants are just a convenience mirror of
production's; they don't need independent verification."

**Confirmed general rule:** When a test file locally redefines a value that has a canonical
production source, treat the redefinition itself as a suspect until confirmed identical to the
source (or better, replaced with a direct import) -- a passing test suite does not prove the
mirrored value is correct if no test actually depends on the mirrored value producing an observable
effect.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-16 | filed | Discovered while writing BUG-183's regression test -- the blended result came back as identity, traced to a channel-lookup key mismatch. |
| 2026-08-16 | fixed | Removed both files' local wrong constants; imported the real production constants from `base::` instead. |
| 2026-08-16 | verified | Read every pre-existing test in both files first to confirm the fix is behaviorally inert for all of them. `cargo nextest run -p renderer --test blender_tests --all-features`: 20/20 passed. `cargo nextest run -p renderer --test scaler_tests --all-features`: 8/8 passed. Full workspace: `cargo nextest run --workspace --all-features --exclude object_picking`: 1909/1909 passed. `cargo clippy --workspace --all-targets --all-features --exclude object_picking -- -D warnings`: clean. |

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 8/8

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Completeness | 🟢 | 🟢 | All FI008 sections present. | — |
| D2 | MRE Validity & Reproducibility | 🟢 | 🟢 | Confirming: diffed local vs. production constants directly. Adversarial: attempted to find a pre-existing test that WOULD regress from this fix -- read all 28 tests across both files line by line, found none. | — |
| D3 | Cross-Reference Integrity | 🟢 | 🟢 | Cross-referenced against BUG-183 (blocked by this), BUG-196 (surfaced by this), BUG-184 (explicitly noted as NOT fixed by this). | — |
| D4 | Root Cause Quality | 🟢 | 🟢 | Backed by direct file reads and diff against `base.rs`, not assumed. | — |
| D5 | Execution Scope | 🟢 | 🟢 | Fix is limited to import statements; no test logic or assertions changed. | — |
| D6 | Crate Scope Unity | 🟢 | 🟢 | Entirely within `renderer`'s own test suite. | — |
| D7 | Crate Locality | 🟢 | 🟢 | Each file's wrong constants fixed at their one definition site. | — |
| D8 | Crate Single Responsibility | 🟢 | 🟢 | Fix only corrects the fixture's channel-name source, no scope change. | — |

**Reproduced:** YES -- confirmed via BUG-183's test failing with an identity result pre-fix
(proving the lookup miss) and succeeding post-fix. Full workspace native suite (1909/1909, 0
skipped), doctests, and clippy all clean (excluding the concurrent actor's unrelated
`object_picking` in-flight refactor), 2026-08-16.

## Refs: src/

None -- test-fixture-only defect, no production source changed.

## Refs: tests/

| File | Change |
|------|--------|
| `module/helper/renderer/tests/blender_tests.rs` | Removed local wrong `TRANSLATION_PREFIX`/`ROTATION_PREFIX`/`SCALE_PREFIX` constants; imported the real ones from `base::`. |
| `module/helper/renderer/tests/scaler_tests.rs` | Removed local wrong `TRANSLATION_PREFIX`/`ROTATION_PREFIX` constants; imported the real ones from `base::`. |
