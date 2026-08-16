# BUG-148: `Vec<E>::interpolate` silently truncates to the shorter length instead of panicking on mismatch

- **Severity:** Medium (silently wrong, shorter-than-expected result; not a crash on the buggy
  path itself, but violates `Animatable::interpolate`'s own boundary contract and this crate's
  own established sibling convention)
- **state:** Completed
- **Affects:** Any `Vec<E>::interpolate( &other, time )` call where `self.len() != other.len()`
- **Component:** `module/helper/animation` (`src/interpolation.rs`)
- **repo_identity:** self
- **Filed:** 2026-08-16
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/verified/
- **verification_date:** 2026-08-16
- **Related Bugs:** None — independent of every other bug filed this session; same defect
  *shape* as the already-fixed `CubicHermite::new`/`apply` length checks in
  `easing/cubic/hermite.rs` (tracked historically as TASK-041, predating this session's BUG-NNN
  numbering), but an entirely separate `Animatable` impl this session discovered was never
  brought into line with that established convention.

## Symptom

```rust
use animation::Animatable;

let start = vec![ 0.0_f32, 1.0_f32, 2.0_f32 ];   // 3 elements
let end = vec![ 10.0_f32, 20.0_f32 ];             // 2 elements

let result = start.interpolate( &end, 0.5 );
// Wrong (pre-fix):  vec![ 5.0, 10.5 ]   -- length 2, `start`'s 3rd element (2.0) silently dropped
// Correct (post-fix): panics with "Vec::interpolate: self and other must have the same length
//                      ( got 3 and 2 )" -- matching CubicHermite's established convention
```

## Impact

**Who is affected:** Any caller of `Vec<E>::interpolate` where the two operand lengths can
legitimately differ at runtime — e.g. data-driven or externally-sourced keyframe arrays whose
lengths aren't statically guaranteed equal by the type system (unlike `mingl::Vector<E, N>`,
whose fixed-size `N` makes a length mismatch impossible by construction).

**What breaks:** `self.iter().zip( other.iter() )` iterates only `min( self.len(), other.len() )`
elements — the standard, well-known truncating behavior of `Iterator::zip`. This silently
violates `Animatable::interpolate`'s own boundary contract, established by every concrete scalar
impl (`f32`/`f64`/`i32`/tuples, all computing `self + ( other - self ) * time`): `interpolate(
other, 0.0 )` must equal `self`, and `interpolate( other, 1.0 )` must equal `other`. For a
`Vec<E>` length mismatch, this fails in either direction — if `self` is longer, its trailing
elements are dropped even at `time == 0.0` (where the result should be exactly `self`); if
`other` is longer, its trailing elements are dropped even at `time == 1.0` (where the result
should be exactly `other`). The returned `Vec` is silently the wrong length either way.

**Established sibling convention violated:** `easing/cubic/hermite.rs`'s `CubicHermite::new` and
`CubicHermite::apply` already guard against this exact defect shape (two runtime-length `Vec`s
that must correspond element-wise) via `assert_eq!` — see their own source comments: "a loud
panic on malformed caller input is the correct fix at this call site," and the existing tests
`test_cubic_hermite_new_panics_on_mismatched_tangent_lengths` /
`test_cubic_hermite_apply_panics_on_mismatched_value_lengths` in `easing_test.rs`. `Vec<E>`'s own
`Animatable::interpolate` impl — structurally the same "two runtime-length Vecs, one operation"
shape — was never brought into line with this already-established, already-tested convention in
the same crate.

