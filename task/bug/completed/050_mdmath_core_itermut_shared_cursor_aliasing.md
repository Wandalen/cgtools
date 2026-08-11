# BUG-050: `Tuple2IterMut`/`Tuple3IterMut`/`Tuple4IterMut`'s shared `index` cursor aliases `&mut` references under mixed-direction iteration

- **Severity:** High
- **state:** Completed
- **Affects:** `mdmath_core::vector::{Tuple2IterMut, Tuple3IterMut, Tuple4IterMut}` — every caller of a `(E,E)`/`(E,E,E)`/`(E,E,E,E)` tuple's `VectorIterMut::vector_iter_mut()` that mixes `.next()` and `.next_back()` on the same live iterator (not pure-forward, and not `.rev()`-then-pure-forward, but a genuine mix of both ends on one instance) — currently zero live call sites (see `## Impact`)
- **Component:** `module/math/mdmath_core` — `vector::tuple2::Tuple2IterMut`, `vector::tuple3::Tuple3IterMut`, `vector::tuple4::Tuple4IterMut`
- **repo_identity:** self
- **Filed:** 2026-08-10
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verification_date:** 2026-08-10
- **Fixed:** 2026-08-10
- **Accepted By:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/ (self — same-session Tier 2 Dual-Role Self-Check, no separate PROC16 acceptance actor)

## Symptom

```bash
# terminal output — Miri Stacked Borrows check against the real crate, ORIGINAL (unfixed) source
$ cargo +nightly miri test -p mdmath_core --all-features -- mixed_direction_no_aliasing next_and_next_back_disjoint
error: Undefined Behavior: attempting a write access using <264200> at alloc97151[0x0], but that tag does not exist in the borrow stack for this location
   --> module/math/mdmath_core/tests/inc/vector_test/tuple2_test.rs:125:5
    |
125 |     *a = 100;
    |     ^^^^^^^^ this error occurs as part of an access at alloc97151[0x0..0x4]
    |
    = help: this indicates a potential bug in the program: it performed an invalid operation, but the Stacked Borrows rules it violated are still experimental
help: <264200> was created by a Unique retag at offsets [0x0..0x4]
   --> module/math/mdmath_core/tests/inc/vector_test/tuple2_test.rs:123:13
    |
123 |     let a = iter.next().unwrap();
    |             ^^^^^^^^^^^^^^^^^^^^
help: <264200> was later invalidated at offsets [0x0..0x4] by a Unique retag
   --> module/math/mdmath_core/src/vector/tuple2.rs:226:31
error: aborting due to 1 previous error

# terminal output — once fixed, same command
$ cargo +nightly miri test -p mdmath_core --all-features -- mixed_direction_no_aliasing next_and_next_back_disjoint
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 73 filtered out; finished in 0.87s
```

