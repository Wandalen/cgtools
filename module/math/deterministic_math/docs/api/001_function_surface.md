# Api: Function Surface

### Scope

- **Purpose**: Give the crate's complete public surface — 28 functions and 7 constants — with each signature, its domain, what it returns outside that domain, and where its behaviour is specified in full.
- **Responsibility**: Be the inventory a caller reads to find the right function, and the register a test reads to confirm nothing is missing.
- **In Scope**: Every `pub use` in `src/lib.rs`, the calling convention, and the deliberate non-inventory.
- **Out of Scope**: How any of them is computed (see `algorithm/`); what they guarantee (see `invariant/`); what they cost (see `non_functional_requirement/`).

### Calling Convention

Every export is a **free function**, called by path or by import:

```rust
use deterministic_math::{ sin_cos, atan2 };

let ( s, c ) = sin_cos( 0.5 );
let bearing  = atan2( dy, dx );
```

Writing `x.sin()` instead calls the platform's libm and forfeits the crate's
only guarantee. It is not a style preference — the two are different functions,
they differ in bits on roughly a fifth of arguments, and no import changes which
one method syntax resolves to. The mechanisms that catch it are in
[pitfall/004](../pitfall/004_method_call_reintroduces_libm.md); reading that
before adopting the crate is worth the five minutes.

There is no trait to import, no extension trait, and no `prelude`. The surface
is flat by design: one crate, one module path, every arithmetic entry point a
reproducibility audit has to examine.

### The 28 Functions

Grouped by the module that implements them, which is also how the `pub use`
lines in `src/lib.rs` are grouped.

#### Algebraic — 4

Built from pinned operations alone; see
[algorithm/006](../algorithm/006_algebraic_compositions.md).

| Function | Signature | Domain | Outside it |
|----------|-----------|--------|------------|
| `sqrt` | `fn( f64 ) -> f64` | `x ≥ 0` | `NaN` — the platform's own, IEEE-pinned |
| `mul_add` | `fn( f64, f64, f64 ) -> f64` | all | fused `a·b + c`, one rounding; the platform's own |
| `hypot` | `fn( f64, f64 ) -> f64` | all | `∞` if either argument is infinite, even against a `NaN` |
| `powi` | `fn( f64, i32 ) -> f64` | all, including negative bases | `powi( x, 0 ) == 1.0` for every `x` |

#### Circular — 4

Cody-Waite reduction; see
[algorithm/001](../algorithm/001_cody_waite_range_reduction.md).

| Function | Signature | Domain | Outside it |
|----------|-----------|--------|------------|
| `sin_cos` | `fn( f64 ) -> ( f64, f64 )` | `\| x \| ≤ SIN_COS_MAX` | `( NaN, NaN )` |
| `sin` | `fn( f64 ) -> f64` | `\| x \| ≤ SIN_COS_MAX` | `NaN` |
| `cos` | `fn( f64 ) -> f64` | `\| x \| ≤ SIN_COS_MAX` | `NaN` |
| `tan` | `fn( f64 ) -> f64` | `\| x \| ≤ SIN_COS_MAX` | `NaN` |

`sin_cos` is the primitive; `sin` and `cos` each take one output of it and
discard the other, which is why `sin_cos` costs no more than `sin` alone
(1.14× the host's, against `sin`'s own 1.14×). **Call `sin_cos` whenever both
are wanted** — it is the single clearest performance decision the surface
offers.

The `SIN_COS_MAX` refusal is a genuine limit, not defensive coding; see
[invariant/004](../invariant/004_refusal_over_meaningless_answer.md).

#### Exponential And Logarithmic — 10

See [algorithm/001](../algorithm/001_cody_waite_range_reduction.md),
[algorithm/003](../algorithm/003_logarithm_by_mantissa_split.md),
[algorithm/005](../algorithm/005_cube_root_by_exponential_round_trip.md).

