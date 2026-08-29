# Algorithm: Arctangent Bin Reduction

### Scope

- **Purpose**: Document how `atan` is brought into range for an eleven-term series, and how `atan2`, `asin` and `acos` are each assembled from it.
- **Responsibility**: Specify the three-stage reduction (sign, inversion, bin), the bin table, the small-argument bypass, and the four functions built on top.
- **In Scope**: `atan`, `atan2`, `asin`, `acos`, `atan_series`, `clamp_unit`, the `ATAN_B`/`ATAN_V` tables, and the `ATAN_DIRECT`/`ASIN_DIRECT` thresholds.
- **Out of Scope**: Why the small-argument bypass is necessary at all — the cancellation mechanism is [pitfall/001](../pitfall/001_small_argument_cancellation.md); the circular functions these invert (see [001](001_cody_waite_range_reduction.md)).

### Abstract

The arctangent has no periodicity to strip, so the Cody-Waite trick does not
apply. What it has instead is an addition formula:

```text
atan t = atan b + atan( ( t − b ) / ( 1 + t·b ) )
```

Pick `b` close to `t` and the second term's argument is small, which is where
the alternating series `u − u³/3 + u⁵/5 − …` converges fast. Four bin centres
covering `[ 0, 1 ]` leave a residual under about `0.125`, at which eleven terms
reach full precision. Without the reduction the same series needs hundreds of
terms near `t = 1` and never actually gets there.

The subtlety is that this reduction is *destructive for a small argument*, in a
way the exponential's is not — and the fix is not a better reduction but the
absence of one. That is the small-argument bypass, and it is the single change
that took `asin`'s worst-case disagreement with libm from 22 268 ulp to 3.

### Algorithm

**Stage 1 — sign.** Record `neg = x < 0`, work with `| x |`, negate at the end.
Exact, and it makes `atan` odd bit-for-bit rather than approximately.

**Stage 2 — inversion.** If `t > 1`, replace `t` with `1/t` and set a flag; at
the end, `a ← π/2 − a`. This folds `[ 1, ∞ )` onto `( 0, 1 ]` so the two halves
share one series. An infinite argument inverts to zero, lands on the direct path
in stage 3, and returns exactly `π/2` — no special case needed.

**Stage 3 — bin, or bypass.**

```text
if t < ATAN_DIRECT ( = 0.0625 = 2⁻⁴ ) :   a = atan_series( t )
else :                                     j = min( ⌊4t⌋, 3 )
                                           b = ATAN_B[ j ]
                                           u = ( t − b ) / ( 1 + t·b )
                                           a = ATAN_V[ j ] + atan_series( u )
```

`ATAN_B` is `[ 0.125, 0.375, 0.625, 0.875 ]` — the four quarter-interval
midpoints — and `ATAN_V` holds `atan` of each to full double precision. The
table is not restated in a test but *re-derived*: bisection on this crate's own
`tan`, which shares nothing with the arctangent path, so a mistyped entry fails
rather than shifting every mid-range answer by a constant.

### The Bypass Is The Point

For `t` near zero the reduced form computes `ATAN_V[ 0 ] + atan_series( u )`
where the two terms are equal and opposite to within the size of `t`. The
leading digits cancel and what survives is the rounding of `0.1243…`, not the
answer. The reduction that makes the series converge fast for a mid-range
argument is precisely what destroys a small one.

Below `ATAN_DIRECT` the series is already well inside its own documented range
(`| u | ≤ 0.2`), so skipping the reduction costs nothing at all and keeps every
digit. The threshold is half the first bin's width, and is asserted to be
exactly `2⁻⁴` rather than a decimal literal that merely looks like it.

`atan2` and `asin` route into `atan`, so both inherited the defect and both
inherited the cure:

| function | before the bypass | after |
|----------|------------------:|------:|
| `atan` | 95 ulp | 2 |
| `atan2` | 8 883 ulp | 3 |
| `asin` | 22 268 ulp | 3 |

### The Four Functions

**`atan2( y, x )`** — quadrant selection around `atan( y/x )`, with the four
axis cases handled before the division so they return exact values rather than
a quotient of zeros. It exists because an angle recovered from a component pair
is only unambiguous when both signs are available: `atan( y/x )` discards one
and lands in the wrong half of the circle for every state west of the origin.

