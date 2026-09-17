# Api Doc Definition

The crate's public surface, as a contract rather than as generated reference.
Rustdoc already lists the signatures; what this collection adds is the part
rustdoc cannot express — which function to reach for when two would compile,
what each returns outside its domain, and which absences are deliberate.

One instance covers the whole surface, because the surface is one flat module
with no types, traits, or generics in it. Splitting 28 free functions across
several documents would fragment the one property they share.

### Scope

- **Purpose**: Navigational hub for the crate's public contract.
- **Responsibility**: Index the surface documentation and its inventory counts.
- **In Scope**: Exported functions and constants, signatures, domains, refusal behaviour, calling convention, and deliberate absences.
- **Out of Scope**: How anything is computed (see `algorithm/`); what the surface guarantees (see `invariant/`); measured cost and accuracy (see `non_functional_requirement/`); the ways callers get it wrong (see `pitfall/`).

### Overview Table

| ID | Name | Purpose | Status |
|----|------|---------|--------|
| 001 | [Function Surface](001_function_surface.md) | All 28 functions and 7 constants with signature, domain and refusal behaviour; the free-function calling convention; the 14 methods deliberately not wrapped | ✅ |

### Inventory At A Glance

| | Count | Note |
|-|------:|------|
| Exported functions | 28 | 21 unary, 6 multi-argument, plus `sin_cos` returning a pair |
| Exported constants | 7 | 6 re-exported from `core`, 1 originated here (`SIN_COS_MAX`) |
| Implementing modules | 5 | `algebraic`, `circular`, `exponential`, `hyperbolic`, `inverse_circular` |
| Deliberately absent | 14 | `abs`, `floor`, `ceil`, … — exact by construction, so `f64`'s own methods already reproduce |

These counts are derived from `src/lib.rs`'s `pub use` lines by three tests
rather than maintained by hand; the tests widen automatically when a function is
added. What they cannot check is whether the numbers in this table were updated
alongside — re-derive with `grep '^pub use' src/lib.rs` if anything looks off.
