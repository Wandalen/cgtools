# Non Functional Requirement Doc Definition

A **non functional requirement** is a quality attribute this crate is held to
rather than a behaviour it implements. Both instances here are quantities with
a ceiling, an instrument, and a standing measurement — and both are answered by
the same one command, which is deliberate: cost and accuracy are the two halves
of one trade, and refreshing either figure without the other would let them
drift apart.

Neither is the crate's guarantee. That is an invariant
([invariant/001](../invariant/001_bit_reproducibility.md)) — a property that
holds absolutely, not one measured against a bound.

### Scope

- **Purpose**: Navigational hub for the two measured quality attributes: what reproducibility costs, and how much accuracy it does or does not buy.
- **Responsibility**: Index each attribute's ceiling and the instrument that decides it.
- **In Scope**: Per-call cost against the host libm; per-function disagreement with it, in ulp.
- **Out of Scope**: Absolute guarantees (see `invariant/`); the procedures whose cost is being bounded (see `algorithm/`).

### Overview Table

| ID | Name | Purpose | Status |
|----|------|---------|--------|
| 001 | [Cost Against Host libm](001_cost_against_host_libm.md) | No function costs more than 5× the host's own; measured, worst is `exp2` at 4.82× | ✅ |
| 002 | [Accuracy Against Host libm](002_accuracy_against_host_libm.md) | Within 8 ulp everywhere but two recorded conditioning exceptions; measured, most functions 0–2 ulp | ✅ |
