# Algorithm Doc Definition

An **algorithm** documents a HOW — a step-by-step computational procedure with correctness properties worth stating explicitly. In `shader_chunks_params`, this collection is the navigational hub for the one algorithmic procedure the crate implements: resolving a range for a tunable parameter that declared none. This collection holds one instance per algorithm; the table below is the index into them.

### Scope

- **Purpose**: Navigational hub for `shader_chunks_params`'s step-by-step computational procedures.
- **Responsibility**: Document each algorithm's abstract behavior and concrete step-by-step procedure.
- **In Scope**: The range-inference heuristic `infer_range` implements.
- **Out of Scope**: The `//@ param:` grammar and taxonomy this algorithm's result feeds into (see `api/001`).

### Overview Table

| ID | Name | Purpose | Status |
|----|------|---------|--------|
| 001 | [Range Inference Heuristic](001_range_inference_heuristic.md) | Two-stage deterministic range resolution: name-substring pattern first, WGSL-type fallback second | ✅ |
