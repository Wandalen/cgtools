# Invariant Doc Definition

An **invariant** is a property this crate holds absolutely — not one measured
against a bound. All four here exist to protect the first: reproducibility is
the guarantee, and the other three are the structural facts that make it hold
rather than merely be intended.

They are stated separately because each fails independently and each is checked
by a different instrument: a scan of the crate's own sources, a look at the
resolved dependency graph, and a set of domain guards with their own tests.

### Scope

- **Purpose**: Navigational hub for the properties callers may rely on without measuring.
- **Responsibility**: Index each invariant's statement, enforcement, and violation consequence.
- **In Scope**: Determinism of returned bits; the libm ban; the dependency count; refusal behaviour at domain edges.
- **Out of Scope**: Bounded quality attributes — cost and accuracy (see `non_functional_requirement/`); the procedures operating under these guarantees (see `algorithm/`).

### Overview Table

| ID | Name | Purpose | Status |
|----|------|---------|--------|
| 001 | [Bit Reproducibility](001_bit_reproducibility.md) | Same input, same 64 bits, every target — the crate's whole contract | ✅ |
| 002 | [Pinned Operations Only](002_pinned_operations_only.md) | No shipping line reaches a libm entry point; mechanically scanned, with two counted exceptions | ✅ |
| 003 | [Zero Dependencies](003_zero_dependencies.md) | Empty dependency tables, because a dependency is a libm the scan cannot see | ✅ |
| 004 | [Refusal Over Meaningless Answer](004_refusal_over_meaningless_answer.md) | `NaN` rather than a plausible finite number where the machinery has run out of bits | ✅ |
