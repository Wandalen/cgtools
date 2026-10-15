# BUG-230: `with_repeat`'s script-facing `i64`-to-`i32` cast silently wraps, and one wraparound value lands exactly on the infinite-repeat sentinel

- **Severity:** High (a script author intending a large but finite repeat count can silently
  get an infinite tween instead -- no error, no warning, a script-reachable footgun with a
  correctness-breaking outcome, not merely a cosmetic one)
- **state:** Completed
- **Affects:** Every Rhai script calling `.with_repeat(count)` on any of the 8 registered
  `Tween<...>` element types (`F32x1`..`F32x4`, `F64x1`..`F64x4`) with a count outside `i32`'s
  range -- reachable by any script author who passes a sufficiently large literal or computed
  value, including the specific value `4294967295` (`u32::MAX`), which wraps to exactly `-1`,
  `Tween`'s documented infinite-repeat sentinel.
- **Component:** `module/helper/scene_script` (`src/tween_binding.rs`)
- **repo_identity:** self
- **Filed:** 2026-08-17
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verification_date:** 2026-08-17
- **Related Bugs:** None. Independent of TASK-165 (`animation::Tween`'s finite `repeat_count`
  overshoot in `repeat_handle`) -- that defect is about an in-range finite count overshooting
  its own budget by one whole repeat; this bug is about an out-of-range count never reaching a
  valid finite value at all, one layer earlier at the script/host boundary.

## Symptom

```rust
// pre-fix -- every with_repeat registration, all 8 element types, identical shape
.register_fn( "with_repeat", | t : Tween< F32x1 >, count : i64 | t.with_repeat( count as i32 ) )
```

```
4294967295i64 as i32 == -1   // Tween's documented infinite-repeat sentinel
```

A script calling `.with_repeat( 4294967295 )` -- a plausible typo or intentional "very large
finite count" -- silently produces an INFINITE tween instead, with no error or warning.

## Impact

**Who is affected:** Any script author passing a repeat count outside `i32`'s range
(`-2147483648..=2147483647`) to any of the 8 `with_repeat` registrations.

**What breaks:** `as i32` truncation is a silent, well-defined-but-surprising Rust operation.
Depending on the exact input value, the wrapped result can land on `-1` (the documented
infinite-repeat sentinel in `animation::interpolation::Tween::repeat_handle`), on some other
unrelated positive or negative `i32`, or anywhere else in range -- none of it validated, none of
it reported to the script author. The most dangerous case is landing exactly on `-1`: a script
author who intended a large but finite repeat count instead gets a tween that never completes.

**Magnitude:** 1 shared cast pattern (`count as i32`), duplicated identically across all 8
`with_repeat` registrations in `tween_binding.rs`.

**Entity Scope:** None -- a code-level defect.

## How Discovered

This session's scouting pass of `scene_script` (previously unaudited), reading
`tween_binding.rs` in full and comparing every `with_repeat` registration's unguarded `as i32`
cast against `easing_from_name`'s existing script-catchable-error convention for invalid input
in the same file.

## Minimum Reproducible Example

```rhai
let t = tween( f32x1(0.0), f32x1(10.0), 10.0 ).with_repeat( 4294967295 );
t.current_repeat()
```

Pre-fix: succeeds silently (`with_repeat` accepts the wrapped `-1`, the infinite-repeat
sentinel). Post-fix: a script-catchable runtime error containing `"out of range"`.

**Verify Command** (<=3 lines, standalone):
```bash
cd module/helper/scene_script && cargo nextest run --all-features -E 'test(tween_with_repeat_rejects_count_that_would_wrap_to_the_infinite_sentinel)'
```

## Hypothesis Table

| ID | Hypothesis | State | Summary | Evidence |
|----|-----------|--------|---------|----------|
| H1 | Every `with_repeat` registration casts its `i64` argument to `i32` via unchecked `as`, which silently wraps out-of-range values instead of erroring. | ✅ Root Cause | Direct read of all 8 registrations (pre-fix) shows the identical `count as i32` pattern in every one; confirmed empirically via temporary-revert-and-rerun showing the call silently succeeds (`Ok`) for `4294967295` instead of erroring. | E1, E2, E4 |
| H2 | This is harmless because no reasonable script would ever pass a repeat count anywhere near `i32`'s bounds. | ❌ Falsified | `Tween::repeat_count`'s own doc comment documents `-1` as a meaningful, reachable sentinel value (infinite repeat) -- the wraparound boundary is not an obscure edge case but sits directly adjacent to a value the API itself treats specially. A script author entering a very large finite count (a plausible "loop basically forever" intent, expressed as a big number rather than the `-1` sentinel) is one wraparound away from silently getting exactly that sentinel instead. | E3 |

