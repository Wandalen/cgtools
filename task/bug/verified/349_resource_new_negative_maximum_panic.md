# BUG-349: `Resource::new`/`with_regeneration` accept a negative `maximum` unclamped, causing a panic on the first `modify`/`current_set` call

- **Severity:** Medium
- **state:** Verified
- **Affects:** Any `Resource::new(maximum)` or `Resource::with_regeneration(maximum, ...)` call
  with a negative `maximum` (e.g., a miscalculated stat, a debuff applied before clamping, bad
  save data), followed by any call to `modify` or `current_set`
- **Component:** `module/helper/tiles_tools` (`src/game_systems.rs`, `Resource::new`,
  `Resource::with_regeneration`)
- **repo_identity:** self
- **Filed:** 2026-08-18
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/module/helper/
- **Fix Task:** [384](../../verifying/384_register_tiles_tools_resource_new_negative_maximum_clamp_fix_closes_bug349.md)

## Symptom

`Resource::maximum_set` correctly clamps its input to a non-negative value, but `Resource::new`
and `Resource::with_regeneration` do not — constructing a `Resource` with a negative `maximum`
succeeds silently, then panics on the very next `modify`/`current_set` call:

```
# Correct (maximum_set already clamps):
let mut r = Resource::new(10.0);
r.maximum_set(-5.0);
r.maximum  -> 0.0                        # clamped, no panic

# Wrong (new/with_regeneration do NOT clamp):
let mut r = Resource::new(-5.0);
r.maximum  -> -5.0                       # unclamped -- constructed successfully
r.modify(1.0);                           # panics:
# thread panicked: min > max, or either was NaN. min = 0.0, max = -5.0
```

## Impact

