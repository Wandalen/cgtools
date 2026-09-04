# BUG-054: `[E]`'s `ArrayMut::vector_mut` casts via `as_ptr()` instead of `as_mut_ptr()`, producing a `&mut` reference with `SharedReadOnly` provenance

- **Severity:** High
- **state:** Completed
- **Affects:** Any caller of `<[E] as mdmath_core::ArrayMut<E, N>>::vector_mut` — currently only the crate's own `test_vector_mut_slice` (no production call sites found workspace-wide; see `## Impact`)
- **Component:** `module/math/mdmath_core` — `vector::slice::<impl ArrayMut<E, N> for [E]>::vector_mut`
- **repo_identity:** self
- **Filed:** 2026-08-10
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verification_date:** 2026-08-10
- **Fixed:** 2026-08-10
- **Accepted By:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/ (self — same-session Tier 2 Dual-Role Self-Check, no separate PROC16 acceptance actor)

## Symptom

```
$ cargo +nightly miri test -p mdmath_core --all-features
...
test inc::vector_test::slice_test::test_vector_mut_slice ... error: Undefined Behavior: trying to retag from <852419> for Unique permission at alloc257739[0x0], but that tag only grants SharedReadOnly permission for this location
 --> module/math/mdmath_core/src/vector/slice.rs:55:13
  |
  = note: this error occurs as part of retag at alloc257739[0x0..0x4]
  = help: this indicates a potential bug in the program: it performed an invalid operation, but the Stacked Borrows rules it violated are still experimental
help: <852419> was created by a SharedReadOnly retag at offsets [0x0..0x4]
 --> module/math/mdmath_core/src/vector/slice.rs:55:21
  = note: stack backtrace:
          0: mdmath_core::vector::slice::<impl mdmath_core::ArrayMut<i32, 1> for [i32]>::vector_mut
              at module/math/mdmath_core/src/vector/slice.rs:55:14: 55:55
          1: inc::vector_test::slice_test::test_vector_mut_slice
              at module/math/mdmath_core/tests/inc/vector_test/slice_test.rs:32:40: 32:58
error: aborting due to 1 previous error
```

Genuine Stacked Borrows Undefined Behavior, not a wrong-value bug — the process aborts under
Miri the moment the returned `&mut [E; N]` is written through.

## Impact

**Who is affected:** Any caller of `<[E] as ArrayMut<E, N>>::vector_mut` — a `pub` trait method,
part of `mdmath_core`'s public API. A workspace-wide grep for `.vector_mut(` found call sites
only inside `mdmath_core`'s own test suite (`tests/inc/vector_test/{slice,array,tuple0..4}_test.rs`);
the trait itself is also implemented for `[E; N]` (`array.rs`, safe — returns `self` directly, no
pointer cast) and `ndarray_cg::Vector<E, N>` (`general.rs`, safe — returns `&mut self.0` directly)
and consumed generically nowhere outside `mdmath_core`/`ndarray_cg` today (confirmed via
`grep -rln "ArrayMut" module/ examples/`).

**What breaks:** Undefined Behavior — under Miri, an immediate process abort on the write. In a
normal (non-Miri) build, this specific access pattern happens to not miscompile with current
rustc/LLVM (confirmed: the pre-fix native `cargo nextest run -p mdmath_core --all-features`
passes cleanly), but Stacked Borrows violations are exactly the class of UB the optimizer is free
to exploit in a future compiler version or under more aggressive optimization — "works today" is
not a safety guarantee once `unsafe` code violates the aliasing model.

**Magnitude — dormant but real, same pattern as BUG-052:** zero non-test call sites exist today,
so nothing observably misbehaves in this workspace right now. Severity is High rather than
Critical for the same reason as BUG-050 (Miri-confirmed real UB, no live exploit path yet) —
distinguishing it from a Critical live-crash bug while still ranking above an ordinary Medium
defect, since any future caller of this public API inherits the UB immediately and silently.

**Entity Scope:** `None` — an ordinary source file, not an entity directory instance.

## How Discovered

