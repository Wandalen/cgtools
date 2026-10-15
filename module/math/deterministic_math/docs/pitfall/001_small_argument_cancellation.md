# Pitfall: Small-Argument Cancellation

### Scope

- **Purpose**: Record the single defect class that accounted for every large accuracy disagreement this crate has had, why the obvious tests miss it entirely, and the shape of the fix in all seven cases.
- **Responsibility**: Document the trap, its measured failure, its mitigation, and the instrument that finds it.
- **In Scope**: Any function whose formula constructs a quantity near `1` and then removes it, or whose argument reduction is applied to an argument that was already reduced.
- **Out of Scope**: The constructive rewrites themselves, which are [algorithm/004](../algorithm/004_cancellation_free_hyperbolic_forms.md) and [algorithm/002](../algorithm/002_arctangent_bin_reduction.md).

### Trap

Believing a function is accurate because it satisfies its identities.

`sinh` satisfying `cosh² − sinh² = 1`, `atan` inverting `tan`, `ln_1p` agreeing
with `ln( 1 + x )` — every one of these held while the function was wrong by
tens of thousands of ulp. They hold because both sides of the identity are
computed by the same machinery on the same argument, so a systematic error in
that machinery cancels out of the comparison. The identity is self-consistent
and says nothing about accuracy.

The trap has a second half, which is what makes it survive review: the error is
invisible at the argument sizes anyone tests by hand. `sinh( 0.5 )` was always
correct to the last bit. The defect appeared only below `| x | ≈ 1e-4` and grew
steadily as the argument shrank, which is precisely the region a sweep over
`[ −1, 1 ]` samples least and an eyeball check never visits.

### Failure

Two distinct mechanisms, one symptom.

**Cancellation in a closed form.** `sinh x = ( eˣ − e⁻ˣ )/2` computes two
numbers both close to `1` and subtracts them. For small `x` the leading bits
agree and cancel, and what survives is not the answer but whatever the two
roundings left behind. At `x = 1e-16`, `exp( x ) − 1.0` returns exactly `0.0`
where the true answer is `1e-16` — every digit gone.

**Reduction applied to an already-small argument.** `atan` reduces onto a bin
centre and computes `ATAN_V[ 0 ] + atan_series( u )`. For `t` near zero those
two terms are equal and opposite to within the size of `t`, so the leading
digits cancel and what survives is the rounding of `0.1243…`. The reduction
that makes the series converge fast for a mid-range argument is exactly what
destroys a small one.

Measured against the platform libm, worst case over 200 000 samples:

| function | before | after | mechanism |
|----------|-------:|------:|-----------|
| `ln_1p` | 32 768 | 5 | `1 + x` rounds `x`'s low bits away before `ln` sees them |
| `exp_m1` | 31 203 | 4 | forms a value near `1`, subtracts `1` |
| `asin` | 22 268 | 3 | inherits `atan`'s reduction |
| `atan2` | 8 883 | 3 | inherits `atan`'s reduction |
| `sinh` | 7 163 | 4 | difference of two exponentials |
| `asinh` | 2 628 | 4 | `ln` of an argument tending to `1` |
| `tanh` | 1 244 | 4 | numerator is a difference of two exponentials |
| `atan` | 95 | 2 | bin reduction near zero |

A related case is worth listing separately because it was found by the same
sweep but is a rounding cost rather than a cancellation: `asin` retained a
**1 ulp** error below `2⁻²⁷` even after the reduction fix, because building
`√( ( 1−c )( 1+c ) )` for a tiny `c` rounds `1−c` and `1+c` separately and their
product lands a step either side of `1`. `asin( 3.7e-12 )` came back one ulp
above `3.7e-12`, where the correct result is the argument itself. Closed by
`ASIN_DIRECT`, a threshold below which `asin` *is* `atan`.

### Why It Escapes Ordinary Testing

Three reinforcing reasons, each of which has to be defeated separately:

- **Identity tests cancel the error.** See Trap above.
- **Relative error is the only bound that shows it.** An absolute bound is
  trivially satisfied by anything near zero, including a completely wrong
  answer. `| sinh( 1e-9 ) − true | < 1e-15` passes even when every significant
  digit is gone.
- **Linear sweeps do not reach the region.** `sweep( −1.0, 1.0, 300 )` steps by
  `6.7e-3` and straddles zero rather than landing on it, so its smallest
  nonzero sample is `3.3e-3`. The defect starts two decades below that and is
  worst eight decades below it — no linear sweep of that interval reaches it at
  any sample count worth running, because halving the step only buys one sample
  closer per doubling.

