# Invariant: Lattice Address Primacy

Game state lives at typed lattice coordinates; pixel positions are derived
projections for presentation and input, never the stored truth. This is the
tile stack's defining invariant — what makes it an extension of the d2 stack
rather than just more 2D code.

### Scope

- **Purpose**: State that authoritative spatial state in `tiles_tools` is addressed by grid coordinate types, with pixel space strictly downstream.
- **Responsibility**: Pin the property, the type-system friction that enforces it, and the failure modes when callers store pixels as state.
- **In Scope**: The role split between the four grid coordinate families and `Pixel` across the crate's own storage and ECS surfaces.
- **Out of Scope**: The coordinate types' internal design (see [type/001_coordinate_system_type_model.md](../type/001_coordinate_system_type_model.md)); conversion formulas (see [algorithm/005_coordinate_system_conversion.md](../algorithm/005_coordinate_system_conversion.md)); the triangular sum constraint (see [001_triangular_coordinate_sum_constraint.md](001_triangular_coordinate_sum_constraint.md)).

### Invariant Statement

Every crate-owned surface that stores or keys spatial state — `Grid2D`
storage, ECS `Position<C>` components, pathfinding, field-of-view — addresses
space by a typed lattice coordinate (`hexagonal::`, `square::`,
`triangular::`, or `isometric::Coordinate`). `Pixel` values appear only as
*outputs* of grid→pixel projection (rendering, layout) or *transient inputs*
to pixel→grid picking; they are never the stored, serialized, or compared
authority.

### Enforcement Mechanism

Enforcement is by type-system friction on the crate's own surfaces, not a
runtime check:

- **Serialization asymmetry**: every grid coordinate type derives (or
  hand-implements) `Serialize`/`Deserialize`; `Pixel` deliberately has
  neither (`src/coordinates/pixel.rs`). A component or save-file type built
  on `Pixel` fails to derive — persisting pixel state is a compile error by
  default, not a lint.
- **Capability asymmetry**: `Distance`, `Neighbors`, pathfinding, and
  field-of-view are implemented for grid coordinate types only; `Pixel` has
  no game-logic vocabulary to misuse.
- **Explicit, direction-marked conversion**: grid→pixel is exact projection;
  pixel→grid exists only as `ApproximateConvert` (see `algorithm/005`),
  marking the lossy direction in the trait name.
- **Boundary**: user-defined components outside this crate can still store
  `Pixel` — the invariant binds the crate's own vocabulary and the friction
  it exports, and cannot bind foreign code.

### Violation Consequences

- State stored as `Pixel` loses the entire grid vocabulary: no neighbors, no
  distance, no pathfinding, no field-of-view — those capabilities exist only
  at lattice addresses.
- Round-tripping state through pixel space and back re-derives cells via
  approximate conversion — accumulated drift moves entities between cells.
- Mixing spaces also mixes axis conventions: `Pixel` documents Y-down while
  `square::Coordinate` documents Y-up (see `type/001`'s validation notes) —
  arithmetic across the boundary silently inverts vertical direction.

### Algorithms

| File | Relationship |
|------|--------------|
| [algorithm/005_coordinate_system_conversion.md](../algorithm/005_coordinate_system_conversion.md) | The sanctioned crossing points between lattice and pixel space, exact one way, approximate the other |

### Types

| File | Relationship |
|------|--------------|
| [type/001_coordinate_system_type_model.md](../type/001_coordinate_system_type_model.md) | The phantom-typed coordinate families, and `Pixel`'s documented lack of `Serialize`/`Deserialize` |
| [type/002_ecs_component_vocabulary.md](../type/002_ecs_component_vocabulary.md) | `Position<C>` — spatial state componentized over lattice coordinates |

### Sources

| File | Relationship |
|------|--------------|
| `src/coordinates.rs` | Grid-only `Distance`/`Neighbors` trait vocabulary |
| `src/coordinates/pixel.rs` | `Pixel` — no phantom marker, no `Serialize`/`Deserialize`, Y-down doc comment |

### Tests

| File | Relationship |
|------|--------------|
| — | No dedicated test pins the asymmetry; it holds structurally (a `#[derive(Serialize)]` on a `Pixel`-bearing type failing to compile is the de facto check) |