Found while independently re-verifying BUG-050's own fix (mdmath_core `Tuple{2,3,4}IterMut`
aliasing) via `cargo +nightly miri test -p mdmath_core --all-features`, run directly (not
delegated) as part of this session's re-verification pass. BUG-050 only touched
`tuple2.rs`/`tuple3.rs`/`tuple4.rs`; this Miri run swept the crate's **entire** test suite (not
scoped to BUG-050's own new tests) and surfaced a second, unrelated UB site in `slice.rs` — a
file BUG-050 never touched. Confirmed pre-existing and unrelated to BUG-050's fix: `slice.rs`'s
`vector_mut` uses a raw-pointer-cast pattern entirely distinct from the cursor-aliasing pattern
BUG-050 fixed, and `git log` shows no recent changes to `slice.rs`.

## Minimum Reproducible Example

The real, existing test (`test_vector_mut_slice`) already reproduces this directly — no synthetic
substitute needed, since `mdmath_core` has no heavy external dependency chain (unlike BUG-050/051's
web-sys-gated equivalents) blocking a minimal Miri-checked repro:

```bash
mkdir -p /tmp/mre054 && cd /tmp/mre054
cat > Cargo.toml <<'EOF'
[package]
name = "mre054"
version = "0.1.0"
edition = "2021"
EOF
mkdir -p src
cat > src/main.rs <<'EOF'
// Mirrors module/math/mdmath_core/src/vector/slice.rs's pre-fix vector_mut:
// casts self.as_ptr() (SharedReadOnly provenance) to *mut instead of
// self.as_mut_ptr() (Unique provenance) before writing through it.
fn vector_mut_buggy( slice : &mut [ i32 ] ) -> &mut [ i32; 1 ]
{
  unsafe { &mut *( slice.as_ptr() as *mut [ i32; 1 ] ) }
}

fn main()
{
  let mut data = [ 42_i32 ];
  let slice : &mut [ i32 ] = &mut data;
  let arr = vector_mut_buggy( slice );
  arr[ 0 ] = 100;
  println!( "{}", data[ 0 ] );
}
EOF
cargo +nightly miri run 2>&1
echo "exit: $?"
```

**Expected** (once fixed — i.e. `as_ptr()` replaced with `as_mut_ptr()`):
```
100
exit: 0
```

**Actual:**
```
error: Undefined Behavior: trying to retag from <TAG> for Unique permission at ALLOC[0x0], but that tag only grants SharedReadOnly permission for this location
 --> src/main.rs:6:12
  |
6 |   unsafe { &mut *( slice.as_ptr() as *mut [ i32; 1 ] ) }
  |            ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ Undefined Behavior occurred here
error: aborting due to 1 previous error
exit: 1
```

**Verify Command:** `cd /tmp/mre054 && cargo +nightly miri run; test $? -eq 1` — **What:**
demonstrates that casting a `&mut [E]`'s `as_ptr()` (not `as_mut_ptr()`) result to `*mut [E; N]`
and dereferencing it mutably is Stacked-Borrows UB, reproducing the exact defect at
`module/math/mdmath_core/src/vector/slice.rs` (pre-fix line 55).

## Hypothesis Table

| ID | Hypothesis | State | Summary | Evidence |
|----|-----------|--------|---------|----------|
| H1 | `vector_mut`'s `unsafe` block casts `self.as_ptr()` (immutable/`SharedReadOnly` provenance) to `*mut [E; N]` and dereferences it as `&mut`, which Stacked Borrows forbids regardless of the outer `&mut self` — the pointer itself was never derived from a mutable borrow | ✅ Root Cause | Miri's own message: "that tag only grants SharedReadOnly permission" — `<[T]>::as_ptr(&self)` takes `&self`, tagging `SharedReadOnly`, before the `*mut` cast | E1, E2, E3 |
| H2 | This is a Miri false positive — Stacked Borrows is "still experimental" per Miri's own hint text, so the flagged access might be sound in practice | ❌ Disproved | The sibling `array_ref` (immutable, line 26) correctly uses `self.as_ptr()`; every OTHER `vector_mut`/`array_ref` implementation in the crate (`array.rs`, `ndarray_cg::Vector`, tuple0-4 via `transmute`) either avoids raw pointers entirely or starts from a genuinely mutable-provenance source — `slice.rs` is the sole outlier, and the fix (`as_mut_ptr()`) is the exact idiom every other implementation already gets right by construction | E4, E5 |
| H3 | The bug is specific to this test's inputs (e.g. zero-length slice) rather than the general pattern | ❌ Disproved | Miri flags the 1-element case (`vector_mut::<i32,1>`) specifically at the `unsafe` cast site itself, not at any bounds-dependent operation; the standalone MRE reproduces with a single-element array with no dependency on `mdmath_core` at all | MRE |

