# BUG-124: `vector::normalize(r, a)` writes `r`'s own values scaled by `a`'s magnitude, never reading `a`'s direction

- **Severity:** Medium (silently wrong result whenever `r != a` at call time — correct only by
  the sole in-crate caller's own pre-seeding convention, not by anything the signature enforces)
- **state:** Completed
- **Affects:** Any caller of `vector::normalize(r: &mut R, a: &A)` where `r`'s pre-call contents
  differ from `a`'s
- **Component:** `module/math/mdmath_core` (`src/vector/arithmetics.rs::normalize`)
- **repo_identity:** self
- **Filed:** 2026-08-15
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/verified/
- **verification_date:** 2026-08-16
- **Related Bugs:** None — independent root cause from BUG-122/123, filed under the same task
  #58 targeted `mdmath_core` review

## Symptom

```bash
# r = [1.0, 0.0, 0.0] (already unit length, unrelated to a), a = [3.0, 4.0, 0.0] (|a| = 5)

# Wrong (pre-fix) -- divides r's OWN elements by a's magnitude, never reads a's direction:
vector::normalize(&mut r, &a);
r == [0.2, 0.0, 0.0]     # = r / |a|, NOT a normalized -- r's own (already-unit) direction
                          #   just got rescaled by an unrelated vector's magnitude

# Correct (post-fix) -- writes a's own direction, normalized, into r:
r == [0.6, 0.8, 0.0]     # = a / |a|
```

## Impact

**Who is affected:** Any caller of `vector::normalize(r, a)` where `r`'s contents at call time
are not already equal to `a`'s. The sole existing in-crate caller, `normalized(a)`, happens to
call `normalize(&mut r, a)` with `r = a.clone()` immediately beforehand — under that specific
calling convention the bug is a byte-identical no-op, which is exactly why it was never observed.

**What breaks:** The function's own signature — `fn normalize<E,R,A,SIZE>(r: &mut R, a: &A)`,
two independent, unconstrained generic type parameters `R`/`A` with no `R = A` or "caller must
pre-seed r = a" precondition documented anywhere — implies `r` receives `a`'s direction
normalized to unit length. Pre-fix, it instead silently normalizes whatever `r` already
contained, scaled by `a`'s magnitude — a different, generally wrong vector whenever `r != a`.

**Magnitude:** Zero current callers affected — grepped all 3 workspace-wide call sites of
`vector::normalize`: the crate's own `normalized()` wrapper (pre-seeds `r = a.clone()`), and two
test call sites (`tests/inc/arithmetics.rs`), both of which also pre-seed `r` equal to `a` before
calling. Any future direct caller passing an `r` unrelated to `a` — the case the two-argument
signature itself invites — would hit this immediately and silently.

**Entity Scope:** None — a code-level math defect, not an operational-entity concern.

## How Discovered

Task #58, a targeted code review of `mdmath_core` dispatched under the standing bug-hunt
mandate. The reviewing agent flagged that `normalize`'s write loop — `for elem in
r.vector_iter_mut() { *elem /= mag; }` (pre-fix) — only ever dereferenced `r`, never called
`a.vector_iter()` beyond the single aggregate `mag(a)` computation, despite the function existing
specifically to write `a`'s normalized direction into `r`. The same-file sibling `project_on(r,
b)` was used as the oracle for what a correctly-written "derive from the other argument" loop
looks like — it reads `b.vector_iter()` inside its own loop (`*elem = *biter.next().unwrap() *
scalar`). Independently confirmed by direct re-read of the pre-fix source before filing:

```bash
$ grep -n "pub fn normalize<\|pub fn project_on<" -A 12 \
    module/math/mdmath_core/src/vector/arithmetics.rs   # (pre-fix)
# normalize:    let mag = mag(a); for elem in r.vector_iter_mut() { *elem /= mag; }
#               -- a's iterator never constructed; only a's aggregate magnitude is read
# project_on:   let mut biter = b.vector_iter(); for elem in r.vector_iter_mut() {
#                 *elem = *biter.next().unwrap() * scalar; }
#               -- correctly reads b's own elements inside the write loop
```

## Minimum Reproducible Example

```bash
rm -rf /tmp/mre124 && mkdir -p /tmp/mre124/src
cat > /tmp/mre124/Cargo.toml <<'EOF'
[package]
name = "mre124"
version = "0.1.0"
edition = "2021"

[dependencies]
mdmath_core = { path = "/home/user1/pro/lib/yrd_gamedev/cgtools/module/math/mdmath_core", features = [ "enabled" ] }
EOF
cat > /tmp/mre124/src/main.rs <<'EOF'
use mdmath_core::vector;

fn main()
{
  // r starts as an unrelated, already-unit-length vector distinct from a.
  let mut r = [ 1.0_f64, 0.0, 0.0 ];
  let a = [ 3.0_f64, 4.0, 0.0 ];
  vector::normalize( &mut r, &a );
  println!( "{r:?}" );
}
EOF
cd /tmp/mre124 && cargo run 2>&1 | tail -1
```

**Expected** (post-fix — `r` receives `a`'s own direction, normalized):
```
[0.6, 0.8, 0.0]
```

**Actual** (pre-fix — `r`'s own pre-existing contents divided by `a`'s magnitude):
```
[0.2, 0.0, 0.0]
```

**Verify Command** (≤3 lines, standalone):
```bash
cd /tmp/mre124 && cargo run 2>&1 | tail -1
# [0.6, 0.8, 0.0] = fixed; [0.2, 0.0, 0.0] = bug present
```
**What:** Violates the function's own name/signature contract — "normalize" over two independent
`R`/`A` parameters implies writing `a`'s normalized direction into `r`, not rescaling whatever
`r` already held.

**Known MRE limitation (check 205):** `mdmath_core` is this workspace's own crate; the MRE
path-depends on it locally rather than a registry version, mirroring BUG-116/118-123's own
documented exception. `3.0`/`4.0`/`5.0`/`0.6`/`0.8` are exactly representable in `f64` with no
floating-point ambiguity this local dependency could be hiding.

## Hypothesis Table

| ID | Hypothesis | State | Summary | Evidence |
|----|-----------|--------|---------|----------|
| H1 | `normalize`'s write loop reads and writes only `r`, never `a`'s own elements beyond the single aggregate `mag(a)` call — correct only by coincidence when the caller pre-seeds `r = a`. | ✅ Root Cause | Direct read of `arithmetics.rs` (pre-fix) confirms the loop body is `*elem /= mag` with `elem` drawn from `r.vector_iter_mut()` and no `a.vector_iter()` constructed anywhere in the function. MRE with `r != a` confirms the wrong-vector symptom. | E1, E2 |
| H2 | `mag(a)` itself is computed incorrectly, independent of the loop's read-source issue. | ❌ Falsified | `mag(a)` correctly computes `5.0` for `a=[3,4,0]` in both pre-fix and post-fix runs (confirmed via the MRE's intermediate value and by `test_magnitude`'s own passing pre-existing coverage) — the aggregate magnitude was always right; only the per-element numerator was wrong. | E2 |
| H3 | The two-argument `normalize(r, a)` signature is intentionally documented as requiring `r == a` on entry (an in-place-only normalize with a redundant second parameter for API-symmetry reasons), making this a documentation gap rather than a logic bug. | ❌ Falsified | No doc comment, parameter name, or type bound anywhere on `normalize` states or enforces `r == a` — `R` and `A` are fully independent, unconstrained generic parameters (`R: VectorIterMut<E,SIZE>`, `A: VectorIter<E,SIZE>`, no shared-type bound), and the sibling `project_on(r, b)` in the same file uses the identical two-argument shape specifically to support `r != b`. | E1 |

## Evidence Table

| # | Location | What it shows | Hypothesis |
|---|----------|---------------|------------|
| E1 | `module/math/mdmath_core/src/vector/arithmetics.rs` (pre-fix `normalize`, and the correctly-written sibling `project_on` a few lines below) | `normalize`: `let mag = mag(a); for elem in r.vector_iter_mut() { *elem /= mag; }` — no `a.vector_iter()` call. `project_on`: `let mut biter = b.vector_iter(); for elem in r.vector_iter_mut() { *elem = *biter.next().unwrap() * scalar; }` — correctly reads `b`'s own elements. | H1 ✅, H3 ❌ |
| E2 | `/tmp/mre124` run, pre-fix vs. post-fix, `r=[1,0,0]`, `a=[3,4,0]` | Pre-fix: `[0.2, 0.0, 0.0]` = `r / |a|` (r's own values, `a`'s magnitude). Post-fix: `[0.6, 0.8, 0.0]` = `a / |a|` (a's own direction, normalized). `mag(a)=5.0` correct in both runs — confirms the aggregate magnitude computation was never the defect. | H1 ✅, H2 ❌ |

## Root Cause

```
normalize< E, R, A, SIZE >( r: &mut R, a: &A )
  let mag = mag( a )                          <- correct: a's aggregate magnitude
  for elem in r.vector_iter_mut()             <- iterates r's OWN elements only
  { *elem /= mag }                            <- writes r[i] / mag, never reads a[i]   ✗
                                                  correct result requires a[i] / mag
```

The function's write loop dereferences and mutates `r`'s own elements in place, using only the
scalar `mag(a)` derived from `a` — it never constructs `a.vector_iter()` to read `a`'s
per-element values at all. This produces `r / |a|` instead of the intended `a / |a|` written
into `r`. The defect is invisible under the sole in-crate calling convention
(`normalized(a)` pre-seeds `r = a.clone()` before calling), where `r[i]` and `a[i]` are always
numerically identical, making `r[i]/mag` and `a[i]/mag` indistinguishable.

## Why Not Caught

Both existing `test_normalize` cases (`tests/inc/arithmetics.rs`) pre-set `r` equal to `a`
before calling `normalize` (`let mut result = vec_a; vector::normalize(&mut result, &vec_a);`),
exactly mirroring the sole in-crate production caller's own convention — so `r[i]` and `a[i]`
were always numerically identical at every existing call site, making the missing read from
`a`'s iterator completely unobservable. No existing test or caller ever exercises `r != a`.

## Fix Location

`module/math/mdmath_core/src/vector/arithmetics.rs`, `pub fn normalize`. One change:

```rust
// before
pub fn normalize< E, R, A, const SIZE : usize >( r : &mut R, a : &A )
where
  R : VectorIterMut< E, SIZE >,
  A : VectorIter< E, SIZE >,
  E : NdFloat,
{
  let mag = mag( a );
  for elem in r.vector_iter_mut()
  {
    *elem /= mag;
  }
}

// after
pub fn normalize< E, R, A, const SIZE : usize >( r : &mut R, a : &A )
where
  R : VectorIterMut< E, SIZE >,
  A : VectorIter< E, SIZE >,
  E : NdFloat,
{
  let mag = mag( a );
  let mut aiter = a.vector_iter();
  for elem in r.vector_iter_mut()
  {
    *elem = *aiter.next().unwrap() / mag;
  }
}
```

For the sole existing in-crate caller (`normalized`, which pre-seeds `r = a.clone()`), this
change is an exact no-op in observable behavior — confirmed by the pre-existing `test_normalize`/
`test_normalized` tests still passing unchanged, since `r[i] == a[i]` at every existing call
site makes `r[i]/mag` and `a[i]/mag` numerically identical.

## Prevention

Added `test_normalize_with_distinct_source_and_destination` to `tests/inc/arithmetics.rs`: seeds
`r = [1.0, 0.0, 0.0]` (an unrelated, already-unit-length vector) and `a = [3.0, 4.0, 0.0]`
(`|a| = 5`), asserting `normalize(&mut r, &a)` produces `[0.6, 0.8, 0.0]` (`a`'s own direction) —
this fails under the pre-fix loop, which would instead produce `[0.2, 0.0, 0.0]` (`r`'s own
value rescaled by `a`'s magnitude).

**Pitfall:** a `fn(r: &mut R, a: &A)`-shaped API that computes a scalar from `a` but only reads/
writes through `r` in its loop body is only correct under a "caller pre-seeds `r = a`" calling
convention that the type signature itself never states or enforces — always check what a write
loop actually dereferences on its right-hand side, not just what it assigns into on its left; a
same-crate sibling with the same "write into r, derived from the other argument" shape (here,
`project_on`) is often the cheapest oracle for what the loop body should have been doing.

## Generalized Version

**Broken assumption:** "A function correctly produces its documented output because its only
existing caller always passes inputs under which the buggy and correct implementations coincide"
— false; the function's own type signature is the contract, not its current caller's specific
usage pattern. An unconstrained two-parameter signature (`R`, `A` independent, no shared-type
bound) invites exactly the general (`r != a`) case the bug silently mishandles.

**Confirmed general rule:** for any `fn(dest: &mut D, src: &S)`-shaped function where `dest` and
`src` are independent, unconstrained type parameters, verify the write loop's right-hand side
actually dereferences `src`'s own iterator/elements, not merely a scalar aggregate derived from
`src` combined with `dest`'s pre-existing contents — the two are indistinguishable only under a
"caller happens to pre-seed dest = src" convention that the signature itself does not enforce or
document.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-15 | filed | Discovered via task #58's targeted code review of `mdmath_core`; confirmed by direct comparison against the same-file `project_on` sibling's correct read-from-other-argument pattern before filing. |
| 2026-08-15 | fixed | Added `let mut aiter = a.vector_iter();` before the loop and changed the write to `*elem = *aiter.next().unwrap() / mag;`. 3-field `Fix(BUG-124)`/`Root cause`/`Pitfall` comment added at the fix site. |
| 2026-08-15 | verified | Added `test_normalize_with_distinct_source_and_destination` (`r != a` case) to `tests/inc/arithmetics.rs`; scoped test run (`verb test_only -p mdmath_core` via `longrun`) passed with the new test green alongside the pre-existing suite; confirmed all 3 existing workspace call sites of `vector::normalize` pre-seed `r == a` and are therefore byte-identical no-ops under this fix. |
| 2026-08-16 | completed | Acceptance verification by a distinct session (filer/fixer 2026-08-15, this verifier 2026-08-16; fix not present anywhere in this session's own context — approached as a fresh reader). Independently re-confirmed via targeted re-read of `arithmetics.rs`'s `normalize` (the `aiter`/`*aiter.next().unwrap()` read genuinely present, matching `Refs: src/`) and the full `bug_reproducer(BUG-124)` test body (non-tautological — asserts the `r≠a` case yields `a`'s own direction). Checked sibling `normalized()` for independent scope-escape: it delegates to the fixed `normalize()` via `r = a.clone()`, no separate implementation. Re-ran `verb/test_only pkg::mdmath_core` via `longrun`: 94/94 passed. `cargo clippy -p mdmath_core --all-features --all-targets -- -D warnings`: clean. MAAV Tier 2 Dual-Role Self-Check (`governance/maav.rulebook.md`), covering BUG-122/123/124 together. State → Completed. |

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 8/8

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Completeness | 🟢 | 🟢 | All 12 FI008 sections + `Refs: src/`/`Refs: tests/` present — confirmed by direct re-read of the full file. | — |
| D2 | MRE Validity & Reproducibility | 🟡 | 🟢 | Confirming pass hand-computed `[0.2,0.0,0.0]`/`[0.6,0.8,0.0]`; adversarial pass independently re-derived both from the pre-fix/post-fix loop bodies (`r[i]/mag` vs `a[i]/mag` with `mag=5`) rather than trusting the confirming pass's arithmetic. | — |
| D3 | Cross-Reference Integrity | 🟢 | 🟢 | Adversarial pass confirmed this file correctly declares no `**Related Bugs:**` (independent root cause from BUG-122/123 — different function, no shared code path) despite being filed under the same task #58 review. | — |
| D4 | Root Cause Quality | 🟢 | 🟢 | Adversarial pass re-read `project_on`'s loop body independently to confirm it genuinely reads `b.vector_iter()` (the oracle claim), not merely assumed from the confirming pass's description. | — |
| D5 | Execution Scope | 🟢 | 🟢 | Adversarial pass checked `normalize_to`/`normalized_to` (the other two functions in the same `## Normalization` grouping) for the same defect shape — both are single-argument-derived (`R` only, no second `A` parameter to potentially misread), so this defect shape cannot apply to them. | — |
| D6 | Crate Scope Unity | 🟢 | 🟢 | Only `mdmath_core`'s own `src/`/`tests/` and this bug-tracking file touched — no cross-crate scope creep. | — |
| D7 | Crate Locality | 🟢 | 🟢 | Fix is local to `normalize`'s own loop body; grepped and confirmed all 3 workspace-wide call sites (1 production, 2 test) pre-seed `r == a`, so no caller's behavior changes. | — |
| D8 | Crate Single Responsibility | 🟢 | 🟢 | Fix does not add any new responsibility — it corrects the function to read the argument its own name and signature already claimed to normalize. | — |

**Reproduced:** YES — `/tmp/mre124` pre-fix: `[0.2, 0.0, 0.0]` instead of `a`'s normalized
direction `[0.6, 0.8, 0.0]`, for `r=[1,0,0]` (unrelated, unit-length), `a=[3,4,0]`, 2026-08-15

## Refs: src/

| File | Change |
|------|--------|
| `module/math/mdmath_core/src/vector/arithmetics.rs` | `normalize`: added `let mut aiter = a.vector_iter();` before the loop; changed `*elem /= mag;` to `*elem = *aiter.next().unwrap() / mag;`. `Fix(BUG-124)`/`Root cause`/`Pitfall` comment added, referencing `project_on` as the correct-pattern precedent. |

## Refs: tests/

| File | Change |
|------|--------|
| `module/math/mdmath_core/tests/inc/arithmetics.rs` | Added `test_normalize_with_distinct_source_and_destination` (`bug_reproducer(BUG-124)`, 5-section doc comment, `r != a` fixture). |
