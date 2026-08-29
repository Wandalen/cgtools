# Algorithm: Logarithm By Mantissa Split

### Scope

- **Purpose**: Document the exact split that turns any positive double into an integer exponent and a mantissa near `1`, and how `ln`, `log2`, `log10` and `log` are each assembled from it.
- **Responsibility**: Specify the split, the re-centring that distinguishes it from the format's own, the `atanh` series, and why three of the four logarithms are exact on their own powers.
- **In Scope**: `mantissa_exponent`, `ln_mantissa`, `atanh_series`, `ln`, `log2`, `log10`, `log`.
- **Out of Scope**: `ln_1p`, which exists for an argument this reduction cannot help with (see [004](004_cancellation_free_hyperbolic_forms.md)); the subnormal defect this split carried (see [pitfall/003](../pitfall/003_subnormal_mantissa_extraction.md)).

### Abstract

`ln( m · 2^k ) = k·ln 2 + ln m`. The exponent term is an integer times a
constant — no series needed — and the mantissa term is a logarithm on a range
narrow enough for a short series. So the whole problem reduces to reading `k`
and `m` out of the bit pattern, which costs a shift and a mask and rounds
nothing at all.

Two choices distinguish this from the textbook version. The mantissa is
re-centred onto `[ √½, √2 )` rather than the `[ 1, 2 )` the format hands over.
And the series is `atanh` rather than the direct `ln( 1 + u )` expansion —
`ln m = 2·atanh( (m−1)/(m+1) )`, which converges in half the terms because the
`atanh` series has only odd powers.

### Algorithm

**Step 1 — split, exactly.** `mantissa_exponent( x )` returns `( m, k )` with
`x = m · 2^k` bit-for-bit:

```text
if x < f64::MIN_POSITIVE :  x ← x · 2⁵⁴ ,  lifted = 54     ( see below )
k = ( ( bits >> 52 ) & 0x7ff ) − 1023 − lifted
m = bits with the exponent field replaced by 1023          ( i.e. m ∈ [ 1, 2 ) )
if m > √2 :  m ← m · 0.5 ,  k ← k + 1                       ( re-centre )
```

Nothing here rounds. `k` is read from the exponent field; `m` is the same
mantissa bits with a different exponent; the halving is exact; the `2⁵⁴` lift is
exact because a subnormal carries at most 52 significant bits and scaling only
shifts its exponent. Asserted by reassembling — `scale2( m, k )` must return the
identical bits `x` went in with.

**The lift is not an optimisation.** A subnormal has a zero exponent field and
*no implicit leading bit*, so the extraction above — which restores that bit
unconditionally — would read its mantissa as a normal one and return an exponent
up to 51 too high. `log2( 5e-324 )` came back `−1023` instead of `−1074`. That
is [pitfall/003](../pitfall/003_subnormal_mantissa_extraction.md), and it is a
wrong answer rather than a precision loss.

**Step 2 — re-centre.** Mapping `m` onto `[ √½, √2 )` instead of `[ 1, 2 )` is
what keeps the series argument small on *both* sides of `1`. With `[ 1, 2 )` the
series argument `s = (m−1)/(m+1)` runs to `1/3`; centred, `| s |` stays under
`0.1716`, which is why eleven terms reach full precision.

More importantly it leaves no cancellation for the reassembly to amplify. This
is the property an `atanh`-series logarithm most needs and most easily loses:
across the 10 000 consecutive doubles astride `1.0` the error is at most 1 ulp,
and `ln( 1.0 )` is exactly `0.0`.

**Step 3 — the series.** `ln_mantissa( m ) = 2 · atanh_series( (m−1)/(m+1) )`,
eleven terms, first omitted term `s²³/23`.

`atanh_series` lives beside `atan_series` in `algebraic.rs` rather than beside
its caller, because the two differ only in whether the signs alternate.
Duplicating a coefficient loop is how two copies of it come to disagree.

**Step 4 — reassemble**, differently per function:

```text
ln    :  k·LN2_HI + ( k·LN2_LO + ln_mantissa( m ) )
log2  :  k + ln_mantissa( m )·LOG2_E
log10 :  k·LOG10_2 + ln_mantissa( m )·LOG10_E
log   :  ln( x ) / ln( base )
```

`ln`'s reassembly uses the same two-part `ln 2` split as the exponential's
reduction ([001](001_cody_waite_range_reduction.md)), and for the same reason:
`k` reaches ±1074, and a single-double `ln 2` loses about as many bits as `k`
has.

### Why Three Of Them Are Exact On Their Own Powers

`log2( 1024.0 )` is exactly `10.0`, and `log2( f64::MIN_POSITIVE )` is exactly
`−1022.0`. The exponent term is an *integer added to a small number*, so when
the mantissa is `1` the series returns zero and nothing rounds.

The composition `ln( x ) / LN_2` cannot do this. It rounds a large logarithm
before dividing, and returns something a hair off an integer — invisible until a
caller floors it. `log10` splits the same way for the same reason, one exact
multiplication rather than a division after a logarithm.

`log( x, base )` is the exception and is documented as such: genuinely two
logarithms and a division, carrying roughly twice their rounding error, with no
reduction that avoids it. Where the base is 2 or 10, the dedicated function is
both faster and exact on powers; where it is a runtime value, this is the honest
cost.

### Verify It Yourself

```sh
cargo test -p deterministic_math mantissa_exponent_is_an_exact_split
cargo test -p deterministic_math log2_and_log10_are_exact_on_their_own_powers
cargo test -p deterministic_math the_logarithms_accept_subnormal_arguments
```

The first reassembles every sampled split and demands bit-identity. The third is
the regression guard for the subnormal defect, and fails by a factor of `2⁵¹` if
the lift is removed.

### Invariants

| File | Relationship |
|------|--------------|
| [../invariant/004_refusal_over_meaningless_answer.md](../invariant/004_refusal_over_meaningless_answer.md) | The domain guards ahead of the split — negative gives `NaN`, zero gives `−INFINITY` |

### Algorithms

| File | Relationship |
|------|--------------|
| [001_cody_waite_range_reduction.md](001_cody_waite_range_reduction.md) | The inverse direction, and the source of the shared `LN2_HI`/`LN2_LO` split step 4 depends on |
| [004_cancellation_free_hyperbolic_forms.md](004_cancellation_free_hyperbolic_forms.md) | `ln_1p`, which reuses `atanh_series` directly rather than going through this split at all |
| [005_cube_root_by_exponential_round_trip.md](005_cube_root_by_exponential_round_trip.md) | `cbrt`, whose cost is one `ln` through this path plus one `exp` back |

### Pitfalls

| File | Relationship |
|------|--------------|
| [../pitfall/003_subnormal_mantissa_extraction.md](../pitfall/003_subnormal_mantissa_extraction.md) | The defect step 1's lift closes, and why the old test sweep could not see it |

### Sources

| File | Relationship |
|------|--------------|
| `src/exponential.rs` | `mantissa_exponent`, `ln_mantissa`, `ln`, `log2`, `log10`, `log` |
| `src/algebraic.rs` | `atanh_series`, shared with `ln_1p` |
| `src/constant.rs` | `LN2_HI`/`LN2_LO`, `LOG2_E`, `LOG10_2`, `LOG10_E` |

### Tests

| File | Relationship |
|------|--------------|
| `src/exponential.rs` — `mod tests` | `mantissa_exponent_is_an_exact_split` (reassembly is bit-identical), `ln_mantissa_inverts_exp_reduced_across_the_recentred_range`, `the_named_log_constants_are_consistent` |
| `src/algebraic.rs` — `mod tests` | `the_two_series_agree_with_their_own_definitions` |
| `tests/inc/exponential_test.rs` | `log2_and_log10_are_exact_on_their_own_powers`, `the_logarithms_accept_subnormal_arguments`, `the_logarithms_reject_what_has_no_logarithm`, the `ln`/`exp` inversion and cross-logarithm agreement tests |