## Evidence Table

| # | Location | What it shows | Hypothesis |
|---|----------|---------------|------------|
| E1 | `module/math/mdmath_core/src/vector/slice.rs` (pre-fix line 55) | `unsafe { &mut *( self.as_ptr() as *mut [ E ; N ] ) }` — casts an immutable pointer to `*mut`, then dereferences mutably | H1 ✅ |
| E2 | Miri output (`## Symptom`) | "trying to retag from <852419> for Unique permission ... but that tag only grants SharedReadOnly permission" — Miri's own diagnosis names the exact provenance mismatch | H1 ✅ |
| E3 | Rust stdlib docs, `<[T]>::as_ptr` vs `<[T]>::as_mut_ptr` | `as_ptr(&self) -> *const T` vs `as_mut_ptr(&mut self) -> *mut T` — two distinct methods exist precisely so a `*mut` pointer starts with `Unique` provenance rather than being cast up from a `*const` one | H1 ✅ |
| E4 | `module/math/mdmath_core/src/vector/slice.rs` (line 26, `array_ref`) | `unsafe { &*( self.as_ptr() as *const [ E ; N ] ) }` — the immutable sibling, correctly using `as_ptr()` for a `*const` result; `vector_mut` appears to have copied this pattern without switching to `as_mut_ptr()` for its `*mut` result | H1 ✅, H2 ❌ |
| E5 | `module/math/mdmath_core/src/vector/array.rs` (lines 34-37), `module/math/ndarray_cg/src/vector/general.rs` (lines 117-120) | Both sibling `ArrayMut` implementations return a direct field reference (`self` / `&mut self.0`) — no raw pointer involved, so neither could exhibit this bug class at all | H2 ❌ |

## Root Cause

```
vector_mut( &mut self ) -> &mut [ E ; N ]
  unsafe { &mut *( self.as_ptr() as *mut [ E ; N ] ) }
                    ^^^^^^^^^^^^ *const E — SharedReadOnly provenance
           ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ retagged Unique here — UB
```

`self.as_ptr()` (`<[T]>::as_ptr(&self) -> *const T`) reborrows `self` as shared before producing
a pointer, tagging that pointer's provenance `SharedReadOnly` under Stacked Borrows. Casting that
`*const` pointer to `*mut [E; N]` and dereferencing it as `&mut` forces a retag to `Unique`
permission — which Stacked Borrows forbids from a `SharedReadOnly`-only tag, regardless of the
fact that the *outer* `self` parameter is `&mut [E]`. The immutable sibling `array_ref` (line 26)
uses the identical `self.as_ptr() as *const [E; N]` pattern correctly, since it only ever needs a
`*const`/shared result — `vector_mut` was authored by copying that pattern and changing the
target type from `*const` to `*mut` without also switching the source accessor from `as_ptr()` to
`as_mut_ptr()` (`<[T]>::as_mut_ptr(&mut self) -> *mut T`, which reborrows `self` as mutable and
tags the resulting pointer `Unique` from the start — the only route to a sound `&mut` retag).

## Why Not Caught

`vector_mut`'s own test (`test_vector_mut_slice`) exercises exactly the write-through-the-result
pattern that triggers this UB, and has since the test was written — but nothing in this
workspace's standard verification commands (`cargo nextest run`, `cargo clippy`, `cargo test`)
runs under Miri; all of them compile this code with the ordinary rustc backend, which does not
enforce Stacked Borrows and happens not to miscompile this particular access today. Miri
(`cargo +nightly miri test`) is the only tool in this toolchain capable of detecting the
violation, and — prior to this session's independent re-verification pass — had apparently never
been run against `mdmath_core`'s full pre-existing test suite (BUG-050's own Miri run was scoped
to confirming its own new tests, not a full-crate sweep, so it never exercised `slice_test.rs`).

