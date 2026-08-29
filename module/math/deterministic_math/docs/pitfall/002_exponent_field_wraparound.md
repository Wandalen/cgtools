# Pitfall: Exponent Field Wraparound

### Scope

- **Purpose**: Record a defect in `scale2` — the crate's own `2ᵏ` multiplier — where an out-of-range exponent wrapped into the sign bit and returned a large negative number for an argument whose answer was zero.
- **Responsibility**: Document the mechanism, the returned value, why the ordinary test sweep could never reach it, and the guard now in place.
- **In Scope**: `scale2`, its three direct callers — `exp`, `exp2`, and the subnormal lift inside `mantissa_exponent` — and everything reaching it through `exp`: `powf`, `cbrt`, and all six hyperbolics.
- **Out of Scope**: The reduction that decides `k` in the first place — see [algorithm/001](../algorithm/001_cody_waite_range_reduction.md).

### Trap

Building `2ᵏ` from its bit pattern without checking that `2ᵏ` is representable.

`scale2( x, k )` multiplies by `2ᵏ` rather than calling `powi`, because a
multiply chain is something the optimiser may reorder and a single exact factor
is not. Constructing that factor is two lines — bias the exponent, shift it into
place:

```rust
let e = ( 1023i32 + k ) as u64;
f64::from_bits( e << 52 )
```

which is correct, and exact, for every `k` in `[ −1022, 1023 ]`. Outside that
window there is no `f64` equal to `2ᵏ` at all, so there is nothing for the code
to build — but nothing in the expression says so. It is total arithmetic: no
division, no root of a negative, no domain it refuses. Every input yields an
output, and the outputs are finite and plausible-looking, which is why nothing
downstream notices.

### Failure

`exp( -745.0 )` returned **`-9.12e292`**, where the answer is `4.94e-324`.

Not merely the wrong magnitude — the wrong *sign*, and wrong by six hundred
orders of magnitude, for the most-used transcendental in the crate.

The mechanism, step by step. The reduction picks `k = -1075`, so the biased
exponent is `1023 − 1075 = −52`. That is an `i32`, and `as u64` on a negative
`i32` sign-extends rather than saturating: `−52` becomes
`0xFFFF_FFFF_FFFF_FFCC`. Shifting left by 52 discards everything above the low
twelve bits and lands `0xFCC` across the top of the word — and the top bit of an
`f64` is not part of the exponent at all. It is the sign. The factor that should
have been `2⁻¹⁰⁷⁵` came out as `−2⁹⁷³`, and the polynomial's `O( 1 )` value
scaled that to `−9.12e292`.

The symmetric failure exists above: a `k` past `1023` pushes the biased exponent
beyond `2047`, which overruns the field from below and sets the same sign bit.

### Why It Escapes Ordinary Testing

It went unnoticed until `exp2` was added — and not because the coverage was
thin. `exp` was already swept over `[ -300, 300 ]`, three hundred times wider
than the band the accuracy comparison actually cares about, and across every
sample of it `scale2` is bit-exact.

The reason is the factor between argument and exponent. `exp` reaches `k` as
`round( x / ln 2 )`, so a sweep to `±300` produces `k` no further than `±433` —
well inside the window, however wide the sweep looks. Reaching `k = −1075`
through `exp` needs an argument near `−745`, which is past where the answer
stops being interesting and becomes `0` or `∞`: exactly the reasoning that
leaves a test point unwritten.

`exp2` has no such factor. Its `k` **is** its argument, so a sweep over its
natural domain hits `k = −1075` immediately. The same defect, in the same shared
helper, sat one function away from being unreachable and one function away from
being unmissable — and which of those it was had nothing to do with how
carefully either function was tested.

It is invisible to the accuracy instrument too. `examples/cost_vs_libm.rs`
samples each function over its useful domain and reports ulp differences; `exp`
at `−745` is not in that domain, and an ulp figure would be a strange way to
report a sign error in any case.

### Mitigation

`scale2` splits an out-of-window `k` in half, so both factors exist:

```rust
if ( -1022 ..= 1023 ).contains( &k )
{
  return x * pow2( k );
}
let half = k / 2;
x * pow2( half ) * pow2( k - half )
```

The range is checked *before* the factor is built, not after. A guard inspecting
the result for a flipped sign would be reading bits that are already corrupted,
and could not distinguish the wraparound from a legitimately negative operand.

**Why halving keeps it correctly rounded.** For an out-of-window `k`, `x` is
`O( 1 )` and `2^half` is comfortably representable, so the first product is
exact — no rounding at all. That leaves the second multiplication as the single
rounding in the whole operation, which is what a correctly-rounded result
requires. Gradual underflow through the subnormal range is preserved for the
same reason, rather than being flushed to zero a step early.

Saturation to `±∞` and `+0` is not coded; it falls out of ordinary float
multiplication once both factors are real numbers.

### Verify It Yourself

