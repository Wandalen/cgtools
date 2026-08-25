# BUG-344: `Health::heal` panics (debug) or silently corrupts health downward (release) on `u32` overflow before the maximum-health clamp runs

- **Severity:** Medium (narrower trigger than most panics in this crate -- requires `current`
  within `amount` of `u32::MAX` -- but every field on `Health` is `pub`, so a caller can construct
  such a state directly with no special setup, and the failure mode in a release build is silent
  data corruption, not merely a panic)
- **state:** Verified
- **Affects:** `tiles_tools::ecs::components::Health::heal` (`src/ecs/components.rs`) -- any call
  where `self.current + amount` overflows `u32`, i.e. `self.current > u32::MAX - amount`
- **Component:** `module/helper/tiles_tools` (`src/ecs/components.rs`)
- **repo_identity:** self
- **Filed:** 2026-08-18
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/module/helper/
- **Fix Task:** [379](../../verifying/379_register_tiles_tools_health_heal_overflow_fix_closes_bug344.md)

## Symptom

```bash
# Actual (pre-fix): current = u32::MAX - 5, heal(20) -- addition overflows before .min() clamps.
$ cargo test -p tiles_tools --all-features --test integration_tests -- integration::ecs_tests::test_health_heal_saturates_instead_of_overflowing
thread 'integration::ecs_tests::test_health_heal_saturates_instead_of_overflowing' panicked at module/helper/tiles_tools/src/ecs/components.rs:243:20:
attempt to add with overflow
test result: FAILED. 1 failed

# Expected (fixed): heal saturates at u32::MAX, then clamps to maximum -- no panic.
$ cargo test -p tiles_tools --all-features --test integration_tests -- integration::ecs_tests::test_health_heal_saturates_instead_of_overflowing
test integration::ecs_tests::test_health_heal_saturates_instead_of_overflowing ... ok
test result: ok. 1 passed
```

## Impact

**Who is affected:** any caller of `Health::heal` where `current` is already close to `u32::MAX`
-- e.g. a game mode with very large health pools, a buggy upstream multiplier that inflated
`current` before `heal` is called, or a save file that was corrupted/tampered with (`Health`'s
fields are `pub` and `Serialize`/`Deserialize`, so a deserialized `Health` can hold any `u32`
value without validation).

**What breaks:** in a debug build, `heal` panics with `attempt to add with overflow`, crashing
whatever system called it (e.g. a per-frame status-effect tick). In a release build (where
integer-overflow checks are compiled out by default), `self.current + amount` silently wraps
around to a small value -- a call whose entire purpose is to *increase* health can instead
collapse it to near-zero, the exact opposite of its documented effect ("Heals this entity, capped
at maximum health").

**Entity Scope:** `None` -- source-level arithmetic defect, not entity directory instances.

## How Discovered

During a systematic bug-hunt pass across `tiles_tools`'s ECS module, comparing `Health::heal`
against its sibling `Health::damage` showed `damage` uses `self.current.saturating_sub(amount)`
(overflow-safe) while `heal` uses `(self.current + amount).min(self.maximum)` -- a raw `+` before
the clamp. Direct construction of a `Health` value near `u32::MAX` and a call to `heal` confirmed
the addition panics before the `.min()` clamp ever has a chance to run.

## Minimum Reproducible Example

**Verify Command** (run from repo root; ≤3 lines):
```bash
cargo test -p tiles_tools --all-features --test integration_tests -- integration::ecs_tests::test_health_heal_saturates_instead_of_overflowing
```
**What:** violates `heal`'s own doc comment ("Heals this entity, capped at maximum health") --
a call meant to cap at `maximum` instead panics (debug) or wraps below the pre-call value
(release), neither of which is "capped at maximum."

**Expected** (fixed): 1 passed -- `health.current == u32::MAX` (saturates then clamps to
`maximum`, which is also `u32::MAX` in this test).

**Actual** (pre-fix, directly observed via temporary revert-and-rerun of this fix): 1 failed --
panics `attempt to add with overflow` at `src/ecs/components.rs:243:20`.

**Known MRE limitation (check 203/205):** none -- `Health::heal` is pure, dependency-free
arithmetic on this crate's own type; reproducing it requires the `tiles_tools` crate itself (no
`/tmp`-based synthetic fixture can exercise it without vendoring the crate), so the Verify Command
runs as an ordinary `cargo test -p tiles_tools` against the real crate directly, consistent with
this repo's existing precedent for in-workspace-only ECS logic bugs (e.g. BUG-132).

