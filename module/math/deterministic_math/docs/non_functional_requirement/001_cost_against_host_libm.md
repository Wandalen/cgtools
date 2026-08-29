# Non Functional Requirement: Cost Against Host libm

### Scope

- **Purpose**: Bound what reproducibility is allowed to cost, per function, against the platform libm each function replaces.
- **Responsibility**: State the ceiling, name the benchmark that measures it, and record the measurement that stands today.
- **In Scope**: Wall-clock cost per call of all 28 exported functions, expressed as a ratio against the same operation on `f64`.
- **Out of Scope**: How close the answer is to the true value — a separate quality attribute with its own instance ([002](002_accuracy_against_host_libm.md)); allocation behaviour, which is trivially zero (nothing here allocates, and the crate is `no_std`-shaped with no collections).

### Quality Attribute

Performance — per-call cost relative to the host's own implementation.

### Statement

No function may cost more than **5×** the corresponding `f64` method on the
measurement host. The bound is deliberately loose, because the trade this crate
offers is *reproducibility for speed* and a caller who did not want to pay
something would not be here. What the bound rules out is a function that has
quietly become an order of magnitude worse — the shape a regression takes when
a reduction stops reducing, a series stops converging in its documented term
count, or a branch that was meant to be rare becomes the common path.

The ratio, not the nanosecond count, is the requirement. Absolute timings move
with the host; the ratio is measured against a reference that moves with it.

### Measurement Method

```sh
cargo run -p deterministic_math --release --example cost_vs_libm
```

`--release` is not optional — a debug build measures missing inlining rather
than the kernel, and reports ratios several times worse than the shipped code's.

The example times 200 000 pseudo-random samples per function drawn from a
fixed-seed `xorshift64*` generator, takes the best of 7 rounds to suppress
scheduler noise, and prints the ratio table reproduced below. The same run also
produces the accuracy figures that [002](002_accuracy_against_host_libm.md)
governs, so one command answers both questions and neither can be refreshed
without the other.

The generator is seeded from a constant, so two runs on the same host draw the
identical sample set and any difference between them is the machine, not the
sampling.

### Acceptance Threshold

**5×.** Every function is inside it; the worst is `exp2` at 4.82×.

The three functions nearest the ceiling — `exp`, `exp2`, `cbrt` — are all
exponential-family, and all pay the same thing: a sixteen-term Taylor series
where the host has a table lookup plus a short polynomial. That is the honest
cost of carrying the coefficients in the source rather than in the platform, and
it is not a defect to be fixed. `cbrt` pays more still because it is an `ln` and
an `exp` back to back, plus a Newton step
([algorithm/005](../algorithm/005_cube_root_by_exponential_round_trip.md)).

### Measurement Of Record

Measured on **aarch64 Linux**, rustc 1.97.1, release profile, 200 000 samples,
best of 7 rounds. Reproduce with the command above; expect the ratios to hold
and the nanoseconds not to.

