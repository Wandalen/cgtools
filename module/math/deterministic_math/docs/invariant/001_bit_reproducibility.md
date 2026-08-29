# Invariant: Bit Reproducibility

The same input produces the same 64 bits, on every target, forever. This is the
crate's entire contract — the one property every design decision in it was made
to protect, and the only one it guarantees.

### Scope

- **Purpose**: State the guarantee precisely, name the three structural facts that make it hold, and record what breaks when it does not.
- **Responsibility**: Distinguish the guarantee from the accuracy it is routinely confused with, and enumerate its enforcement.
- **In Scope**: Determinism of every exported function's returned bits, across targets, toolchains and libc versions.
- **Out of Scope**: How close those bits are to the true value (see [non_functional_requirement/002](../non_functional_requirement/002_accuracy_against_host_libm.md)); reproducibility of code the *caller* writes around these calls.

### Invariant Statement

For every exported function `f` and every input `x`, `f( x )` returns the
identical bit pattern on any conforming IEEE-754 double-precision target, under
any toolchain version, linked against any libc, now and after any future change
that keeps this invariant.

`NaN` inputs are included in the shape of the guarantee but not in its letter:
a `NaN` in produces a `NaN` out, and the payload bits of that `NaN` are not
specified. Nothing may branch on them.

### Why It Does Not Come For Free

IEEE-754 fixes the result of exactly six operations: `+`, `−`, `×`, `÷`, `√`
and `fma`. Given the same inputs and rounding mode, every conforming machine
produces the same bits for those. It says nothing of the sort about `sin`,
`cos`, `atan`, `exp` or `ln` — for those, correct rounding is *recommended* and
not required, because proving the last bit right in every case runs into the
table-maker's dilemma: the exact result can sit arbitrarily close to a rounding
boundary, and no bounded working precision decides every input.

Conforming implementations are therefore free to differ in the final place, and
they do: between glibc and musl, between two glibc releases, and between an
x86-64 build and an aarch64 one compiled from identical source.

### Enforcement Mechanism

Three structural facts, each checkable:

- **No libm is reached.** Every transcendental is built from the pinned six.
  This is not a convention but a mechanically-scanned property — see
  [002](002_pinned_operations_only.md), which is the enforcement, not a
  restatement.
- **The coefficients travel with the crate.** Every series coefficient and
  reduction constant is a literal in `src/constant.rs` — including the
  three-part `PI/2` split, the two-part `ln 2` split, the arctangent bin table
  and the reciprocal-factorial table. Nothing is read from the platform, so the
  numbers are identical on every target by construction rather than by the
  coincidence of two vendors having chosen alike. Each table is additionally
  rebuilt from its own definition at run time and compared, so a mistyped digit
  fails a test rather than shifting an answer everywhere at once.
- **Nothing reads ambient state.** No function consults a rounding mode, an
  environment variable, a clock, an allocator or a global. Every function is a
  pure function of its arguments, which is what makes "same input, same bits" a
  statement about the whole call rather than about one evaluation of it.

Two smaller decisions serve the same end and are worth naming because they look
like fussiness otherwise. `scale2` uses bit manipulation rather than
`x * 2f64.powi( k )`, because that is a multiply chain whose association the
optimiser may reorder. And `powi` writes its own squaring chain rather than
calling `f64::powi`, for the same reason — here the order of every
multiplication is fixed by the loop.

### Violation Consequences

The drift itself is harmless. One ulp in a position is nothing, and a renderer
will never see it.

The first *branch* is not. Two machines evaluating the same threshold against
the same state return `true` and `false`, and from that instant they are
simulating different worlds rather than the same world with a rounding error in
it. Nothing downstream reconciles them, because there is no longer a shared
answer to converge on — the two runs are answering different questions.

An iterative solver reaches that point quickly, since each iteration multiplies
the input difference it was given. This is why the failure mode is
characteristically *late and total* rather than early and small: a lockstep
session desynchronises after some minutes of agreement, a replay diverges
partway through, a golden test passes on one CI architecture and fails on
another with no diff in the source.

### Example

The guarantee is about agreement across machines, which no single-machine test
can observe directly. What a single machine *can* check is the property that
makes the agreement possible — that the function is a pure function of its
argument, with no path-dependence, no accumulated state, and no dependence on
what was evaluated before it:

```rust
use deterministic_math::sin;
let once = sin( 0.5 );
for _ in 0 .. 1000
{
  let _ = sin( 1.7 );
  let _ = sin( -3.2e5 );
}
assert_eq!( sin( 0.5 ).to_bits(), once.to_bits() );
```

`tests/inc/contract_test.rs`'s `evaluation_is_bit_reproducible` asserts the same
property over every exported function, listed individually rather than sampled —
"representative" is a judgement, and a stateful function would be exactly the
one the judgement missed. It evaluates each function twice at the same argument
across 400 points and compares bits; the interleaving above is the stronger form
a reader can run by hand.

To see the property this crate exists to restore, compare the two spellings on
any two machines you have:

```sh
cargo run -p deterministic_math --release --example cost_vs_libm
```

The `max Δulp` column is the per-function disagreement between this crate and
*that host's* libm. Run it on a second architecture: the `det` column will be
identical between the two runs and the `libm` column will not.

### Non Functional Requirements

| File | Relationship |
|------|--------------|
| [../non_functional_requirement/002_accuracy_against_host_libm.md](../non_functional_requirement/002_accuracy_against_host_libm.md) | The property this one is constantly mistaken for; that instance opens by separating them |
| [../non_functional_requirement/001_cost_against_host_libm.md](../non_functional_requirement/001_cost_against_host_libm.md) | What this guarantee costs — the bounded price of not calling the host's own |

### Invariants

| File | Relationship |
|------|--------------|
| [002_pinned_operations_only.md](002_pinned_operations_only.md) | The mechanically-enforced ban that makes this guarantee hold rather than merely be intended |
| [003_zero_dependencies.md](003_zero_dependencies.md) | A dependency could reach a libm this crate cannot see; zero of them is how that stays impossible |

### Pitfalls

| File | Relationship |
|------|--------------|
| [../pitfall/004_method_call_reintroduces_libm.md](../pitfall/004_method_call_reintroduces_libm.md) | How a *caller* loses this guarantee — the crate's own enforcement stops at its own source |

### Sources

| File | Relationship |
|------|--------------|
| `src/lib.rs` | The crate-level documentation stating the guarantee and its limits |
| `src/constant.rs` | Every coefficient and reduction constant, carried rather than queried |
| `src/algebraic.rs` | `scale2` and `powi`, whose bit-manipulation and explicit squaring chain both exist to deny the optimiser a reassociation |

### Tests

| File | Relationship |
|------|--------------|
| `tests/inc/contract_test.rs` | `evaluation_is_bit_reproducible` (purity across the unary surface), `nan_propagates_rather_than_being_swallowed`, `constants_are_the_core_ones` |
| `src/constant.rs` — via the `mod tests` of its consumers | Each table rebuilt from its own definition: `RECIP_FACT` and both circular series in `circular.rs`, `ATAN_B`/`ATAN_V` in `inverse_circular.rs`, the `PI/2` split in `circular.rs`, both `atan`/`atanh` series in `algebraic.rs` |