## Evidence Table

| # | Location | What it shows | Hypothesis |
|---|----------|---------------|------------|
| E1 | `module/helper/scene_script/src/tween_binding.rs`, all 8 `with_repeat` registrations (pre-fix, direct read) | Identical unguarded `count as i32` cast, duplicated 8 times, no range check anywhere. | H1 ✅ |
| E2 | `module/helper/scene_script/src/tween_binding.rs`, `easing_from_name` (direct read) | The same file already establishes the correct contract for invalid script input: a script-catchable `Err`, never a silent fallback -- `with_repeat` was the one script-facing conversion in this file that didn't follow it. | H1 ✅ |
| E3 | `module/helper/animation/src/interpolation.rs` lines 74-75, 269 (direct read) | `repeat_count : i32` field doc comment: `"Number of times to repeat ( 0 = no repeat, -1 = infinite )"`; `repeat_handle` branches explicitly on `self.repeat_count == -1` for the infinite case -- confirms `-1` is a real, specially-handled sentinel, not an arbitrary value. | H2 ❌ |
| E4 | Temporary direct-source-edit revert-and-rerun (this fix) | Reverting the shared helper to an unchecked `Ok( count as i32 )` produced test failure `called \`Result::unwrap_err()\` on an \`Ok\` value: 0` for input `4294967295` -- an exact, unambiguous empirical confirmation that the pre-fix code accepted this value silently. | H1 ✅ |

## Root Cause

`with_repeat`'s Rhai registrations accepted a script-supplied `i64` (Rhai's only integer type)
and narrowed it to the `i32` `animation::Tween::with_repeat` actually stores, via unchecked
`as`. Integer-to-integer `as` casts in Rust never panic and never signal truncation -- any
value outside the target type's range wraps silently. Because `Tween` additionally treats one
specific `i32` value (`-1`) as a meaningful "infinite repeat" sentinel, this wasn't merely lossy
-- one identifiable input value (`4294967295`, and any other value congruent to `-1` mod
2^32) silently converts a script author's large-finite-count intent into an infinite one.

## Why Not Caught

No existing test drove any `with_repeat` registration with a value outside `i32`'s range -- the
2 pre-existing repeat-related tests in `tests/engine_test.rs` both use the small literal `5`.

## Fix Location

