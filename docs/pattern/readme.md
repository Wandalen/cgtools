# Pattern Doc Definition

A **pattern** here is a reusable design rule applied — or intended to be applied — across more than one crate, distinct from a single-crate pattern that lives inside that crate's own `docs/pattern/`. This collection holds one **instance** — one pattern file — per rule; the table below is the index into them.

### Scope

- **Purpose**: Document the reusable cross-crate design rules the rendering ecosystem is built on — solutions applied (or intended) across multiple crates, not inside one.
- **Responsibility**: Document each pattern's problem, solution, applicability, and consequences, with its known uses in this workspace.
- **In Scope**: Ecosystem-shaping patterns — the rules [ADR-001](../adr/001_multi_stack_rendering_architecture.md) rests on, and the reusable forms its layers take (e.g. the two L5 script forms).
- **Out of Scope**: Single-crate patterns (see that crate's own `docs/pattern/`, e.g. `tilemap_renderer/docs/pattern/001_ports_and_adapters_backend_architecture.md`); the decision record that adopts these patterns (see [../adr/readme.md](../adr/readme.md)).

### Overview Table

| ID | Name | Purpose | Status |
|----|------|---------|--------|
| 001 | [Invariant-Defined Stack](001_invariant_defined_stack.md) | A stack is its table of rendering invariants; contradiction founds a sibling, addition founds an extension | ✅ |
| 002 | [Strict Layering with One-Step Drill-Down](002_strict_layering_one_step_drilldown.md) | Depend only one layer down; expose the layer below through an explicit handle | ✅ |
| 003 | [Cross-Stack Bridge via Foundation Resources](003_cross_stack_bridge_via_foundation_resources.md) | Stacks compose through textures and command streams, never through each other's scene abstractions | ✅ |
| 004 | [Script-as-Data](004_script_as_data.md) | The scene script is a declarative document; a deterministic compiler is its only executor | ✅ |
| 005 | [Script-as-Glue](005_script_as_glue.md) | The scene script is a program over a deliberately bound engine API | ✅ |
