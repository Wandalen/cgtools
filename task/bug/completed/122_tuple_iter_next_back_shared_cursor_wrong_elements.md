# BUG-122: `Tuple{2,3,4}Iter::next_back()` returns the wrong element under mixed `.next()`/`.next_back()` traversal

- **Severity:** Medium (non-crashing, silently wrong values — read-only siblings of the already-fixed BUG-050 `*Mut` types)
- **state:** Completed
- **Affects:** Any caller of `(E,E)`/`(E,E,E)`/`(E,E,E,E)`'s `VectorIter::vector_iter()` (the immutable, shared-reference iterator) that mixes `.next()` and `.next_back()` calls on the same iterator instance instead of using it in one direction only
- **Component:** `module/math/mdmath_core` (`src/vector/tuple2.rs::Tuple2Iter`, `src/vector/tuple3.rs::Tuple3Iter`, `src/vector/tuple4.rs::Tuple4Iter`)
- **repo_identity:** self
- **Filed:** 2026-08-15
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/verified/
- **verification_date:** 2026-08-16
- **Related Bugs:** BUG-050 (`module/math/mdmath_core`) — identical shared-single-`index`-cursor root cause in the sibling `Tuple{2,3,4}IterMut` types; that fix never propagated to these read-only counterparts, which is exactly what this bug closes

## Symptom

```bash
# A 3-tuple (42, 43, 44), iterated with one .next() then one .next_back():

# Wrong (pre-fix) -- next_back() reinterprets next()'s leftover index as if counted from the back:
iter.next()       -> Some(&42)   # correct: front element
iter.next_back()  -> Some(&43)   # WRONG: should be &44 (the true back element); &43 was never
                                  #        supposed to be yielded yet, and &44 is never reached

# Correct (post-fix):
iter.next()       -> Some(&42)   # front element
iter.next_back()  -> Some(&44)   # back of the remaining {43,44} range
```

## Impact

**Who is affected:** Any caller of `vector_iter()` on a 2/3/4-element tuple that calls `.next()`
and `.next_back()` on the *same* iterator instance in a mixed order — e.g. consuming from both
ends of a small fixed-size vector simultaneously. Pure forward iteration (`.next()` only) and
pure reverse iteration (`.rev()` then `.next()` only) are both unaffected, since those never
exercise the cross-direction interaction between the two methods.

**What breaks:** `next_back()` silently returns the wrong element — either a duplicate of an
already-yielded element, or skips an element entirely — with no panic, no `None` early, and no
type-level signal that anything went wrong. The only observable symptom is a wrong value read
downstream of the iteration.

**Magnitude:** Zero current in-crate callers mix `.next()`/`.next_back()` on these three
immutable iterator types (confirmed: every existing `vector_iter` test in
`tests/inc/vector_test/tuple{2,3,4}_test.rs` uses either pure-forward or pure-`.rev()`
iteration). Any future caller that does — e.g. a two-pointer algorithm over a fixed-size tuple
vector — would hit this immediately and silently.

**Entity Scope:** None — a code-level iterator-correctness defect, not an operational-entity
concern.

## How Discovered

Task #58, a targeted code review of `mdmath_core` dispatched under the standing bug-hunt
mandate, using the same "read every source file directly, hand-derive expected behavior rather
than trusting existing tests" methodology that surfaced BUG-250 through BUG-121 in `ndarray_cg`.
The reviewing agent flagged that `Tuple2Iter`/`Tuple3Iter`/`Tuple4Iter` (the plain, immutable
`Iterator`/`DoubleEndedIterator` impls in `vector/tuple{2,3,4}.rs`) still used the single shared
`index : usize` field with hardcoded per-direction match arms — the exact shape BUG-050 already
fixed, but only for the `*Mut` sibling types in the same three files. Independently confirmed by
direct re-read of the pre-fix source before filing:

```bash
$ grep -n "index : usize\|fn next(\|fn next_back(" \
    module/math/mdmath_core/src/vector/tuple2.rs   # (pre-fix)
# Tuple2Iter (immutable):   index : usize            <- shared cursor, BUG-050's exact shape
# Tuple2IterMut (mutable):  front : usize, back : usize   <- already fixed by BUG-050
```

## Minimum Reproducible Example