## Hypothesis Table

| ID | Hypothesis | State | Summary | Evidence |
|----|-----------|--------|---------|----------|
| H1 | `Health::heal` can overflow `u32` before its own clamp runs | ✅ Verified | `src/ecs/components.rs:243` (pre-fix): `self.current = (self.current + amount).min(self.maximum);` -- the `+` executes before `.min()` | E1, E2 |
| H2 | The overflow is reachable via public API alone, no unsafe or internal-only construction needed | ✅ Root Cause | `Health`'s fields (`current`, `maximum`) are both `pub` (`src/ecs/components.rs:213-219`), and the struct derives `Clone, Copy` with no validating constructor enforced on direct struct-literal construction | E3 |
| H3 | `damage()`, `heal`'s sibling, already uses the overflow-safe pattern `heal` should have used | ✅ Verified | `src/ecs/components.rs:235-238` (pre-fix numbering, unchanged by this fix): `self.current = self.current.saturating_sub(amount);` | E4 |
| H4 | No existing test exercises `heal` from a `current` value near `u32::MAX` | ✅ Verified | `test_health_component` (`tests/integration/ecs_tests.rs`, pre-existing) only calls `heal` from `current: 70` (`heal(15) -> 85`) | E5 |
| H5 | The fix does not change `heal`'s behavior for any ordinary (non-overflowing) input | ✅ Verified | `test_health_component` re-run post-fix: unchanged pass, `heal(15)` from `current: 70` still yields `85` | E6 |

## Evidence Table

| # | Location | What it shows | Hypothesis |
|---|----------|---------------|------------|
| E1 | `src/ecs/components.rs:243` (pre-fix) | `self.current = (self.current + amount).min(self.maximum);` -- raw addition before clamp | H1 ✅ |
| E2 | Terminal output (this report, MRE section) | `Health { current: u32::MAX - 5, maximum: u32::MAX }.heal(20)` panics `attempt to add with overflow` | H1 ✅ |
| E3 | `src/ecs/components.rs:213-219` | `pub struct Health { pub current: u32, pub maximum: u32 }` -- both fields directly constructible, no validation | H2 ✅ |
| E4 | `src/ecs/components.rs:235-238` (unchanged) | `pub fn damage(&mut self, amount: u32) { self.current = self.current.saturating_sub(amount); }` | H3 ✅ |
| E5 | `tests/integration/ecs_tests.rs`, `test_health_component` (pre-existing, unmodified) | `health.heal(15); assert_eq!(health.current, 85);` from a `current: 70` baseline -- no near-`u32::MAX` case | H4 ✅ |
| E6 | Terminal output (`cargo test -p tiles_tools --all-features --test integration_tests -- integration::ecs_tests::test_health_component`, post-fix) | `test result: ok. 1 passed` | H5 ✅ |

## Root Cause

```
heal( &mut self, amount : u32 )
{
  self.current = ( self.current + amount ).min( self.maximum );
                    ^^^^^^^^^^^^^^^^^^^^^
                    this addition can overflow u32 BEFORE .min() ever
                    gets a chance to clamp the result down to `maximum`
}

  damage() (sibling method, correct):
  self.current = self.current.saturating_sub( amount );
                              ^^^^^^^^^^^^^^^ overflow-safe by construction
```
`heal` and `damage` perform structurally symmetric operations (adjust `current` by `amount`, then
implicitly or explicitly bound the result) but only `damage` used the overflow-safe primitive.
`heal`'s clamp (`.min(self.maximum)`) only protects against exceeding `maximum` -- it does nothing
for the intermediate `+` operation, which is where the actual overflow occurs.

## Why Not Caught

