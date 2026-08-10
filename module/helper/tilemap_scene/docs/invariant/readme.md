# Invariant Doc Definition

An **invariant** is a guarantee this crate enforces and callers may rely on. In `tilemap_scene`, that covers `RenderSpec` referential integrity, edge/vertex canonical uniqueness, compilation target purity, and deterministic compilation — each pinned down with its precise statement, enforcement mechanism, and violation consequences. This collection holds one instance per invariant, each pinned to where it is enforced in code; the table below is the index into them.

### Scope

- **Purpose**: Navigational hub for `tilemap_scene`'s correctness properties that must always hold.
- **Responsibility**: Document each invariant's precise statement, enforcement mechanism, and violation consequences.
- **In Scope**: `RenderSpec` referential integrity, edge/vertex canonical uniqueness, compilation target purity, deterministic compilation.
- **Out of Scope**: The schema fields the invariants constrain (see `format/`); the consumer-facing trap created where enforcement is incomplete (see `pitfall/`).

### Overview Table

| ID | Name | Purpose | Status |
|----|------|---------|--------|
| 001 | [RenderSpec Referential Integrity](001_renderspec_referential_integrity.md) | Every id reference resolves; enforced-vs-declared-only breakdown | ⚠️ |
| 002 | [Edge and Vertex Canonical Uniqueness](002_edge_and_vertex_canonical_uniqueness.md) | Each physical edge/vertex has exactly one canonical encoding | ✅ |
| 003 | [Compiles to Renderer Commands Only](003_compiles_to_renderer_commands_only.md) | Output is purely the `tilemap_renderer` command stream — no GPU or platform code | ✅ |
| 004 | [Deterministic Compilation](004_deterministic_compilation.md) | Same `(spec, scene, time, seed)` yields the identical command stream every run | ✅ |

Status ⚠️ marks an invariant whose enforcement is partial — see the file's own Enforcement Mechanism section.
