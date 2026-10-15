# Doc Definitions

## Master Doc Definitions Table

| Type | Purpose | Master File | Instances |
|------|---------|-------------|----------:|
| `api/` | The public surface as a contract — signatures, domains, deliberate absences | [api/readme.md](../api/readme.md) | 1 |
| `invariant/` | The four properties the crate guarantees, and their enforcement | [invariant/readme.md](../invariant/readme.md) | 4 |
| `algorithm/` | The six reduction and composition procedures behind all 28 functions | [algorithm/readme.md](../algorithm/readme.md) | 6 |
| `pitfall/` | Defects actually found, with the value each returned and why tests missed it | [pitfall/readme.md](../pitfall/readme.md) | 4 |
| `non_functional_requirement/` | The measured cost and accuracy ceilings, with the standing measurement | [non_functional_requirement/readme.md](../non_functional_requirement/readme.md) | 2 |

## Master Doc Instances Table

| Definition | ID  | Name | File |
|---------|-----|------|------|
| api | 001 | Function Surface | [api/001_function_surface.md](../api/001_function_surface.md) |
| invariant | 001 | Bit Reproducibility | [invariant/001_bit_reproducibility.md](../invariant/001_bit_reproducibility.md) |
| invariant | 002 | Pinned Operations Only | [invariant/002_pinned_operations_only.md](../invariant/002_pinned_operations_only.md) |
| invariant | 003 | Zero Dependencies | [invariant/003_zero_dependencies.md](../invariant/003_zero_dependencies.md) |
| invariant | 004 | Refusal Over Meaningless Answer | [invariant/004_refusal_over_meaningless_answer.md](../invariant/004_refusal_over_meaningless_answer.md) |
| algorithm | 001 | Cody-Waite Range Reduction | [algorithm/001_cody_waite_range_reduction.md](../algorithm/001_cody_waite_range_reduction.md) |
| algorithm | 002 | Arctangent Bin Reduction | [algorithm/002_arctangent_bin_reduction.md](../algorithm/002_arctangent_bin_reduction.md) |
| algorithm | 003 | Logarithm By Mantissa Split | [algorithm/003_logarithm_by_mantissa_split.md](../algorithm/003_logarithm_by_mantissa_split.md) |
| algorithm | 004 | Cancellation-Free Hyperbolic Forms | [algorithm/004_cancellation_free_hyperbolic_forms.md](../algorithm/004_cancellation_free_hyperbolic_forms.md) |
| algorithm | 005 | Cube Root By Exponential Round Trip | [algorithm/005_cube_root_by_exponential_round_trip.md](../algorithm/005_cube_root_by_exponential_round_trip.md) |
| algorithm | 006 | Algebraic Compositions | [algorithm/006_algebraic_compositions.md](../algorithm/006_algebraic_compositions.md) |
| pitfall | 001 | Small-Argument Cancellation | [pitfall/001_small_argument_cancellation.md](../pitfall/001_small_argument_cancellation.md) |
| pitfall | 002 | Exponent Field Wraparound | [pitfall/002_exponent_field_wraparound.md](../pitfall/002_exponent_field_wraparound.md) |
| pitfall | 003 | Subnormal Mantissa Extraction | [pitfall/003_subnormal_mantissa_extraction.md](../pitfall/003_subnormal_mantissa_extraction.md) |
| pitfall | 004 | Method Call Reintroduces libm | [pitfall/004_method_call_reintroduces_libm.md](../pitfall/004_method_call_reintroduces_libm.md) |
| non_functional_requirement | 001 | Cost Against Host libm | [non_functional_requirement/001_cost_against_host_libm.md](../non_functional_requirement/001_cost_against_host_libm.md) |
| non_functional_requirement | 002 | Accuracy Against Host libm | [non_functional_requirement/002_accuracy_against_host_libm.md](../non_functional_requirement/002_accuracy_against_host_libm.md) |

## Reading Paths

Three entry points, depending on why the crate is open:

**Using it.** [api/001](../api/001_function_surface.md) for the surface, then
[pitfall/004](../pitfall/004_method_call_reintroduces_libm.md) for the one
mistake that silently voids the guarantee, then
[invariant/001](../invariant/001_bit_reproducibility.md) for what is actually
promised — and, just as important, what is not.

**Auditing it.** [invariant/002](../invariant/002_pinned_operations_only.md) and
[invariant/003](../invariant/003_zero_dependencies.md) are the two mechanically
enforced claims; each names its guard test and the command that runs it.
[non_functional_requirement/001](../non_functional_requirement/001_cost_against_host_libm.md)
and [002](../non_functional_requirement/002_accuracy_against_host_libm.md) carry
the standing measurements, reproducible with one example run.

**Changing it.** `algorithm/` first — every function reduces to one of its six
procedures, and each instance names the naive form it replaces and why. Then
`pitfall/`, which is the more valuable half: the fixes are short and obvious in
hindsight, but the reason a passing test suite ran green over each defect is
not, and repeating any of those testing mistakes is easier than repeating the
implementation ones.
