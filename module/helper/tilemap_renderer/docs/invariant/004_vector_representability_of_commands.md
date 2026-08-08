# Invariant: Vector Representability of Commands

Every command in the stream can be rendered as resolution-independent,
declarative output — proven continuously by the SVG backend. This is the d2
stack's export guarantee: what this crate can draw, it can also *serialize
as a drawing*.

### Scope

- **Purpose**: Guarantee that the command vocabulary never grows a member expressible only as GPU side effects — the d2 stack's vector-representability invariant.
- **Responsibility**: State the property, name the structural enforcement, and record what would break if it were violated.
- **In Scope**: The public `RenderCommand` set and the asset vocabulary it references.
- **Out of Scope**: Rendering *quality* parity between backends (a backend may approximate — see `Capabilities`); which commands an individual adapter has implemented so far (see each `feature/` instance and `roadmap.md`).

### Invariant Statement

Every `RenderCommand`, with the assets it references, has a complete
declarative representation: it can be rendered to a resolution-independent
text format (SVG) with no access to a GPU, a window, or any ambient runtime
state. Raster content (bitmap sprites) participates as *embedded data*
(PNG-encoded `data:` URIs), which bounds its fidelity but not its
representability.

### Enforcement Mechanism

- **Closed, POD command set**: all commands are plain-old-data `Copy` types
  (`src/commands.rs`) referencing assets by typed `ResourceId<T>` — no GPU
  handles, callbacks, or backend objects can appear in a command, so no
  command can *depend* on GPU state to be meaningful.
- **The SVG adapter is the living proof**: `src/adapters/svg.rs` implements
  the full command set (paths, text, sprites, meshes, batches, groups,
  effects, gradients, patterns, blend modes — see `roadmap.md`, "SVG adapter —
  full implementation"). A proposed command that cannot be given an SVG
  rendering fails this invariant and must be rejected or redesigned.
- **Capability declaration**: where a backend approximates or omits (e.g.
  blend modes), `Backend::capabilities` declares it — degradation is
  *declared*, never silent redefinition of a command's meaning.

### Violation Consequences

- Adding a command expressible only as GPU side effects (e.g. "bind this
  native texture handle", a readback-dependent effect) would break the SVG
  and terminal backends, and with them the d2 stack's headless/off-screen
  export path — snapshot testing, server-side rendering, and document output
  all rest on this invariant.
- Downstream, `tilemap_scene` compiles scenes to this command set
  (`tilemap_scene/docs/invariant/003`); a non-representable command would
  make whole scenes silently unexportable.

### Features

| File | Relationship |
|------|--------------|
| [feature/001_svg_backend_adapter.md](../feature/001_svg_backend_adapter.md) | The backend whose completeness constitutes the proof |
| [feature/002_webgl2_backend_adapter.md](../feature/002_webgl2_backend_adapter.md) | GPU backend — accelerates the same declarative stream, adds nothing unrepresentable |

### Sources

| File | Relationship |
|------|--------------|
| `roadmap.md` | Records SVG adapter completeness and the POD-command design decision |
| `src/adapters/svg.rs` | Full-command-set SVG implementation |
| `src/assets.rs` | Asset vocabulary (images, sprites, geometries, gradients, patterns) — all declarative |
| `src/commands.rs` | The closed POD `RenderCommand` set |

### Tests

| File | Relationship |
|------|--------------|
| `tests/commands_test.rs` | Pins the POD command vocabulary |
| `tests/assets_test.rs` | Pins the declarative asset vocabulary |