## Fix Location

`module/math/mdmath_core/src/vector/slice.rs`, `<impl ArrayMut<E, N> for [E]>::vector_mut`
(pre-fix lines 47-57):

```rust
// Before:
unsafe { &mut *( self.as_ptr() as *mut [ E ; N ] ) }

// After:
unsafe { &mut *( self.as_mut_ptr() as *mut [ E ; N ] ) }
```

Single-token fix: `as_ptr()` → `as_mut_ptr()`. `as_mut_ptr(&mut self) -> *mut E` reborrows `self`
as mutable and tags the resulting pointer `Unique` from the start, so the subsequent cast to
`*mut [E; N]` and mutable dereference no longer requires an invalid `SharedReadOnly` → `Unique`
retag.

## Fix Applied

Applied exactly as documented above (`slice.rs`), with the fix-time comment in the standard
3-field form (`Fix(BUG-054)` / root cause / pitfall, `slice.rs:53-58`). The pre-existing
`test_vector_mut_slice` (`tests/inc/vector_test/slice_test.rs`) already exercised the exact
sequence that triggers this UB — rather than duplicate that coverage with a new test, it was
annotated `// test_kind: bug_reproducer(BUG-054)` plus a doc comment naming the mechanism, since
adding a second test performing the identical operations would violate this project's
no-duplication rule. RED→GREEN, run for real via `longrun`:

- **Red (pre-fix):** `cargo +nightly miri test -p mdmath_core --all-features` — Miri aborts:
  `error: Undefined Behavior: trying to retag from <852419> for Unique permission ... but that
  tag only grants SharedReadOnly permission` at `slice.rs:55:13`, `test_vector_mut_slice ...`
  reported as the failing test; process exit 1.
- **Green (post-fix):** same command — `76 passed; 0 failed` (unit tests, includes
  `test_vector_mut_slice ... ok`), `3 passed; 0 failed; 4 ignored` (doc tests); exit 0.
- **Native regression (`RUSTFLAGS="-D warnings" cargo nextest run -p mdmath_core
  --all-features`):** `76 tests run: 76 passed, 0 skipped` — same count as the Miri run, no
  regression from the ordinary (non-Miri) backend's perspective, confirming this was a pure
  aliasing-provenance defect with no behavioral change to fix.
- **`cargo clippy -p mdmath_core --all-targets --all-features -- -D warnings`:** clean, 0
  warnings (after also correcting a `clippy::doc_markdown` hit on this fix's own new doc comment
  — `SharedReadOnly`/`Unique` needed backticks).

## Prevention

Any `unsafe` block that casts a pointer obtained from `&self`/`as_ptr()` to a `*mut` type and
dereferences it mutably is unsound regardless of whether the *enclosing* function signature takes
`&mut self` — the pointer's own provenance is what Stacked Borrows tracks, not the function
signature it was produced inside. Detection:

```bash
grep -n "as_ptr() as \*mut" module/math/mdmath_core/src/vector/*.rs
```

should show no matches — any such cast should start from `as_mut_ptr()` instead. More generally,
any crate with `unsafe` pointer-cast code exercised only by ordinary (non-Miri) tests should
schedule a periodic full-crate `cargo +nightly miri test` sweep — not just Miri runs scoped to a
single bug's own new tests — since a scoped Miri run proves only that the new tests are UB-free,
never that pre-existing, untouched code in the same crate is.

