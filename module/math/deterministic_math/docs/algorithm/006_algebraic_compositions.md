# Algorithm: Algebraic Compositions

### Scope

- **Purpose**: Document the two functions built from nothing but pinned operations — `hypot` and `powi` — and the specific failure each one's naive form has.
- **Responsibility**: Give each procedure, the naive expression it replaces, and the failure that expression exhibits.
- **In Scope**: `hypot`, `powi`, and the reason `sqrt` and `mul_add` are exported despite needing no implementation at all.
- **Out of Scope**: Anything reaching a series or a reduction — these two need neither, which is what puts them in their own module.

### Abstract

Neither of these needed reimplementing to be reproducible. Both are
compositions of `+`, `×`, `÷` and `√`, which IEEE-754 already pins, so the
platform's own versions were never at risk of differing between targets.

They are here for a different reason, and it is worth separating from the
determinism argument: each replaces a naive expression that *fails* — not
imprecisely, but by returning infinity, zero, or `NaN` where an ordinary finite
answer exists. And having them here means a reader auditing a codebase for libm
calls finds every arithmetic entry point in one crate, including the ones that
were never at risk.

### `hypot` — Scaling By The Larger Magnitude

```text
if either is NaN :  ∞ if the other is infinite, else NaN
( large, small ) = ( max( |x|, |y| ), min( |x|, |y| ) )
if large == 0 :   0
if large == ∞ :   ∞
r = small / large                                 ( r ∈ [ 0, 1 ] )
return  large · √( 1 + r² )
```

**The naive form's failure.** `√( x² + y² )` overflows to infinity as soon as
either argument passes `1.34e154`, even though the answer is an ordinary finite
number, and underflows to zero for arguments below `1.5e-154`. Both failures
are silent and both are reachable from real data — a distance in metres between
two points a light-year apart is `9.5e15`, and squaring a stiffness ratio gets
small fast.

Squaring only a ratio in `[ 0, 1 ]` cannot overflow, and the leading `large`
carries the exponent unharmed.

**`∞` beats `NaN` in the other argument.** The magnitude is known to be infinite
whatever that component turns out to be. This follows the IEEE recommendation,
and it is asserted against the host's own `hypot` so the two cannot silently
diverge.

**It is the one function faster than the host's**, at `0.53×`. Not an
achievement — glibc's `hypot` is scrupulous about the last ulp across the whole
exponent range and pays for it; this one accepts 2 ulp. Both are defensible;
they answer slightly different questions.

### `powi` — Binary Exponentiation

```text
if n == 0 :  1.0
e = n.unsigned_abs()                              ( not −n )
base = x ;  acc = 1.0
loop :  if e is odd :  acc ·= base
        e >>= 1 ;  if e == 0 :  break
        base ·= base
return  1/acc if n < 0 else acc
```

**Why it is not `powf( x, n as f64 )`.** `powf` routes through `ln` and is `NaN`
for every negative base, so the obvious substitution is silently wrong over half
the domain — `powi( −2.0, 3 )` is `−8.0`, and `powf( −2.0, 3.0 )` is `NaN`. For
a small exponent it is also both faster and more accurate, being multiplication
alone rather than an `exp ∘ ln` round trip.

**`n.unsigned_abs()`, never `−n`.** `i32::MIN` has no positive counterpart, so
an implementation reaching the magnitude by negating overflows there: in a debug
build it panics, and in release it wraps back to `i32::MIN` and computes an
enormous positive power instead of a vanishing negative one. Asserted directly —
`powi( 2.0, i32::MIN )` is `0.0` and `powi( 0.5, i32::MIN )` is infinite.

**The squaring chain is written out rather than delegated to `f64::powi`,**
whose association the optimiser is free to reorder. Here the order of every
multiplication is fixed by the loop, which is what
[invariant/001](../invariant/001_bit_reproducibility.md) needs of it.

### `sqrt` And `mul_add` — Exported Without Being Implemented

Both delegate to the platform in one line, because both are among the six
operations IEEE-754 requires to be correctly rounded. There is exactly one right
answer and the platform already produces it.

`mul_add` is worth a note anyway, and the reason is not determinism. `a·b + c`
and `fma( a, b, c )` are *different functions* — one rounding versus two — and
both are correct. Two call sites that ought to agree will not if one is written
each way, and at a glance `x.mul_add( y, z )` and `x * y + z` read as the same
expression. Routing the fused form through a named function makes the choice
legible. Rust never contracts `a * b + c` into an FMA on its own, so the unfused
spelling stays unfused and the two forms never swap underfoot.

That distinction is asserted rather than assumed: the test sweeps for an
argument where the fused and unfused forms actually diverge, so the bit-equality
check above it cannot be passing for the wrong reason.

### Verify It Yourself

```rust
use deterministic_math::{ hypot, powi, powf };

// Where the naive forms fail.
let big : f64 = 1.0e200;
assert!( ( big * big ).is_infinite() );
assert!( hypot( big, big ).is_finite() );

let small : f64 = 1.0e-200;
assert_eq!( small * small, 0.0 );
assert!( hypot( small, small ) > 0.0 );

assert!( powf( -2.0, 3.0 ).is_nan() );
assert_eq!( powi( -2.0, 3 ), -8.0 );

// The exponent with no positive counterpart.
assert_eq!( powi( 2.0, i32::MIN ), 0.0 );
assert!( powi( 0.5, i32::MIN ).is_infinite() );
```

### Invariants

| File | Relationship |
|------|--------------|
| [../invariant/002_pinned_operations_only.md](../invariant/002_pinned_operations_only.md) | The two counted `.sqrt(` / `.mul_add(` call sites are both in this module, and are the only permitted libm calls in the crate |
| [../invariant/001_bit_reproducibility.md](../invariant/001_bit_reproducibility.md) | Why `powi`'s squaring chain is written out rather than delegated |

### Algorithms

| File | Relationship |
|------|--------------|
| [005_cube_root_by_exponential_round_trip.md](005_cube_root_by_exponential_round_trip.md) | `cbrt`, the other function that exists because `powf` is silently wrong for a negative base |

### Non Functional Requirements

| File | Relationship |
|------|--------------|
| [../non_functional_requirement/001_cost_against_host_libm.md](../non_functional_requirement/001_cost_against_host_libm.md) | `hypot` at 0.53×, and `sqrt`/`mul_add`/`powi` at 1.00× — the reason those three are in the table at all |

### Sources

| File | Relationship |
|------|--------------|
| `src/algebraic.rs` | `hypot`, `powi`, `sqrt`, `mul_add`; also `atan_series`, `atanh_series`, `scale2` and `round_half_away`, which are crate-internal helpers documented with their consumers |

### Tests

| File | Relationship |
|------|--------------|
| `tests/inc/algebraic_test.rs` | `hypot_survives_arguments_the_naive_form_cannot` (asserting the naive form's failure first, so the guard cannot pass vacuously), `hypot_matches_libm_and_its_own_definition`, `hypot_handles_zero_infinity_and_nan`, `powi_is_repeated_multiplication`, `powi_accepts_the_negative_bases_powf_refuses`, `powi_handles_the_exponent_that_cannot_be_negated`, `sqrt_is_bit_identical_to_the_hardware`, `mul_add_is_the_fused_form_and_not_the_unfused_one` |