`module/helper/scene_script/src/tween_binding.rs`: added a shared `repeat_count_from_i64`
helper (mirroring `easing_from_name`'s existing pattern) using `i32::try_from` and mapping the
error to a script-catchable message. All 8 `with_repeat` registrations changed from the
unchecked closure form to the fallible `-> Result< Tween< X >, Box< EvalAltResult > >` form
already used by this file's own 4-arg named-easing `tween(...)` overloads, calling
`repeat_count_from_i64( count )?` in place of the raw `as` cast.

## Prevention

`tests/engine_test.rs::tween_with_repeat_rejects_count_that_would_wrap_to_the_infinite_sentinel`
pins the single most dangerous wraparound value (`4294967295`, which lands exactly on the
infinite-repeat sentinel) as a permanent regression guard, asserting the script-level `eval`
call now returns a catchable error rather than succeeding silently.

## Pitfall

`as` between integer types is a silent, non-panicking, non-`Result`-returning operation --
every occurrence reachable from script (or any other untrusted/external) input needs its own
explicit range check, because nothing else in the language will ever surface the truncation.
When the narrower target type additionally carries a special sentinel value (here, `-1` for
"infinite"), an unchecked narrowing cast isn't just imprecise -- it can silently convert
ordinary input into that sentinel, turning a magnitude error into a qualitatively different,
much more damaging outcome (finite becomes infinite) rather than merely a wrong finite number.

## Generalized Version

**Broken assumption:** "a script-facing numeric narrowing cast is safe as long as no reasonable
script would pass a value anywhere near the target type's bounds."

**Confirmed general rule:** Any narrowing cast reachable from external (script, network,
file-format) input must be range-checked and turned into a catchable error on overflow --
`as` never does this automatically. This is doubly true when the narrower type reserves any
specific value as a sentinel: a silent wraparound can land exactly on that sentinel, converting
an otherwise-bounded input error into qualitatively different (and potentially unbounded)
behavior, not just a numerically wrong result.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-17 | filed | Found via this session's `scene_script` scouting pass, comparing all 8 `with_repeat` registrations' unguarded `as i32` cast against `easing_from_name`'s existing script-catchable-error convention in the same file, then confirming the `-1` infinite-repeat sentinel via direct read of `animation::interpolation::Tween`. |
| 2026-08-17 | fixed | Added shared `repeat_count_from_i64` helper (`i32::try_from`, script-catchable error on overflow); updated all 8 `with_repeat` registrations to the fallible closure form. |
| 2026-08-17 | verified | `cargo nextest run -p scene_script --all-features`: 57/57 passed, 0 skipped. `cargo test --doc -p scene_script --all-features`: clean (0 doc tests). `cargo clippy -p scene_script --all-targets --all-features -- -D warnings`: clean. Fix verified via a temporary direct-source-edit revert-and-rerun (`called \`Result::unwrap_err()\` on an \`Ok\` value: 0` pre-fix, passed post-fix). |

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 6/6

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Completeness | — | 🟢 | All FI008 sections present. | — |
| D2 | MRE Validity & Reproducibility | — | 🟢 | Confirming pass: deterministic MRE using the exact value that wraps to `-1`. Adversarial pass: noted the test's `current_repeat()` call happens before any `update()`, so it can't itself demonstrate the "tween never completes" consequence directly -- but the test's actual assertion (error vs. silent success) is exactly what the fix changes, and the infinite-sentinel consequence is independently established via direct source evidence (E3), not left to the test alone to prove. | — |
| D3 | Cross-Reference Integrity | — | 🟢 | Correctly distinguished from the separate, still-pending finite `repeat_count` overshoot defect (in `animation::Tween::repeat_handle` itself) as an independent, earlier-layer defect at the script/host boundary. | — |
| D4 | Root Cause Quality | — | 🟢 | Backed by direct reads of both `tween_binding.rs` (all 8 sites) and `animation::interpolation::Tween`'s sentinel documentation, plus empirical revert-rerun proof. | — |
| D5 | Execution Scope | — | 🟢 | Confirming pass: fix confined to adding one shared helper and updating exactly the 8 `with_repeat` registrations. Adversarial pass: grepped for `count as i32`/`as i32` across the file post-fix to confirm no other unguarded cast of this shape remains. | — |
| D6 | Crate Scope Unity | — | 🟢 | Fix lives entirely in `scene_script`'s Rhai-registration layer; `animation::Tween::with_repeat`'s own Rust-level signature is unchanged, so no downstream Rust caller needed updating -- confirmed this is a pure script-facing behavior change, not a Rust API break. | — |

**Reproduced:** Confirmed via `cargo nextest` (fail pre-fix with an `Ok` value where an `Err`
was expected, pass post-fix) and temporary direct-source-edit revert-and-rerun. 2026-08-17.

## Refs: src/

| File | Change |
|------|--------|
| `module/helper/scene_script/src/tween_binding.rs` | Added `repeat_count_from_i64` helper (range-checked `i32::try_from`, script-catchable error on overflow); all 8 `with_repeat` registrations changed to the fallible closure form calling it (full `Fix(BUG-230)` comment block); updated the shared module doc comment describing `.with_repeat()`'s behavior. |

## Refs: tests/

| File | Change |
|------|--------|
| `module/helper/scene_script/tests/engine_test.rs` | Added `tween_with_repeat_rejects_count_that_would_wrap_to_the_infinite_sentinel` (`bug_reproducer(BUG-230)`, 5-section doc comment). |

## Refs: docs/

| File | Change |
|------|--------|
| — | None — the fix eliminates the trap rather than leaving a permanent API characteristic to document; unlike `docs/pitfall/004`/`006` (both by-design, ongoing limitations), this defect has no residual pitfall for script authors once fixed. This bug report's own Pitfall/Generalized Version sections carry the lesson instead, matching this session's established convention for fixed (not by-design) defects. |
