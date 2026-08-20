# Pattern: Invariant-Defined Stack

### Scope

- **Purpose**: Give the ecosystem an objective rule for what a "stack" is, which stack a crate belongs to, and when a new stack must be founded.
- **Responsibility**: Define stack identity as an invariant table plus the sibling/extension/crate classification rules and the membership test.
- **In Scope**: The classification rules and the current d2 / tile / d3 instantiation.
- **Out of Scope**: The full text of each invariant — pinned in crate-level `docs/invariant/` instances (linked below); the layering discipline inside a stack (see [002_strict_layering_one_step_drilldown.md](002_strict_layering_one_step_drilldown.md)).

### Problem

Grouping rendering crates by intuitive labels — "2D", "3D", "tile games" —
gives no decision procedure. Where does a vector-line crate belong? Is an
isometric renderer 2D or 3D? When a new crate half-fits two groups, teams
either force it in (eroding the group's guarantees) or found yet another
group (fragmenting the ecosystem). The labels name aesthetics, not contracts,
so nothing can be *checked*.

### Solution

A **stack is its table of rendering invariants and limitations** — the
assumptions every layer of that stack may rely on and the capabilities it
deliberately renounces. The table, not the name, is the identity.

Classification rules, applied to any candidate crate or capability:

1. **Contradiction founds a sibling.** If the candidate requires violating an
   existing stack's invariant (e.g. needs a depth buffer where the stack
   guarantees vector representability), it belongs to a *sibling stack* with
   its own table.
2. **Pure addition founds an extension.** If the candidate keeps every
   invariant of a stack and only adds new ones, it is an *extension stack* —
   it may reuse everything below the layers it constrains further (the tile
   stack extends d2 this way).
3. **Neither founds anything.** A candidate that fits inside an existing
   table is just a crate in that stack.

**Membership test**: a crate is in a stack iff it assumes no more than that
stack's invariant table. **Variance rule**: shareability decreases upward —
foundation layers serve every stack, mid layers serve compatible stacks,
top layers are stack-private. Foundation APIs therefore stay free of stack
vocabulary (no sprite, tile, or camera-space terms below L2).

Current instantiation (living identity cards in
[../render_stack/](../render_stack/readme.md); decision context in
[ADR-001](../adr/001_multi_stack_rendering_architecture.md); each invariant
pinned where it is enforced):

| Stack | Invariant instances |
|-------|---------------------|
| `d2` | `module/helper/tilemap_renderer/docs/invariant/` — 001 Y-up, 003 z-layer draw ordering, 004 vector representability |
| `tile` (extends `d2`) | `module/helper/tiles_tools/docs/invariant/002_lattice_address_primacy.md`; `module/helper/tilemap_scene/docs/invariant/` — 003 compiles to the command set only, 004 deterministic compilation |
| `d3` | `module/helper/renderer/docs/invariant/` — 001 depth-buffer visibility with OIT, 002 PBR metallic-roughness baseline, 003 HDR-internal tone-mapped output |

### Applicability

Apply when deciding where a new rendering crate lives, whether a proposed
feature is admissible in its host crate, or whether a new family of crates is
justified. Not applicable to non-rendering utility crates (math, input,
logging) — they sit beside the stacks, not in them.

### Consequences

- Membership and stack-founding become checkable claims instead of taste;
  reviews can cite a specific invariant instance.
- A feature request that contradicts the table gets a principled "not here —
  sibling stack or bridge" answer instead of scope creep.
- Cost: every load-bearing invariant must actually be written down as a
  crate-level `docs/invariant/` instance with an enforcement mechanism —
  an unpinned invariant protects nothing.
- Borderline crates (e.g. isometric rendering: lattice addresses over planar
  projection — tile stack, not d3) must be classified explicitly, once.

### ADRs

| File | Relationship |
|------|--------------|
| [../adr/001_multi_stack_rendering_architecture.md](../adr/001_multi_stack_rendering_architecture.md) | Adopts this pattern and aggregates the current stack tables |

### Render Stacks

| File | Relationship |
|------|--------------|
| [../render_stack/001_d2.md](../render_stack/001_d2.md) | The base invariant table this pattern's classification rules formalize |
| [../render_stack/002_tile.md](../render_stack/002_tile.md) | Instantiates rule 2 (pure addition founds an extension) over d2's table |
| [../render_stack/003_d3.md](../render_stack/003_d3.md) | Instantiates rule 1 (contradiction founds a sibling) against d2's table |

### Sources

| File | Relationship |
|------|--------------|
| `module/helper/tilemap_renderer/docs/invariant/` | d2 stack invariants pinned at enforcement site |
| `module/helper/tiles_tools/docs/invariant/` | tile stack — lattice address primacy |
| `module/helper/tilemap_scene/docs/invariant/` | tile stack — compilation target and determinism |
| `module/helper/renderer/docs/invariant/` | d3 stack invariants pinned at enforcement site |

### Tests

No workspace-level test can pin a classification rule; each linked invariant
instance carries its own Tests section where per-crate enforcement is
testable.
