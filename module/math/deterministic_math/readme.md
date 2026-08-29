# deterministic_math

Transcendental functions that return the **same bits on every machine**.

`sin`, `exp`, `atan2` and their neighbours are not pinned by IEEE-754 — only
`+ − × ÷ √ fma` are. Two conforming libm implementations may disagree in the
last place and both be right, and they do disagree: between glibc and musl,
between two glibc releases, and between x86-64 and aarch64 builds of identical
source. This crate rebuilds all 28 from the pinned six, so the answer is fixed
by the source rather than by the host.

```toml
[dependencies]
deterministic_math = "0.1"
```

```rust
// Call by path. `x.sin()` is the platform's libm; this is not.
let ( s, c ) = deterministic_math::sin_cos( 0.5 );
assert!( ( s * s + c * c - 1.0 ).abs() < 1.0e-15 );
```

## When this matters

Anywhere a float feeds a **branch** that two machines must take the same way:
lockstep multiplayer, deterministic replay, a recorded run that has to
reproduce after a libc upgrade, a golden test that runs on more than one CI
architecture. The drift itself is harmless — one ulp in a position is nothing.
The first threshold comparison that resolves differently is not, because from
that instant the two runs are answering different questions and no downstream
precision reconciles them.

Anywhere else — rendering, analysis, a single-machine tool — the platform's
libm is faster and often more accurate. Use it.

## What it guarantees, and what it does not

**Guaranteed:** identical bits for identical inputs, on every target, forever.
Nothing here reads ambient state, and every operation is one the standard
requires to be correctly rounded.

**Not guaranteed:** that the answer is the closest representable double to the
true value. It usually is, and for a few functions it is a couple of ulp
further out than a good libm. Reproducibility and accuracy are separate
properties and only the first is on offer.

Measure both yourself:

```sh
cargo run -p deterministic_math --release --example cost_vs_libm
```

`--release` is not optional — a debug build measures missing inlining, not the
kernel. The example prints per-function timing against the host libm and the
worst ulp disagreement found over 200 000 samples.

## The surface

28 functions, covering every `f64` method whose result an implementation is
free to choose.

| Family | Functions |
|--------|-----------|
| Algebraic (already pinned; here for one-place auditing) | `sqrt`, `mul_add`, `hypot`, `powi` |
| Exponential and logarithmic | `exp`, `exp2`, `exp_m1`, `ln`, `ln_1p`, `log`, `log2`, `log10`, `powf`, `cbrt` |
| Circular | `sin_cos`, `sin`, `cos`, `tan` |
| Inverse circular | `atan`, `atan2`, `asin`, `acos` |
| Hyperbolic | `sinh`, `cosh`, `tanh`, `asinh`, `acosh`, `atanh` |

Constants: `PI`, `TAU`, `FRAC_PI_2`, `FRAC_PI_4`, `LN_2`, `LN_10`,
`SIN_COS_MAX`.

Deliberately absent: `abs`, `floor`, `ceil`, `round`, `trunc`, `fract`,
`signum`, `recip`, `clamp`, `min`, `max`, `to_degrees`, `to_radians`,
`rem_euclid`. Each is exact by construction or a single pinned operation, so
`f64`'s own method is already bit-identical everywhere. Wrapping them would add
a call and no guarantee.

## No dependencies

Zero, and that is the point. Reproducible arithmetic is wanted by simulation
code that has no use for a graphics stack, so reaching it should not cost one.

```sh
cargo tree -p deterministic_math
```

One line. Verify it before and after any change that adds a dependency.

## Documentation

Design documentation lives in [`docs/`](docs/definition/readme.md) — 17
instances across five types, indexed there with three reading paths depending on
whether you are using the crate, auditing it, or changing it.

The four worth knowing about before anything else:

| | |
|-|-|
| [api/001](docs/api/001_function_surface.md) | All 28 functions and 7 constants, with domain and refusal behaviour |
| [pitfall/004](docs/pitfall/004_method_call_reintroduces_libm.md) | `x.sin()` is not `sin( x )`, and no test here can catch it — read before adopting |
| [invariant/001](docs/invariant/001_bit_reproducibility.md) | Exactly what is promised, and what is deliberately not |
| [non_functional_requirement/001](docs/non_functional_requirement/001_cost_against_host_libm.md) | The measured cost of every function against the host's own |