### Mitigation

**For a caller.** Prefer `exp_m1` and `ln_1p` over `exp( x ) − 1` and
`ln( 1 + x )` whenever `x` may be small — that is what they are for, and it is
the argument they exist for rather than a weakness in `exp` or `ln`. Note that
no accuracy inside `ln` can rescue a caller who forms `1.0 + u` *before* calling
it: that addition has already discarded `u`'s low bits.

**For this crate.** Every affected function carries a branch whose only job is
to compute the same quantity without ever forming the `1` that the subtraction
would have had to remove
([algorithm/004](../algorithm/004_cancellation_free_hyperbolic_forms.md)), or —
for `atan` and `asin` — a threshold below which the reduction is simply skipped
([algorithm/002](../algorithm/002_arctangent_bin_reduction.md)).

**For anyone adding a function.** Test relatively, across decades, and compare
against something that is not built from the same machinery. Concretely, the
three instruments that found all of this:

```sh
cargo run -p deterministic_math --release --example cost_vs_libm
```

prints a small-argument sweep over eight decades — a flat row means the relative
accuracy holds all the way down, a row fanning out as the argument shrinks is
this defect — plus two amplification checks, where the caller's own arithmetic
magnifies whatever error the function returned. The `( sinh( s ) − s )/s³` check
is the sharpest: its subtraction multiplies `sinh`'s error by roughly `1/s²`,
which is how a defect at `1e-6` in the output becomes an `8e-5` relative error
in something a caller actually computes.

And in the test suite, each family carries a
`the_small_argument_band_keeps_every_digit` that asserts bit-identity where the
function is its own argument to the last place — the strongest form of the
check, and one no tolerance can be loosened to pass.

### Algorithms

| File | Relationship |
|------|--------------|
| [../algorithm/004_cancellation_free_hyperbolic_forms.md](../algorithm/004_cancellation_free_hyperbolic_forms.md) | The rewrite table — this document's constructive half |
| [../algorithm/002_arctangent_bin_reduction.md](../algorithm/002_arctangent_bin_reduction.md) | The bypass thresholds, for the two functions where the fix was removing a reduction rather than rewriting a formula |

### Non Functional Requirements

| File | Relationship |
|------|--------------|
| [../non_functional_requirement/002_accuracy_against_host_libm.md](../non_functional_requirement/002_accuracy_against_host_libm.md) | The instrument, the standing measurement, and the same before/after table in its accuracy-ceiling context |

### Pitfalls

| File | Relationship |
|------|--------------|
| [002_exponent_field_wraparound.md](002_exponent_field_wraparound.md) | Found by widening the same test coverage; a defect the accuracy sweep could *not* see, because its domain never reached the failing range |
| [003_subnormal_mantissa_extraction.md](003_subnormal_mantissa_extraction.md) | Likewise — the other defect this crate's extraction exposed |

### Sources

| File | Relationship |
|------|--------------|
| `src/hyperbolic.rs` | The module header stating this trap as the reason every function there has a second branch |
| `src/exponential.rs` | `exp_m1`, `ln_1p`, and `SERIES_BAND`'s derivation |
| `src/inverse_circular.rs` | `atan`'s small-argument path and `asin`'s bypass |
| `src/constant.rs` | `SERIES_BAND`, `ATAN_DIRECT`, `ASIN_DIRECT` — each recording the measurement that set it |
| `examples/cost_vs_libm.rs` | `small_argument_sweep()` and `amplification()` |

### Tests

| File | Relationship |
|------|--------------|
| `tests/inc/hyperbolic_test.rs` | `the_small_argument_band_keeps_every_digit` — bit-identity below `1e-20` for `sinh`, `tanh`, `asinh`, `atanh`, and `cosh == 1.0` |
| `tests/inc/inverse_circular_test.rs` | `the_small_argument_band_keeps_every_digit` — bit-identity below `1e-9` for `atan` and `asin` |
| `tests/inc/circular_test.rs` | `sin_keeps_its_relative_accuracy_near_zero` — relative, across 300 decades |
| `tests/inc/exponential_test.rs` | The `exp_m1` and `ln_1p` cancellation tests over 300 decades |
| `tests/inc/internal_test.rs` | `the_direct_thresholds_are_the_powers_of_two_they_claim_to_be` — the seam, with a bound tracking `c²/2` because the two functions genuinely differ by that much |