| Function | Signature | Domain | Outside it |
|----------|-----------|--------|------------|
| `exp` | `fn( f64 ) -> f64` | all | saturates to `+0` / `+∞`, correctly through subnormals |
| `exp2` | `fn( f64 ) -> f64` | all | as `exp`; exact at every integer argument |
| `exp_m1` | `fn( f64 ) -> f64` | all | — |
| `ln` | `fn( f64 ) -> f64` | `x > 0` | `NaN` for `x < 0`; `−∞` at `0` |
| `ln_1p` | `fn( f64 ) -> f64` | `x > −1` | as `ln` of `1 + x` |
| `log2` | `fn( f64 ) -> f64` | `x > 0` | as `ln`; exact on powers of two |
| `log10` | `fn( f64 ) -> f64` | `x > 0` | as `ln`; exact on powers of ten |
| `log` | `fn( f64, f64 ) -> f64` | `x > 0`, `base > 0` | as `ln`; see the note below |
| `powf` | `fn( f64, f64 ) -> f64` | `x ≥ 0` | `NaN` for a negative base — use `powi` or `cbrt` |
| `cbrt` | `fn( f64 ) -> f64` | all, including negatives | `±0`, `±∞`, `NaN` pass through |

**`log( x, base )` is the one documented accuracy exception.** It is
`ln( x ) / ln( base )`, two roundings rather than one, so it is measurably
looser than the three dedicated logarithms. Reach for `log2` or `log10` when the
base is 2 or 10 — they are both more accurate and exact on their own powers.

**`exp_m1` and `ln_1p` are not conveniences.** They exist because `exp( x ) − 1`
and `ln( 1 + x )` destroy every significant digit for small `x`, by up to
32 768 ulp ([pitfall/001](../pitfall/001_small_argument_cancellation.md)). If
`x` can be small, these are the correct functions, not the polite ones.

#### Hyperbolic — 6

Cancellation-free rewrites; see
[algorithm/004](../algorithm/004_cancellation_free_hyperbolic_forms.md).

| Function | Signature | Domain | Outside it |
|----------|-----------|--------|------------|
| `sinh` | `fn( f64 ) -> f64` | all | saturates rather than overflowing |
| `cosh` | `fn( f64 ) -> f64` | all | saturates rather than overflowing |
| `tanh` | `fn( f64 ) -> f64` | all | exactly `±1.0` past `\| x \| = 20` |
| `asinh` | `fn( f64 ) -> f64` | all | odd; correct to `\| x \| ≈ 1e250` |
| `acosh` | `fn( f64 ) -> f64` | `x ≥ 1` | `NaN` |
| `atanh` | `fn( f64 ) -> f64` | `\| x \| ≤ 1` | `NaN` past the pole; `±∞` at `±1` |

`atanh` is the crate's least accurate function at 24 ulp, and that figure is
conditioning rather than a defect — its relative error is `4.0e-15` and its
derivative near the sample is ≈49
([non_functional_requirement/002](../non_functional_requirement/002_accuracy_against_host_libm.md)).

#### Inverse Circular — 4

Bin reduction with a near-zero bypass; see
[algorithm/002](../algorithm/002_arctangent_bin_reduction.md).

| Function | Signature | Domain | Outside it |
|----------|-----------|--------|------------|
| `atan` | `fn( f64 ) -> f64` | all | `±FRAC_PI_2` at `±∞` |
| `atan2` | `fn( f64, f64 ) -> f64` | all, both arguments | full four-quadrant result on `( −π, π ]` |
| `asin` | `fn( f64 ) -> f64` | `\| x \| ≤ 1` | **clamps** — see below |
| `acos` | `fn( f64 ) -> f64` | `\| x \| ≤ 1` | **clamps** — see below |

**`asin` and `acos` clamp rather than refusing**, which is the crate's one
deliberate departure from
[invariant/004](../invariant/004_refusal_over_meaningless_answer.md). Their
arguments habitually arrive as a normalised dot product, where an exact `1.0`
routinely lands a rounding step above and a `NaN` there would be a defect in the
caller's arithmetic reported as a domain error. Note the asymmetry with
`acosh`, which does refuse — a value marginally below `1` reaching `acosh` is
not the same everyday rounding artefact.

Argument order on `atan2` is `( y, x )`, matching C and Rust's own
`f64::atan2` — the vertical component first, which reads backwards against
almost every other two-argument function here.

