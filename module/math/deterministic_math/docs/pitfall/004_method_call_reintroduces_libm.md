# Pitfall: Method Call Reintroduces libm

### Scope

- **Purpose**: Record the one way a caller silently loses everything this crate provides — writing `x.sin()` instead of `sin( x )` — and the fact that no test inside this crate can detect it.
- **Responsibility**: Document why the two spellings do not mean the same thing, why nothing warns, why the crate's own guard cannot reach it, and the two mechanisms a consumer can install instead.
- **In Scope**: Any code depending on this crate; all 28 method spellings in `LIBM_SPELLINGS`.
- **Out of Scope**: The scan that covers this crate's *own* sources, which is [invariant/002](../invariant/002_pinned_operations_only.md).

### Trap

Believing the import decides which function runs.

```rust
use deterministic_math::sin;

let a = sin( x );      // this crate
let b = x.sin();       // f64::sin — the platform's libm
```

Both lines are in scope of the same `use`. Both read as "the sine of `x`". They
call different functions, and the second one is the one the crate exists to
avoid.

The reason is that method syntax never consults imported free functions at all.
`x.sin()` resolves against `f64`'s *inherent* methods, and an inherent method
always wins — there is nothing a `use` statement can do about it, and no way to
shadow, hide, or override `f64::sin` from another crate. The import is not
ignored so much as irrelevant: it makes `sin( x )` available and leaves
`x.sin()` exactly as it was.

Nothing warns. Not `rustc`, not `cargo clippy` by default. Both spellings are
valid, both type-check, both return an `f64` of the right magnitude — the
difference is invisible at every layer that ordinarily catches mistakes.

### Failure

The two functions genuinely differ, and on a single machine, not only across
platforms. Sampling `x = i × 0.000137` for `i` in `1 ..= 200 000` on the
reference host:

| function | samples differing from the host in bits | first divergence |
|----------|----------------------------------------:|------------------|
| `tan` | 89 151 / 200 000 | `x = 0.000274` |
| `atan` | 59 786 / 200 000 | `x = 0.000548` |
| `sin` | 38 379 / 200 000 | `x = 0.000274` |
| `exp` | 20 178 / 200 000 | `x = 0.035072` |
| `ln` | 15 449 / 200 000 | `x = 0.020413` |

Around a fifth of `sin` calls and nearly half of `tan` calls land on a different
bit pattern. Each is within the accuracy ceiling
([non_functional_requirement/002](../non_functional_requirement/002_accuracy_against_host_libm.md))
— an ulp or so, nothing that looks like a bug — and that is precisely the
problem. The values are close enough that no assertion fails, no output looks
wrong, and no reviewer notices. What breaks is not the number but the
*reproducibility*: two builds that disagree about which spelling was used in one
place will diverge, and having diverged in the low bit they eventually take
different branches ([invariant/001](../invariant/001_bit_reproducibility.md)).

A mixed codebase is the worst case, because it looks the most correct. Someone
imports the crate, converts the call sites they can see, misses one inside a
closure or a macro, and the file now reads as fully converted. The remaining
`.sin(` is the only thing that matters and it is the one thing that looks
ordinary.

### Why The Crate's Own Guard Cannot Catch It

`no_libm_call_survives_in_shipping_code` scans a fixed table of seven
`include_str!`-ed files:

```rust
const SOURCES : [ ( &str, &str ) ; 7 ] =
[
  ( "lib.rs",       include_str!( "../../src/lib.rs" ) ),
  ( "constant.rs",  include_str!( "../../src/constant.rs" ) ),
  // ... five more
];
```

`include_str!` is what makes the guard trustworthy for this crate — it reads the
sources the test binary was actually compiled from, so it cannot be defeated by
a stale checkout or a relocated tree
([invariant/002](../invariant/002_pinned_operations_only.md)). It is also
exactly what confines it: a consumer's files are not in that list and cannot be
added to it. A crate cannot `include_str!` its dependents.

So the guarantee has a hard edge, and the edge is the crate boundary. Inside it,
"no libm is reached" is mechanically enforced on every test run. Outside it, the
crate has no visibility whatsoever, and a consumer who calls `x.sin()` gets libm
with no indication that anything is unusual. This is a structural limit, not a
gap someone forgot to close.

### Mitigation

Two mechanisms, in order of preference. A consumer wanting the crate's guarantee
to actually hold **must install one of them** — the crate cannot supply it.

**1. `clippy::disallowed_methods` — the cheap, complete one.**

Put a `clippy.toml` at the consuming crate's root:

```toml
disallowed-methods = [
  { path = "f64::sin", reason = "use deterministic_math::sin" },
  { path = "f64::cos", reason = "use deterministic_math::cos" },
  # ... one line per spelling in LIBM_SPELLINGS
]
```

and turn the lint on, in the crate root or in `[lints.clippy]`:

