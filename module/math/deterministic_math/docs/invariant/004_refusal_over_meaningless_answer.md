# Invariant: Refusal Over Meaningless Answer

Where a function's own machinery has run out of bits, it returns `NaN` rather
than a finite number. No exported function ever returns a plausible-looking
double that carries no information.

### Scope

- **Purpose**: Pin the decision that an out-of-range argument is refused rather than answered approximately, and record the incident that made it a checked property instead of a documented one.
- **Responsibility**: State the rule, enumerate every refusal site, and explain why `NaN` rather than a clamp, a panic or a best effort.
- **In Scope**: Domain and range limits of `sin_cos`/`sin`/`cos`/`tan`, `ln`/`log2`/`log10`, `powf`, `acosh`, `atanh`, `asin`/`acos`.
- **Out of Scope**: Accuracy inside the accepted domain (see [non_functional_requirement/002](../non_functional_requirement/002_accuracy_against_host_libm.md)).

### Invariant Statement

Every exported function either returns a value it can stand behind, or returns
`NaN` / the correct infinity. It never returns a finite value outside the
precision it documents.

`NaN` in always gives `NaN` out — asserted across the whole surface by
`tests/inc/contract_test.rs`'s `nan_propagates_rather_than_being_swallowed`.
A function that swallowed a `NaN` into a finite result would be the same defect
in the other direction: an answer with nothing behind it.

### The Refusal Sites

| Function | Refused | Returns | Why |
|----------|---------|---------|-----|
| `sin_cos`, `sin`, `cos`, `tan` | `\| x \| > 1e8`, `±INFINITY`, `NaN` | `NaN` | The three-part `PI/2` reduction runs out of bits and the quarter-turn count is an `i32` a larger argument saturates |
| `ln`, `log2`, `log10` | `x < 0`, `NaN` | `NaN` | No real logarithm exists |
| `ln`, `log2`, `log10` | `x == 0` | `−INFINITY` | The correct limit, not a refusal |
| `powf` | `x < 0` | `NaN` | A negative base to a real power is not a real number; for an integer power the caller wants `powi`, which accepts one |
| `acosh` | `x < 1` | `NaN` | Outside the domain |
| `atanh` | `\| x \| > 1` | `NaN` | Outside the domain |
| `atanh` | `\| x \| == 1` | `±INFINITY` | The correct limit at the pole |
| `asin`, `acos` | — | *clamps* | The deliberate exception, below |

### Why `NaN` And Not A Best Effort

Past `SIN_COS_MAX` there is no answer to give. Nothing about a `1e20`-radian
angle survives to the fifteenth digit — the argument's own last representable
step is larger than a full turn — so a finite return would be a number with no
information in it. Those propagate silently: they pass every `is_finite` check,
they plot, they compare, and they carry a completely arbitrary value into
whatever consumed them.

`NaN` is the value that does not do that. It fails the checks a caller already
writes, and it does so at the call that produced it.

### Why Not A Panic

The limit was once documented and not enforced, and the resulting behaviour is
the reason this is an invariant. An argument past the range did not produce a
poor answer — it produced an arithmetic overflow inside `round_half_away`,
which panics in a debug build and wraps in a release one. So the same input gave
an abort in one profile and a garbage angle in the other.

It was reached from a diverging Newton iterate: a solver that had wandered
handed a large angle to `sin`. That is exactly the situation in which a caller
has least reason to expect a math library to abort the process, and most reason
to want a value it can test and recover from.

### The Deliberate Exception: `asin` And `acos` Clamp

Both accept arguments slightly outside `[ −1, 1 ]` and clamp rather than
refusing. This is not an inconsistency with the rule above; it is the rule
applied to a different situation.

An argument of `1.0000000000000002` arriving at `asin` is almost always a
normalised dot product that rounded, and the true value was `1`. There *is* an
answer, and the clamp is wrong by less than the rounding that produced the
argument. Returning `NaN` would instead propagate a failure into a computation
that had not actually failed — and would do it silently, since the `NaN` appears
one call later than the rounding that caused it.

The distinction: `sin_cos( 1e20 )` refuses because no answer exists.
`asin( 1 + 2e-16 )` clamps because the answer exists and is known to a better
accuracy than the argument was.

### Violation Consequences

Removing a refusal does not produce a wrong answer at the call site — it
produces a plausible one. The value flows into a position, a threshold, a
transform, and surfaces as a physically impossible state some distance away with
nothing linking it back. This is the same shape as
[001](001_bit_reproducibility.md)'s failure mode, and for the same reason:
a number that is wrong but well-formed defeats every check written for numbers
that are ill-formed.

### Example

```rust
use deterministic_math::{ sin_cos, sin, asin };

// Past the reduction's range: refused, in both members of the pair.
assert!( sin_cos( 1.0e20 ).0.is_nan() );
assert!( sin( f64::INFINITY ).is_nan() );

// Just inside it: still answered.
assert!( sin( 5.0e7 ).is_finite() );

// And the clamp, which is the other decision, not a weaker one.
assert_eq!( asin( 1.0 + 2.0e-16 ), asin( 1.0 ) );
```

### Invariants

| File | Relationship |
|------|--------------|
| [001_bit_reproducibility.md](001_bit_reproducibility.md) | Shares the failure shape — a well-formed wrong number defeats the checks written for ill-formed ones |

### Algorithms

| File | Relationship |
|------|--------------|
| [../algorithm/001_cody_waite_range_reduction.md](../algorithm/001_cody_waite_range_reduction.md) | The reduction whose exhaustion sets `SIN_COS_MAX`, and the `i32` quarter-turn count that would saturate past it |

### Sources

| File | Relationship |
|------|--------------|
| `src/constant.rs` | `SIN_COS_MAX`, with the reasoning for its value |
| `src/circular.rs` | `sin_cos`'s range guard, ahead of the reduction |
| `src/exponential.rs` | `ln`/`log2`/`log10`/`powf` domain guards |
| `src/hyperbolic.rs` | `acosh` and `atanh` domain guards and pole handling |
| `src/inverse_circular.rs` | `clamp_unit` — the deliberate exception |

### Tests

| File | Relationship |
|------|--------------|
| `tests/inc/circular_test.rs` | `the_range_limit_is_enforced_rather_than_merely_documented` — refusal past the limit *and* an answer just inside it, so the guard cannot pass by refusing everything |
| `tests/inc/exponential_test.rs` | `the_logarithms_reject_what_has_no_logarithm` |
| `tests/inc/hyperbolic_test.rs` | `acosh_inverts_cosh_and_refuses_what_has_no_inverse`, `atanh_inverts_tanh_and_refuses_what_has_no_inverse` |
| `tests/inc/inverse_circular_test.rs` | `asin_and_acos_clamp_rather_than_returning_nan` |
| `tests/inc/contract_test.rs` | `nan_propagates_rather_than_being_swallowed` |
| `tests/inc/internal_test.rs` | `clamp_unit_is_a_boundary_not_a_rescale` — in-domain arguments pass through bit-identically, so the clamp cannot be quietly rescaling |