**`asin( x )`** — `atan2( c, √( ( 1−c )( 1+c ) ) )`, where `c` is `x` clamped to
`[ −1, 1 ]` ([invariant/004](../invariant/004_refusal_over_meaningless_answer.md)
explains the clamp). The companion side `√( 1−c² )` is written as
`√( ( 1−c )( 1+c ) )`, which is the accurate factoring near `| c | = 1`.

Below `ASIN_DIRECT` (`2⁻²⁷`) `asin` returns `atan( c )` directly and builds no
companion at all. Two reasons, and the second is the operative one. `asin c` and
`atan c` differ by `c³/2`, which under this threshold is below half the last
place of `c` — so they are the same double and the companion contributes nothing
but arithmetic. And that arithmetic *costs*: `1−c` and `1+c` each round, and
their product lands a step either side of `1` even though the exact product
rounds *to* `1`. Measured: `asin( 3.7e-12 )` came back one ulp above
`3.7e-12`, where the correct answer is the argument itself.

**`acos( x )`** — `atan2( √( ( 1−c )( 1+c ) ), c )`, the same two pieces with
the arguments swapped, and deliberately **no** bypass. Its result near zero is
`π/2` rather than something small, so a last-place wobble in the companion is a
last-place wobble in `1.57` — eight orders of magnitude less significant. The
asymmetry is visible in the measurements: before the reduction work `asin`
disagreed with libm by 22 268 ulp and `acos`, built from identical pieces, by 69.

### Verify It Yourself

```sh
cargo test -p deterministic_math the_bin_table_holds_the_arctangent_of_each_centre
cargo test -p deterministic_math the_direct_path_and_the_reduced_path_agree_at_the_seam
cargo test -p deterministic_math the_direct_thresholds_are_the_powers_of_two_they_claim_to_be
```

The seam test evaluates *both* strategies at the same argument rather than
sampling either side of the threshold — sampling instead measures
`d(atan)/dx ≈ 1` times the sampling gap, which looks exactly like a
discontinuity of that size while saying nothing about continuity.

### Invariants

| File | Relationship |
|------|--------------|
| [../invariant/004_refusal_over_meaningless_answer.md](../invariant/004_refusal_over_meaningless_answer.md) | `clamp_unit` — the one place the crate answers an out-of-domain argument instead of refusing it, and why |

### Algorithms

| File | Relationship |
|------|--------------|
| [001_cody_waite_range_reduction.md](001_cody_waite_range_reduction.md) | The reduction this one is the table-driven analogue of, for the functions that do have a periodicity to strip |
| [004_cancellation_free_hyperbolic_forms.md](004_cancellation_free_hyperbolic_forms.md) | The same small-argument repair carried out by rewriting the identity rather than by bypassing a reduction |

### Pitfalls

| File | Relationship |
|------|--------------|
| [../pitfall/001_small_argument_cancellation.md](../pitfall/001_small_argument_cancellation.md) | The mechanism the bypass exists to avoid, and the seven functions it was found in |

### Non Functional Requirements

| File | Relationship |
|------|--------------|
| [../non_functional_requirement/002_accuracy_against_host_libm.md](../non_functional_requirement/002_accuracy_against_host_libm.md) | The measurement that made the 22 268 ulp visible and now records the 3 |

### Sources

| File | Relationship |
|------|--------------|
| `src/inverse_circular.rs` | `atan` (all three stages), `atan2`, `asin`, `acos`, `clamp_unit` |
| `src/algebraic.rs` | `atan_series` — kept here because the logarithm's `atanh_series` is the same shape and duplicating a coefficient loop is how two copies come to disagree |
| `src/constant.rs` | `ATAN_B`, `ATAN_V`, `ATAN_DIRECT`, `ASIN_DIRECT` |

### Tests

| File | Relationship |
|------|--------------|
| `src/inverse_circular.rs` — `mod tests` | `the_bin_table_holds_the_arctangent_of_each_centre` (re-derived by bisection on `tan`), `the_direct_path_and_the_reduced_path_agree_at_the_seam`, `the_direct_thresholds_are_the_powers_of_two_they_claim_to_be`, `clamp_unit_is_a_boundary_not_a_rescale` |
| `src/algebraic.rs` — `mod tests` | `the_two_series_agree_with_their_own_definitions` — both series rebuilt term by term at run time |
| `tests/inc/inverse_circular_test.rs` | `the_small_argument_band_keeps_every_digit` (bit-identity below `1e-9`), `atan_joins_its_two_strategies_without_a_step`, `atan2_lands_on_the_axes_exactly`, `asin_and_acos_clamp_rather_than_returning_nan` |