**Magnitude:** Silent wrong-length result, not a crash. Any downstream code assuming the returned
`Vec`'s length matches `self`'s (or `other`'s) length gets a shorter collection with no signal
that truncation occurred.

**Entity Scope:** None — a code-level defect, not an operational-entity concern.

## How Discovered

Deferred investigation task from this session's `animation` crate review (tracked as "Investigate
Vec<E>::interpolate silent truncation via .zip()"). Confirmed by direct read of
`Vec<E>::interpolate`'s pre-fix body, cross-referenced against `CubicHermite::new`/`apply`'s
already-existing `assert_eq!` guards and tests in the same crate — the same defect shape, already
fixed once, left unfixed in a sibling `Animatable` impl.

## Minimum Reproducible Example

```bash
cd module/helper/animation && cargo test --test interpolation_test test_vec_interpolate_panics_on_mismatched_lengths 2>&1 | tail -10
```

**Expected** (post-fix):
```
test tests::test_vec_interpolate_panics_on_mismatched_lengths ... ok
```

**Actual** (pre-fix — confirmed by temporarily reverting the `assert_eq!` guard back to the bare
`.zip()`, then restoring the fix immediately after capturing the failure):
```
note: test did not panic as expected at module/helper/animation/tests/interpolation_test.rs:71:6
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 16 filtered out; finished in 0.00s
```

**Verify Command** (≤3 lines, standalone):
```bash
cd module/helper/animation && cargo test --test interpolation_test test_vec_interpolate_panics_on_mismatched_lengths
# 1 passed = fixed; 1 failed ("test did not panic as expected") = bug present
```

**Known MRE limitation (check 205):** none — pure, synchronous, dependency-free state; the
regression test runs as an ordinary native `cargo test` against the real crate directly.

## Hypothesis Table

| ID | Hypothesis | State | Summary | Evidence |
|----|-----------|--------|---------|----------|
| H1 | `Vec<E>::interpolate`'s `self.iter().zip( other.iter() )` silently truncates to the shorter length on a mismatch, instead of surfacing it, unlike the crate's own established `CubicHermite` convention for the same defect shape. | ✅ Root Cause | Direct read of pre-fix `Vec<E>::interpolate` shows no length check at all; direct read of `easing/cubic/hermite.rs` shows `CubicHermite::new`/`apply` both already `assert_eq!` on exactly this kind of mismatch. | E1 |
| H2 | `mingl::Vector<E, N>::interpolate` (the fixed-size sibling impl in the same file) has the identical latent defect, since it also uses `.zip()`. | ❌ Falsified | `mingl::Vector<E, N>` is a compile-time-sized array wrapper — both `self` and `other` are always exactly `N` elements by the type system itself; no runtime length mismatch is possible, so `.zip()` there is safe by construction, unlike `Vec<E>`'s runtime-arbitrary length. | E2 |
| H3 | The correct fix is to define specific mismatch-handling semantics (e.g. pass through the longer side's extra elements unchanged) rather than panicking. | ❌ Falsified | `Animatable::interpolate` returns `Self` directly with no `Result` in the trait signature (same constraint noted in `CubicHermite`'s own fix comments), so a malformed-input precondition can only be surfaced as a loud panic at this call site — matching the exact reasoning already applied to `CubicHermite`. | E3 |

## Evidence Table

| # | Location | What it shows | Hypothesis |
|---|----------|---------------|------------|
| E1 | `src/interpolation.rs` (pre-fix `Vec<E>::interpolate`) vs. `src/easing/cubic/hermite.rs` (`CubicHermite::new`/`apply`) | Only `Vec<E>::interpolate` lacks the length guard the crate's own sibling convention already establishes. | H1 ✅ |
| E2 | `src/interpolation.rs`, `impl< E, const N : usize > Animatable for mingl::Vector< E, N >` | `N` is a compile-time const generic shared by both operands' types — no runtime mismatch is representable. | H2 ❌ |
| E3 | `Animatable` trait definition (`traits.rs`): `fn interpolate( &self, other : &Self, time : f64 ) -> Self;` — no `Result` | Return type has no room for a recoverable error; a loud panic is the only option consistent with the trait's own signature, matching `CubicHermite`'s prior fix. | H3 ❌ |

## Root Cause

```
Vec<E>::interpolate()   (pre-fix)
  self.iter().zip( other.iter() )   // <-- truncates to min(self.len(), other.len()) on mismatch
  .map( |(elem, other_elem)| elem.interpolate( other_elem, time ) )
  .collect()

CubicHermite::new() / apply()   (pre-existing, correct, same crate)
  assert_eq!( m1.len(), m2.len(), "CubicHermite::new: m1 and m2 must have the same length ( got {} and {} )", ... );
  assert_eq!( start.len(), end.len(), "CubicHermite::apply: start and end must have the same length ( got {} and {} )", ... );
```

`Vec<E>::interpolate` is structurally the same "two runtime-length `Vec`s combined element-wise"
operation `CubicHermite` already guards, but was never checked against that established
convention when written.

## Why Not Caught

No existing test exercised `Vec<E>::interpolate` at all — not even the equal-length happy path,
let alone a length mismatch. The only `Animatable` trait tests prior to this fix covered `f32`
and `i32` directly.

## Fix Location

`module/helper/animation/src/interpolation.rs`, `impl< E > Animatable for Vec< E >`:

```rust
// before
fn interpolate( &self, other : &Self, time : f64 ) -> Self
{
  self.iter().zip( other.iter() )
  .map( |( elem, other_elem )| elem.interpolate( other_elem, time ) )
  .collect::< Vec< _ > >()
}

// after
fn interpolate( &self, other : &Self, time : f64 ) -> Self
{
  assert_eq!
  (
    self.len(), other.len(),
    "Vec::interpolate: self and other must have the same length ( got {} and {} )", self.len(), other.len()
  );

  self.iter().zip( other.iter() )
  .map( |( elem, other_elem )| elem.interpolate( other_elem, time ) )
  .collect::< Vec< _ > >()
}
```

Added an `assert_eq!` guard before the existing `.zip()`, matching `CubicHermite`'s established
message format exactly (`"<Type>::<method>: <a> and <b> must have the same length ( got {} and
{} )"`). No signature change.

## Prevention

Added two tests to `tests/interpolation_test.rs`: `test_vec_interpolation` (baseline equal-length
coverage — none existed before) and `test_vec_interpolate_panics_on_mismatched_lengths`
(`bug_reproducer(BUG-148)`), which constructs two `Vec<f32>` of differing length and asserts
`interpolate` panics naming both lengths, rather than silently returning a shorter-than-expected
result.

**Pitfall:** invisible whenever every caller happens to pass equal-length Vecs — the exact same
"invisible unless lengths genuinely differ" pitfall already documented for `CubicHermite`'s own
fix, restated here because `Vec<E>::interpolate` fell into it independently despite the sibling
precedent already existing in the same crate.

## Generalized Version

**Broken assumption:** "an established convention for one type implementing a shared trait
(`CubicHermite`'s length-checking discipline) automatically applies to every other implementor of
that trait." False — each `impl` is a separate piece of code; a convention established in one
sibling implementation does not propagate to another unless explicitly checked against.

**Confirmed general rule:** when a crate already has an established, tested convention for
handling a specific defect shape (here: "two runtime-length collections combined element-wise
must have matching lengths, enforced via `assert_eq!` with a `Type::method: a and b must have the
same length ( got {} and {} )` message"), every other implementation with the same shape must be
audited against that convention explicitly — grep sibling implementations of the same trait (or
the same general operation) for the pattern before assuming a new/divergent implementation is
exempt, matching BUG-145's own Generalized Version lesson for a different trait in this same
crate.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-16 | filed | Deferred investigation task from this session's `animation` crate review; confirmed by direct comparison against `CubicHermite::new`/`apply`'s already-established, already-tested length-check convention in the same crate. |
| 2026-08-16 | fixed | Added an `assert_eq!` guard on `self.len() == other.len()`, matching `CubicHermite`'s message format exactly. |
| 2026-08-16 | verified | Added `test_vec_interpolation` (baseline) and `test_vec_interpolate_panics_on_mismatched_lengths`; confirmed the latter fails pre-fix with "test did not panic as expected" and passes against the fix; full crate suite (17 tests in `interpolation_test.rs`, 41 total incl. doctests across the crate) + `cargo clippy --all-targets -- -D warnings` clean. |
| 2026-08-16 | completed | Acceptance verification by a distinct session, same batch as BUG-138 (see its completed-row note for the shared 40/40 `animation` run and MAAV batch scope). Independently re-read `impl< E > Animatable for Vec< E >::interpolate` (confirmed the `assert_eq!` length guard genuinely present, matching `CubicHermite`'s message format, `Fix(BUG-148)` comment intact) and `test_vec_interpolate_panics_on_mismatched_lengths` (non-tautological: `#[should_panic(expected = "self and other must have the same length")]` on a genuine 3-vs-2-length mismatch). State → Completed. |

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 8/8

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Completeness | 🟢 | 🟢 | All 12 FI008 sections present. | — |
| D2 | MRE Validity & Reproducibility | 🟢 | 🟢 | Confirming pass traced the `.zip()` truncation directly from source; adversarial pass specifically checked whether the fixed-size `mingl::Vector<E,N>` sibling shares the same latent defect (H2, falsified — compile-time `N` prevents mismatch) and whether a non-panic fix (pass-through semantics) was viable given the trait's `Result`-less signature (H3, falsified) before accepting the panic-based fix as correct and matching precedent. | — |
| D3 | Cross-Reference Integrity | 🟢 | 🟢 | Independent of every other bug filed this session; explicitly cross-referenced against `CubicHermite`'s pre-existing, already-tested fix for the identical defect shape in the same crate. | — |
| D4 | Root Cause Quality | 🟢 | 🟢 | Root cause is a precedent-violation (established sibling convention not applied to a divergent implementor), stated and evidenced directly rather than merely asserted. | — |
| D5 | Execution Scope | 🟢 | 🟢 | Grepped every `Vec<E>::interpolate`/`.interpolate(` call site involving a `Vec` in the workspace (crate-local tests only, both newly added) — no production call site currently passes mismatched-length Vecs, consistent with zero prior test coverage. | — |
| D6 | Crate Scope Unity | 🟢 | 🟢 | Only `animation` src+test+bug file touched. | — |
| D7 | Crate Locality | 🟢 | 🟢 | Fix local to one trait impl method. | — |
| D8 | Crate Single Responsibility | 🟢 | 🟢 | No new public surface; existing `Animatable` contract now enforced consistently with the crate's own established sibling convention. | — |

**Reproduced:** YES — temporarily reverting the fixed `interpolate()` back to the bare `.zip()`
with no length guard (marked `// TEMPORARY BUG-148 REVERT FOR MRE VERIFICATION`) and running
`cargo test --test interpolation_test test_vec_interpolate_panics_on_mismatched_lengths` produced
the exact predicted `test did not panic as expected` failure; restoring the fix returned the full
suite (17 tests in `interpolation_test.rs` incl. both new tests, 41 total across the crate incl.
doctests) to passing plus a clean `cargo clippy --all-targets -- -D warnings`, 2026-08-16.

## Refs: src/

| File | Change |
|------|--------|
| `module/helper/animation/src/interpolation.rs` | `impl< E > Animatable for Vec< E >`: added an `assert_eq!` length guard before the existing `.zip()`. `Fix(BUG-148)`/`Root cause`/`Pitfall` comment added. |

## Refs: tests/

| File | Change |
|------|--------|
| `module/helper/animation/tests/interpolation_test.rs` | Two new tests: `test_vec_interpolation` (baseline equal-length coverage) and `test_vec_interpolate_panics_on_mismatched_lengths` (`bug_reproducer(BUG-148)`, 5-section doc comment). |
