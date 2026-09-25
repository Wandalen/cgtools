# Pitfall Doc Definition

A **pitfall** here is a defect that was actually found in this crate, or one a
consumer of it reliably walks into. Not a hypothetical — each instance names the
value the code returned, the mechanism that produced it, and the test that now
fails if it comes back.

Every one of them shares a shape worth naming: **the wrong answer looked
right**. `-9.12e292` from `exp`, `-1023` from `log2`, a `sinh` satisfying its own
identity to the last bit while wrong by 7 163 ulp, an `x.sin()` reading exactly
like the `sin( x )` beside it. None of these announced themselves, and none was
caught by a test that was already passing.

So each instance carries a `Why It Escapes Ordinary Testing` section, and that
section is the point of the document. The fixes are short and mostly obvious in
hindsight; the reason a competent test suite ran green over each defect for as
long as it did is not.

### Scope

- **Purpose**: Navigational hub for the defects this crate has had and the trap a consumer is most likely to hit.
- **Responsibility**: Index each defect, its mechanism, and the generalisable lesson it carries.
- **In Scope**: Found-and-fixed defects with a recorded failing value; consumer-side traps the crate cannot defend against itself.
- **Out of Scope**: The constructive procedures that resolve them (see `algorithm/`); the properties they threatened (see `invariant/`); the measurements that graded the repairs (see `non_functional_requirement/`).

### Overview Table

| ID | Name | Purpose | Status |
|----|------|---------|--------|
| 001 | [Small-Argument Cancellation](001_small_argument_cancellation.md) | The class behind every large accuracy defect — eight functions, worst 32 768 ulp; why identity tests and absolute bounds are blind to it | ✅ |
| 002 | [Exponent Field Wraparound](002_exponent_field_wraparound.md) | `exp( -745 )` returned `-9.12e292`; a sign-extending cast in the `2ᵏ` factor, and why a `[ -300, 300 ]` sweep could not reach it | ✅ |
| 003 | [Subnormal Mantissa Extraction](003_subnormal_mantissa_extraction.md) | `log2( 5e-324 )` returned `-1023`; the implicit leading bit that subnormals do not have, and why the round trip stayed self-consistent through it | ✅ |
| 004 | [Method Call Reintroduces libm](004_method_call_reintroduces_libm.md) | `x.sin()` is not `sin( x )`, no import changes that, and no test in this crate can see it — the consumer-side mechanisms that can | ✅ |

### Reading Order

001 first — it is the largest class and the one whose lesson (test relatively,
across decades, against something built differently) generalises furthest.
002 and 003 are a matched pair on the same format detail from opposite sides,
and are best read together. 004 last, because it is the only one aimed at a
reader outside this crate rather than inside it, and it is the only one that
cannot be fixed here.
