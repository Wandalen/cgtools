# BUG-123: `[E]::vector_iter_mut` mutates past the logical vector length `N` on longer slices

- **Severity:** Medium (silent out-of-bounds-of-contract mutation, no panic, no UB — a logical
  correctness defect confined to whatever the caller does with the extra mutated elements)
- **state:** Completed
- **Affects:** Any caller of `<[E] as VectorIterMut<E,N>>::vector_iter_mut()` where the backing
  slice's length is strictly greater than `N`
- **Component:** `module/math/mdmath_core` (`src/vector/slice.rs::impl VectorIterMut<E,N> for [E]`)
- **repo_identity:** self
- **Filed:** 2026-08-15
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/verified/
- **verification_date:** 2026-08-16
- **Related Bugs:** None — independent root cause from BUG-122/124, filed under the same task
  #58 targeted `mdmath_core` review

## Symptom

```bash
# data = [1, 2, 3, 4, 5], treated as a logical 3-element vector via VectorIterMut<i32, 3>:

# Wrong (pre-fix) -- vector_iter_mut::<i32,3>() yields all 5 elements, not just the first 3:
for x in <[i32] as VectorIterMut<i32,3>>::vector_iter_mut(&mut data) { *x += 100; }
data == [101, 102, 103, 104, 105]   # indices 3,4 (outside the logical length-3 vector) mutated

# Correct (post-fix) -- bounded to the first N=3 elements, matching the VectorIter sibling:
data == [101, 102, 103, 4, 5]       # only the logical vector's own elements touched
```

## Impact

**Who is affected:** Any caller that uses `[E]`'s `VectorIterMut<E,N>` impl on a slice longer
than `N` — i.e. treating a prefix of a larger buffer as a fixed-size logical vector for mutation
purposes, the exact "first N of a possibly-longer slice" contract `array_ref`/`vector_mut`/
`vector_iter` in the same file already document and enforce via their shared `len() >= N`
(not `== N`) assertion.