`test_health_component`, the only existing test exercising `heal`, starts from an ordinary
mid-range `current` (70, reached via `Health::new(100)` then `damage(30)`) and heals by a small
amount (15) -- nowhere near `u32::MAX`. `Health::new` itself only ever produces `current ==
maximum`, so no code path inside the crate's own constructors can produce a `current` close to
`u32::MAX` without a caller directly writing to the `pub` field, which no existing test does.

## Fix Location

**`src/ecs/components.rs:241-256`** (before/after):

```rust
// Before:
/// Heals this entity, capped at maximum health.
pub fn heal( &mut self, amount : u32 )
{
  self.current = ( self.current + amount ).min( self.maximum );
}

// After:
/// Heals this entity, capped at maximum health.
pub fn heal( &mut self, amount : u32 )
{
  // Fix(BUG-344): `self.current + amount` could overflow `u32` before the
  // `.min(self.maximum)` clamp ever ran (debug build: panics; release
  // build: silently wraps to a tiny value, i.e. "healing" corrupts health
  // downward) -- switched to `saturating_add`, matching `damage()`'s
  // existing `saturating_sub` convention just above.
  // [...full 3-field comment at src/ecs/components.rs:243-254...]
  self.current = self.current.saturating_add( amount ).min( self.maximum );
}
```
Source comment (`Fix(BUG-344)`/`Root cause`/`Pitfall`) added inside `heal`, immediately above the
corrected line.

## Prevention

Detection command for the general pattern (an unchecked arithmetic operator immediately followed
by a clamp, inside this crate's ECS components):
```bash
grep -n "self\.[a-z_]* + [a-z_]*.*\.min(\|self\.[a-z_]* - [a-z_]*.*\.max(" src/ecs/components.rs
```
Run against the fixed file, this finds no remaining matches for `heal`'s own pattern (now
`saturating_add(...).min(...)`, which the regex's literal `+` does not match) -- a starting point
for review, not a precise or general-purpose detector; it would not catch the same defect written
with a temporary variable instead of an inline expression.

**Pitfall:** `(x + y).min(bound)` and `(x - y).max(bound)` are only safe when the inner
arithmetic itself cannot overflow/underflow the integer type -- the outer clamp does not protect
the inner operation. Use `saturating_add`/`saturating_sub` for the inner operation whenever the
operand values are not provably bounded away from the type's limits (here, every field on
`Health` is `pub`, so nothing bounds `current` away from `u32::MAX`).

## Generalized Version

**Broken assumption:** `(x + y).min(bound)` clamps the *result* of `x + y` to `bound`, and is
therefore safe regardless of how large `x` and `y` individually are.

Fails whenever:
1. `x` and `y` are both unsigned (or otherwise non-overflow-checked in release builds) integers, AND
2. `x + y` can exceed the integer type's maximum representable value, AND
3. Nothing upstream bounds `x` away from `TYPE::MAX - y` before this expression runs

