# Algorithm: Cube Root By Exponential Round Trip

### Scope

- **Purpose**: Document how `cbrt` is computed, why it costs an `ln` and an `exp` rather than a dedicated kernel, and why the obvious substitution for it is not merely less accurate but wrong on half the domain.
- **Responsibility**: Specify the sign handling, the exponential round trip, the Newton refinement, and the accuracy the combination reaches.
- **In Scope**: `cbrt`.
- **Out of Scope**: The `ln` and `exp` it is built from (see [003](003_logarithm_by_mantissa_split.md) and [001](001_cody_waite_range_reduction.md)); `powf`, which shares the round trip but not the sign handling.

### Abstract

Every real number has exactly one real cube root, including every negative one.
`powf` does not: it routes through `ln`, which is undefined for a negative
argument, so `powf( x, 1.0/3.0 )` returns `NaN` for every `x < 0`. The identity
that looks like it should work is therefore not an approximation of `cbrt` — it
is silently wrong on half the domain, which is exactly the failure shape this
crate exists to make impossible.

So `cbrt` is a separate function. It applies odd symmetry around the sign, takes
the round trip on the magnitude, and then spends one Newton step buying back
what the round trip's two roundings cost.

### Algorithm

```text
if x == 0 or not finite :  return x            ( ±0, ±∞ and NaN pass through )
m = | x |
r = exp( ln( m ) / 3 )                          ( the round trip )
r = r − ( r − m / r² ) / 3                      ( one Newton step on r³ = m )
return  −r if x < 0 else r
```

**Odd symmetry, not a branch on the formula.** `cbrt( −x ) = −cbrt( x )`
exactly, so working on the magnitude and negating costs nothing and makes the
negative half bit-for-bit the mirror of the positive one.

**`ln( m ) / 3`, never `ln( m ) · ( 1.0/3.0 )`.** One third is not
representable, so forming it as an `f64` rounds the exponent *before* it is
used, and `exp` then amplifies that absolute error into a relative error in the
result. Dividing by the exact integer `3` instead leaves one rounding where
there would have been two.

**The Newton step.** `r³ = m` refined by Newton's method is
`r ← r − ( r − m/r² )/3`, which is the standard update rearranged so it costs
one division and one subtraction rather than a cube. It converges quadratically,
so a starting point already accurate to about 1e-16 relative reaches the
correctly-rounded answer in one step — which is what the measurement shows:
`cbrt` is within **1 ulp** across `[ −1e6, 1e6 ]`, better than `powf`'s 18 ulp
despite being built on the same round trip.

Notably the step reintroduces no libm call
([invariant/002](../invariant/002_pinned_operations_only.md)) — it is two
divisions, a multiplication and a subtraction, all pinned.

### The Cost

`cbrt` is the third most expensive function in the crate at **3.74×** the host's
own ([non_functional_requirement/001](../non_functional_requirement/001_cost_against_host_libm.md)),
and the reason is structural rather than fixable: it is a full `ln` plus a full
`exp` plus a Newton step, where the host has a bit-manipulation initial guess
and a short polynomial. A dedicated kernel — seed from the exponent field
divided by three, then two or three Newton steps — would likely be faster.

It is not written, because nothing in the crate's consumers calls `cbrt` in a
hot path and the current form is one composition of two already-tested
functions rather than a fourth reduction to get right. If that changes, this is
the place the alternative is recorded.

### Verify It Yourself

```rust
use deterministic_math::{ cbrt, powf };

// The substitution that looks equivalent, on the half of the domain where it is not.
assert!( powf( -27.0, 1.0 / 3.0 ).is_nan() );
assert!( ( cbrt( -27.0 ) + 3.0 ).abs() < 1.0e-12 );

// And the round trip, on both signs.
for x in [ 27.0_f64, -1e6, 1e-9, -0.5 ]
{
  let r = cbrt( x );
  assert!( ( r * r * r / x - 1.0 ).abs() < 1.0e-15 );
}
```

`cargo run -p deterministic_math --release --example cost_vs_libm` reports both
the 3.74× and the 1 ulp.

### Invariants

| File | Relationship |
|------|--------------|
| [../invariant/002_pinned_operations_only.md](../invariant/002_pinned_operations_only.md) | Why the Newton step is written out in arithmetic rather than calling anything |

### Algorithms

| File | Relationship |
|------|--------------|
| [003_logarithm_by_mantissa_split.md](003_logarithm_by_mantissa_split.md) | The `ln` half of the round trip |
| [001_cody_waite_range_reduction.md](001_cody_waite_range_reduction.md) | The `exp` half |
| [006_algebraic_compositions.md](006_algebraic_compositions.md) | `powi`, the other function that exists because `powf` is wrong for a negative base |

### Non Functional Requirements

| File | Relationship |
|------|--------------|
| [../non_functional_requirement/001_cost_against_host_libm.md](../non_functional_requirement/001_cost_against_host_libm.md) | The 3.74× and the note that the exponential family sits nearest the ceiling |

### Sources

| File | Relationship |
|------|--------------|
| `src/exponential.rs` | `cbrt`, and the `powf` whose domain restriction makes it necessary |

### Tests

| File | Relationship |
|------|--------------|
| `tests/inc/exponential_test.rs` | The `cbrt` negative-argument, odd-symmetry and cubes-back tests |
