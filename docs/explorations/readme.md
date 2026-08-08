# Design Exploration Doc Definition

### Scope

- **Purpose**: Hold open multi-crate design investigations — questions under active study that have not yet produced a decision.
- **Responsibility**: Document each exploration's objective, the approaches investigated, their comparison, and a recommendation with next steps.
- **In Scope**: Ecosystem-level questions whose answer would bind multiple crates.
- **Out of Scope**: Decisions already made (see [../adr/readme.md](../adr/readme.md) — an exploration that concludes graduates into an ADR); single-crate evaluations (see that crate's own `docs/`, e.g. `tiles_tools/docs/architectural_evaluation/`).

### Overview Table

| ID | Name | Purpose | Status |
|----|------|---------|--------|
| 001 | [GPU HAL: Buy vs Build](001_gpu_hal_buy_vs_build.md) | Whether the L1 hardware abstraction layer should be `wgpu`, an in-house crate, or deferred | 🔄 Open |