**Who is affected:** any code constructing a `Resource` (health, mana, stamina, or any other
regenerating/clampable stat) from a value that can be negative — a computed stat before its own
clamping, a debuff/multiplier applied to a base value, deserialized save data, or a test fixture
— then later calling `modify` or `current_set` on it (the two most common ways to change a
resource's current value during gameplay).

**What breaks:** a hard panic (`f32::clamp`'s internal `assert!(min <= max)`) crashes the calling
thread the first time `modify`/`current_set` runs against a `Resource` built with a negative
`maximum` — not a silent logic error like BUG-347/348, but an outright crash, and one that can be
far removed in time and call stack from the actual `Resource::new`/`with_regeneration` call that
created the invalid state.

**Magnitude:** 2 constructors (`new`, `with_regeneration`); every value-producing path into
`Resource` except `maximum_set` (which already clamps correctly) is affected. `modify` and
`current_set` are the crash sites, not the defect's origin.

**Entity Scope:** `None` — a code-level invariant-enforcement gap, not entity directory instances.

## How Discovered

```bash
$ cargo test -p tiles_tools --all-features --test game_systems_test \
    test_resource_new_with_negative_maximum_does_not_panic_on_modify -- --exact

thread 'test_resource_new_with_negative_maximum_does_not_panic_on_modify'
panicked at /rustc/8bab26f4f68e0e26f0bb7960be334d5b520ea452/library/core/src/num/f32.rs:1565:9:
min > max, or either was NaN. min = 0.0, max = -5.0
test result: FAILED. 9 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

A prior investigation pass identified the missing clamp by direct reading of `Resource::new`
alongside its sibling `maximum_set` (§ Hypothesis Table below); this report re-confirms it with
the permanent reproducer test above, run against the pre-fix source.

## Minimum Reproducible Example

**Verify Command** (run from repo root; ≤3 lines):
```bash
cargo test -p tiles_tools --all-features --test game_systems_test \
  test_resource_new_with_negative_maximum_does_not_panic_on_modify -- --exact
```
**What:** `Resource::new` with a negative `maximum` must clamp to a non-negative value (matching
`maximum_set`'s existing invariant), not construct an invalid `Resource` that panics on the next
`modify`/`current_set` call.

**Expected** (fixed): test passes — `test test_resource_new_with_negative_maximum_does_not_panic_on_modify ... ok`.

**Actual** (pre-fix, directly confirmed by running the same test against the current, unfixed
source before applying the fix below):
```
thread 'test_resource_new_with_negative_maximum_does_not_panic_on_modify'
panicked at /rustc/8bab26f4f68e0e26f0bb7960be334d5b520ea452/library/core/src/num/f32.rs:1565:9:
min > max, or either was NaN. min = 0.0, max = -5.0
test result: FAILED. 9 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

## Hypothesis Table

| ID | Hypothesis | State | Summary | Evidence |
|----|-----------|--------|---------|----------|
| H1 | `Resource::new` (game_systems.rs:564, pre-fix) stores `maximum` directly into both `current` and `maximum` fields with no clamping, unlike `maximum_set` (game_systems.rs:603, pre-fix) which clamps via `self.maximum = value.max(0.0)` | ✅ Root Cause | Direct read: `new`'s body is `Self { current: maximum, maximum, regeneration: 0.0 }` — no `.max(0.0)` anywhere; `maximum_set`'s body explicitly clamps | E1 |
| H2 | `Resource::with_regeneration` (game_systems.rs:574, pre-fix) shares the identical unclamped-construction defect as `new` | ✅ Verified | Direct read: `with_regeneration`'s body is `Self { current: maximum, maximum, regeneration }` — same unclamped pattern as `new` | E1 |
| H3 | `modify` (game_systems.rs:593) and `current_set` (game_systems.rs:598) both call `.clamp(0.0, self.maximum)`, and Rust's `f32::clamp` panics via an unconditional `assert!(min <= max)` whenever `self.maximum` is negative (making `0.0 <= self.maximum` false) | ✅ Verified | Direct read of both methods' bodies; standard library panic message (`min > max, or either was NaN`) confirmed verbatim in the reproducer's terminal output, sourced from `library/core/src/num/f32.rs:1565` | E2 |
| H4 | The panic site (inside `f32::clamp`, called from `modify`/`current_set`) is a different location from the defect's actual origin (`new`/`with_regeneration`), so a stack trace alone points at the wrong function to fix | ✅ Verified | Terminal evidence (E3): panic location is `f32.rs:1565` (standard library internals) with no `Resource::new`/`with_regeneration` frame visible without `RUST_BACKTRACE`, even though the actual defect is 2 calls upstream, at construction time | E3 |

## Evidence Table

| # | Location | What it shows | Hypothesis |
|---|----------|---------------|------------|
| E1 | `module/helper/tiles_tools/src/game_systems.rs:564-579` (`new`, `with_regeneration`, pre-fix, direct read via `git show HEAD:...`) | Both constructors assign `maximum` straight into the struct with no `.max(0.0)`, unlike `maximum_set` | H1, H2 |
| E2 | `module/helper/tiles_tools/src/game_systems.rs:593-601` (`modify`, `current_set`, direct read) | Both call `.clamp(0.0, self.maximum)`; `maximum_set` (603-606) clamps `self.maximum = value.max(0.0)` — establishing the invariant `new`/`with_regeneration` fail to enforce | H3 |
| E3 | Terminal output (this report, MRE section; also captured in `-0001_longrun.log:107-118`, pre-fix combined test run) | Panic fires inside `core::num::f32::clamp` (`f32.rs:1565:9`) with message `min > max, or either was NaN. min = 0.0, max = -5.0`, confirming `self.maximum` was `-5.0` at the `modify` call site | H3, H4 |

## Root Cause

```
Resource::new(-5.0)
  -> Self { current: -5.0, maximum: -5.0, regeneration: 0.0 }   # constructed successfully, no clamp

r.modify(1.0)
  -> self.current = (self.current + amount).clamp(0.0, self.maximum)
                                              ^^^^^^^^^^^^^^^^^^^^^^^
                                              .clamp(0.0, -5.0)
                                              -> f32::clamp asserts min <= max unconditionally
                                              -> 0.0 <= -5.0 is false -> PANIC
```
`modify` and `current_set` both assume `self.maximum >= 0.0` (a precondition `f32::clamp` itself
enforces via a hard `assert!`), and `maximum_set` correctly maintains that invariant by clamping
its input. But `new` and `with_regeneration` — the two paths that establish a `Resource`'s initial
`maximum` — never enforce the same invariant, so a negative constructor argument produces a
`Resource` that is invalid from the moment it exists, deferring the actual failure to whichever
`modify`/`current_set` call happens to run first.

## Why Not Caught

Every existing `Resource` test constructed instances with positive `maximum` values (health pools,
mana pools, stamina — all naturally non-negative in the test fixtures used) — none exercised
`Resource::new`/`with_regeneration` with a negative argument, so the missing clamp had no path to
surface. `maximum_set`'s own tests confirm *that* method clamps correctly, but no test cross-checked
whether the *constructors* enforced the same invariant `maximum_set` does.

## Fix Location

**`module/helper/tiles_tools/src/game_systems.rs:564`** (`Resource::new`, pre-fix) and
**`:574`** (`Resource::with_regeneration`, pre-fix):

```rust
// Before:
pub fn new(maximum: f32) -> Self {
  Self {
    current: maximum,
    maximum,
    regeneration: 0.0,
  }
}

pub fn with_regeneration(maximum: f32, regeneration: f32) -> Self {
  Self {
    current: maximum,
    maximum,
    regeneration,
  }
}

// After:
pub fn new(maximum: f32) -> Self {
  // Fix(BUG-349): clamp maximum to a non-negative value, matching the
  // invariant maximum_set already enforces (`self.maximum = value.max(0.0)`).
  // Root cause: modify/current_set both call `.clamp(0.0, self.maximum)`,
  // and f32::clamp asserts `min <= max` unconditionally -- a negative
  // maximum stored here made every later modify/current_set call panic.
  // Pitfall: a sibling setter (maximum_set) enforcing an invariant
  // correctly is not evidence every value-producing path (new,
  // with_regeneration) enforces the same invariant -- check each one.
  let maximum = maximum.max(0.0);
  Self {
    current: maximum,
    maximum,
    regeneration: 0.0,
  }
}

pub fn with_regeneration(maximum: f32, regeneration: f32) -> Self {
  // Fix(BUG-349): see `Resource::new` -- same clamp, same root cause.
  let maximum = maximum.max(0.0);
  Self {
    current: maximum,
    maximum,
    regeneration,
  }
}
```

## Prevention

Detection command for the general pattern (a struct with a setter that clamps a field, and one or
more constructors that assign the same field without the equivalent clamp):
```bash
grep -n "fn new(maximum\|fn with_regeneration\|fn maximum_set" module/helper/tiles_tools/src/game_systems.rs
```
This is a starting point for review, not a precise check — confirming correctness requires a test
that constructs via each value-producing path with an out-of-invariant input and asserts the
result is still valid, which is exactly what the new reproducer test adds. Any future
`Resource`-producing constructor (or any other type with a clamped setter) should apply the same
clamp its setter uses, not just its setter.

**Pitfall:** a sibling setter enforcing an invariant correctly is not evidence every
value-producing path (constructors included) enforces the same invariant — every path that can
produce a value must be checked individually, especially when the eventual failure (a panic deep
inside a stdlib function called much later) is far removed from the actual point of construction.

## Generalized Version

**Broken assumption:** "if one method on a type (a setter) correctly clamps/validates a field,
every other method that can set that same field (constructors, other setters) does too."

Fails for any type whenever:
1. A field has an invariant (non-negative, bounded range, non-empty) enforced by at least one
   setter, AND
2. The same field can also be assigned by a constructor or a different setter that was added
   separately (or earlier) and never updated to match, AND
3. Something downstream (here, `f32::clamp`'s own internal assertion) trusts the invariant
   unconditionally rather than re-validating it, turning a silent bad value into a deferred panic.

**Detection invariant:**
```
for every field F with an invariant enforced by at least one setter S:
  every constructor and setter that can assign F must apply the same invariant as S
  -- an invariant enforced by only SOME value-producing paths is not actually enforced
```
Confirmed as a single instance in this crate (`maximum` is the only `Resource` field with a
clamping setter (`maximum_set`) whose invariant is not mirrored by all constructors; `current` is
always derived from an already-clamped `maximum` in both `new` and `with_regeneration`, so it
does not share this gap once the fix is applied). Dedup search:
`grep -rli "Resource::new\|with_regeneration" task/bug/` found no prior hits — not a duplicate of
any existing bug report in this repository.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-18 | filed | Re-confirmed via direct source reading and a new permanent reproducer test, following up a prior investigation pass's finding |
| 2026-08-18 | note | SUBMIT: state Draft -> Unverified; reproducer confirmed FAIL pre-fix and PASS post-fix, fix applied, full scoped suite (`cargo test -p tiles_tools --all-features`) green |
| 2026-08-18 | VERIFY Gate | Reproducer test test_resource_new_with_negative_maximum_does_not_panic_on_modify confirmed passing against current source (`cargo test -p tiles_tools --all-features --test game_systems_test ... -- --exact`: 1 passed; 0 failed); fix in module/helper/tiles_tools/src/game_systems.rs confirmed present at line 573 (`Resource::new`, `let maximum = maximum.max(0.0);`) and line 585 (`Resource::with_regeneration`, same clamp). state: Unverified -> Verified |

## Refs: src/

- `module/helper/tiles_tools/src/game_systems.rs` — `Resource::new` and `Resource::with_regeneration` now clamp `maximum` to `.max(0.0)`, mirroring `maximum_set`'s existing invariant

## Refs: tests/

- `module/helper/tiles_tools/tests/game_systems_test.rs` — new reproducer: constructs a `Resource` with a negative `maximum`, calls `modify`, and asserts both `maximum` and `current` stay non-negative instead of panicking

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 8/8

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Completeness | — | 🟢 | Adversarial pass re-scanned all 12 sections plus header for `_Investigation ongoing._` placeholders and confirmed `filed_by` matches the `$HUMAN_ID` shape already established by other verified bugs in this repo (e.g. BUG-311) — none found thin | — |
| D2 | MRE Validity & Reproducibility | — | 🟢 | Confirming pass executed the crate's full test suite fresh (`cargo nextest run -p tiles_tools --all-features`, detached via longrun): 272/272 passed, including the exact reproducer. Adversarial pass checked whether the MRE's repo-root `cargo test` invocation (no `/tmp/` paths) violates check 203 — confirmed this matches this repo's own established, previously-accepted MRE convention (e.g. BUG-311's Verification Record), not a gap; also checked whether the reproducer's exclusive use of `.modify()` (not `.current_set()`) undersells coverage — both call the byte-identical `.clamp(0.0, self.maximum)`, so the fix closes the shared root cause for both | — |
| D3 | Cross-Reference Integrity | — | 🟢 | 4 Hypothesis rows (H1 marked Root Cause), all H/E rows cross-cited and bidirectional; `grep -n "Fix(BUG-349)" src/game_systems.rs` and `grep -n "BUG-349" tests/game_systems_test.rs` both confirm backreferences matching `## Refs:` | — |
| D4 | Root Cause Quality | — | 🟢 | Direct source read confirms the Root Cause trace exactly: `new`/`with_regeneration` (game_systems.rs:564,583) now clamp via `.max(0.0)` at lines 573/585, mirroring `maximum_set`. Adversarial pass checked for a raw-struct-literal bypass (all 3 `Resource` fields are `pub`) and a NaN-input edge case — struct-literal construction is a pre-existing, explicitly out-of-scope characteristic shared by every `pub`-field type in this crate (not a gap this fix could or should close), and `f32::max(0.0)` on NaN input already returns `0.0` per Rust semantics, so no regression | — |
| D5 | Execution Scope | — | 🟢 | `repo_identity: self`; fix resolves inside `module/helper/tiles_tools/src/game_systems.rs`, same repo | — |
| D6 | Crate Scope Unity | — | 🟢 | `**Component:**` (`module/helper/tiles_tools`) matches the crate `## Fix Location`'s `game_systems.rs` resolves to | — |
| D7 | Crate Locality | — | 🟢 | `tiles_tools` is the leaf crate directly owning `Resource`/`game_systems.rs` — not a pushed-up aggregator | — |
| D8 | Crate Single Responsibility | — | 🟢 | Fix stays within `tiles_tools`'s existing tile-logic-library responsibility; no scope expansion | — |
| **Total** | | — | 🟢 | 0 open | 0/0 |

**Reproduced:** YES — `cargo nextest run -p tiles_tools --all-features` exit 0, 272/272 passed (includes `test_resource_new_with_negative_maximum_does_not_panic_on_modify`), 2026-08-18.
