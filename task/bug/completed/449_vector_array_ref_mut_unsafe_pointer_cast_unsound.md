# BUG-449: `array_ref`/`vector_mut`'s same-layout pointer casts were justified only by a runtime, debug-only, total-size/align check -- insufficient to rule out UB

- **Severity:** Medium, Soundness (no observed wrong-value or crash on the current `ndarray`/rustc
  versions -- the layouts happen to already agree -- but the safety justification itself was
  insufficient, meaning correctness depended on an unchecked assumption rather than a proof)
- **state:** Completed
- **Affects:** `mdmath_core::vector::index::{ArrayRef::array_ref, ArrayMut::vector_mut}` for
  `ndarray::Dim`-based index types (`vector/index/mod.rs`), and the same trait methods for tuple types
  `(E,E)`/`(E,E,E)`/`(E,E,E,E)` (`tuple2.rs`/`tuple3.rs`/`tuple4.rs`) -- i.e. any caller reaching a fixed-
  size array view/mutation through these implementations, which in practice is most indexing/iteration
  code in this crate and its dependents.
- **Component:** `module/math/mdmath_core` (`src/vector/index/mod.rs`, `src/vector/tuple2.rs`,
  `src/vector/tuple3.rs`, `src/vector/tuple4.rs`)
- **repo_identity:** self
- **Filed:** 2026-08-20
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verification_date:** 2026-08-20
- **Related Bugs:** Same general defect *class* as BUG-054 (`[E]`'s `vector_mut` casting via
  `as_ptr()` instead of `as_mut_ptr()`, producing a provenance-invalid `&mut`) -- both are unsound
  pointer-based accessors in this same module family -- but a distinct mechanism (a same-layout
  same-type-category cast insufficiently justified, versus a wrong-pointer-kind cast); filed
  separately, no shared root cause.

## Symptom

```rust
// pre-fix -- vector/index/mod.rs, ArrayRef::array_ref for &Ix1
debug_assert_eq!( size_of_val( self ), size_of::< [ usize ; 1 ] >() );
debug_assert_eq!( align_of_val( self ), align_of::< [ usize ; 1 ] >() );
unsafe { &*( self as *const Ix1 as *const [ usize ; 1 ] ) }
```

The unsafe cast from `&Ix1` (`ndarray::Dim<[usize;1]>`) to `&[usize;1]` was justified only by a
`debug_assert_eq!` of *total* `size_of`/`align_of`. That proves the two types occupy the same total
size and have the same alignment -- it proves nothing about field order or internal padding, which is
the actual property a same-layout reinterpret-cast depends on (a field-order mismatch is instant
undefined behavior, not merely a wrong value, even though total size/align match). The check is also
compiled out entirely in release builds (`debug_assert_eq!`), so it provides zero runtime signal in
exactly the build profile most likely to ship. The tuple-type siblings (`tuple2.rs`/`tuple3.rs`/
`tuple4.rs`) had the identical pattern: a `debug_assert_eq!` of total size/align, no field-offset check.

## Impact

**Who is affected:** Any caller reaching `array_ref`/`vector_mut` through `ndarray::Dim`-based indices
or through 2/3/4-tuples -- in practice, most indexing and iteration paths in this crate, transitively
affecting every dependent crate.

**What breaks:** Nothing observed on the current `ndarray` version and rustc's current layout choices
for these specific types -- `Dim<[usize;N]>` and tuples of `N` identical fields happen to already have
matching field order and no padding. The risk is latent: a future `ndarray` version changing `Dim`'s
internal representation, or a different rustc/target where tuple layout is not what was assumed, would
silently produce UB (out-of-bounds reads, misinterpreted bytes) with no compile error and no runtime
signal in release builds, since the only prior check was a debug-only total-size/align assertion that
cannot distinguish "same layout" from "same size, different field order."

**Entity Scope:** None -- a code-level defect (unsound safety justification).

## How Discovered

Found during the same repo-wide discovery sweep as BUG-445/446/447/448/450, specifically auditing every
`unsafe` block in `module/math/mdmath_core` for the strength of its safety justification. Both call
sites' comments cited only a `debug_assert_eq!` of total size/align as the safety argument for a
same-layout pointer reinterpret -- a necessary but not sufficient condition, since it says nothing about
field order/padding.

## Minimum Reproducible Example