**`atan2` does not distinguish signed zero in `y`.** `atan2( -0.0, -1.0 )`
returns `+π` here where `f64::atan2` returns `−π`; likewise `atan2( 0.0, 0.0 )`
is `0.0` by the same convention. This is the one place the surface departs from
the host's result rather than merely from its last bit, and it is a convention
rather than a defect — the sign of a zero `y` carries no direction. It matters
only to a caller that feeds the result back through something sign-sensitive at
exactly the negative-x axis. Everywhere else the four-quadrant result agrees
with the host to within `1e-12`, which
`atan2_matches_libm_in_every_quadrant` asserts across all four.

### The 7 Constants

Exported from `constant`, which is itself `pub` — reachable as
`deterministic_math::PI` or `deterministic_math::constant::PI`.

| Constant | Value | Source |
|----------|-------|--------|
| `PI` | `core::f64::consts::PI` | re-exported unchanged |
| `TAU` | `core::f64::consts::TAU` | re-exported unchanged |
| `FRAC_PI_2` | `core::f64::consts::FRAC_PI_2` | re-exported unchanged |
| `FRAC_PI_4` | `core::f64::consts::FRAC_PI_4` | re-exported unchanged |
| `LN_2` | `core::f64::consts::LN_2` | re-exported unchanged |
| `LN_10` | `core::f64::consts::LN_10` | re-exported unchanged |
| `SIN_COS_MAX` | `1.0e8` | this crate's own — the reduction's limit |

Six of the seven are `core`'s values re-exported rather than retyped, so there
is no possibility of a transcribed digit differing from the standard one, and a
test asserts the identity rather than the literal. `SIN_COS_MAX` is the only
constant this crate originates, and it is a documented capability boundary
rather than a mathematical value.

The reduction and series coefficients — `LN2_HI`, `LN2_LO`, `PIO2_HI`,
`ATAN_B`, `ATAN_V`, `SERIES_BAND`, and the rest — are `pub( crate )` and
deliberately not exported. They are implementation detail whose split points a
future rewrite must be free to change, and each carries its derivation in a doc
comment where it is defined.

### What Is Deliberately Absent

These are **not** here, and their absence is not a gap:

```
abs  floor  ceil  round  trunc  fract  signum  recip  clamp  min  max
to_degrees  to_radians  rem_euclid
```

Each is either exact by construction or a single pinned operation, so `f64`'s
own method is already bit-identical on every target. Wrapping them would add a
call and no guarantee — and worse, it would blur the line the crate draws:
everything exported here is a function whose platform version is *free to
differ*. A surface padded with exact operations makes that signal harder to
read, not easier.

`sqrt` and `mul_add` are the two counterexamples, and they are counterexamples
on purpose. Both are IEEE-pinned and neither needed reimplementing; they are
exported so that one crate holds every arithmetic entry point a reproducibility
audit has to look at, and so the scan in
[invariant/002](../invariant/002_pinned_operations_only.md) can ban all 28
method spellings uniformly rather than maintaining an exception list.

Vector and matrix algebra also needs no treatment here. It is built from `+`,
`−` and `×`, which IEEE-754 pins, and Rust never contracts `a * b + c` into an
FMA on its own — so the ordinary operators are already reproducible.

### How This Inventory Stays True

The surface is not maintained by hand against this document. Three tests read
`src/lib.rs`'s own `pub use` lines and derive the register from them:

- `the_export_list_is_actually_being_parsed` — asserts the parser finds at least
  28 names including several spelled distinctly, so the two tests below cannot
  pass by iterating an empty list. That is their one failure mode that cannot
  report itself.
- `the_unary_surface_covers_every_exported_one_argument_function` — every export
  that is not among the 6 multi-argument names or `sin_cos` must appear in the
  cross-cutting purity and `NaN` tests.
- `the_benchmark_measures_every_exported_function` — every export must be named
  in `examples/cost_vs_libm.rs`, so nothing ships measured against neither the
  cost ceiling nor the accuracy ceiling.

