# Algorithm: Cancellation-Free Hyperbolic Forms

### Scope

- **Purpose**: Document the substitutions that let `exp_m1`, `ln_1p` and the six hyperbolic functions keep a small argument's digits, where each one's textbook closed form destroys them.
- **Responsibility**: Give each function's rewritten identity, the algebra that justifies it, the band it applies on, and the saturation branch at the other end.
- **In Scope**: `exp_m1`, `ln_1p`, `sinh`, `cosh`, `tanh`, `asinh`, `acosh`, `atanh`, and the `SERIES_BAND`, `HYPERBOLIC_SATURATION`, `LARGE_ARGUMENT` thresholds.
- **Out of Scope**: Why cancellation destroys digits at all — the mechanism is [pitfall/001](../pitfall/001_small_argument_cancellation.md), which this document is the constructive half of.

### Abstract

Every function here has a textbook closed form, and every one of those forms
cancels catastrophically somewhere in its domain: near zero for the direct
functions, near zero *or* near a domain edge for the inverses. The pattern is
always the same — the formula constructs a quantity near `1` and then removes
the `1`, so the leading bits agree and subtract away, leaving only what the
intermediate roundings happened to leave behind.

The cure is never a longer series or a tighter tolerance. It is an algebraic
rewrite that never forms the `1` in the first place. `exp_m1` and `ln_1p` are
the two primitives that make the rewrites possible, and this module is their
largest consumer.

At the other end of each domain the opposite problem appears — a term that
cannot change a bit but can still overflow computing it — and each function
carries a saturation branch that is an *identity*, not an approximation.

### The Two Primitives

**`exp_m1( x )`** — for `| x | < SERIES_BAND` (`0.25`), the series
`x·( 1 + x/2! + x²/3! + … )`, which never forms the `1` the subtraction would
have had to remove. Outside the band, plainly `exp( x ) − 1`.

**`ln_1p( x )`** — for `| x | < SERIES_BAND`, the identity
`ln( 1 + x ) = 2·atanh( x/( 2 + x ) )`, reusing `atanh_series`
([003](003_logarithm_by_mantissa_split.md)). The series argument
`s = x/( 2 + x )` is computed from `x` directly and keeps every bit of it, so
`1 + x` is never formed and `x`'s low bits are never rounded away before the
logarithm sees them. Outside the band, plainly `ln( 1 + x )`.

**The band is `0.25`, and its previous value of `1e-5` was four decades too
low.** Both naive identities carry an absolute error near half an ulp of `1`
(≈`1.1e-16`) into a result whose magnitude is roughly `| x |`, so their relative
error is about `1.1e-16 / | x |` — which passes one part in `1e15` only around
`| x | = 0.11`. Across the whole band from `1e-5` to `0.25` the old code
returned bit-identical results to the naive expression it was called to avoid,
losing three to four decimal digits exactly where callers had been told it would
not. At `0.25` the naive form is within about 1.5 ulp while both series still
have orders of margin, so the two paths overlap comfortably rather than meeting
at a seam — which is asserted, at the seam, on both functions.

### The Rewrites

Each row is the same move: name the small quantity, and express the answer in it
without ever reconstructing the large one.

| Function | Textbook form | Used instead | Substitution |
|----------|---------------|--------------|--------------|
| `sinh`, `\| x \| < 1` | `( eˣ − e⁻ˣ )/2` | `u( u + 2 )/( 2( 1 + u ) )` | `u = exp_m1( x )`, so `eˣ = 1 + u` |
| `tanh`, `\| x \| ≤ 20` | `( e²ˣ − 1 )/( e²ˣ + 1 )` | `u/( u + 2 )` | `u = exp_m1( 2x )` |
| `asinh`, `\| x \| < 1` | `ln( x + √( x² + 1 ) )` | `ln_1p( x + x²/( 1 + √( 1 + x² ) ) )` | the bracket is `x + √( x²+1 ) − 1` |
| `acosh`, `x ≤ 2²⁸` | `ln( x + √( x² − 1 ) )` | `ln_1p( t + √( t( t + 2 ) ) )` | `t = x − 1`, exact by Sterbenz on `[ 0.5, 2 ]` |
| `atanh` | `ln( ( 1+x )/( 1−x ) )/2` | `ln_1p( 2x/( 1 − x ) )/2` | `( 1+x )/( 1−x ) = 1 + 2x/( 1−x )` |
| `cosh` | `( eˣ + e⁻ˣ )/2` | *unchanged* | both terms positive — nothing to cancel |

`cosh` is the one member of the family with no cancellation anywhere, and it is
worth stating explicitly: it needs only the saturation branch, and only to stay
finite.

`acosh`'s `t = x − 1` deserves its own note. Sterbenz's lemma makes that
subtraction *exact* for every `x` in `[ 0.5, 2 ]` — no rounding at all — which
is what makes the whole rewrite work at the domain edge. The naive form at
`x = 1 + 1e-10` has already lost six digits, because `x²` rounds and subtracting
`1` then removes most of what distinguished it, leaving the square root to be
computed from a handful of surviving bits.

### The Saturation Branches

Three thresholds, each chosen so the branch is an identity rather than an
approximation:

**`HYPERBOLIC_SATURATION = 20`** — at `| x | = 20`, `e⁻ˣ` is `2e-9` times `eˣ`,
so `e⁻²ˣ` — the relative weight of the second term in `sinh`, `cosh` and `tanh`
alike — is `4e-18`, below half an ulp of the first. Past this point the smaller
exponential cannot change a single bit, so evaluating it is pure cost. On `tanh`
it is worse than cost: the answer is exactly `1.0`, and forming it from two
large numbers risks an overflow that returns `NaN` for an argument whose answer
is `1`.

`sinh` and `cosh` take `exp( a − ln 2 )` there rather than `0.5·exp( a )`,
halving inside the exponent so the result stays finite for an `a` whose `eᵃ`
would already have overflowed.

The threshold sits deliberately far below where `exp` overflows, so the branch
is chosen for being an identity rather than for dodging an edge case — and that
identity is asserted bit-exactly at the threshold itself.

**`LARGE_ARGUMENT = 2²⁸`** — where `asinh` and `acosh` become `ln( 2x )`. Also
an identity: `x² ± 1` rounds to `x²` there, since the gap between neighbouring
doubles at `7.2e16` is already `16`, and `√( x² )` is exact. Without it, `x·x`
overflows above `1.34e154` — barely a tenth of the way through the exponent
range, and far short of what a hyperbolic anomaly actually asks for. An argument
of `1e250` names a real point, at a result near `576`.

**Negative arguments to `asinh`** go by odd symmetry rather than directly,
because `x + √( x² + 1 )` cancels catastrophically for large negative `x` — the
two terms agree to as many digits as `x` has.

### The Measured Result

| function | before | after |
|----------|-------:|------:|
| `exp_m1` | 31 203 ulp | 4 |
| `ln_1p` | 32 768 ulp | 5 |
| `sinh` | 7 163 ulp | 4 |
| `tanh` | 1 244 ulp | 4 |
| `asinh` | 2 628 ulp | 4 |

### Verify It Yourself

```sh
cargo test -p deterministic_math the_saturation_branch_is_an_identity_not_an_approximation
cargo test -p deterministic_math the_branches_meet_without_a_step
cargo test -p deterministic_math the_series_band_is_where_both_paths_are_accurate
cargo run -p deterministic_math --release --example cost_vs_libm
```

The example's last two tables are aimed directly at this document: `exp_m1` and
`ln_1p` against the naive expressions they replace across seven decades, and
`( sinh( s ) − s )/s³`, where a caller's own subtraction amplifies `sinh`'s
error by roughly `1/s²`. Both make visible what a per-call ulp figure alone
does not — that these functions exist for what happens *after* they return.

Both branch-agreement tests evaluate the two formulas at the **same** argument
rather than either side of the threshold. Sampling across instead measures the
function's own slope times the sampling gap — `cosh( 1 ) ≈ 1.54` times `2e-12`
reads as a jump of `3e-12` while saying nothing about continuity.

### Algorithms

| File | Relationship |
|------|--------------|
| [003_logarithm_by_mantissa_split.md](003_logarithm_by_mantissa_split.md) | `atanh_series`, which `ln_1p` reuses directly, and the `ln` that the out-of-band paths fall back to |
| [001_cody_waite_range_reduction.md](001_cody_waite_range_reduction.md) | The `exp` every direct hyperbolic function is built on |
| [002_arctangent_bin_reduction.md](002_arctangent_bin_reduction.md) | The same repair carried out by removing a reduction rather than rewriting an identity |

### Pitfalls

| File | Relationship |
|------|--------------|
| [../pitfall/001_small_argument_cancellation.md](../pitfall/001_small_argument_cancellation.md) | The failure mode; this document is its constructive half |

### Non Functional Requirements

| File | Relationship |
|------|--------------|
| [../non_functional_requirement/002_accuracy_against_host_libm.md](../non_functional_requirement/002_accuracy_against_host_libm.md) | The before/after figures above, and the amplification instruments that produced them |

### Sources

| File | Relationship |
|------|--------------|
| `src/hyperbolic.rs` | All six hyperbolic functions and their branch structure |
| `src/exponential.rs` | `exp_m1`, `ln_1p` — the two primitives |
| `src/constant.rs` | `SERIES_BAND`, `HYPERBOLIC_SATURATION`, `LARGE_ARGUMENT`, each carrying its own derivation |

### Tests

| File | Relationship |
|------|--------------|
| `src/hyperbolic.rs` — `mod tests` | `the_saturation_branch_is_an_identity_not_an_approximation` (bit-exact at the threshold), `the_branches_meet_without_a_step` |
| `src/exponential.rs` — `mod tests` | `the_series_band_is_where_both_paths_are_accurate` |
| `tests/inc/hyperbolic_test.rs` | `the_small_argument_band_keeps_every_digit`, `the_saturating_branches_agree_with_the_ones_they_replace`, `acosh_inverts_cosh_and_refuses_what_has_no_inverse` (including the just-above-1 band), the identity and inversion round trips |
| `tests/inc/exponential_test.rs` | `the_two_series_bands_join_without_a_step`, the `exp_m1`/`ln_1p` cancellation tests over 300 decades |
