# Algorithm: Cody-Waite Range Reduction

### Scope

- **Purpose**: Document the shared reduction that makes a short Taylor series sufficient for `exp`, `exp2`, `sin`, `cos`, `tan` and `sin_cos` — strip an integer multiple of a constant, evaluate the series on what is left, reassemble.
- **Responsibility**: Specify the reduction, the multi-part constant splits it depends on, the reassembly, and where it runs out.
- **In Scope**: `exp`, `exp2`, `sin_cos` and their helpers `round_half_away`, `exp_reduced`, `sin_reduced`, `cos_reduced`, `scale2`; the `LN2_HI`/`LN2_LO` and `PIO2_HI`/`PIO2_MD`/`PIO2_LO` splits.
- **Out of Scope**: The logarithm's own reduction, which strips an exponent rather than a multiple (see [003](003_logarithm_by_mantissa_split.md)); the arctangent's, which reduces onto a bin centre (see [002](002_arctangent_bin_reduction.md)).

### Abstract

A Taylor series converges quickly near its expansion point and slowly away from
it. `exp( 20 )` by direct summation needs dozens of terms and loses digits to
cancellation on the way; `sin( 100 )` never converges usefully at all. The cure
is periodicity or its exponential analogue: write `x = k·C + r` where `C` is
`ln 2` or `π/2`, `k` is an integer, and `| r |` is small — then evaluate the
series on `r`, where sixteen terms reach full precision, and put `k` back
afterwards by an operation that costs nothing.

The whole difficulty is in forming `r`. `k` can be a few thousand, and
`x − k·C` computed with a single-double `C` loses about as many bits as `k` has,
which is precisely the range where the reduction was supposed to help. The
Cody-Waite technique splits `C` into parts whose leading one is exactly
representable in the top mantissa bits, so `k · C_hi` is *exact* and each
subsequent part corrects what the previous one could not carry.

### Algorithm

**Step 1 — choose `k`.** `k = round_half_away( x · 1/C )`, where the reciprocal
is a compile-time constant (`LOG2_E` for `exp`, `INV_PIO2` for `sin_cos`;
`exp2` needs no scaling since `C = 1` in its own base). Rounding to *nearest*
rather than truncating is what bounds `| r |` by `C/2` rather than `C`, halving
the series' range.

`round_half_away` is written out rather than calling `f64::round`, which has
exactly these semantics and is exact for every value the reductions produce —
but is a libm entry point on some targets, and the two comparisons that replace
it are not ([invariant/002](../invariant/002_pinned_operations_only.md)).

**Step 2 — form the residual by successive subtraction.**

```text
exp     : r = x − k·LN2_HI − k·LN2_LO                       | r | ≤ ln 2 / 2 ≈ 0.347
exp2    : r = x − k          then  r·LN2_HI + r·LN2_LO       | r | ≤ 0.5, scaled ≤ 0.347
sin_cos : r = x − k·PIO2_HI − k·PIO2_MD − k·PIO2_LO          | r | ≤ π/4  ≈ 0.785
```

Two parts suffice for `ln 2` because `k` there is bounded by the exponent range,
about ±1075. Three are needed for `π/2` because `x` may be a few thousand
radians and `k` correspondingly larger, and a two-part split starts losing bits
in that range. The parts sum to the correctly-rounded constant — asserted
bit-exactly, and the high part is additionally asserted to have its low 26
mantissa bits clear, since a `PIO2_HI` that is *not* exactly representable in
the top bits makes `k · PIO2_HI` round and the split buys nothing.

**Step 3 — evaluate the series on `r`**, in Horner form, with coefficients from
`RECIP_FACT`:

- `exp_reduced( r )` — sixteen terms of `Σ rⁿ/n!`. The first omitted term is
  `r¹⁷/17!`, under `1e-20` at the range edge, so the truncation error sits well
  below the rounding error of evaluating the polynomial.
- `sin_reduced( r )` / `cos_reduced( r )` — Horner on `r²` rather than `r`,
  which halves the multiply count and is exact to do because both series contain
  only odd or only even powers.

**Step 4 — reassemble.**

- Exponential: `scale2( series, k )` multiplies by `2^k`, which for `k` inside
  the exponent field is one exact multiplication —
  see [pitfall/002](../pitfall/002_exponent_field_wraparound.md) for the case
  where it is not.
- Circular: `k mod 4` selects which of four sign-and-swap pairings to emit —
  `( s, c )`, `( c, −s )`, `( −s, −c )`, `( −c, s )`. A wrong entry here is
  invisible to `sin² + cos² = 1`, since every pairing satisfies it, so the
  quarter-turn values are checked directly instead.

### One Reduction, Both Outputs