Adding a function to `lib.rs` therefore widens all three immediately, and none
of them can be satisfied by editing a list. What no test can check is that
*this document* was updated too — the counts here (28 functions, 7 constants,
21 unary, 6 multi-argument) are the part a reader should re-derive from
`src/lib.rs` if anything looks off.

### Verify It Yourself

```sh
# The surface, straight from the source of truth.
grep '^pub use' src/lib.rs

# The three register tests.
cargo test -p deterministic_math the_export_list_is_actually_being_parsed
cargo test -p deterministic_math the_unary_surface_covers_every_exported_one_argument_function
cargo test -p deterministic_math the_benchmark_measures_every_exported_function
```

```rust
use deterministic_math::{ sin_cos, sin, cos, log, log2, powi, powf, cbrt, asin };

// sin_cos is the primitive, and its outputs are the other two exactly.
let ( s, c ) = sin_cos( 0.5 );
assert_eq!( s, sin( 0.5 ) );
assert_eq!( c, cos( 0.5 ) );

// The dedicated logarithm is exact where the general one is not.
assert_eq!( log2( 8.0 ), 3.0 );
assert!( ( log( 8.0, 2.0 ) - 3.0 ).abs() < 1.0e-15 );

// Negative bases: powf refuses, powi and cbrt do not.
assert!( powf( -8.0, 2.0 ).is_nan() );
assert_eq!( powi( -8.0, 2 ), 64.0 );
assert_eq!( cbrt( -8.0 ), -2.0 );

// asin clamps rather than refusing, at the edge a dot product lands on.
assert_eq!( asin( 1.0 + 2.0e-16 ), asin( 1.0 ) );
```

### Invariants

| File | Relationship |
|------|--------------|
| [../invariant/001_bit_reproducibility.md](../invariant/001_bit_reproducibility.md) | The single property every function on this surface guarantees |
| [../invariant/002_pinned_operations_only.md](../invariant/002_pinned_operations_only.md) | Why `sqrt` and `mul_add` are exported despite needing no implementation |
| [../invariant/004_refusal_over_meaningless_answer.md](../invariant/004_refusal_over_meaningless_answer.md) | The refusal column above, in full, including the `asin`/`acos` exception |
| [../invariant/003_zero_dependencies.md](../invariant/003_zero_dependencies.md) | Why the surface is flat `pub use` rather than `mod_interface` |

### Algorithms

| File | Relationship |
|------|--------------|
| [../algorithm/readme.md](../algorithm/readme.md) | How each group above is computed — six procedures covering all 28 |

### Pitfalls

| File | Relationship |
|------|--------------|
| [../pitfall/004_method_call_reintroduces_libm.md](../pitfall/004_method_call_reintroduces_libm.md) | The calling convention's failure mode, and the two consumer-side mechanisms that catch it |
| [../pitfall/001_small_argument_cancellation.md](../pitfall/001_small_argument_cancellation.md) | Why `exp_m1` and `ln_1p` are on the surface at all |

### Non Functional Requirements

| File | Relationship |
|------|--------------|
| [../non_functional_requirement/001_cost_against_host_libm.md](../non_functional_requirement/001_cost_against_host_libm.md) | Per-function cost for all 28, including the `sin_cos` figure quoted above |
| [../non_functional_requirement/002_accuracy_against_host_libm.md](../non_functional_requirement/002_accuracy_against_host_libm.md) | Per-function accuracy for all 28, including `atanh`'s recorded exception |

### Sources

| File | Relationship |
|------|--------------|
| `src/lib.rs` | The five `pub use` lines this inventory is derived from, and the crate-level documentation of the calling convention and the deliberate absences |
| `src/constant.rs` | The 7 exported constants and the `pub( crate )` coefficients that are not exported |
| `readme.md` | The same surface as a short table, for a reader who has not opened `docs/` |

### Tests

| File | Relationship |
|------|--------------|
| `tests/inc/contract_test.rs` | `exported_function_names` and the three register tests described above |
| `tests/inc/contract_test.rs` | `constants_are_the_core_ones` — asserts the six re-exports are `core`'s own values rather than transcriptions |
| `tests/inc/contract_test.rs` | `nan_propagates_rather_than_being_swallowed` — over every unary export, so no function on this surface can silently absorb a `NaN` |
