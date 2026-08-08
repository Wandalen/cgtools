# Invariant Doc Entity

### Scope

- **Purpose**: Navigational hub for `tilemap_scene`'s correctness properties that must always hold.
- **Responsibility**: Document each invariant's precise statement, enforcement mechanism, and violation consequences.
- **In Scope**: `RenderSpec` referential integrity, edge/vertex canonical uniqueness.
- **Out of Scope**: The schema fields the invariants constrain (see `format/`); the consumer-facing trap created where enforcement is incomplete (see `pitfall/`).

### Overview Table

| ID | Name | Purpose | Status |
|----|------|---------|--------|
| 001 | [RenderSpec Referential Integrity](001_renderspec_referential_integrity.md) | Every id reference resolves; enforced-vs-declared-only breakdown | ⚠️ |
| 002 | [Edge and Vertex Canonical Uniqueness](002_edge_and_vertex_canonical_uniqueness.md) | Each physical edge/vertex has exactly one canonical encoding | ✅ |

Status ⚠️ marks an invariant whose enforcement is partial — see the file's own Enforcement Mechanism section.