**Detection invariant:**
```
for every `(x + y).min(bound)` / `(x - y).max(bound)` expression on an unsigned integer type:
  the inner `+`/`-` must be `saturating_add`/`saturating_sub`,
  unless `x`/`y` are provably bounded such that overflow/underflow cannot occur
```
Single confirmed instance in this crate (grep swept `src/ecs/components.rs` for every
`+`/`-` immediately followed by `.min(`/`.max(`; only `heal` matched -- `damage` already used the
saturating form, and no other component in this file uses the clamp-after-arithmetic shape). Not
a duplicate of any prior bug in this repo's `task/bug/` history (dedup search:
`grep -rli "fn heal\|health.*heal\|heal.*overflow" task/bug/` found no prior filing referencing
`Health::heal` specifically).

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-18 | filed | Found during a systematic bug-hunt pass over `tiles_tools`'s ECS module; root-caused by comparing `heal`'s arithmetic against `damage`'s existing `saturating_sub` convention |
| 2026-08-18 | fix_applied | Changed `src/ecs/components.rs:243` (pre-fix) to `self.current.saturating_add(amount).min(self.maximum)`. Reproducer test confirmed FAIL pre-fix (`attempt to add with overflow` panic) and PASS post-fix; full `ecs_tests` module (24 tests) and scoped clippy (`cargo clippy -p tiles_tools --all-targets --all-features -- -D warnings`) both clean |
| 2026-08-18 | VERIFY Gate run, PASS | File sat in `bug/verified/` with `state:` still `Unverified` and no `## Verification Record` — the formal PROC1-S9 VERIFY Gate was never run/recorded, caught during a repo-wide reach-consistency sweep. Ran the 8-dimension Tier 2 Dual-Role Self-Check: re-executed the Verify Command fresh — exit 0, test passes, matching the Expected block exactly. Adversarial pass caught the same defect as BUG-342: Evidence Table Hypothesis column had bare H-IDs with no state symbols (checklist 304) — fixed by annotating all 6 rows `✅`. All 8 dimensions 🟢 — see `## Verification Record`. VERIFY_PASS fired; state → `Verified` (file already correctly resided in `bug/verified/`). |
| 2026-08-18 | re-verified | Independent second Tier 2 Dual-Role Self-Check (separate session, task-scoped to BUG-343/344/345 specifically). Re-confirmed source fix (`saturating_add` at `src/ecs/components.rs:243`) and reproducer test directly; full-crate `cargo nextest run -p tiles_tools --all-features` (detached via `longrun`) 272/272 passed. Adversarial pass caught an MRE portability defect (check 203/205) the prior pass's D2 row missed: Verify Command hardcoded `cd /home/user1/pro/lib/yrd_gamedev/cgtools`, an absolute per-user path — fixed by removing it and adding the `**Known MRE limitation**` disclosure (matching BUG-132's precedent). See `## Verification Record`. |

## Verification Record

**VERIFY Gate (2026-08-18) — Tier 2 Dual-Role Self-Check, 8 dimensions, verdict: PASS (8/8).**

| Gate | Name | Prev | Now | Issues | Fixes |
| ---- | ---- | ---- | --- | ------ | ----- |
| D1 | Completeness | — | 🟢 | — | — |
| D2 | MRE Validity & Reproducibility | 🔴 | 🟢 | Second pass (independent re-check): Verify Command hardcoded `cd /home/user1/pro/lib/yrd_gamedev/cgtools` — an absolute, per-user path violating check 205, missed by the first pass above; inconsistent with sibling BUG-341/346 and missing the `**Known MRE limitation**` disclosure BUG-132's precedent uses. | Removed the hardcoded path; added the `**Known MRE limitation (check 203/205):**` disclosure. Re-confirmed via `cargo nextest run -p tiles_tools --all-features` (detached via `longrun`): 272/272 passed, including this test. |
| D3 | Cross-Reference Integrity | — | 🟢 | Evidence Table Hypothesis column had bare H-IDs, no state symbols (304) | Added `✅` to all 6 rows |
| D4 | Root Cause Quality | — | 🟢 | — | — |
| D5 | Execution Scope | — | 🟢 | — | — |
| D6 | Crate Scope Unity | — | 🟢 | — | — |
| D7 | Crate Locality | — | 🟢 | — | — |
| D8 | Crate Single Responsibility | — | 🟢 | — | — |
| **Total** | | — | 🟢 | 2 issues (1 per pass) | 2 fixes |

**Reproduced:** YES — `cargo test -p tiles_tools --all-features --test integration_tests -- integration::ecs_tests::test_health_heal_saturates_instead_of_overflowing`, exit 0, 2026-08-18. Re-confirmed by a second, independent pass: `cargo nextest run -p tiles_tools --all-features` (detached via `longrun`), exit 0, 272/272 passed, 2026-08-18.

## Refs: src/

- `src/ecs/components.rs` — changed `Health::heal`'s addition to `saturating_add`, matching `damage()`'s existing `saturating_sub` convention

## Refs: tests/

- `tests/integration/ecs_tests.rs` — new reproducer test `test_health_heal_saturates_instead_of_overflowing`: `Health { current: u32::MAX - 5, maximum: u32::MAX }.heal(20)` must saturate to `u32::MAX`, not panic