**Pitfall:** copy-pasting an immutable pointer-cast implementation (`array_ref`'s `as_ptr() as
*const [E;N]`) to author its mutable sibling (`vector_mut`) and changing only the target type
(`*const` → `*mut`) without also changing the source accessor (`as_ptr()` → `as_mut_ptr()`)
produces code that compiles, passes ordinary tests, and is still Undefined Behavior.

## Generalized Version

**Broken assumption:** "Casting `T::as_ptr()`'s result to a `*mut` type is fine as long as the
enclosing function already takes `&mut self`." False — a raw pointer's Stacked-Borrows provenance
is determined by which accessor produced it (`as_ptr()` → `SharedReadOnly`, `as_mut_ptr()` →
`Unique`), not by the ambient mutability of the reference the accessor was called through.

**Detection invariant:**
```
for every `unsafe` block that dereferences a `*mut T` pointer,
the pointer's value must trace back to a `*mut`-producing accessor
( e.g. `as_mut_ptr()`, `&mut expr as *mut _` ),
never to a `*const`-producing accessor ( e.g. `as_ptr()` ) cast to `*mut`.
```

## Verification

### Checklist

- [x] C1 — Is `vector_mut`'s cast fixed from `self.as_ptr()` to `self.as_mut_ptr()`? Direct read of `slice.rs:50-63` → `unsafe { &mut *self.as_mut_ptr().cast::<[E;N]>() }`; `grep -n "as_ptr() as \*mut" module/math/mdmath_core/src/vector/*.rs` (the bug's own `## Prevention` detection command) → 0 matches.
- [x] C2 — Is the `Fix(BUG-054)` 3-field comment present, and does it explain the provenance mechanism (not just "fixed")? Direct read of `slice.rs:53-59` → present, states `as_ptr()` carries only `SharedReadOnly` provenance, names the root cause (copy-pasted from `array_ref`'s immutable sibling without switching accessors), and the pitfall.
- [x] C3 — Does the pre-existing `test_vector_mut_slice` remain the sole coverage (per this bug's own "no duplicate test added" decision), correctly tagged? `slice_test.rs:20-27` → `// test_kind: bug_reproducer(BUG-054)` present directly above `test_vector_mut_slice`; no second, duplicate test was added alongside it.
- [x] C4 — Does the fixed source pass clean under Miri (the bug's own primary evidence mechanism)? This session's fresh `cargo +nightly miri test -p mdmath_core --all-features` shows `inc::vector_test::slice_test::test_vector_mut_slice ... ok` with zero Stacked-Borrows errors anywhere in the run (I3) — matches the bug's documented "Green (post-fix)" expectation.
- [x] C5 — Is the "dormant, zero live callers" Impact/Magnitude claim still true? Workspace-wide `grep -rn "\.vector_mut(" --include="*.rs" .` → every hit is inside `mdmath_core/tests/vector_test/*_test.rs`; `grep -rln "ArrayMut" module/ examples/ --include="*.rs"` → only `mdmath_core`'s own impls/tests plus one safe-by-construction impl in `ndarray_cg/src/vector/general.rs` (`&mut self.0`, no raw pointer) — no generic consumer exists that could be affected either way.

### Measurements

- [x] M1 — `as_ptr() as *mut` occurrences in `mdmath_core/src/vector/`: `0` (was: `1`, at `slice.rs`'s `vector_mut` — `git show 9b71cf39^:module/math/mdmath_core/src/vector/slice.rs` line 55 shows `unsafe { &mut *( self.as_ptr() as *mut [ E ; N ] ) }`).
- [x] M2 — Miri UB errors on the crate's full test suite: `0` (was: `1`, reproduced in `## Symptom`: `error: Undefined Behavior: trying to retag from <852419> for Unique permission ... but that tag only grants SharedReadOnly permission`, at pre-fix `slice.rs:55` — this session's fresh run against current source shows the same test now passing clean).

### Invariants

- [x] I1 — Test suite (crate-scoped): `cargo nextest run -p mdmath_core --all-features` (via `longrun`) → exit 0, 89 tests run: 89 passed, 0 skipped, including `test_vector_mut_slice` (log `-0014_longrun.log`).
- [x] I2 — Lints clean: `cargo clippy -p mdmath_core --all-targets --all-features -- -D warnings` (via `longrun`) → exit 0, zero warnings (log `-0018_longrun.log`).
- [x] I3 — Miri Stacked Borrows (this bug's own primary evidence mechanism): `cargo +nightly miri test -p mdmath_core --all-features` (via `longrun`) → exit 0, 89 passed, 0 failed, zero UB anywhere in the crate (log `-0019_longrun.log`); Miri's availability confirmed first (`cargo +nightly miri --version` → `miri 0.1.0`) rather than assumed.

### Anti-faking checks

- [x] AF1 — Guards against the `as_ptr()`-cast-to-`*mut` pattern reappearing in this or any sibling accessor: re-run C1's `grep -n "as_ptr() as \*mut"` across `mdmath_core/src/vector/*.rs` — must return 0 matches; the bug's own `## Prevention` names this exact command.
- [x] AF2 — Guards against a future edit re-widening `vector_mut`'s real callers beyond today's test-only usage without also re-running Miri first: any new production call site to `<[E] as ArrayMut<E,N>>::vector_mut` should trigger a fresh `cargo +nightly miri test -p mdmath_core --all-features` before being trusted, since ordinary (non-Miri) `cargo test`/`cargo clippy` cannot detect this UB class at all (confirmed clean natively both before and after the fix, per this bug's own `## Why Not Caught`).

## History

| Date       | Event  | Notes                                                                                                     |
|------------|--------|-------------------------------------------------------------------------------------------------------------|
| 2026-08-10 | filed  | Found while independently re-verifying BUG-050's fix via a full-crate (not scoped) `cargo +nightly miri test -p mdmath_core --all-features` run; root cause confirmed via source comparison against the crate's own correct sibling implementations (`array_ref`, `array.rs`, `ndarray_cg::Vector`) plus a standalone `/tmp` MRE |
| 2026-08-10 | confirmed | VERIFY_PASS — Tier 2 Dual-Role Self-Check, 8/8 dimensions PASS; MRE re-executed and reproduces (Miri exit 1) |
| 2026-08-10 | completed | `as_ptr()` → `as_mut_ptr()` in `vector_mut`; pre-existing `test_vector_mut_slice` annotated as this bug's reproducer (no duplicate test added). Miri clean post-fix (76 unit + 3 doc tests passed), native `cargo nextest run -p mdmath_core --all-features` unchanged at 76/76, `cargo clippy -p mdmath_core --all-targets --all-features -- -D warnings` clean. Same-session, self-administered (filer = fixer = verifier) — Tier 2 Dual-Role Self-Check per `governance/maav.rulebook.md`'s default, not an independent PROC16-style acceptance pass. State → Completed. |

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 8/8

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Completeness | 🟢 | 🟢 | — | — |
| D2 | MRE Validity & Reproducibility | 🟢 | 🟢 | — | — |
| D3 | Cross-Reference Integrity | 🟢 | 🟢 | — | — |
| D4 | Root Cause Quality | 🟢 | 🟢 | — | — |
| D5 | Execution Scope | 🟡 | 🟢 | Adversarial pass: initial draft asserted "zero non-test call sites" from a narrower `.vector_mut(` grep alone, without also checking generic `ArrayMut` trait-bound consumers | Re-ran `grep -rln "ArrayMut"` workspace-wide and confirmed only `ndarray_cg::Vector` implements it (itself safe by construction) — no generic consumer exists |
| D6 | Crate Scope Unity | 🟢 | 🟢 | Single crate (`mdmath_core`), single file fixed | — |
| D7 | Crate Locality | 🟢 | 🟢 | — | — |
| D8 | Crate Single Responsibility | 🟢 | 🟢 | — | — |
| **Total** | | 🔴 | 🟢 | 1 fixed | 1/1 |

**Reproduced:** YES — Miri exit 1 pre-fix (`error: Undefined Behavior ... SharedReadOnly`), exit 0
post-fix, both via `cargo +nightly miri test -p mdmath_core --all-features`, 2026-08-10, verbatim
output captured into `## Symptom` and `## Fix Applied`.

**Aggregate verdict:** PASS — all 8 dimensions 🟢 after one Fix-and-Recheck Loop round
(`governance/maav.rulebook.md § MAAV : Fix-and-Recheck Loop`); self-administered, no subagent
dispatch (Verification Delegation would be forbidden per `file.rulebook.md § Report New Bug :
Step 9 - VERIFY Gate`).

## Refs: src/

- `module/math/mdmath_core/src/vector/slice.rs` — `vector_mut` (lines 47-58): `as_ptr()` →
  `as_mut_ptr()`, with a `Fix(BUG-054)` source comment explaining the provenance mechanism.

## Refs: tests/

- `module/math/mdmath_core/tests/inc/vector_test/slice_test.rs` — `test_vector_mut_slice`
  (lines 20-27): annotated `// test_kind: bug_reproducer(BUG-054)` plus a doc comment; no new
  test added since the existing test already exercises the exact bug-triggering sequence.
