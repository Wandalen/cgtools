# Invariant: Pinned Operations Only

No shipping line of this crate calls a libm entry point. Every transcendental is
built from `+`, `−`, `×`, `÷`, `√` and `fma` — the six operations IEEE-754
requires to be correctly rounded — and a mechanical scan over the crate's own
sources fails the test suite if that stops being true.

### Scope

- **Purpose**: Turn "does not call libm" from an intention into a checked property, and record exactly which calls are permitted and how many.
- **Responsibility**: State the ban, name its two declared exceptions, describe the scan and the two meta-checks that keep the scan itself honest.
- **In Scope**: Method-form calls to any transcendental in `src/**`, outside doc comments and `#[ cfg( test ) ]` modules.
- **Out of Scope**: What *callers* write (see [pitfall/004](../pitfall/004_method_call_reintroduces_libm.md)); `examples/` and `tests/`, which compare against libm deliberately.

### Invariant Statement

For each file in `src/`, the shipping portion — everything before the trailing
`#[ cfg( test ) ]` module, with comment lines removed — contains exactly zero
occurrences of each of 28 banned method spellings, except for two declared
sites, each of which must occur exactly once:

| File | Spelling | Count |
|------|----------|------:|
| `algebraic.rs` | `.sqrt(` | 1 |
| `algebraic.rs` | `.mul_add(` | 1 |

The banned list covers every `f64` transcendental method: `.sin(`, `.cos(`,
`.tan(`, `.sin_cos(`, `.asin(`, `.acos(`, `.atan(`, `.atan2(`, `.sinh(`,
`.cosh(`, `.tanh(`, `.asinh(`, `.acosh(`, `.atanh(`, `.exp(`, `.exp2(`,
`.exp_m1(`, `.ln(`, `.ln_1p(`, `.log(`, `.log2(`, `.log10(`, `.powf(`,
`.powi(`, `.cbrt(`, `.hypot(`, `.sqrt(`, `.mul_add(`.

### Why `sqrt` And `mul_add` Are Banned Too

Both are pinned by IEEE-754. Both are as reproducible as `+`. Their presence on
the banned list is not a correctness claim.

It is an auditability one. The crate's premise is that a reader can find every
arithmetic entry point by looking in one place, and an unannounced call bypasses
that whatever its rounding behaviour. So the two legitimate uses are declared —
by *count*, not by permission. A permission would let a second, unwrapped
`.sqrt(` hide in the same file where one was expected; a count of exactly one
will not.

`round_half_away` exists for the same reason with the opposite conclusion.
`f64::round` has precisely the semantics the reductions need and is exact for
every value they produce — but it is a libm entry point on some targets, and the
two comparisons that replace it are not.

### Enforcement Mechanism

`tests/inc/contract_test.rs`'s `no_libm_call_survives_in_shipping_code` scans
the sources and asserts the counts above. Three properties make it a guard
rather than decoration:

- **The sources are `include_str!`-ed, not walked.** The check runs against the
  files this test binary was actually compiled from, so it cannot be defeated by
  a stale tree, a relocated directory, or a build from a different checkout.
- **Doc comments are stripped first.** The documentation quotes the banned calls
  deliberately — naming what not to write is part of its job. Scanning the raw
  text would make the check fail on correct code, and the usual response to that
  is to weaken it.

  The stripper cuts at `#[ cfg( test ) ]` as well, which for *this* crate now
  matches nothing: its tests live in `tests/`, so `src/` contains no inline test
  module for the cut to remove. That half is retained because the scan is
  written to be copied — [../pitfall/004](../pitfall/004_method_call_reintroduces_libm.md)
  hands it to consuming crates, where an inline `mod tests` comparing against
  libm is the point rather than a violation.
- **The scan's own scope is checked.** `include_str!` means a new module must be
  added to the `SOURCES` table by hand, which is exactly the step someone
  forgets. `every_source_file_is_covered_by_the_libm_scan` parses the `mod` and
  `pub mod` lines out of `lib.rs` and fails if any declared module is missing
  from `SOURCES`.

And the stripper is checked in turn. `the_scan_would_actually_catch_something`
runs it over a small synthetic source containing a real violation in shipping
code and another inside a `mod tests`, and asserts the first survives stripping
while the second does not. Without it, a stripper that over-eagerly removed
everything would make the whole scan pass vacuously — which is the failure mode
a scan-based guard actually has.

### Violation Consequences

A single reintroduced libm call silently voids
[001](001_bit_reproducibility.md) for every function downstream of it, and
nothing observable changes on the machine that introduced it. The value it
returns is correct there. It is correct on the CI runner. It differs on a user's
machine with a different libc, and the first symptom is a desynchronised
lockstep session or a replay that diverges partway through — arbitrarily far in
time and in call depth from the line that caused it.

That distance is the reason this is enforced mechanically rather than by review.
The defect is invisible at the point of introduction, invisible in the diff
(`x.sqrt()` next to `sqrt( x )` reads as a formatting preference), and only
visible in a symptom that does not name it.

### Example

Add an unannounced call to any shipping file:

```rust
// in src/circular.rs
pub fn tan( x : f64 ) -> f64
{
  x.tan()
}
```

then:

```sh
cargo test -p deterministic_math --test tests no_libm_call_survives
```

fails naming the file, the spelling, and both counts. Revert and it passes. To
see the second guard, instead add a new `mod` to `src/lib.rs` without adding it
to `SOURCES`, and watch `every_source_file_is_covered_by_the_libm_scan` fail.

That second guard is not a hypothetical. `pub mod measure;` was the most recent
module added to `lib.rs`, and it is the case above run for real: with the
`measure.rs` row held out of `SOURCES`, the guard fails with

```
`measure.rs` is declared in lib.rs but missing from the libm scan's SOURCES table
```

and with the row present it passes. That gap is the one worth closing — a module
missing from `SOURCES` is exempt from the ban above while reading exactly like a
module covered by it.

### Invariants

| File | Relationship |
|------|--------------|
| [001_bit_reproducibility.md](001_bit_reproducibility.md) | The guarantee this ban exists to make hold; a violation here voids it silently |
| [003_zero_dependencies.md](003_zero_dependencies.md) | The same ban at the other boundary — this scan sees only this crate's own sources, so a dependency could reach a libm it cannot check |

### Pitfalls

| File | Relationship |
|------|--------------|
| [../pitfall/004_method_call_reintroduces_libm.md](../pitfall/004_method_call_reintroduces_libm.md) | The identical trap on the caller's side of the boundary, where no scan of this crate can reach |

### Sources

| File | Relationship |
|------|--------------|
| `src/algebraic.rs` | The two declared call sites — `sqrt` and `mul_add`, each wrapping the pinned operation once — and `round_half_away`, which replaces `f64::round` for this reason |

### Tests

| File | Relationship |
|------|--------------|
| `tests/inc/contract_test.rs` | `no_libm_call_survives_in_shipping_code` (the scan), `every_source_file_is_covered_by_the_libm_scan` (its scope), `the_scan_would_actually_catch_something` (its stripper) |
