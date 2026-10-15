# Non Functional Requirement Doc Definition

A **non-functional requirement** instance documents one quality attribute the crate is held to — a constraint every feature must respect, rather than a feature in itself. In `minwebgpu`, that means tracking measurable targets, such as abstraction-overhead performance, that aren't verifiable by reading the source alone, together with the method intended to measure each one. This collection holds one instance per requirement; the table below is the index into them.

### Scope

- **Purpose**: Track measurable quality-attribute targets that are not verifiable by reading source alone.
- **Responsibility**: Document performance/quality targets and their intended measurement method.
- **In Scope**: Abstraction-overhead performance target.
- **Out of Scope**: Functional correctness (see `feature/`).

### Overview Table

| ID | Name | Purpose | Status |
|----|------|---------|--------|
| 001 | [Minimal Abstraction Overhead](001_minimal_abstraction_overhead.md) | Under-5%-CPU-overhead-vs-raw-`web-sys` target (currently unbenchmarked) | ⚠️ |