| function | domain | det ns | libm ns | ratio | max Δulp |
|----------|--------|--------|---------|-------|----------|
| `sqrt` | `[ 0, 1e6 ]` | 2.39 | 2.38 | 1.00x | 0 |
| `mul_add` | `x,y ∈ [ -100, 100 ], c = 1` | 1.91 | 1.91 | 1.00x | 0 |
| `hypot` | `x,y ∈ [ -1e6, 1e6 ]` | 5.78 | 10.83 | 0.53x | 2 |
| `powi` | `[ -10, 10 ], n = 7` | 2.06 | 2.04 | 1.01x | 0 |
| `exp` | `[ -20, 20 ]` | 30.60 | 6.64 | 4.61x | 1 |
| `exp2` | `[ -20, 20 ]` | 29.23 | 6.06 | 4.82x | 1 |
| `exp_m1` | `[ -1, 1 ]` | 30.72 | 15.90 | 1.93x | 4 |
| `ln` | `[ 1e-6, 1e6 ]` | 21.52 | 7.79 | 2.76x | 1 |
| `ln_1p` | `[ -0.5, 1 ]` | 21.91 | 15.96 | 1.37x | 5 |
| `log2` | `[ 1e-6, 1e6 ]` | 21.33 | 8.47 | 2.52x | 2 |
| `log10` | `[ 1e-6, 1e6 ]` | 21.47 | 12.77 | 1.68x | 1 |
| `log` | `x ∈ [ 1e-6, 1e6 ], base ∈ [ 2, 10 ]` | 40.02 | 15.49 | 2.58x | 3 |
| `powf` | `x ∈ [ 0.1, 10 ], y ∈ [ -5, 5 ]` | 65.12 | 18.49 | 3.52x | 18 |
| `cbrt` | `[ -1e6, 1e6 ]` | 75.13 | 20.09 | 3.74x | 1 |
| `sin` | `[ -100, 100 ]` | 30.81 | 22.05 | 1.40x | 2 |
| `cos` | `[ -100, 100 ]` | 31.24 | 22.30 | 1.40x | 2 |
| `tan` | `[ -1.5, 1.5 ]` | 29.61 | 20.81 | 1.42x | 3 |
| `sin_cos` | `[ -100, 100 ]` | 30.59 | 26.93 | 1.14x | 2 |
| `atan` | `[ -100, 100 ]` | 22.40 | 15.36 | 1.46x | 2 |
| `atan2` | `y,x ∈ [ -100, 100 ]` | 35.63 | 30.43 | 1.17x | 3 |
| `asin` | `[ -1, 1 ]` | 40.37 | 19.29 | 2.09x | 3 |
| `acos` | `[ -1, 1 ]` | 38.48 | 20.24 | 1.90x | 3 |
| `sinh` | `[ -20, 20 ]` | 39.25 | 29.49 | 1.33x | 4 |
| `cosh` | `[ -20, 20 ]` | 35.36 | 13.32 | 2.65x | 2 |
| `tanh` | `[ -20, 20 ]` | 40.21 | 28.39 | 1.42x | 4 |
| `asinh` | `[ -100, 100 ]` | 30.26 | 17.04 | 1.78x | 4 |
| `acosh` | `[ 1, 100 ]` | 30.49 | 16.06 | 1.90x | 4 |
| `atanh` | `[ -0.99, 0.99 ]` | 31.42 | 24.74 | 1.27x | 24 |

### Reading The Table

**`hypot` is faster than the host's, at 0.53×.** Not a measurement artifact and
not an achievement — glibc's `hypot` is scrupulous about the last ulp over the
full exponent range and pays for it; this one scales by the larger magnitude,
takes one `sqrt`, and accepts 2 ulp. Both are defensible; they are answering
slightly different questions.

**`sqrt`, `mul_add` and `powi` sit at 1.00×** because they are the host's own
operations, called directly. IEEE-754 already pins `sqrt` and `fma`, and `powi`
is a multiply chain. They are exported so that one crate holds every arithmetic
entry point a reproducibility audit has to look at — see
[api/001](../api/001_function_surface.md) — not because they needed replacing.

**`sin_cos` costs the same as `sin` alone** (1.14× versus 1.40×), because it
*is* `sin` alone: one reduction feeds both polynomials, and `sin`/`cos` are
wrappers that discard half the pair. A caller needing both should say so.

### Invariants

| File | Relationship |
|------|--------------|
| [../invariant/002_pinned_operations_only.md](../invariant/002_pinned_operations_only.md) | The ban that makes this cost unavoidable — every function here is forbidden the host's own transcendental, which is what it is being timed against |

### Algorithms

| File | Relationship |
|------|--------------|
| [../algorithm/001_cody_waite_range_reduction.md](../algorithm/001_cody_waite_range_reduction.md) | The reduction plus sixteen-term series behind the exponential family's 4.6–4.8× — the ratios closest to the ceiling |
| [../algorithm/005_cube_root_by_exponential_round_trip.md](../algorithm/005_cube_root_by_exponential_round_trip.md) | Why `cbrt` costs an `ln`, an `exp` and a Newton step rather than one kernel |

### Sources

| File | Relationship |
|------|--------------|
| `examples/cost_vs_libm.rs` | The benchmark that produces this table; `all_rows()` enumerates the 28 measured functions |

### Tests

No test asserts the ratio, and none should — a timing assertion fails on a
loaded CI runner and says nothing about the code. The benchmark is an
instrument, run deliberately.

What *is* asserted mechanically is that the instrument covers the whole surface.
`tests/inc/contract_test.rs`'s `the_benchmark_measures_every_exported_function`
parses `lib.rs`'s own `pub use` lines and fails if any exported function is
never named in `examples/cost_vs_libm.rs` — so a function cannot ship held to
neither this ceiling nor [002](002_accuracy_against_host_libm.md)'s by simple
omission from the table. Its companion
`the_export_list_is_actually_being_parsed` guards the parser itself, since a
parser silently returning nothing would let that check pass over an empty list.