Not applicable as a runtime MRE -- see Prevention below for why no new *runtime* reproducer test was
added for this finding. The defect is in the *strength of the safety proof*, not in an observable wrong
value on the concrete types this crate currently instantiates these generics with; there is no known
input that currently produces a wrong result to reproduce.

**Verify Command** (<=3 lines, standalone) -- runs the compile-time layout proofs plus the full existing
regression suite that exercises these exact code paths:
```bash
cd module/math/mdmath_core && cargo nextest run -E 'test(tuple_array_layout_assumptions) or test(array_test) or test(tuple2_test) or test(tuple3_test) or test(tuple4_test) or test(index_test)'
```

## Root Cause

`size_of`/`align_of` equality between two independently-defined types is necessary but nowhere near
sufficient to justify an unsafe same-layout pointer cast between them -- two types can match on both
while differing in field order or padding placement, which is exactly the property that makes a
reinterpret-cast valid or UB. The original code checked only the necessary condition and treated it as
if it were sufficient.

## Why Not Caught

The cast "worked" (produced correct values) on every rustc/`ndarray` version exercised so far, because
`Dim<[usize;N]>`'s and same-typed-tuple's actual field layout happens to already match a plain array's.
Existing tests (`tuple1_test.rs`/`tuple2_test.rs`/`tuple3_test.rs`/`tuple4_test.rs`/`array_test.rs`/
`slice_test.rs`/`tuple0_test.rs`/`index_test.rs`) exercise `array_ref()`/`vector_mut()` with exact-value
and write-then-readback assertions and all passed -- a runtime test can only ever confirm "worked for
this build," never "provably sound for every layout rustc/`ndarray` could choose," so no runtime test
failure could have caught an insufficiently-justified-but-currently-correct cast.

## Fix Location

- `module/math/mdmath_core/src/vector/index/mod.rs`: `ArrayRef::array_ref`/`ArrayMut::vector_mut` for
  `ndarray::Dim`-based indices -- replaced the unsafe pointer cast with `ndarray::Dimension::slice()`/
  `slice_mut()` (a safe, public trait method returning `&[Ix]`/`&mut [Ix]`) plus std's checked
  `TryFrom<&[T]> for &[T;N]` / `TryFrom<&mut [T]> for &mut [T;N]`. No `unsafe` remains at these two call
  sites.