`Tuple2IterMut`, `Tuple3IterMut`, and `Tuple4IterMut` (`vector/tuple2.rs`, `vector/tuple3.rs`,
`vector/tuple4.rs`) each implement `DoubleEndedIterator` by keying both `next()` and
`next_back()` off one shared `index : usize` field, with per-value match arms hardcoded for a
single traversal direction. The moment a caller mixes `.next()` and `.next_back()` on the same
live iterator — exactly the usage pattern `DoubleEndedIterator` advertises as supported — the
two methods misinterpret the shared counter and hand out a second `&mut` reference to a tuple
field a previous call already returned a live `&mut` reference to. This is genuine Undefined
Behavior (confirmed by Miri's Stacked Borrows checker above), not merely a wrong final value.

## Impact

**Who is affected:** Any current or future caller of `(E,E)`/`(E,E,E)`/`(E,E,E,E)`'s
`VectorIterMut::vector_iter_mut()` (directly, or transitively through `mdmath_core::Vector`'s
own `VectorIterMut` blanket delegation, `vector/mod.rs:340-358`) that calls `.next()` and
`.next_back()` on the same iterator instance without fully draining one direction first — no
`unsafe` block is required on the caller's side to trigger this; the unsoundness is entirely
internal to the iterator's own hand-rolled implementation.

**What breaks:** Silent memory-safety violation — two live `&mut` references pointing at the
same field simultaneously, which Miri's Stacked Borrows checker classifies as Undefined
Behavior. In a real (non-Miri) build, the visible symptom is silent data corruption: whichever
of the two aliased writes executes last "wins," and the other is lost with no panic, no
warning, and no compiler diagnostic — worse under optimization, where the compiler is entitled
to assume `&mut` references never alias and may reorder, cache, or eliminate accesses in ways
that produce results even the "last write wins" model doesn't predict.

**Why High, not Critical or Medium (contrast with BUG-007/BUG-043/BUG-052):** This project's own
closed-bug precedent (`task/bug/readme.md`) reserves **Critical** for defects that block the
whole workspace right now regardless of caller — BUG-007's yanked dependency stopped every
build. This bug is dormant (see Magnitude below): zero live callers currently mix
`.next()`/`.next_back()`, so it blocks nothing today, which rules out Critical. It is
nonetheless categorically worse than **Medium** BUG-043 (a wrong-value bug, no unsafe code, no
UB — worst case a stale-but-defined number) and at least as serious as **High** BUG-052 (a live,
reachable panic with no UB): this bug is Miri-confirmed real Undefined Behavior, reachable from
100% safe caller code exercising a publicly advertised trait capability (`DoubleEndedIterator`)
with zero `unsafe` on the caller's part — precisely the class of defect Rust's safety model
exists to rule out, and precisely why the workspace-wide triage plan that originated this
investigation (`task/draft/009_mdmath_core_itermut_aliasing_ub.md`) bucketed it separately as
"P1 — soundness," a categorically distinct, high-priority tier. Net: worse-in-kind than a
reachable panic, offset by zero current reachability — **High**, matching BUG-052's tier rather
than exceeding it.

**Magnitude — currently zero live callers, confirmed by exhaustive search:** a workspace-wide
`grep -rn "vector_iter_mut"` found every real (non-test) call site
(`module/math/mdmath_core/src/vector/arithmetics.rs` — 14 sites; `module/math/ndarray_cg/src/d2/arithmetics/mul.rs:74`;
`module/math/ndarray_cg/src/vector/general.rs:139`, itself a pass-through) uses only
`for elem in r.vector_iter_mut()`, `.zip(a.vector_iter())`, `.enumerate()`, or a bare sequential
`*iter.next().unwrap() = ...` chain — pure single-direction forward traversal in every case; a
second grep confirms zero production call sites invoke `.next_back()` or `.rev()` on the result
of `vector_iter_mut()` anywhere in this workspace. The defect is real and Miri-confirmed, but
dormant: it will silently corrupt the first future caller that mixes directions on one of these
iterators, which is why this is filed and fixed now rather than deferred — soundness bugs are
this project's own highest-priority triage bucket regardless of current caller topology.

**Entity Scope:** `None` — the affected code lives in ordinary source files
(`src/vector/tuple2.rs`, `tuple3.rs`, `tuple4.rs`), not an entity directory instance;
`## Affected Entity Collections` does not apply.

## How Discovered

Carried forward from `task/draft/009_mdmath_core_itermut_aliasing_ub.md`, itself filed from a
workspace-wide Delete/Rewrite/Fix triage plan's P1 (soundness) bucket. That task's own text
states the original file/line citation was **not** preserved when filed and must be
re-confirmed from scratch against current source before any change — so this investigation
started from zero, not from a trusted prior citation:

```bash
$ grep -rn "qqq : not sure it's sound" module/math/mdmath_core/src/vector/
module/math/mdmath_core/src/vector/tuple1.rs:162:      // qqq : not sure it's sound, either prove it or find a sound solution
module/math/mdmath_core/src/vector/tuple1.rs:193:      // qqq : not sure it's sound, either prove it or find a sound solution
module/math/mdmath_core/src/vector/tuple2.rs:180:        // qqq : not sure it's sound, either prove it or find a sound solution
module/math/mdmath_core/src/vector/tuple2.rs:216:        // qqq : not sure it's sound, either prove it or find a sound solution
module/math/mdmath_core/src/vector/tuple3.rs:182:        // qqq : not sure it's sound, either prove it or find a sound solution
module/math/mdmath_core/src/vector/tuple3.rs:224:        // qqq : not sure it's sound, either prove it or find a sound solution
module/math/mdmath_core/src/vector/tuple4.rs:192:        // qqq : not sure it's sound, either prove it or find a sound solution
module/math/mdmath_core/src/vector/tuple4.rs:244:        // qqq : not sure it's sound, either prove it or find a sound solution
```

The author's own contemporaneous doubt (`qqq : not sure it's sound, either prove it or find a
sound solution`) on every `unsafe` block in all four `Tuple{1,2,3,4}IterMut` types was the
starting lead. Manually tracing `next()`/`next_back()` call sequences by hand against each
type's actual match arms (not assumed) confirmed real aliasing for `Tuple2IterMut`,
`Tuple3IterMut`, and `Tuple4IterMut`, but proved `Tuple1IterMut` coincidentally sound (`N=1`
collapses `next()`'s and `next_back()`'s own conditions into the same single check, so it can
never double-yield) — `Tuple1IterMut` is therefore **not** part of this bug's scope; its
still-open `qqq` doubt is now a resolved-but-unremoved comment, tracked separately by this
workspace's own marker-backlog task (`task/draft/038_workspace_marker_backlog_cleanup.md`), not
touched here to avoid scope creep into iterator code this bug does not affect.

## Minimum Reproducible Example

Fully self-contained — plain `rustc`, no cargo project, no external crates, no cgtools paths.
`mdmath_core::vector::Tuple2IterMut` is a private, unpublished workspace type, so the script
below reproduces the exact defect *pattern* instead — a 2-element tuple wrapper with a
`DoubleEndedIterator` keyed off one shared `index` field, direction-hardcoded per arm —
structurally identical to the real bug at `module/math/mdmath_core/src/vector/tuple2.rs:171-231`
(pre-fix; see `## Root Cause` for the exact original source).

```bash
mkdir -p /tmp/mre050
cat > /tmp/mre050/repro.rs <<'EOF'
struct Pair( i32, i32 );

struct PairIterMut< 'p >
{
  pair : &'p mut Pair,
  index : usize,
}

impl< 'p > Iterator for PairIterMut< 'p >
{
  type Item = &'p mut i32;
  fn next( &mut self ) -> Option< Self::Item >
  {
    match self.index
    {
      0 => { self.index += 1; unsafe { Some( &mut *( &mut self.pair.0 as *mut i32 ) ) } },
      1 => { self.index += 1; unsafe { Some( &mut *( &mut self.pair.1 as *mut i32 ) ) } },
      _ => None,
    }
  }
}

impl< 'p > DoubleEndedIterator for PairIterMut< 'p >
{
  fn next_back( &mut self ) -> Option< Self::Item >
  {
    match self.index
    {
      // BUG: uses the FORWARD cursor's current value to pick the BACKWARD element, so once
      // next() has advanced index past 0, next_back() re-derives an already-yielded field.
      0 => { self.index += 1; unsafe { Some( &mut *( &mut self.pair.1 as *mut i32 ) ) } },
      1 => { self.index += 1; unsafe { Some( &mut *( &mut self.pair.0 as *mut i32 ) ) } },
      _ => None,
    }
  }
}

fn main()
{
  let mut pair = Pair( 42, 43 );
  let mut iter = PairIterMut { pair: &mut pair, index: 0 };
  let a = iter.next().unwrap();       // index 0 -> 1, yields &mut pair.0
  let b = iter.next_back().unwrap();  // index now 1 -> 2, yields &mut pair.0 AGAIN (aliases `a`)
  *a = 100;
  *b = 200;
  println!( "pair = ({}, {})", pair.0, pair.1 );
  assert_eq!( ( pair.0, pair.1 ), ( 100, 200 ), "next() and next_back() must yield disjoint elements, not alias the same field" );
}
EOF
rustc --edition 2021 /tmp/mre050/repro.rs -o /tmp/mre050/repro 2>&1
echo "compile exit: $?"
/tmp/mre050/repro
echo "run exit: $?"
```

**Expected:**
```
compile exit: 0
pair = (100, 200)
run exit: 0
```

**Actual:**
```
compile exit: 0
pair = (200, 43)

thread 'main' (2760835) panicked at /tmp/mre050/repro.rs:47:3:
assertion `left == right` failed: next() and next_back() must yield disjoint elements, not alias the same field
  left: (200, 43)
 right: (100, 200)
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
run exit: 101
```

**Verify Command:** `/tmp/mre050/repro; test $? -eq 101` — **What:** demonstrates that
`next()` then `next_back()` on a shared-`index`-cursor `DoubleEndedIterator` silently double
-writes one field and never reaches the other, reproducing the value-level symptom of the exact
invariant violated by `Tuple2IterMut` at `module/math/mdmath_core/src/vector/tuple2.rs:171-231`
(pre-fix). The real crate's own test suite additionally proves this is genuine Undefined
Behavior, not just a wrong value — see `## Symptom`'s Miri output, captured directly against
`mdmath_core` itself (Miri cannot run standalone against this synthetic MRE's `assert_eq!`
-based harness in the same way; the aliasing *mechanism* is identical in both).

## Hypothesis Table

| ID | Hypothesis | State | Summary | Evidence |
|----|-----------|--------|---------|----------|
| H1 | `next()`/`next_back()` share one `index : usize` field with match arms hardcoded per-direction; mixing the two methods on one iterator makes them misinterpret the shared counter and double-yield an already-returned field | ✅ Root Cause | Original `tuple2.rs:161-231`: `next()`'s `0` arm yields `tuple.0`; `next_back()`'s `1` arm (reached once `next()` has advanced `index` to `1`) also yields `tuple.0` | E1, E3, E4 |
| H2 | The `qqq` comment marks a deliberately-accepted, already-reviewed tradeoff, not a real gap | ❌ Disproved | Comment text is literally "not sure it's sound, either prove it or find a sound solution" — unresolved doubt, not acceptance | E2 |
| H3 | This shared-cursor shape is equivalent to `core::slice::IterMut`'s own (real, sound) two-ended design, so it should be equally sound | ❌ Disproved | `core::slice::IterMut` advances two genuinely independent pointers that only ever converge, never alias; `Tuple2IterMut` had exactly one field, reinterpreted differently by each method — a fundamentally different, unsound shape | E1, E3 |

## Evidence Table

| # | Location | What it shows | Hypothesis |
|---|----------|---------------|------------|
| E1 | `git show HEAD:module/math/mdmath_core/src/vector/tuple2.rs` lines 161-231 | Original `Tuple2IterMut`: one `index : usize` field; `next()`'s arms `0→tuple.0, 1→tuple.1`; `next_back()`'s arms `0→tuple.1, 1→tuple.0` — both keyed off the same field, each direction's arms authored independently of the other | H1 ✅, H3 ❌ |
| E2 | `git show HEAD:module/math/mdmath_core/src/vector/tuple2.rs` lines 180, 216 (and the identical pattern in `tuple3.rs:182,224` / `tuple4.rs:192,244`) | Every `unsafe` block in all three types carries `// qqq : not sure it's sound, either prove it or find a sound solution` — contemporaneous author-flagged, unresolved doubt | H1 ✅, H2 ❌ |
| E3 | `cargo +nightly miri test -p mdmath_core --all-features -- mixed_direction_no_aliasing next_and_next_back_disjoint` against original source (captured in `## Symptom`) | `error: Undefined Behavior: attempting a write access using <264200> ... but that tag does not exist in the borrow stack` — the write through `a` (from `next()`) is invalid because `next_back()`'s later reborrow of the same field (`tuple2.rs:226`) invalidated `a`'s unique tag | H1 ✅ (direct UB confirmation) |
| E4 | Manual trace, N=3 (`next()`,`next()`,`next_back()`) and N=4 (`next()`,`next_back()`,`next()`,`next_back()`), against original `tuple3.rs`/`tuple4.rs` source | Same shared-`index` mechanism: N=3 re-yields `tuple.0` while `tuple.2` is never reached; N=4 re-yields both `tuple.0` and `tuple.2` while `tuple.1`/`tuple.3` are never reached — byte-identical authoring pattern copy-pasted across all three arities | H1 ✅ |

## Root Cause

```
Tuple2IterMut::next()      when index == 0 -> yields tuple.0, index := 1   (original, unfixed)
Tuple2IterMut::next_back() when index == 1 -> yields tuple.0, index := 2   (original, unfixed)
                                                       ^^^^^^^ same field, already live as `a`
```

`next()` and `next_back()` each treat `self.index` as "how many elements have been consumed
overall," then independently map that count to a concrete tuple field — but each method's
mapping silently assumes *every* prior call went through *its own* direction only. The two
mappings were authored independently (one arm-set per direction, `tuple2.rs:171-201` vs.
`tuple2.rs:207-231` in the original source) and never cross-checked against each other. The
moment a caller interleaves the two methods, `next_back()` receives a count that already
includes a `next()` call, but re-derives "which field is still available from the back" using
only the raw count — landing back on a field `next()` already returned a live `&mut` reference
to. This confirms **H1 (✅ Root Cause)** over the disproved H2/H3: the shape is not a deliberate
tradeoff (H2 disproved by the `qqq` comment's own wording) and is not equivalent to std's sound
two-cursor design (H3 disproved — std's `IterMut` never lets one shared value stand in for both
directions' bookkeeping). `Tuple3IterMut` and `Tuple4IterMut` (`tuple3.rs`, `tuple4.rs`) carry
the identical pattern, scaled to 3 and 4 arms respectively — same root cause, three files, one
fix shape (see `## Fix Applied`).

## Why Not Caught

Every pre-existing `vector_iter_mut` test in this crate (`tuple{0,1,2,3,4}_test.rs`,
`array_test.rs`, `slice_test.rs`) exercised exactly one of two shapes: repeated `.next()` calls
only, or `.rev()` applied once up front followed by repeated `.next()` calls only (i.e. fully
reversed, still single-direction from the iterator's own perspective — `.rev()` swaps which
method `.next()` delegates to, it does not itself interleave `next()`/`next_back()`). In both
shapes, every call funnels through exactly one of the two match-arm sets, so that one set's
self-consistent (if direction-siloed) counting never collides with the other's. No existing
test — nor any production call site (see `## Impact`'s magnitude search) — ever called
`.next()` and `.next_back()` on the same live, non-`.rev()`-wrapped iterator, which is the
precise and only trigger condition for the double-yield. The `qqq` comments correctly flagged
uncertainty at authoring time, but no test was ever added to resolve that uncertainty one way
or the other until this investigation.

## Fix Location

Three files, identical fix shape, each replacing the shared `index : usize` field with
independent `front`/`back` cursors in the type's `Tuple{2,3,4}IterMut` struct, `Iterator::next`,
`DoubleEndedIterator::next_back`, and the `VectorIterMut::vector_iter_mut()` constructor:

- `module/math/mdmath_core/src/vector/tuple2.rs:161-278` (struct at 170, `next()`/`next_back()`,
  constructor at 268-278)
- `module/math/mdmath_core/src/vector/tuple3.rs:163-287` (struct at 172, constructor at 277-287)
- `module/math/mdmath_core/src/vector/tuple4.rs:173-315` (struct at 182, constructor at 304-315)

```rust
// Before (tuple2.rs, representative — tuple3.rs/tuple4.rs follow the identical shape):
struct Tuple2IterMut< 'tuple_ref, E >
{
  tuple : &'tuple_ref mut ( E, E ),
  index : usize,
}
// next(): match self.index { 0 => { index += 1; tuple.0 }, 1 => { index += 1; tuple.1 }, _ => None }
// next_back(): match self.index { 0 => { index += 1; tuple.1 }, 1 => { index += 1; tuple.0 }, _ => None }
// vector_iter_mut(): Tuple2IterMut { tuple: self, index: 0 }

// After:
struct Tuple2IterMut< 'tuple_ref, E >
{
  tuple : &'tuple_ref mut ( E, E ),
  front : usize,
  back : usize,
}
// next(): if front >= back { None } else { let i = front; front += 1; yield element i }
// next_back(): if front >= back { None } else { back -= 1; yield element back }
// vector_iter_mut(): Tuple2IterMut { tuple: self, front: 0, back: 2 }
```

## Fix Applied

Applied exactly as documented above to all three files, mirroring `core::slice::IterMut`'s own
two-cursor design: `front`/`back` converge monotonically (`front` only increases, `back` only
decreases) and are guarded by `front >= back => None` in both `next()` and `next_back()`, so
each tuple field is reborrowed at most once across the whole iteration regardless of call
interleaving. Each match arm's `unsafe` block carries an explicit SAFETY comment stating this
invariant; the stale `qqq` doubt-comments are removed and replaced with the standard 3-field
form (`Fix(BUG-050)` / `Root cause` / `Pitfall`) directly above each `struct Tuple{2,3,4}IterMut`
definition (`tuple2.rs:161-169`, `tuple3.rs:163-171`, `tuple4.rs:173-181`).

New reproducer tests — `test_vector_iter_mut_mixed_direction_no_aliasing_tuple2`
(`tuple2_test.rs`, `.next()` then `.next_back()`), `test_vector_iter_mut_next_and_next_back_disjoint_tuple3`
(`tuple3_test.rs`, `.next()`,`.next()`,`.next_back()`), and
`test_vector_iter_mut_next_and_next_back_disjoint_tuple4` (`tuple4_test.rs`, alternating
`.next()`/`.next_back()`×2), each `// test_kind: bug_reproducer(BUG-050)` — confirmed failing
before the fix (`cargo nextest run -p mdmath_core --all-features -E 'test(mixed_direction_no_aliasing) + test(next_and_next_back_disjoint)'`
→ `3 tests run: 0 passed, 3 failed`) and passing after (`3 tests run: 3 passed`). Miri
additionally confirms real Stacked Borrows UB against the original source and zero UB against
the fixed source (`## Symptom`). Full crate suite: `cargo nextest run -p mdmath_core --all-features`
→ `76 tests run: 76 passed, 0 skipped`. `cargo clippy -p mdmath_core --all-targets --all-features -- -D warnings`
(this crate denies `undocumented_unsafe_blocks`) → clean, zero warnings.

## Prevention

Add at least one mixed-direction `.next()`/`.next_back()` reproducer for every hand-rolled
`DoubleEndedIterator` yielding `&mut` references in this crate, and treat it as a required test
shape (alongside pure-forward and `.rev()`-then-forward) for any future one. Detection:

```bash
cargo nextest run -p mdmath_core --all-features -E 'test(next_and_next_back) + test(mixed_direction)'
```

should exist and pass for every hand-rolled `&mut`-yielding `DoubleEndedIterator` in the crate.
Additionally, since this class of bug can be invisible to ordinary safe-Rust assertions in some
call-sequence/arity combinations (the "wrong value" symptom depends on write-ordering, while the
underlying aliasing is unconditional), periodic `cargo +nightly miri test -p mdmath_core` runs
are the only way to *prove* absence of UB rather than merely fail to observe a wrong value.

**Pitfall:** A `DoubleEndedIterator` whose `next()` and `next_back()` are authored as two
independent per-direction match-arm sets sharing one counter field is unsound the instant both
methods are actually exercised on the same instance — the shared field's "how many consumed"
meaning is direction-agnostic, but each method's arm independently (and silently) assumes every
prior call came from its own direction. Always use two independent cursors (front/back) that
only ever converge, mirroring `core::slice::IterMut`, and always test the mixed-direction case.

## Generalized Version

**Broken assumption:** "A single shared position/index field can serve both `next()` and
`next_back()` of a hand-rolled `DoubleEndedIterator`, as long as each method's match arms are
written correctly for its own direction." False whenever the iterator yields unique (`&mut`)
references and the two methods' arm-sets were authored independently against the shared field.

Fails for any hand-rolled `DoubleEndedIterator<Item = &mut T>` where:
1. `next()`/`next_back()` are both implemented via one shared position/index field, AND
2. each method's match arms independently hardcode which field/slot that field's current value
   maps to for their own direction, without cross-referencing the other method's mapping, AND
3. no test exercises a genuine mix of both methods on one live (non-`.rev()`-wrapped) iterator
   instance.

**Detection invariant:**
```
for every hand-rolled DoubleEndedIterator<Item = &mut T> of length N,
for every valid split k + m <= N,
calling .next() k times then .next_back() m times yields k+m pairwise-distinct
memory addresses, matching what an independent front/back-cursor traversal would produce
```

## History

| Date       | Event  | Notes                                                                                                     |
|------------|--------|-------------------------------------------------------------------------------------------------------------|
| 2026-08-10 | filed  | Carried forward from `task/draft/009_mdmath_core_itermut_aliasing_ub.md` (workspace-wide P1 soundness triage plan); citation re-derived from scratch per that task's own instruction. Root cause confirmed via direct source read, the pre-existing `qqq` doubt-comments, manual trace for N=2/3/4, and a Miri Stacked Borrows run before filing. |
| 2026-08-10 | confirmed | VERIFY_PASS — Tier 2 Dual-Role Self-Check, 8/8 dimensions PASS (see `## Verification Record`); reproducer tests confirmed RED pre-fix (`cargo nextest`: 3/3 failed) and GREEN post-fix (3/3 passed); Miri confirms real UB pre-fix and zero UB post-fix. |
| 2026-08-10 | completed | Fix applied to all 3 files (`tuple2.rs`, `tuple3.rs`, `tuple4.rs`): shared `index` replaced with independent `front`/`back` cursors; 3-field `Fix(BUG-050)` comments added; stale `qqq` doubt-comments removed. Full `mdmath_core` suite: 76/76 passed. `cargo clippy -p mdmath_core --all-targets --all-features -- -D warnings`: clean. Same-session, self-administered (filer = fixer = verifier) — Tier 2 Dual-Role Self-Check per `governance/maav.rulebook.md`'s default, not an independent PROC16-style acceptance pass. State → Completed. |

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 8/8

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Completeness | 🟢 | 🟢 | — | — |
| D2 | MRE Validity & Reproducibility | 🟢 | 🟢 | — | — |
| D3 | Cross-Reference Integrity | 🟢 | 🟢 | — | — |
| D4 | Root Cause Quality | 🟢 | 🟢 | — | — |
| D5 | Execution Scope | 🟢 | 🟢 | — | — |
| D6 | Crate Scope Unity | 🟢 | 🟢 | — | — |
| D7 | Crate Locality | 🟢 | 🟢 | — | — |
| D8 | Crate Single Responsibility | 🟢 | 🟢 | — | — |
| **Total** | | 🟢 | 🟢 | 0 | 0/0 |

**Reproduced:** YES — exit 101, 2026-08-10 (`/tmp/mre050/repro`, verbatim output captured and
matched into `## Minimum Reproducible Example`); real crate UB additionally confirmed via Miri
(`## Symptom`).

**Aggregate verdict:** PASS — self-administered, no subagent dispatch (Verification Delegation
would be forbidden per `file.rulebook.md § Report New Bug : Step 9 - VERIFY Gate`).

## Refs: src/

- `module/math/mdmath_core/src/vector/tuple2.rs` — replaced `index : usize` with `front`/`back`
  cursors in `Tuple2IterMut` (struct: line 170; constructor `vector_iter_mut()`: lines 268-278);
  `Fix(BUG-050)` comment at lines 161-169
- `module/math/mdmath_core/src/vector/tuple3.rs` — same fix shape in `Tuple3IterMut` (struct:
  line 172; constructor: lines 277-287); `Fix(BUG-050)` comment at lines 163-171
- `module/math/mdmath_core/src/vector/tuple4.rs` — same fix shape in `Tuple4IterMut` (struct:
  line 182; constructor: lines 304-315); `Fix(BUG-050)` comment at lines 173-181

## Refs: tests/

- `module/math/mdmath_core/tests/inc/vector_test/tuple2_test.rs` — new reproducer
  `test_vector_iter_mut_mixed_direction_no_aliasing_tuple2` (`// test_kind: bug_reproducer(BUG-050)`,
  lines 94-130)
- `module/math/mdmath_core/tests/inc/vector_test/tuple3_test.rs` — new reproducer
  `test_vector_iter_mut_next_and_next_back_disjoint_tuple3` (lines 105-145)
- `module/math/mdmath_core/tests/inc/vector_test/tuple4_test.rs` — new reproducer
  `test_vector_iter_mut_next_and_next_back_disjoint_tuple4` (lines 116-158)