```rust
#![ deny( clippy::disallowed_methods ) ]
```

Verified on `rustc 1.97.1` / clippy 1.97.0: `x.sin()` is reported at the call
site with the `reason` string attached as a note, and — importantly — a method
*not* on the list is not reported. The list has to be complete to be worth
anything; there is no wildcard, and both `{ path, reason }` tables and bare
`"f64::cos"` strings are accepted in the same array.

This is the better mechanism because it fires at the exact call site, during an
ordinary `cargo clippy`, before the code is ever run.

**2. Copy the scan.** If clippy is not in the consumer's loop, the test in
`tests/inc/contract_test.rs` is written to be copied: `LIBM_SPELLINGS`,
`shipping_code`, and the assertion are about eighty lines with no dependencies,
and the consumer substitutes its own `SOURCES` table. Copy
`every_source_file_is_covered_by_the_libm_scan` along with it — a scan whose
file list can silently fall out of date is the failure mode that turns the guard
into decoration.

**What does not work:** trying to shadow `f64::sin`. There is no trait import,
`use` form, or prelude trick that makes `x.sin()` resolve elsewhere, because
inherent methods take priority over every trait method by construction. Do not
spend time looking for one.

### Verify It Yourself

That the import does not help:

```rust
use deterministic_math::sin;

let x = 1.0_f64;
assert_eq!( x.sin().to_bits(), f64::sin( x ).to_bits() );   // method syntax → the host
assert_eq!( sin( x ).to_bits(), sin( x ).to_bits() );       // free function → this crate
```

That the two are genuinely different functions — this prints a concrete argument
rather than asserting a difference that might not exist on every host:

```rust
use deterministic_math::sin;

for i in 1 ..= 200_000i64
{
  let v = i as f64 * 0.000_137;
  if sin( v ).to_bits() != v.sin().to_bits()
  {
    println!( "diverges at {v}: {:#x} vs {:#x}", sin( v ).to_bits(), v.sin().to_bits() );
    break;
  }
}
```

On the reference host this prints `diverges at 0.000274`, one ulp apart.

That the lint is actually wired up, once installed — add a deliberate violation
and confirm it is rejected:

```sh
cargo clippy 2>&1 | grep 'disallowed method'
```

A clean run with no violation present proves nothing; the check is to see the
lint fire on a call you added on purpose, then remove it.

### The Generalisable Lesson

**A guarantee enforced by scanning source has the scan's file list as its exact
boundary**, and that boundary is worth stating in the documentation rather than
leaving for a consumer to discover. This crate's scan is airtight over seven
files and empty over everything else; the useful thing to publish is not "no
libm is reached" but "no libm is reached *here*, and here is the mechanism you
need at your end."

### Invariants

| File | Relationship |
|------|--------------|
| [../invariant/002_pinned_operations_only.md](../invariant/002_pinned_operations_only.md) | The scan itself, its 28 spellings, its two counted exceptions, and the three-layer enforcement this document describes the outer boundary of |
| [../invariant/001_bit_reproducibility.md](../invariant/001_bit_reproducibility.md) | What is actually lost — the branch divergence a one-ulp difference eventually produces |

### Non Functional Requirements

| File | Relationship |
|------|--------------|
| [../non_functional_requirement/002_accuracy_against_host_libm.md](../non_functional_requirement/002_accuracy_against_host_libm.md) | Why the substituted values look fine: every divergence above is inside the crate's own accuracy ceiling |

### Pitfalls

| File | Relationship |
|------|--------------|
| [001_small_argument_cancellation.md](001_small_argument_cancellation.md) | The other pitfall with a caller-side half — there the advice is to reach for `exp_m1`/`ln_1p`, here it is to reach for the free function at all |
| [002_exponent_field_wraparound.md](002_exponent_field_wraparound.md) | An internal defect, for contrast: that one the crate could find and fix in its own sources, this one it structurally cannot |

### Api

| File | Relationship |
|------|--------------|
| [../api/001_function_surface.md](../api/001_function_surface.md) | The free-function surface to call instead, and the deliberate non-inventory of methods the crate does *not* wrap because they are exact |

### Sources

| File | Relationship |
|------|--------------|
| `src/lib.rs` | The `pub use` surface — every name here is a free function, and none of them is a method |
| `readme.md` | States the calling convention up front, this being the first thing a consumer gets wrong |

### Tests

| File | Relationship |
|------|--------------|
| `tests/inc/contract_test.rs` | `no_libm_call_survives_in_shipping_code` — the guard, and the template a consumer copies |
| `tests/inc/contract_test.rs` | `every_source_file_is_covered_by_the_libm_scan` — the reason the file list cannot silently go stale, and the second thing to copy |
| `tests/inc/contract_test.rs` | `the_scan_would_actually_catch_something` — proves the stripper leaves genuine calls intact while removing doc comments and `mod tests`, so a clean scan is not clean by accident |
