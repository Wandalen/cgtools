# Pitfall: Subnormal Mantissa Extraction

### Scope

- **Purpose**: Record a defect in `mantissa_exponent` — the bit-level split every logarithm is built on — where a subnormal argument's exponent was read up to 51 powers of two too high, and the logarithm returned a confidently wrong number.
- **Responsibility**: Document the format detail that causes it, the returned value, why a wide sweep still missed it, and the lift that closes it.
- **In Scope**: `mantissa_exponent`, and its four consumers `ln`, `log2`, `log10`, `log`.
- **Out of Scope**: The series that consumes the split — see [algorithm/003](../algorithm/003_logarithm_by_mantissa_split.md).

### Trap

Assuming every `f64` has an implicit leading mantissa bit.

Reading a float apart is a three-line idiom, and it is correct for the
overwhelming majority of values:

```rust
let k = ( ( bits >> 52 ) & 0x7ff ) as i32 - 1023;              // unbias the exponent
let m = f64::from_bits( ( bits & 0x000f_ffff_ffff_ffff ) | ( 1023u64 << 52 ) );
```

The `| ( 1023u64 << 52 )` restores the leading `1` that the format leaves out —
normal `f64`s store `1.xxx` and keep only the `xxx`, so the bit has to be put
back by hand.

**Subnormals do not have it.** Below `2⁻¹⁰²², ` the format switches
representations: the exponent field is zero, the implicit bit is *absent*, and
the stored mantissa is the whole significand. The idiom above is not
approximately right there — it restores a bit that was never elided and unbiases
an exponent field that is not an exponent. Both halves of the split come back
wrong, and neither the compiler nor a runtime check has anything to object to.

### Failure

`log2( 5e-324 )` returned **`-1023`**, where the answer is `-1074`.

Off by 51 powers of two — a factor of `2.3e15` — and returned as a clean,
finite, entirely plausible number. Nothing about `-1023` looks like an error
value.

The mechanism. Every subnormal has a zero exponent field, so
`( bits >> 52 ) & 0x7ff` is `0` and the unbiasing yields `k = -1023` *for every
subnormal alike* — the smallest one and the largest one produce the same
exponent. All the information distinguishing `2⁻¹⁰⁷⁴` from `2⁻¹⁰²³` lives in the
mantissa bits, and forcing the implicit bit on top of them discards exactly
that. For `5e-324` the stored mantissa is a single set bit at the bottom; with
the implicit bit forced on it reads as `1 + 2⁻⁵²`, so the split reported
`1.0000000000000002 × 2⁻¹⁰²³` for a number that is `1 × 2⁻¹⁰⁷⁴`.

The error is worst at the bottom of the subnormal range and shrinks toward the
top, reaching zero at `2⁻¹⁰²²` where normal representation resumes. That
gradient is the second half of the trap: a test that samples the range's upper
edge sees a small discrepancy and reads it as ordinary rounding.

### Why It Escapes Ordinary Testing

The logarithms were swept across three hundred decades, and every sample was
correct. `1e-300` is a perfectly ordinary normal number — the subnormal range
does not begin until `2.2e-308`, and it is only `16` decades deep out of the
`632` an `f64` spans. A sweep by decade that stops at `1e-300`, or one that
starts from a "very small" constant chosen by eye, lands short of it every time.

Round-trip tests do not help either, and this is the part worth internalising:
`exp2( log2( x ) )` was *self-consistent* through the defect. `log2` reported
`-1023`, `exp2` faithfully evaluated `2⁻¹⁰²³`, and the two agreed with each other
about a number 51 powers of two away from the input. The identity holds because
both functions share the same mistaken idea of where the argument sits — which
is the same reason identity tests miss cancellation
([001](001_small_argument_cancellation.md)). An identity between two functions
built on one broken helper tests the helper against itself.

What found it was widening the sweep until it crossed a *representation*
boundary rather than a magnitude one. The format changes behaviour at
`f64::MIN_POSITIVE`, and no amount of sampling on one side of that boundary says
anything about the other.

### Mitigation

Lift any subnormal into the normal range before touching its bits, and subtract
the lift back out of the exponent:

```rust
/// Enough to lift the smallest subnormal, `2^-1074`, clear of `2^-1022`.
const LIFT : i32 = 54;

let ( x, lifted ) = if x < f64::MIN_POSITIVE { ( scale2( x, LIFT ), LIFT ) } else { ( x, 0 ) };
// ... then the ordinary extraction, with `- lifted` folded into `k`.
```

**The lift is exact, which is what makes this a fix rather than a mitigation.** A
subnormal carries at most 52 significant bits, and multiplying by `2⁵⁴` only
shifts its exponent — no bit falls off the bottom, nothing rounds. So the lifted
value is the *same number*, and the split of it is the split of the original with
a known integer offset. Subtracting `LIFT` from `k` recovers the original
exactly, rather than approximately.

**Why 54 and not 52.** `2⁻¹⁰⁷⁴ × 2⁵²` is exactly `2⁻¹⁰²²`, which is
`f64::MIN_POSITIVE` — normal, but sitting precisely on the boundary the branch
is testing against. Two extra powers put every lifted value unambiguously clear
of it, so no argument can land on the seam.