```rust
use deterministic_math::{ exp, exp2, powf };

// The exact failing argument. The answer is the smallest subnormal — not zero,
// and the crate reaches it bit-for-bit.
assert_eq!( exp( -745.0 ).to_bits(), 1 );
assert_eq!( exp( -745.0 ), f64::exp( -745.0 ) );

// Both ends, on functions that reassemble through `scale2`.
assert!( exp( 1000.0 ).is_infinite() );
assert_eq!( exp( -800.0 ), 0.0 );
assert_eq!( exp2( -1200.0 ), 0.0 );
assert!( exp2( 1200.0 ).is_infinite() );
assert_eq!( powf( 2.0, -2000.0 ), 0.0 );

// Gradual underflow survives — this is what the exactness of the first
// multiplication buys.
assert!( exp2( -1060.0 ) > 0.0 );
assert!( exp2( -1060.0 ) < f64::MIN_POSITIVE );
assert_eq!( exp2( -1060.0 ), f64::exp2( -1060.0 ) );
```

Against the single-factor implementation the first two are wrong-signed and
enormous rather than off by an ulp, so these assertions genuinely discriminate
between the two versions.

### The Generalisable Lesson

**A wide sweep is not a wide sweep of the thing that breaks.** `[ -300, 300 ]`
looks exhaustive and is, for the argument — but the quantity that had a failing
range was `k`, and the reduction divides by `ln 2` on the way there. Whatever
internal quantity a helper is actually indexed by, that is the one whose ends
need reaching; the argument's ends are a proxy, and the conversion factor
between them decides whether the proxy is any good.

The corollary is that a shared helper needs its own tests at its own domain
ends, not only the coverage it inherits from callers. `scale2` has them now —
`scale2_stays_correct_past_the_exponent_field` — and they are stated in `k`
directly, so no caller's reduction stands between the test and the failing
range.

### Pitfalls

| File | Relationship |
|------|--------------|
| [003_subnormal_mantissa_extraction.md](003_subnormal_mantissa_extraction.md) | The mirror image — that one *reads* an exponent field where this one *builds* one, and it fails on the same subnormal range this one now underflows through correctly |
| [001_small_argument_cancellation.md](001_small_argument_cancellation.md) | The other defect class, opposite in character: a steady accuracy loss the sweep could measure, where this is a total failure the sweep could not reach |

### Algorithms

| File | Relationship |
|------|--------------|
| [../algorithm/001_cody_waite_range_reduction.md](../algorithm/001_cody_waite_range_reduction.md) | Where `k` is chosen, and the reassembly step that consumes it |
| [../algorithm/004_cancellation_free_hyperbolic_forms.md](../algorithm/004_cancellation_free_hyperbolic_forms.md) | The saturation branches, which keep the hyperbolic functions out of the range where `k` gets extreme in the first place |

### Invariants

| File | Relationship |
|------|--------------|
| [../invariant/004_refusal_over_meaningless_answer.md](../invariant/004_refusal_over_meaningless_answer.md) | Why the fix saturates to `±∞`/`+0` rather than returning `NaN` — these are the correct answers, not refusals |

### Sources

| File | Relationship |
|------|--------------|
| `src/algebraic.rs` | `scale2` and `pow2`, with the fix and a doc comment recording this failure — including the note that it "mattered nowhere until `exp2` was added" |
| `src/exponential.rs` | `exp` and `exp2`, the two reassembly call sites; `mantissa_exponent`, the third caller, whose own use of `scale2` is the subject of [003](003_subnormal_mantissa_extraction.md); `powf` and `cbrt`, which reach it through `exp` |
| `src/hyperbolic.rs` | Reaches `scale2` only through `exp` and `exp_m1`, and is kept away from extreme `k` by its own saturation branches |

### Tests

| File | Relationship |
|------|--------------|
| `src/algebraic.rs` — `mod tests` | `scale2_stays_correct_past_the_exponent_field` — the direct guard, stated in `k` so no caller's reduction stands between it and the failing range. Bit-exact rather than tolerant, because a sign flip is not something a tolerance should be able to absorb; graded against `f64::exp2`, which reaches these values by an unrelated route, and explicitly *not* against `f64::powi`, which returns zero here because its reciprocal's denominator overflows first |
| `src/algebraic.rs` — `mod tests` | `scale2_is_exact_multiplication_by_a_power_of_two` — the in-window companion, `k` from `−1020` to `1021`, so the fast path cannot regress unnoticed while the guarded path passes |
| `tests/inc/exponential_test.rs` | `exp_stays_correct_where_the_result_is_subnormal` — 400 samples across `[ −745, −700 ]` asserting `got >= 0.0` before the ulp bound, so a sign flip is reported as itself; plus both domain ends |
| `tests/inc/exponential_test.rs` | `exp2_matches_libm_across_its_whole_domain` and `exp2_is_exact_on_integers` — the function that made the defect reachable, swept over its full `[ −1074, 1023 ]` and asserted bit-exact at every integer in it |