```bash
rm -rf /tmp/mre122 && mkdir -p /tmp/mre122/src
cat > /tmp/mre122/Cargo.toml <<'EOF'
[package]
name = "mre122"
version = "0.1.0"
edition = "2021"

[dependencies]
mdmath_core = { path = "/home/user1/pro/lib/yrd_gamedev/cgtools/module/math/mdmath_core", features = [ "enabled" ] }
EOF
cat > /tmp/mre122/src/main.rs <<'EOF'
use mdmath_core::VectorIter;

fn main()
{
  let tuple : ( i32, i32, i32 ) = ( 42, 43, 44 );
  let mut iter = tuple.vector_iter();
  let a = iter.next();       // front
  let b = iter.next_back();  // should be the true back of the remaining {43,44} range
  println!( "a = {a:?}, b = {b:?}" );
}
EOF
cd /tmp/mre122 && cargo run 2>&1 | tail -1
```

**Expected** (post-fix — independent front/back cursors):
```
a = Some(42), b = Some(44)
```

**Actual** (pre-fix — shared `index` reinterpreted across the direction switch):
```
a = Some(42), b = Some(43)
```

**Verify Command** (≤3 lines, standalone):
```bash
cd /tmp/mre122 && cargo run 2>&1 | tail -1
# b = Some(44) = fixed; b = Some(43) = bug present
```
**What:** Violates `DoubleEndedIterator`'s contract that `next()` and `next_back()` consume from
opposite ends of the same remaining range without overlap or gaps.

**Known MRE limitation (check 205):** `mdmath_core` is this workspace's own crate; the MRE
path-depends on it locally rather than a registry version, mirroring BUG-116/118-121's own
documented exception. The values involved (`42`,`43`,`44`) are plain `i32` literals with no
floating-point ambiguity this local dependency could be hiding.

## Hypothesis Table

| ID | Hypothesis | State | Summary | Evidence |
|----|-----------|--------|---------|----------|
| H1 | `Tuple2Iter`/`Tuple3Iter`/`Tuple4Iter` share BUG-050's exact defect shape (one `index : usize` field, hardcoded per-direction match arms) — the fix simply never propagated from the `*Mut` types to these immutable siblings. | ✅ Root Cause | Direct read of `tuple2.rs`/`tuple3.rs`/`tuple4.rs` (pre-fix) confirmed all three immutable iterators still used a single shared `index` field while all three `*Mut` counterparts already used `front`/`back`. MRE confirms the exact wrong-value symptom. | E1, E2 |
| H2 | The defect is UB/aliasing-related, like BUG-050, just not yet caught by Miri for the shared-reference case. | ❌ Falsified | `&E` (shared reference) yields have no exclusivity requirement — two live `&E` pointing at the same slot is always sound under Stacked Borrows; the defect here is purely a wrong-value correctness bug, not a soundness one. Confirmed by reasoning about Rust's aliasing rules for `&T` vs `&mut T`, and by the fact BUG-050's own Miri run only ever targeted the `*Mut` types. | E1 |
| H3 | `.rev()`-then-`.next()` iteration (already tested for all three tuple sizes) is sufficient to catch this class of defect, making a dedicated mixed-direction test redundant. | ❌ Falsified | `.rev()` swaps which method is called for "forward" traversal but still only exercises a *single* direction end-to-end (fully reversed) — the shared-`index` defect only manifests when `.next()` and `.next_back()` are both called, unreversed, on the *same* iterator instance. Confirmed: every pre-fix `test_vector_iter_rev_tupleN` test passes even with the bug present. | E2 |

## Evidence Table

| # | Location | What it shows | Hypothesis |
|---|----------|---------------|------------|
| E1 | `module/math/mdmath_core/src/vector/tuple2.rs` (pre-fix, `Tuple2Iter`), and the identical shape in `tuple3.rs`/`tuple4.rs` | `struct Tuple2Iter { tuple: &'_ (E,E), index: usize }`; `next()` matched `index` then incremented it, `next_back()` matched a *decremented* `index` — both operating on the same field with direction-specific arms, byte-identical in structure to BUG-050's pre-fix `Tuple2IterMut`. | H1 ✅, H2 ❌ |
| E2 | `/tmp/mre122` run, pre-fix vs. post-fix, tuple `(42,43,44)`, one `.next()` then one `.next_back()` | Pre-fix: `b = Some(43)` (wrong — re-yields the pending middle element instead of the true back). Post-fix: `b = Some(44)` (correct). Confirms the defect is a wrong-value, non-UB correctness bug, and that pure-`.rev()` tests (already passing pre-fix) cannot detect it. | H1 ✅, H3 ❌ |

## Root Cause

```
Tuple2Iter { tuple: &(42,43), index: 0 }
  .next()                          -> index=0 matched -> &tuple.0 (=42); index becomes 1
  .next_back()                     -> index(=1) matched in next_back()'s own arms, which
                                       independently treat "1" as "the back position" rather
                                       than tracking how much has already been consumed from
                                       the front -> &tuple.1 (=43)   ✗ should be the tuple's
                                       true remaining back element
```

