# Algorithm Doc Definition

An **algorithm** here is a procedure that turns an argument the series cannot
handle into one it can, and puts the result back together afterwards. All 28
exported functions reduce to six such procedures plus two Taylor kernels — the
kernels are the easy part, and every design decision worth documenting is in the
reduction around them.

Each instance names the naive form it replaces, because in this crate the naive
form is usually not merely less accurate but *wrong* — returning `NaN` for a
real answer, or infinity for a finite one, or the argument's leading digits
subtracted away.

### Scope

- **Purpose**: Navigational hub for the reduction and composition procedures behind the exported functions.
- **Responsibility**: Index each procedure, the functions built on it, and the failure it exists to avoid.
- **In Scope**: Argument reduction, series selection, branch thresholds, and reassembly.
- **Out of Scope**: The properties these procedures uphold (see `invariant/`); the defects that shaped them (see `pitfall/`); their measured cost and accuracy (see `non_functional_requirement/`).

### Overview Table

| ID | Name | Purpose | Status |
|----|------|---------|--------|
| 001 | [Cody-Waite Range Reduction](001_cody_waite_range_reduction.md) | Strip an integer multiple of `ln 2` or `π/2` using a multi-part split — `exp`, `exp2`, `sin`, `cos`, `tan`, `sin_cos` | ✅ |
| 002 | [Arctangent Bin Reduction](002_arctangent_bin_reduction.md) | Reduce onto one of four bin centres, and bypass entirely near zero — `atan`, `atan2`, `asin`, `acos` | ✅ |
| 003 | [Logarithm By Mantissa Split](003_logarithm_by_mantissa_split.md) | Read the exponent out of the bit pattern, re-centre the mantissa, run an `atanh` series — `ln`, `log2`, `log10`, `log` | ✅ |
| 004 | [Cancellation-Free Hyperbolic Forms](004_cancellation_free_hyperbolic_forms.md) | Rewrite each closed form so the leading `1` is never constructed — `exp_m1`, `ln_1p`, and all six hyperbolics | ✅ |
| 005 | [Cube Root By Exponential Round Trip](005_cube_root_by_exponential_round_trip.md) | `exp( ln m / 3 )` plus one Newton step, with odd symmetry `powf` cannot express — `cbrt` | ✅ |
| 006 | [Algebraic Compositions](006_algebraic_compositions.md) | The two built from pinned operations alone, each replacing a naive form that overflows or refuses — `hypot`, `powi` | ✅ |