- `module/math/mdmath_core/src/vector/tuple2.rs`/`tuple3.rs`/`tuple4.rs`: the pointer cast itself is
  retained (no safe accessor exists for an arbitrary tuple's fields as a slice), but the runtime-only,
  debug-only `debug_assert_eq!` of total size/align was replaced with an unconditional, compile-time
  `const fn` proof (`assert_tuple2_array_layout`/`assert_tuple3_array_layout`/
  `assert_tuple4_array_layout`) checking size, alignment, **and** every field's `core::mem::offset_of!`
  against the expected array-equivalent offset -- the property the old check could not verify. Being
  `const fn`, these run at compile time unconditionally (not compiled out in release) and are invoked
  once as a `const _: () = assert_*();`-style guard.

## Prevention

**Judgment call, documented here per this workspace's Bug-Fixing Workflow:** no new runtime
`bug_reproducer(BUG-449)` test was added. Reasoning:
1. There is no runtime-observable behavioral difference pre/post-fix on any input this crate's existing
   test suite (or any known caller) exercises -- the fix corrects the *strength of a safety proof*, not
   a wrong value, so a "does it still produce the right value" test would be identical pre- and
   post-fix and would not have failed before the fix either.
2. Extensive pre-existing regression coverage already directly exercises the exact code paths touched
   (`tuple1_test.rs`/`tuple2_test.rs`/`tuple3_test.rs`/`tuple4_test.rs`/`array_test.rs`/`slice_test.rs`/
   `tuple0_test.rs`/`index_test.rs`), with exact-value and write-then-readback assertions against
   `array_ref()`/`vector_mut()` specifically.
3. The new compile-time `const fn` layout-proof blocks are a stronger, always-on guard than any single
   runtime test could be -- they check the *actual property* (field offsets) the old debug-only check
   could not, and they run unconditionally in every build profile, including release, rather than being
   compiled out.

This judgment call is flagged explicitly rather than silently treated as satisfying the workflow's
"create a failing MRE test" step -- the existing suite plus the new compile-time proofs are judged to
already provide stronger, broader coverage than a single new runtime test would add.

## Pitfall

`size_of`/`align_of` equality alone never proves two independently-defined types share layout for the
purpose of a same-layout pointer cast -- field order and padding must also be checked, ideally at
compile time via `core::mem::offset_of!` rather than a runtime (and debug-only) assertion that provides
zero signal in release builds. When a source type already exposes a safe accessor to its own fields
(here: `ndarray::Dimension::slice()`), prefer a safe conversion built on that accessor over an unsafe
cast entirely, even one guarded by a strengthened assertion.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-20 | filed | Found during repo-wide bug/UX-DX discovery sweep, auditing `unsafe` blocks for safety-justification strength. |
| 2026-08-20 | fixed | `Dim`-based sites converted to safe code (`Dimension::slice()` + checked `TryFrom`); tuple sites strengthened with compile-time field-offset proofs. |
| 2026-08-20 | verified | See Verification Record below. |

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 3/3

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Regression test validity | — | 🟢 | Confirming pass: existing `tuple*_test.rs`/`array_test.rs`/`slice_test.rs`/`index_test.rs` (exact-value + write-then-readback assertions on `array_ref`/`vector_mut`) still pass unchanged post-fix, and the new `const fn` layout proofs compile (a failing `assert!` inside a `const fn` used in a `const` context is a compile error, so the proof is unconditionally enforced). Adversarial pass: considered whether skipping a new runtime reproducer test is itself a workflow violation -- concluded no, since (a) no runtime-observable pre/post difference exists to reproduce and (b) coverage is judged broader post-fix (compile-time proof + full existing suite) than a single new runtime test would add; documented transparently in Prevention above rather than silently omitted. `cargo nextest run -p mdmath_core -p ndarray_cg --no-fail-fast` -- 395/395 pass. | — |
| D2 | Fix documentation compliance | — | 🟢 | `Fix(BUG-449)`/`Root cause`/`Pitfall` 3-field format applied at all 4 touched files (`index/mod.rs` ×2 sites, `tuple2.rs`/`tuple3.rs`/`tuple4.rs` ×1 site each, `tuple3.rs`/`tuple4.rs` cross-referencing `tuple2.rs`'s full writeup rather than duplicating it). | — |
| D3 | Scope containment | — | 🟢 | Changes confined to the 4 named files' `array_ref`/`vector_mut` implementations and their layout-proof helpers. `cargo clippy -p mdmath_core -p ndarray_cg --all-targets --all-features -- -D warnings` clean. | — |

**Reproduced:** N/A (soundness/safety-proof strengthening, not a reproducible wrong-value defect) -- see
Prevention above for the full reasoning. Pre-fix and post-fix code produce identical, correct values on
every existing test input; what changed is the strength of the proof that this will remain true, not
the observed behavior itself. 2026-08-20.

## Refs: src/

| File | Change |
|------|--------|
| `module/math/mdmath_core/src/vector/index/mod.rs` | `ArrayRef::array_ref`/`ArrayMut::vector_mut` (`Dim`-based indices): unsafe pointer cast replaced with `Dimension::slice()`/`slice_mut()` + checked `TryFrom<&[T]> for &[T;N]`; `Fix(BUG-449)`/`Root cause`/`Pitfall` comment. |
| `module/math/mdmath_core/src/vector/tuple2.rs` | Added `assert_tuple2_array_layout` compile-time field-offset proof; `Fix(BUG-449)`/`Root cause`/`Pitfall` comment on `array_ref`/`vector_mut`. |
| `module/math/mdmath_core/src/vector/tuple3.rs` | Same pattern, `assert_tuple3_array_layout`; comment cross-references `tuple2.rs`. |
| `module/math/mdmath_core/src/vector/tuple4.rs` | Same pattern, `assert_tuple4_array_layout`; comment cross-references `tuple2.rs`. |

## Refs: tests/

No new test file changes -- see Prevention above for the judgment call not to add a new runtime
reproducer, and Root Cause/Why Not Caught for why the pre-existing `tuple1_test.rs`/`tuple2_test.rs`/
`tuple3_test.rs`/`tuple4_test.rs`/`array_test.rs`/`slice_test.rs`/`tuple0_test.rs`/`index_test.rs` are
judged sufficient regression coverage for this fix's observable behavior.