A single shared `index` field cannot simultaneously represent "how far consumed from the
front" and "how far consumed from the back" — `next()` and `next_back()` each interpreted it
under their own direction-specific assumption, correct only when the other method is never
called on the same iterator instance. This is byte-for-byte the same defect shape BUG-050 fixed
in `Tuple2IterMut`/`Tuple3IterMut`/`Tuple4IterMut` — the fix was simply never applied to these
three immutable sibling types (`Tuple2Iter`/`Tuple3Iter`/`Tuple4Iter`) in the same files.

## Why Not Caught

Every existing `vector_iter` test (`tests/inc/vector_test/tuple{2,3,4}_test.rs`) called either
`.next()` repeatedly (pure forward) or `.rev()` then `.next()` repeatedly (pure, fully-reversed
traversal) — never mixed `.next()`/`.next_back()` calls on the same unwrapped iterator, the
exact trigger condition. BUG-050's own fix and its regression tests were scoped to the `*Mut`
types only (where the consequence is Miri-detectable UB), so the parallel defect in the
immutable types — merely wrong values, non-crashing — was never independently checked.

## Fix Location

`module/math/mdmath_core/src/vector/tuple2.rs`, `tuple3.rs`, `tuple4.rs`. Same change in each:
replaced the `Tuple{2,3,4}Iter` struct's single `index : usize` field with independent
`front : usize, back : usize` cursors (mirroring the already-fixed `*Mut` siblings and
`core::slice::Iter`'s own two-cursor design), and rewrote `next()`/`next_back()` with a shared
`if front >= back { return None }` guard plus direction-local increment/decrement before the
match.

```rust
// before (tuple2.rs, representative of all three files)
struct Tuple2Iter< 'tuple_ref, E >
{
  tuple : &'tuple_ref ( E, E ),
  index : usize,
}
// next(): match self.index { 0 => ..., 1 => ..., _ => None }, self.index += 1
// next_back(): match self.index { ... }, self.index -= 1   <- SAME field, direction-blind

// after
struct Tuple2Iter< 'tuple_ref, E >
{
  tuple : &'tuple_ref ( E, E ),
  front : usize,
  back : usize,
}
// next(): if front >= back { None } else { let i = front; front += 1; match i { ... } }
// next_back(): if front >= back { None } else { back -= 1; match back { ... } }
```

The `vector_iter()` constructor in each file changed from `Tuple{2,3,4}Iter { tuple: self, index:
0 }` to `Tuple{2,3,4}Iter { tuple: self, front: 0, back: N }` (`N` = 2/3/4 per file).

## Prevention

Added `test_vector_iter_next_and_next_back_disjoint_tuple{2,3,4}()` to each of
`tests/inc/vector_test/tuple2_test.rs`, `tuple3_test.rs`, `tuple4_test.rs` — mixes `.next()`/
`.next_back()` on `vector_iter()` (the immutable trait) and asserts every yielded value against
what a correct front/back traversal produces, mirroring BUG-050's own mixed-direction test
pattern but for the read-only iterators.

**Pitfall:** a `DoubleEndedIterator` backed by one shared index counter is only correct under
single-direction iteration, regardless of whether its yielded references are shared (`&E`,
safe-but-wrong under misuse) or exclusive (`&mut E`, unsound under misuse per BUG-050) — fixing
the unsound `*Mut` variant does not imply the merely-wrong immutable sibling was also fixed;
each must be independently verified.

## Generalized Version

**Broken assumption:** "Fixing a shared-cursor `DoubleEndedIterator` defect in a type's mutable
variant also covers its immutable sibling, since the immutable case can't cause UB anyway" —
false; UB-severity and correctness-severity are independent axes. The absence of a soundness
consequence does not imply the absence of a correctness one.

**Confirmed general rule:** when a defect class is fixed in one member of a family of
near-identical sibling types (here: `*Mut` vs. plain, across 3 tuple arities), check every other
member of that family independently — do not infer "already fixed" from one sibling's fix,
especially when the un-fixed sibling's failure mode (wrong value, not a crash) is less likely to
be caught by chance during unrelated development.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-15 | filed | Discovered via task #58's targeted code review of `mdmath_core`; confirmed as BUG-050's un-propagated immutable-sibling counterpart via direct source comparison before filing. |
| 2026-08-15 | fixed | Replaced the shared `index` field with independent `front`/`back` cursors in `Tuple2Iter`/`Tuple3Iter`/`Tuple4Iter` (`tuple2.rs`/`tuple3.rs`/`tuple4.rs`). 3-field `Fix(BUG-122)`/`Root cause`/`Pitfall` comments added at each fix site. |
| 2026-08-15 | verified | Added one mixed-direction regression test per tuple arity to `tests/inc/vector_test/tuple{2,3,4}_test.rs`; scoped test run (`verb test_only -p mdmath_core` via `longrun`) passed with all 3 new tests green alongside the pre-existing suite. |
| 2026-08-16 | completed | Acceptance verification by a distinct session (filer/fixer 2026-08-15, this verifier 2026-08-16; fix not present anywhere in this session's own context — approached as a fresh reader). Independently re-confirmed via full re-read of `tuple2.rs`/`tuple3.rs`/`tuple4.rs` (front/back cursor rewrite genuinely present and correct at all 3 arities, hand-traced independently of this file's own trace) and the 3 `bug_reproducer(BUG-122)` test bodies (non-tautological — assert the back element is not a repeat of the front). Re-ran `verb/test_only pkg::mdmath_core` via `longrun`: 94/94 passed. `cargo clippy -p mdmath_core --all-features --all-targets -- -D warnings`: clean. MAAV Tier 2 Dual-Role Self-Check (`governance/maav.rulebook.md`), covering BUG-122/123/124 together. State → Completed. |

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 8/8

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Completeness | 🟢 | 🟢 | All 12 FI008 sections + `Refs: src/`/`Refs: tests/` present — confirmed by direct re-read of the full file. | — |
| D2 | MRE Validity & Reproducibility | 🟡 | 🟢 | Confirming pass wrote the MRE from the fixed struct's field names; adversarial pass independently re-ran the exact `next()`/`next_back()` sequence by hand against the pre-fix match-arm logic to confirm `Some(43)` (not some other wrong value) is what pre-fix code actually produces. | — |
| D3 | Cross-Reference Integrity | 🟢 | 🟢 | Adversarial pass confirmed the `**Related Bugs:** BUG-050` link is accurate (same root-cause shape, same source files, sibling types) and not a broken/invented reference. | — |
| D4 | Root Cause Quality | 🟢 | 🟢 | Adversarial pass re-traced `next()`/`next_back()`'s match-arm logic from the pre-fix source independently rather than trusting the confirming pass's description. | — |
| D5 | Execution Scope | 🟢 | 🟢 | Adversarial pass checked whether any other iterator type in the crate (e.g. `[E]`'s slice-backed iterators) shares this shared-cursor shape — it doesn't; `VectorIter for [E]` delegates to `core::slice::Iter`, which already has independent front/back cursors internally. | — |
| D6 | Crate Scope Unity | 🟢 | 🟢 | Only `mdmath_core`'s own `src/`/`tests/` and this bug-tracking file touched — no cross-crate scope creep. | — |
| D7 | Crate Locality | 🟢 | 🟢 | Fix is local to the three `Tuple{2,3,4}Iter` structs and their own `Iterator`/`DoubleEndedIterator` impls; no other function constructs or depends on these structs' internal field layout. | — |
| D8 | Crate Single Responsibility | 🟢 | 🟢 | Fix does not add any new responsibility — it corrects the existing `DoubleEndedIterator` contract the type already claimed to implement. | — |

**Reproduced:** YES — `/tmp/mre122` pre-fix: `b = Some(43)` for a 3-tuple's true back element `44`, 2026-08-15

## Refs: src/

| File | Change |
|------|--------|
| `module/math/mdmath_core/src/vector/tuple2.rs` | `Tuple2Iter`: `index : usize` → `front : usize, back : usize`; `next()`/`next_back()` rewritten with bounds-checked cursor logic; `vector_iter()` constructor updated. `Fix(BUG-122)`/`Root cause`/`Pitfall` comment added. |
| `module/math/mdmath_core/src/vector/tuple3.rs` | Same transformation for `Tuple3Iter` (`back: 3`). |
| `module/math/mdmath_core/src/vector/tuple4.rs` | Same transformation for `Tuple4Iter` (`back: 4`). |

## Refs: tests/

| File | Change |
|------|--------|
| `module/math/mdmath_core/tests/inc/vector_test/tuple2_test.rs` | Added `test_vector_iter_next_and_next_back_disjoint_tuple2` (`bug_reproducer(BUG-122)`, 5-section doc comment). |
| `module/math/mdmath_core/tests/inc/vector_test/tuple3_test.rs` | Added `test_vector_iter_next_and_next_back_disjoint_tuple3` (same pattern). |
| `module/math/mdmath_core/tests/inc/vector_test/tuple4_test.rs` | Added `test_vector_iter_next_and_next_back_disjoint_tuple4` (same pattern). |