`sin_cos` runs the reduction once and evaluates *both* polynomials on the same
residual. This is not only cheaper than two calls — measured at 1.14× the host's
`sin_cos` against 1.40× for `sin` alone
([non_functional_requirement/001](../non_functional_requirement/001_cost_against_host_libm.md))
— it is the reason the pair always agree. `sin² + cos² = 1` holds to rounding
rather than to twice the reduction error, because there is only one reduction.

`sin` and `cos` are wrappers that discard half the pair, and are documented as
such; the tests assert their agreement with `sin_cos` bit-exactly, so the three
cannot drift apart.

### Why `exp2` Is Not `exp( x · ln 2 )`

That composition rounds the product before exponentiating it, and the rounding
is proportional to `x`. Reducing in base two directly instead moves the integer
part of `x` into the exponent field untouched and sends only the fraction to a
series — so `exp2` of any integer is exact, across the entire domain from
`k = −1074` to `k = 1023`. Asserted for every one of those integers.

### Where It Runs Out

Past `SIN_COS_MAX` (`1e8`) the three-part `π/2` split stops carrying enough
bits for the residual to mean anything, and `x · INV_PIO2` approaches the `i32`
that `k` is held in. The functions refuse rather than answer — see
[invariant/004](../invariant/004_refusal_over_meaningless_answer.md), which
records what the unenforced version of that limit actually did.

The exponential has no equivalent cliff: its `k` is bounded by the exponent
range, and arguments beyond `[ −745, 709 ]` are answered with `0.0` or
`INFINITY`, which are the correct results rather than refusals.

### Verify It Yourself

The reduction is internal, but its two load-bearing constants are checkable in
one line each:

```sh
cargo test -p deterministic_math the_three_part_split_reassembles_to_pi_over_two
cargo test -p deterministic_math exp2_is_exact_on_integers
```

The first asserts `PIO2_HI + PIO2_MD + PIO2_LO` is bit-identical to
`FRAC_PI_2` and that `PIO2_HI`'s low 26 mantissa bits are zero. The second
asserts `exp2( k ) == 2^k` exactly for every representable integer exponent —
which is the property the base-two reduction exists to provide.

### Invariants

| File | Relationship |
|------|--------------|
| [../invariant/002_pinned_operations_only.md](../invariant/002_pinned_operations_only.md) | Why `round_half_away` exists rather than a call to `f64::round` |
| [../invariant/004_refusal_over_meaningless_answer.md](../invariant/004_refusal_over_meaningless_answer.md) | What happens past this reduction's range, and the incident that made it enforced |

### Algorithms

| File | Relationship |
|------|--------------|
| [002_arctangent_bin_reduction.md](002_arctangent_bin_reduction.md) | The same idea with a table of centres instead of one constant, for a function with no periodicity to exploit |
| [003_logarithm_by_mantissa_split.md](003_logarithm_by_mantissa_split.md) | The inverse direction — an exact split read out of the bit pattern rather than computed |

### Pitfalls

| File | Relationship |
|------|--------------|
| [../pitfall/002_exponent_field_wraparound.md](../pitfall/002_exponent_field_wraparound.md) | Step 4's reassembly, and the defect it carried for every `k` outside the exponent field |
| [../pitfall/001_small_argument_cancellation.md](../pitfall/001_small_argument_cancellation.md) | The general failure mode of reducing an argument that was already small — why `atan` has a bypass and these do not need one |

### Sources

| File | Relationship |
|------|--------------|
| `src/circular.rs` | `sin_cos` (steps 1–4), `sin_reduced`, `cos_reduced` |
| `src/exponential.rs` | `exp`, `exp2`, `exp_reduced` |
| `src/algebraic.rs` | `round_half_away` (step 1), `scale2` (step 4) |
| `src/constant.rs` | `LN2_HI`/`LN2_LO`, `PIO2_HI`/`PIO2_MD`/`PIO2_LO`, `INV_PIO2`, `LOG2_E`, `RECIP_FACT`, `SIN_COS_MAX` |

### Tests

| File | Relationship |
|------|--------------|
| `src/circular.rs` — `mod tests` | `the_three_part_split_reassembles_to_pi_over_two`, `the_reduced_series_match_their_own_definitions` (series rebuilt from run-time factorials) |
| `src/exponential.rs` — `mod tests` | `exp_reduced_matches_its_own_series_definition` |
| `src/algebraic.rs` — `mod tests` | `round_half_away_breaks_ties_away_from_zero`, `scale2_is_exact_multiplication_by_a_power_of_two` |
| `tests/inc/circular_test.rs` | `the_quadrant_reassembly_lands_on_the_right_branch` (step 4's four-way selection), `sin_cos_agrees_with_the_separate_calls` |
| `tests/inc/exponential_test.rs` | `exp2_is_exact_on_integers`, `exp2_matches_libm_across_its_whole_domain` |