**What breaks:** Every element from index `N` to the slice's actual end also gets mutated,
silently — not just the logical vector's own first `N` elements. Any caller relying on the
"only the first N elements are touched" contract (implied by the shared `>=`-style length
assertion and by the sibling `VectorIter::vector_iter`'s own `.take(N)`) gets unexpected writes
past the intended boundary.

**Magnitude:** Zero current in-crate callers pass a slice strictly longer than `N` to
`vector_iter_mut` (confirmed: every existing `tests/inc/vector_test/slice_test.rs` case uses a
slice whose length exactly equals `N`). Any future caller treating a longer buffer's prefix as a
fixed-size vector for in-place mutation would hit this immediately and silently.

**Entity Scope:** None — a code-level trait-implementation defect, not an operational-entity
concern.

## How Discovered

Task #58, a targeted code review of `mdmath_core` dispatched under the standing bug-hunt
mandate. The reviewing agent flagged that `VectorIterMut<E,N>::vector_iter_mut` for `[E]`
asserts the same `self.len() >= N` precondition as its `VectorIter<E,N>::vector_iter` sibling in
the same file, but returns the slice's full, unbounded `IterMut` instead of taking only the
first `N` elements — while `vector_iter` correctly appends `.take(N)`. Independently confirmed
by direct re-read of the pre-fix source before filing:

```bash
$ grep -n "assert!.*len() >= N\|iter( self )\|iter_mut( self )\|\.take( N )" \
    module/math/mdmath_core/src/vector/slice.rs   # (pre-fix)
# vector_iter:      <[E]>::iter( self ).take( N )         <- bounded
# vector_iter_mut:  <[E]>::iter_mut( self )                <- NOT bounded, missing .take( N )
```

## Minimum Reproducible Example

```bash
rm -rf /tmp/mre123 && mkdir -p /tmp/mre123/src
cat > /tmp/mre123/Cargo.toml <<'EOF'
[package]
name = "mre123"
version = "0.1.0"
edition = "2021"

[dependencies]
mdmath_core = { path = "/home/user1/pro/lib/yrd_gamedev/cgtools/module/math/mdmath_core", features = [ "enabled" ] }
EOF
cat > /tmp/mre123/src/main.rs <<'EOF'
use mdmath_core::VectorIterMut;

fn main()
{
  let mut data : [ i32; 5 ] = [ 1, 2, 3, 4, 5 ];
  let slice : &mut [ i32 ] = &mut data;
  for x in <[ i32 ] as VectorIterMut< i32, 3 >>::vector_iter_mut( slice )
  {
    *x += 100;
  }
  println!( "{data:?}" );
}
EOF
cd /tmp/mre123 && cargo run 2>&1 | tail -1
```

**Expected** (post-fix — bounded to the first `N`=3 elements):
```
[101, 102, 103, 4, 5]
```

**Actual** (pre-fix — the full 5-element slice is mutated, not just the first 3):
```
[101, 102, 103, 104, 105]
```

**Verify Command** (≤3 lines, standalone):
```bash
cd /tmp/mre123 && cargo run 2>&1 | tail -1
# [101, 102, 103, 4, 5] = fixed; [101, 102, 103, 104, 105] = bug present
```
**What:** Violates the "first `N` of a possibly-longer slice" contract the file's own `>=`-style
length assertion establishes and the sibling `vector_iter` already upholds.

**Known MRE limitation (check 205):** `mdmath_core` is this workspace's own crate; the MRE
path-depends on it locally rather than a registry version, mirroring BUG-116/118-122's own
documented exception. All values are plain `i32` literals with no floating-point ambiguity this
local dependency could be hiding.

## Hypothesis Table

| ID | Hypothesis | State | Summary | Evidence |
|----|-----------|--------|---------|----------|
| H1 | `vector_iter_mut` for `[E]` omits the `.take(N)` its `vector_iter` sibling applies, so mutation reaches every element of the backing slice rather than just the logical vector's first `N`. | ✅ Root Cause | Direct read of `slice.rs` (pre-fix) confirms `vector_iter` ends in `.take(N)` while `vector_iter_mut` does not, despite both sharing the identical `assert!(self.len() >= N, ...)` precondition one line above. MRE confirms all 5 elements of a length-5 slice get mutated when `N=3`. | E1, E2 |
| H2 | The `assert!(self.len() >= N)` precondition itself is wrong and should require `== N` instead, which would make this a documentation/assertion bug rather than a missing-`.take()` bug. | ❌ Falsified | The `>=`-style assertion is shared identically by `array_ref`/`vector_mut`/`vector_iter` — all four accessors in this file are built on the same deliberate "treat the first N elements of a possibly-longer slice as the logical vector" contract, and three of the four already correctly bound their own traversal to exactly `N` (via `[E;N]` pointer casts or `.take(N)`). Only `vector_iter_mut` fails to uphold the shared contract; the contract itself is consistent and intentional. | E1 |
| H3 | Because `IterMut` and `Iter` have different underlying types, `.take(N)` cannot be applied to `vector_iter_mut`'s return value the same way it is applied to `vector_iter`'s. | ❌ Falsified | `core::slice::IterMut` implements `Iterator` exactly like `core::slice::Iter` does, and `Iterator::take` is a default trait method available on both without any additional bound — the fix is a direct, mechanical `.take(N)` append, identical in form to the sibling. Confirmed by the fix compiling cleanly with no signature or bound changes required. | E1 |

## Evidence Table

| # | Location | What it shows | Hypothesis |
|---|----------|---------------|------------|
| E1 | `module/math/mdmath_core/src/vector/slice.rs` (pre-fix) | `VectorIter::vector_iter`: `assert!(self.len() >= N, ...); <[E]>::iter(self).take(N)`. `VectorIterMut::vector_iter_mut`: `assert!(self.len() >= N, ...); <[E]>::iter_mut(self)` — identical precondition, missing `.take(N)` on the mutable path only. | H1 ✅, H2 ❌, H3 ❌ |
| E2 | `/tmp/mre123` run, pre-fix vs. post-fix, `data=[1,2,3,4,5]`, `N=3` | Pre-fix: `[101,102,103,104,105]` — all 5 elements mutated. Post-fix: `[101,102,103,4,5]` — only the first 3 (the logical vector) mutated, indices 3/4 untouched. | H1 ✅ |

## Root Cause

```
<[E] as VectorIterMut<E,N>>::vector_iter_mut( self )
  assert!( self.len() >= N )       <- establishes "first N of possibly more" contract
  <[E]>::iter_mut( self )          <- returns ALL of self's elements, unbounded   ✗
                                       (sibling vector_iter correctly ends .take(N) here)
```

`array_ref`/`vector_mut` (via `[E;N]` pointer casts) and `vector_iter` (via `.take(N)`) all
independently bound their traversal to exactly the first `N` elements, upholding the file's
shared `>=`-style length contract. `vector_iter_mut` asserts the identical precondition but
never applies the corresponding bound to its own returned iterator — a straightforward omission,
not a deeper design inconsistency, since the fix is a one-token addition matching an
already-established sibling pattern in the same file.

## Why Not Caught

Every existing `vector_iter_mut` slice test (`tests/inc/vector_test/slice_test.rs`) used a slice
whose length exactly equals `N` (0, 1, or 3 elements sliced with a matching `N`) — under
`len() == N`, `.take(N)` and no-`.take(N)` are behaviorally indistinguishable, since there is
nothing past index `N` to over-mutate. The defect only surfaces when the slice is strictly
longer than `N`, a case no existing test constructs.

## Fix Location

`module/math/mdmath_core/src/vector/slice.rs`, `impl<E,const N:usize> VectorIterMut<E,N> for
[E]`. One change:

```rust
// before
fn vector_iter_mut< 'data >( &'data mut self ) -> impl VectorIterator< 'data, &'data mut E >
where
  E : 'data,
{
  assert!( self.len() >= N, "Slice must have at least {N} elements" );
  <[ E ]>::iter_mut( self )
}

// after
fn vector_iter_mut< 'data >( &'data mut self ) -> impl VectorIterator< 'data, &'data mut E >
where
  E : 'data,
{
  assert!( self.len() >= N, "Slice must have at least {N} elements" );
  <[ E ]>::iter_mut( self ).take( N )
}
```

For every existing caller (all of which pass a slice whose length exactly equals `N`), this
change is an exact no-op in observable behavior — confirmed by the pre-existing
`test_vector_iter_mut_slice` test still passing unchanged.

## Prevention

Added `test_vector_iter_mut_slice_longer_than_n_leaves_tail_untouched` to
`tests/inc/vector_test/slice_test.rs`: uses a 5-element slice with `N=3`, mutates every element
the iterator yields, and asserts both that exactly 3 elements were yielded and that indices 3/4
remain untouched — this would fail under the pre-fix unbounded `iter_mut(self)`, since it yields
all 5 elements.

**Pitfall:** an `>=`-style length assertion documents "first N of possibly more" — every
accessor built on that contract must independently bound its own traversal to `N`; a sibling
method already doing so (here, `vector_iter`'s `.take(N)`) is not evidence that a related method
sharing the identical precondition also applies the bound.

## Generalized Version

**Broken assumption:** "Two sibling accessor methods sharing an identical precondition
assertion also share identical downstream bounding behavior" — false; the assertion only
establishes what inputs are *accepted*, not what the method *does* with the accepted input's
extra length. Each method's body must be checked independently.

**Confirmed general rule:** when a type exposes multiple accessors over the same "first N of a
possibly-longer container" contract (established by a shared `>=`-style length assertion), audit
each accessor's body individually for whether it actually bounds its own traversal/write range
to `N` — do not infer from one correct sibling that the others are too, especially across an
immutable/mutable pair where the mutable variant's failure mode (extra writes) is harder to
notice by accident than the immutable variant's (extra reads, often simply unused).

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-15 | filed | Discovered via task #58's targeted code review of `mdmath_core`; confirmed via direct comparison of `vector_iter`/`vector_iter_mut`'s bodies in `slice.rs` before filing. |
| 2026-08-15 | fixed | Added `.take(N)` to `vector_iter_mut`'s return expression, matching the `vector_iter` sibling. 3-field `Fix(BUG-123)`/`Root cause`/`Pitfall` comment added at the fix site. |
| 2026-08-15 | verified | Added `test_vector_iter_mut_slice_longer_than_n_leaves_tail_untouched` to `tests/inc/vector_test/slice_test.rs`; scoped test run (`verb test_only -p mdmath_core` via `longrun`) passed with the new test green alongside the pre-existing suite. |
| 2026-08-16 | completed | Acceptance verification by a distinct session (filer/fixer 2026-08-15, this verifier 2026-08-16; fix not present anywhere in this session's own context — approached as a fresh reader). Independently re-confirmed via targeted re-read of `slice.rs`'s `vector_iter_mut` (the `.take(N)` append genuinely present, matching `Refs: src/`) and the full `bug_reproducer(BUG-123)` test body (non-tautological — asserts indices ≥N are explicitly untouched, not just that the test exists). Re-ran `verb/test_only pkg::mdmath_core` via `longrun`: 94/94 passed. `cargo clippy -p mdmath_core --all-features --all-targets -- -D warnings`: clean. MAAV Tier 2 Dual-Role Self-Check (`governance/maav.rulebook.md`), covering BUG-122/123/124 together. State → Completed. |

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 8/8

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Completeness | 🟢 | 🟢 | All 12 FI008 sections + `Refs: src/`/`Refs: tests/` present — confirmed by direct re-read of the full file. | — |
| D2 | MRE Validity & Reproducibility | 🟡 | 🟢 | Confirming pass wrote the MRE from the fixed source; adversarial pass independently re-checked that `core::slice::IterMut::take(N)` genuinely stops after `N` items (standard library `Iterator::take` semantics) rather than assuming it without confirming against the `core` docs. | — |
| D3 | Cross-Reference Integrity | 🟢 | 🟢 | Adversarial pass confirmed this file correctly declares no `**Related Bugs:**` (independent root cause from BUG-122/124 — different function, no shared code path) despite being filed under the same task #58 review. | — |
| D4 | Root Cause Quality | 🟢 | 🟢 | Adversarial pass re-read `array_ref`/`vector_mut`'s pointer-cast bounding mechanism independently to confirm the "first N of possibly more" contract is genuinely shared crate-wide within this file, not asserted without checking. | — |
| D5 | Execution Scope | 🟢 | 🟢 | Adversarial pass checked whether `ArrayMut::vector_mut` (the other mutable accessor in the same file) has an equivalent gap — it doesn't; its `[E;N]` pointer-cast approach is structurally bounded to exactly `N` elements by the cast's own type, with no separate `.take`-style step that could be omitted. | — |
| D6 | Crate Scope Unity | 🟢 | 🟢 | Only `mdmath_core`'s own `src/`/`tests/` and this bug-tracking file touched — no cross-crate scope creep. | — |
| D7 | Crate Locality | 🟢 | 🟢 | Fix is local to the single `vector_iter_mut` method body; no other function depends on its pre-fix unbounded behavior (confirmed: zero other in-crate callers of this trait method exist). | — |
| D8 | Crate Single Responsibility | 🟢 | 🟢 | Fix does not add any new responsibility — it corrects the method to uphold the contract its own precondition assertion already declared. | — |

**Reproduced:** YES — `/tmp/mre123` pre-fix: `[101,102,103,104,105]` for a length-3 logical
vector over a length-5 slice, 2026-08-15

## Refs: src/

| File | Change |
|------|--------|
| `module/math/mdmath_core/src/vector/slice.rs` | `impl VectorIterMut<E,N> for [E]::vector_iter_mut`: appended `.take(N)` to `<[E]>::iter_mut(self)`, matching the `VectorIter::vector_iter` impl. `Fix(BUG-123)`/`Root cause`/`Pitfall` comment added. |

## Refs: tests/

| File | Change |
|------|--------|
| `module/math/mdmath_core/tests/inc/vector_test/slice_test.rs` | Added `test_vector_iter_mut_slice_longer_than_n_leaves_tail_untouched` (`bug_reproducer(BUG-123)`, 5-section doc comment, length-5 slice with `N=3`). |
