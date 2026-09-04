# BUG-149: `Squad::apply` blends tangents 1/3 toward the endpoints instead of slerping them directly, producing the wrong mid-curve quaternion

- **Severity:** Medium (silently wrong mid-curve interpolation result; not a crash, and
  invisible at both `time == 0.0` and `time == 1.0` — see Pitfall)
- **state:** Completed
- **Affects:** Any `Squad::apply( start, end, time )` call where `time` is strictly between
  `0.0` and `1.0`
- **Component:** `module/helper/animation` (`src/easing/squad.rs`)
- **repo_identity:** self
- **Filed:** 2026-08-16
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/verified/
- **verification_date:** 2026-08-16
- **Related Bugs:** None — independent of every other bug filed this session.

## Symptom

```rust
use animation::easing::{ base::EasingFunction, Squad };
use mingl::{ QuatF32, F32x3 };
use std::f32::consts::PI;

let start = QuatF32::default();
let end = QuatF32::from_axis_angle( F32x3::new( 0.0, 0.0, 1.0 ), PI / 2.0 );
let out_tangent = QuatF32::from_axis_angle( F32x3::new( 1.0, 0.0, 0.0 ), PI / 6.0 );
let in_tangent = QuatF32::from_axis_angle( F32x3::new( 0.0, 1.0, 0.0 ), PI / 3.0 );

let squad = Squad::new( in_tangent, out_tangent );
let result = squad.apply( start, end, 0.4 );
// Wrong (pre-fix):    result.x() == 0.026101308  -- computed via an extraneous 1/3-blend of
//                      start/end toward the tangents before the second slerp
// Correct (post-fix): result.x() == 0.07900075   -- matches Slerp(Slerp(start,end,t),
//                      Slerp(out_tangent,in_tangent,t), 2t(1-t)), the formula this file's own
//                      cited sources define
```

## Impact

**Who is affected:** Any caller of `Squad::apply` at any `time` strictly inside `(0.0, 1.0)` —
i.e. every actual animation frame except the first and last. `Squad` is the crate's only
quaternion-interpolation easing function, used for smooth spline-style rotation animation
between keyframes with precomputed tangent quaternions.

**What breaks:** The pre-fix `apply` computed two intermediate quaternions,
`b_start = start.slerp( &out_tangent, 1/3 )` and `b_end = end.slerp( &in_tangent, 1/3 )`, then
slerped `b_start`/`b_end` against each other in the second (outer) slerp. This 1/3-blend step
does not appear in the SQUAD formula defined by any of the three sources this file's own doc
comment cites — the correct formula slerps the tangent quaternions `out_tangent`/`in_tangent`
**directly** against each other, with no intermediate blend toward `start`/`end` at all. The
result is a plausible-looking but numerically wrong rotation at every mid-curve `time` value —
the interpolated orientation deviates from the correct SQUAD curve, most severely near
`time == 0.5` where the outer `2t(1-t)` coefficient (which weights the tangent-slerp
contribution) is at its maximum.

**Boundary values unaffected:** at `time == 0.0` and `time == 1.0`, the outer coefficient
`2t(1-t)` is exactly `0`, so `apply` returns precisely `start`/`end` regardless of what the
second slerp computes — under both the buggy and fixed formula. This is why the defect is
invisible to boundary-only testing (see Pitfall).

**Magnitude:** Silent wrong rotation, not a crash. Any consumer relying on `Squad` for smooth
tangent-aware quaternion interpolation gets a curve that passes through the correct keyframes but
takes the wrong path between them.

**Entity Scope:** None — a code-level defect, not an operational-entity concern.

## How Discovered

