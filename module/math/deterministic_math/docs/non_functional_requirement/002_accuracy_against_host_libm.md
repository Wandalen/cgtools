# Non Functional Requirement: Accuracy Against Host libm

### Scope

- **Purpose**: State how far from the true value this crate is allowed to be, and separate that question completely from the reproducibility guarantee it is routinely confused with.
- **Responsibility**: Set the ulp ceiling, name the instrument, record the standing measurement, and explain each figure that looks alarming and is not.
- **In Scope**: Disagreement with the host libm, per function, in ulp and in relative terms.
- **Out of Scope**: Bit-reproducibility, which is a hard invariant rather than a bounded quality attribute — see [invariant/001](../invariant/001_bit_reproducibility.md); per-call cost, see [001](001_cost_against_host_libm.md).

### Quality Attribute

Accuracy — distance from the correctly-rounded result, measured against the
host libm as a stand-in for it.

### Statement

Every function must agree with the host libm to within **8 ulp** across its
documented domain, except where a larger figure is explained by the
*conditioning of the function itself* rather than by the implementation, and
that explanation is recorded here.

Two things this deliberately does **not** say.

It does not say "correctly rounded". Proving the last bit right for every input
runs into the table-maker's dilemma — the exact result can sit arbitrarily close
to a rounding boundary, and no bounded working precision decides every case — so
the claim would be unbacked. IEEE-754 itself only *recommends* correct rounding
for these functions, which is precisely why they differ between platforms and
why this crate exists.

And it does not treat accuracy as the crate's contract. A function reproducibly
one ulp off on every machine satisfies the whole contract
([invariant/001](../invariant/001_bit_reproducibility.md)). A function nearer
the truth on one machine and differently near it on another satisfies none of
it. Accuracy is a quality this crate tries hard to have; reproducibility is the
one it guarantees.

### Measurement Method

The same single command as [001](001_cost_against_host_libm.md):

```sh
cargo run -p deterministic_math --release --example cost_vs_libm
```

Beyond the ratio table, the run prints three instruments aimed specifically at
accuracy:

1. **Worst-case ulp per function**, over 200 000 fixed-seed samples, with the
   input that produced it and the relative difference at that input. The
   relative column is what separates a real disagreement from cancellation near
   a zero of the function.
2. **A small-argument sweep** — eight functions across eight decades from `1e-1`
   to `1e-8`, sampling off the decade so neither implementation is being asked
   about a value it might special-case. This is the instrument that made the
   defects in [pitfall/001](../pitfall/001_small_argument_cancellation.md)
   visible; a flat row means the relative accuracy holds all the way down, and a
   row that fans out as the argument shrinks is the signature of a reduction
   destroying its own input.
3. **Two amplification checks**, where a caller's own arithmetic magnifies
   whatever error the function returned — `exp_m1`/`ln_1p` against the naive
   expressions they replace, and `( sinh( s ) − s ) / s³`, whose subtraction
   multiplies `sinh`'s error by roughly `1 / s²`.

`ulp` is computed sign-aware, so it stays meaningful across zero and into the
subnormal range, where consecutive representable values are up to 6% apart and
any *relative* bound would be either unmeetable or vacuous.

### Acceptance Threshold

**8 ulp**, with two recorded exceptions, both conditioning rather than
implementation:

- **`atanh`, 24 ulp** at `x = −0.9898…`, relative difference `4.0e-15`. `atanh`
  has a pole at `±1`; its derivative at this sample is about 49, so a
  last-place perturbation of the argument moves the true result by tens of ulp
  before any implementation has done anything. Both implementations are near the
  truth; the ulp grid is simply fine relative to the function's own sensitivity.
  Away from the pole `atanh` measures 0–2e-16 relative across eight decades.
- **`powf`, 18 ulp** at `9.67^−4.92`, relative difference `2.2e-15`. `powf` is
  `exp( y · ln x )`, so the argument handed to `exp` carries `ln`'s rounding
  multiplied by `y`, and `exp` then amplifies an absolute error in its argument
  into a relative error in its output. At `| y | ≈ 5` a couple of ulp of `ln`
  is exactly this many ulp of result. This is inherent to the identity, not to
  this implementation of it; the crate's own doc comment on `powf` points a
  caller wanting an integer power at `powi` instead, which is both faster and
  more accurate.

Everything else measures **0–5 ulp**. The five worst inside the bound —
`ln_1p` at 5, `exp_m1`/`sinh`/`tanh`/`asinh`/`acosh` at 4 — are all functions
whose relative figure is 4.5e-16 to 6.1e-16, i.e. two to three roundings' worth,
which is what a two-or-three-step composition costs.

### Measurement Of Record

Measured on **aarch64 Linux**, rustc 1.97.1, release profile. Worst case per
function, with the input that produced it:

| function | max Δulp | rel | note |
|----------|----------|-----|------|
| `sqrt`, `mul_add`, `powi` | 0 | 0 | Pinned or exact by construction — bit-identical, not merely close |
| `exp`, `exp2`, `ln`, `log10`, `cbrt` | 1 | ~1.2–1.6e-16 | One rounding |
| `hypot`, `log2`, `sin`, `cos`, `sin_cos`, `atan`, `cosh` | 2 | ~2.2–3.1e-16 | |
| `tan`, `log`, `atan2`, `asin`, `acos` | 3 | ~3.3–4.0e-16 | |
| `exp_m1`, `sinh`, `tanh`, `asinh`, `acosh` | 4 | ~4.5–6.1e-16 | Two-to-three-step compositions |
| `ln_1p` | 5 | 6.1e-16 | |
| `powf` | 18 | 2.2e-15 | Recorded exception — `exp ∘ ln` amplification |
| `atanh` | 24 | 4.0e-15 | Recorded exception — conditioning at the pole |

Relative disagreement across eight decades, for the eight functions whose
small-argument behaviour was the thing being repaired:

| function | x≈1e-1 | x≈1e-2 | x≈1e-3 | x≈1e-4 | x≈1e-5 | x≈1e-6 | x≈1e-7 | x≈1e-8 |
|----------|--------|--------|--------|--------|--------|--------|--------|--------|
| `exp_m1` | 0 | 0 | 0 | 1e-16 | 0 | 1e-16 | 1e-16 | 0 |
| `ln_1p` | 0 | 2e-16 | 1e-16 | 1e-16 | 2e-16 | 0 | 1e-16 | 2e-16 |
| `asin` | 0 | 0 | 1e-16 | 0 | 0 | 0 | 1e-16 | 2e-16 |
| `atan` | 0 | 2e-16 | 1e-16 | 0 | 0 | 0 | 0 | 2e-16 |
| `sinh` | 1e-16 | 2e-16 | 0 | 1e-16 | 2e-16 | 0 | 1e-16 | 2e-16 |
| `tanh` | 2e-16 | 2e-16 | 1e-16 | 1e-16 | 0 | 1e-16 | 0 | 2e-16 |
| `asinh` | 0 | 2e-16 | 1e-16 | 1e-16 | 0 | 1e-16 | 1e-16 | 0 |
| `atanh` | 0 | 2e-16 | 1e-16 | 1e-16 | 2e-16 | 1e-16 | 1e-16 | 0 |

Flat, all eight, all the way down. That flatness is the whole result of the
repair recorded in [pitfall/001](../pitfall/001_small_argument_cancellation.md);
before it these rows fanned out to `1e-11` at the right-hand end.

### What The Figures Were Before

Kept because the delta is the evidence that the repair is real, and because a
future change that reintroduces a reduction will reproduce these numbers rather
than something new:

| function | before | after |
|----------|-------:|------:|
| `atan` | 95 | 2 |
| `atan2` | 8 883 | 3 |
| `asin` | 22 268 | 3 |
| `exp_m1` | 31 203 | 4 |
| `ln_1p` | 32 768 | 5 |
| `sinh` | 7 163 | 4 |
| `tanh` | 1 244 | 4 |
| `asinh` | 2 628 | 4 |

### Invariants

| File | Relationship |
|------|--------------|
| [../invariant/001_bit_reproducibility.md](../invariant/001_bit_reproducibility.md) | The property that *is* guaranteed, and which this one is routinely mistaken for |

### Pitfalls

| File | Relationship |
|------|--------------|
| [../pitfall/001_small_argument_cancellation.md](../pitfall/001_small_argument_cancellation.md) | The defect class the before/after table measures, and the branch structure that closed it |
| [../pitfall/002_exponent_field_wraparound.md](../pitfall/002_exponent_field_wraparound.md) | A defect this instrument did *not* find, because the benchmark's domains never reached it — the reason the ulp table is not the whole verification story |

### Sources

| File | Relationship |
|------|--------------|
| `examples/cost_vs_libm.rs` | The worst-case table, `small_argument_sweep()`, and `amplification()` — everything on this page is one run of it |
| `src/measure.rs` | `ulp_diff`, the metric the Δulp column is measured with. Library code rather than example-local, so the test suite's bounds and this page's table are read off the same ruler |
| `tests/inc/measure_test.rs` | Grades that ruler before anything is measured with it — an error in `ulp_diff` would silently restate every figure on this page rather than fail |

### Tests

The per-function agreement bounds are asserted, not merely measured. Each family
test compares against the host libm over its own domain —
`tests/inc/exponential_test.rs`'s `exp2_matches_libm_across_its_whole_domain`
(≤2 ulp over `k ∈ [ −1074, 1023 ]`) and `exp_stays_correct_where_the_result_is_subnormal`
(≤2 ulp over `[ −745, −700 ]`) are the tightest; `circular_test.rs`,
`inverse_circular_test.rs` and `hyperbolic_test.rs` each carry a
`the_small_argument_band_keeps_every_digit` that asserts bit-identity where the
function is its own argument to the last place.

The benchmark's own coverage of the surface is asserted by
`tests/inc/contract_test.rs`'s `the_benchmark_measures_every_exported_function`,
so no function can be exported and held to this ceiling only nominally.