The branch condition is `x < f64::MIN_POSITIVE`, the format's own definition of
subnormal, rather than a hand-chosen threshold. Callers have already excluded
zero, negatives, infinities and `NaN`
([invariant/004](../invariant/004_refusal_over_meaningless_answer.md)), so the
comparison sees only positive finite values and the branch means exactly what it
says.

### Verify It Yourself

```rust
use deterministic_math::{ ln, log2, log10, exp2 };

// The exact failing argument: the smallest positive f64.
assert_eq!( log2( 5.0e-324 ), -1074.0 );

// Exact on every subnormal power of two, where the answer is a statable integer.
for k in -1074 ..= -1023
{
  let x = f64::exp2( f64::from( k ) );
  assert!( x < f64::MIN_POSITIVE );        // guard: these really are subnormal
  assert_eq!( log2( x ), f64::from( k ) );
}

// And the boundary itself is continuous — no step where the branch switches.
assert_eq!( log2( f64::MIN_POSITIVE ), -1022.0 );
assert_eq!( log2( f64::MIN_POSITIVE / 2.0 ), -1023.0 );

// ln and log10 come along, being the same split scaled.
assert!( ( ln( 5.0e-324 ) - f64::ln( 5.0e-324 ) ).abs() < 1.0e-12 );
assert!( ( log10( 5.0e-324 ) + 323.3062153431158 ).abs() < 1.0e-10 );

// The round trip, which the defect broke by 51 powers of two while still
// appearing self-consistent.
assert_eq!( exp2( log2( 5.0e-324 ) ), 5.0e-324 );
```

The `log2` assertions are exact equalities rather than tolerances, which is
available here because a subnormal power of two has an integer logarithm the
test can simply state — no reference implementation needed, and no tolerance
that could be loosened until the defect fits inside it.

### The Generalisable Lesson

**Sweep to representation boundaries, not to round numbers.** `1e-300` is a
round number; `f64::MIN_POSITIVE` is where the format changes what the bits
mean. Only the second kind of boundary can hide a defect of this shape, and a
sweep that stops at the first kind can be arbitrarily wide and still never reach
it.

The `f64` boundaries worth naming in a test: `MIN_POSITIVE` (normal/subnormal),
`MAX` and `2⁵²` (where integers stop being exactly representable), `1.0` (where
`atanh`-style series arguments change sign), and the two infinities. This crate
has a test crossing each of them.

### Pitfalls

| File | Relationship |
|------|--------------|
| [002_exponent_field_wraparound.md](002_exponent_field_wraparound.md) | The mirror image, and named as such in this test's own comment — that one produces a wrong subnormal *result*, this one reads a wrong subnormal *argument*. Its `scale2` is also the tool this fix lifts with, so the two defects are one call apart |
| [001_small_argument_cancellation.md](001_small_argument_cancellation.md) | Where the round-trip blindness described above is the same failure in its general form: an identity between two functions sharing a helper tests the helper against itself |

### Algorithms

| File | Relationship |
|------|--------------|
| [../algorithm/003_logarithm_by_mantissa_split.md](../algorithm/003_logarithm_by_mantissa_split.md) | The four-step procedure this split is step 1 of, including the `2⁵⁴` lift in its pseudocode |

### Invariants

| File | Relationship |
|------|--------------|
| [../invariant/004_refusal_over_meaningless_answer.md](../invariant/004_refusal_over_meaningless_answer.md) | The caller-side exclusions that let the subnormal branch test a bare `<` and mean it |
| [../invariant/002_pinned_operations_only.md](../invariant/002_pinned_operations_only.md) | Why the split is bit manipulation rather than a `frexp` call |

### Non Functional Requirements

| File | Relationship |
|------|--------------|
| [../non_functional_requirement/002_accuracy_against_host_libm.md](../non_functional_requirement/002_accuracy_against_host_libm.md) | The 8 ulp ceiling the logarithms hold, which is measured over the normal range and says nothing on its own about this one |

### Sources

| File | Relationship |
|------|--------------|
| `src/exponential.rs` | `mantissa_exponent`, its `LIFT`, and the doc comment recording this failure under its own `# Subnormals are lifted first` heading |
| `src/algebraic.rs` | `scale2`, which performs the lift — and whose own defect is [002](002_exponent_field_wraparound.md) |

### Tests

| File | Relationship |
|------|--------------|
| `tests/inc/exponential_test.rs` | `the_logarithms_accept_subnormal_arguments` — exact equality on all 52 subnormal powers of two, a relative bound across arbitrary subnormals from `decades( 3.7, 323 )`, and the `exp2 ∘ log2` round trip across the boundary that the defect broke by 51 powers of two |
| `tests/inc/internal_test.rs` | `mantissa_exponent_is_an_exact_split` — asserts `m · 2ᵏ` reassembles to the input bit-for-bit, which is the property the lift preserves and the unlifted extraction destroyed |
| `tests/inc/exponential_test.rs` | `the_logarithms_reject_what_has_no_logarithm` — the exclusions the subnormal branch relies on having already happened |