Deferred investigation task from this session's `animation` crate review (tracked as "Investigate
`Squad::apply` tangent/endpoint pairing"). The original framing suspected a simple
tangent/endpoint pairing swap. An internal mathematical reversal-symmetry check (swap
`start`/`end`, swap `in_tangent`/`out_tangent`, and `time -> 1 - time` should reproduce the
identical result) was carried out first but proved genuinely inconclusive — it held equally for
both the current pairing and its swap, so it could not by itself distinguish "correct" from
"incorrectly paired." The investigation was escalated to the three external sources `squad.rs`'s
own doc comment already cites: a GitHub C++ reference implementation
(`ROBOOP/source/quaternion.cpp`, fetched via `WebFetch`) and an MIT thesis PDF
(`QuaternionReport1.pdf`, Section 6.2.1, Definition 17, read directly via the `Read` tool after
`WebFetch`'s own summarization failed to extract that section) both independently confirmed the
same formula shape:
`Squad(q_i,q_{i+1},s_i,s_{i+1},h) = Slerp( Slerp(q_i,q_{i+1},h), Slerp(s_i,s_{i+1},h), 2h(1-h) )`,
with the precomputed control quaternions `s_i`/`s_{i+1}` used **directly** in the inner slerp —
no further blending toward the endpoints. (The third cited source, `math.stackexchange.com`,
could not be fetched at all — a tool-level restriction, not a content issue — but the two
successfully-verified independent sources already agreed exactly, so no further source was
needed.) This revealed the actual defect is more fundamental than a pairing swap: the pairing
itself (`out_tangent` with `start`, `in_tangent` with `end`) was already correct; the bug is an
entirely extraneous blending step that does not belong in the formula at all.

## Minimum Reproducible Example

```bash
cd module/helper/animation && cargo test --test squad_test test_squad_matches_reference_formula_mid_curve 2>&1 | tail -10
```

**Expected** (post-fix):
```
test tests::test_squad_matches_reference_formula_mid_curve ... ok
```

**Actual** (pre-fix — confirmed by writing the test against the unfixed code before applying the
fix, test-first):
```
0.026101308 != 0.07900075 (eps 0.0001)
FAIL [   0.007s] (1/2) animation::squad_test tests::test_squad_matches_reference_formula_mid_curve
Summary [   0.012s] 2 tests run: 1 passed, 1 failed, 0 skipped
```

**Verify Command** (≤3 lines, standalone):
```bash
cd module/helper/animation && cargo test --test squad_test test_squad_matches_reference_formula_mid_curve
# 1 passed = fixed; 1 failed ("0.026101308 != 0.07900075") = bug present
```

**Known MRE limitation (check 205):** none — pure, synchronous, dependency-free state; the
regression test runs as an ordinary native `cargo test` against the real crate directly.

## Hypothesis Table

| ID | Hypothesis | State | Summary | Evidence |
|----|-----------|--------|---------|----------|
| H1 | `apply`'s `b_start`/`b_end` 1/3-blend step is an extraneous addition not part of the correct SQUAD formula; the fix is to slerp `out_tangent`/`in_tangent` directly. | ✅ Root Cause | Two independent authoritative sources (ROBOOP C++ reference, MIT thesis Definition 17) both define the formula with the tangent quaternions used directly in the inner slerp, no intermediate blend. | E1, E2 |
| H2 | The defect is a simple `in_tangent`/`out_tangent` pairing swap relative to `start`/`end` (the task's original framing). | ❌ Falsified | Both external sources confirm the existing pairing (`out_tangent` acts as `start`'s outgoing control point / `s_i`, `in_tangent` as `end`'s incoming control point / `s_{i+1}`) is already correct — the fix preserves this exact pairing and only removes the blend step. An internal reversal-symmetry check was tried first but proved inconclusive for distinguishing pairing correctness either way. | E1, E2, E3 |
| H3 | A boundary-value test (`time == 0.0`/`1.0`) would have caught this defect had one existed. | ❌ Falsified | The outer `2t(1-t)` coefficient is exactly `0` at both boundaries, so `apply` returns precisely `start`/`end` regardless of the second slerp's value — confirmed directly: `test_squad_boundaries` (written against the unfixed code) already passes pre-fix. | E4 |

## Evidence Table

| # | Location | What it shows | Hypothesis |
|---|----------|---------------|------------|
| E1 | `github.com/phuicy/ROBOOP` `source/quaternion.cpp:722` (fetched via `WebFetch`) | `Squad(p,a,b,q,t) = Slerp(Slerp(p,q,t), Slerp(a,b,t), 2*t*(1-t))` — control points `a`/`b` used directly, no blend with `p`/`q`. | H1 ✅, H2 ❌ |
| E2 | MIT thesis `QuaternionReport1.pdf`, Section 6.2.1, Definition 17 (PDF absolute pages 55-57, read directly via `Read` after `WebFetch` summarization missed the section) | `Squad(q_i,q_{i+1},s_i,s_{i+1},h) = Slerp(Slerp(q_i,q_{i+1},h), Slerp(s_i,s_{i+1},h), 2h(1-h))`, with `s_i`/`s_{i+1}` precomputed and used directly — independently confirms E1's formula shape. | H1 ✅, H2 ❌ |
| E3 | Internal reversal-symmetry analysis (swap `start`/`end`, swap `in_tangent`/`out_tangent`, `time -> 1-time`) | Holds equally under both the current pairing and its swap — mathematically valid but inconclusive for settling H2 on its own; recorded as a dead end, not used as evidence either way. | H2 (inconclusive on its own) |
| E4 | `tests/squad_test.rs`, `test_squad_boundaries` run against unfixed `apply` | Passes pre-fix — confirms the boundary-only blind spot directly rather than by inference. | H3 ❌ |

## Root Cause

```
Squad::apply()   (pre-fix)
  let b_start = start.slerp( &self.out_tangent, 1/3 );        // <-- extraneous
  let b_end   = end.slerp( &self.in_tangent, 1/3 );            // <-- extraneous
  let slerp1  = start.slerp( &end, t );
  let slerp2  = b_start.slerp( &b_end, t );                    // <-- should slerp tangents directly
  slerp1.slerp( &slerp2, 2*t*(1-t) )

Correct formula (Shoemake, Definition 17 / ROBOOP reference — E1, E2)
  Squad(q_i, q_{i+1}, s_i, s_{i+1}, h)
    = Slerp( Slerp(q_i, q_{i+1}, h), Slerp(s_i, s_{i+1}, h), 2h(1-h) )
```

`s_i`/`s_{i+1}` are themselves precomputed via a separate log/exp-map formula over neighboring
keyframes — entirely outside `Squad::apply` itself, matching how `out_tangent`/`in_tangent` are
already passed in as ready-to-use fields on `Squad`. The pre-fix code additionally blended them
1/3 toward `start`/`end` before the second slerp — a step with no basis in the cited formula.

## Why Not Caught

`Squad` had zero test coverage of any kind prior to this fix — no test constructed a `Quat` or
called `Squad::apply` anywhere in the crate.

## Fix Location

`module/helper/animation/src/easing/squad.rs`, `impl< E > EasingFunction for Squad< E >`:

```rust
// before
fn apply( &self, start : Quat< E >, end : Quat< E >, time : f64 ) -> Quat< E >
{
  let time_e = E::from( time ).unwrap();
  let b_start = start.slerp( &self.out_tangent, E::from( 1.0 / 3.0 ).unwrap() );
  let b_end = end.slerp( &self.in_tangent, E::from( 1.0 / 3.0 ).unwrap() );
  let slerp1 = start.slerp( &end, time_e );
  let slerp2 = b_start.slerp( &b_end, time_e );

  slerp1.slerp( &slerp2, E::from( 2.0 * time * ( 1.0 - time ) ).unwrap() )
}

// after
fn apply( &self, start : Quat< E >, end : Quat< E >, time : f64 ) -> Quat< E >
{
  let time_e = E::from( time ).unwrap();
  let slerp1 = start.slerp( &end, time_e );
  let slerp2 = self.out_tangent.slerp( &self.in_tangent, time_e );

  slerp1.slerp( &slerp2, E::from( 2.0 * time * ( 1.0 - time ) ).unwrap() )
}
```

Removed the `b_start`/`b_end` intermediate computation entirely; the second slerp now uses
`self.out_tangent`/`self.in_tangent` directly. No signature change, no field change.

## Prevention

Added `tests/squad_test.rs` (new file — no prior test file existed for this source file):
`test_squad_boundaries` (locks in the `time == 0.0`/`1.0` exact-match property, explicitly
documented as necessary-but-insufficient — see Pitfall) and
`test_squad_matches_reference_formula_mid_curve` (`bug_reproducer(BUG-149)`, pinning a mid-curve
value against a reference independently composed via the confirmed-correct formula).

## Pitfall

The outer `2t(1-t)` coefficient is exactly `0` at `time == 0.0` and `time == 1.0`, so `apply`
returns precisely `start`/`end` at both boundaries under **both** the buggy and fixed formula —
a boundary-only test (the style already used elsewhere in this crate, e.g.
`test_cubic_boundaries_and_properties`) can never catch this class of defect; only a pinned
mid-curve value can. This is also why an internal symmetry-style analysis alone was insufficient
to resolve the investigation (see Hypothesis Table H2/E3) — some formula-correctness questions
cannot be settled by reasoning about the buggy code's own structure and require checking against
an independent, authoritative external definition.

## Generalized Version

**Broken assumption:** "code implementing a named mathematical formula, with citations to
authoritative sources already in its own doc comment, has been checked against those sources."
False — the citations documented *where the formula comes from*, not that the implementation was
ever verified against them line-by-line.

**Confirmed general rule:** for a bug investigation into a named-formula implementation, when
internal reasoning (code reading, symmetry/invariant analysis) proves inconclusive, escalate to
the exact external sources the code itself already cites before concluding either "no bug" or
guessing at a fix — the citations are the cheapest, most direct check available and were already
present in this case. Distinguish this from BUG-144/145/146/147/148, where the crate's own
internal code (a sibling implementation, an enum variant, an early return) was sufficient
evidence on its own; this bug required going outside the codebase entirely.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-16 | filed | Deferred investigation task from this session's `animation` crate review; original "pairing swap" framing revised after external verification revealed the actual defect (extraneous blend step). |
| 2026-08-16 | fixed | Removed the `b_start`/`b_end` 1/3-blend step; second slerp now uses `out_tangent`/`in_tangent` directly. |
| 2026-08-16 | verified | Added `tests/squad_test.rs` (2 tests, written test-first against the unfixed code); confirmed `test_squad_matches_reference_formula_mid_curve` fails pre-fix with `0.026101308 != 0.07900075` and passes post-fix, while `test_squad_boundaries` passes both pre- and post-fix (confirming the boundary blind spot directly); full crate suite (40 tests incl. both new, 3 doctests) + `cargo clippy --all-targets --all-features -- -D warnings` clean. |
| 2026-08-16 | completed | Acceptance verification by a distinct session, same batch as BUG-138 (see its completed-row note for the shared 40/40 `animation` run and MAAV batch scope). Independently re-read `Squad::apply` (confirmed the `b_start`/`b_end` 1/3-blend step genuinely removed, `self.out_tangent.slerp( &self.in_tangent, time_e )` used directly, `Fix(BUG-149)`/`Root cause`/`Pitfall` comment intact, matching this file's own cited ROBOOP/MIT-thesis sources) and `test_squad_matches_reference_formula_mid_curve` (non-tautological: pins a mid-curve value against a reference independently reconstructed via the confirmed-correct formula, not merely re-deriving the implementation under test). State → Completed. |

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 8/8

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Completeness | 🟢 | 🟢 | All 12 FI008 sections present. | — |
| D2 | MRE Validity & Reproducibility | 🟢 | 🟢 | Confirming pass wrote the test against unfixed code and captured the exact pre-fix failure value; adversarial pass specifically checked whether the boundary test alone would have been sufficient evidence (H3, falsified by running it directly pre-fix, not by assumption) and whether the internal symmetry analysis alone could have substituted for external verification (E3, explicitly recorded as inconclusive rather than silently discarded). | — |
| D3 | Cross-Reference Integrity | 🟢 | 🟢 | Independent of every other bug filed this session; both external sources (E1, E2) are the exact URLs already cited in `squad.rs`'s own doc comment, not newly introduced. | — |
| D4 | Root Cause Quality | 🟢 | 🟢 | Root cause backed by two independent authoritative external sources, not asserted from internal reasoning alone — internal reasoning (E3) was explicitly insufficient and is recorded as such. | — |
| D5 | Execution Scope | 🟢 | 🟢 | Grepped the whole workspace for `Squad::` / `Squad(` usage — zero call sites outside `squad.rs` itself and the new test file, consistent with zero prior test coverage. | — |
| D6 | Crate Scope Unity | 🟢 | 🟢 | Only `animation` src+test+bug file touched. | — |
| D7 | Crate Locality | 🟢 | 🟢 | Fix local to one method body; no field/signature change. | — |
| D8 | Crate Single Responsibility | 🟢 | 🟢 | No new public surface; existing `EasingFunction::apply` contract now matches the crate's own cited formula sources. | — |

**Reproduced:** YES — `test_squad_matches_reference_formula_mid_curve` was written and run against
the unfixed `apply` first (test-first), producing the exact predicted `0.026101308 != 0.07900075`
failure; applying the fix and re-running returned both tests to passing, and the full crate suite
(40 tests incl. 3 doctests) + `cargo clippy --all-targets --all-features -- -D warnings` remained
clean, 2026-08-16.

## Refs: src/

| File | Change |
|------|--------|
| `module/helper/animation/src/easing/squad.rs` | `impl< E > EasingFunction for Squad< E >`: removed the `b_start`/`b_end` 1/3-blend step; second slerp now uses `self.out_tangent`/`self.in_tangent` directly. `Fix(BUG-149)`/`Root cause`/`Pitfall` comment added. |

## Refs: tests/

| File | Change |
|------|--------|
| `module/helper/animation/tests/squad_test.rs` | New file. `test_squad_boundaries` (baseline boundary coverage) and `test_squad_matches_reference_formula_mid_curve` (`bug_reproducer(BUG-149)`, 5-section doc comment). |
