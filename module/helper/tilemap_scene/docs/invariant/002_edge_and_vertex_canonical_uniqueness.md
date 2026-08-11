# Invariant: Edge and Vertex Canonical Uniqueness

### Scope

- **Purpose**: State the property that every edge and every vertex in a hex grid is represented by exactly one canonical encoding, and that a renderer emits each exactly once per frame.
- **Responsibility**: Document the canonical-form rule for `Edge` and `Vertex` anchors and how the compile layer enforces single-emission.
- **In Scope**: Edge canonicalization (`(hex, direction)` pair selection), Vertex canonicalization (sorted corner tuple + rotation), the single-emission guarantee this enables.
- **Out of Scope**: General cross-reference resolution (see `invariant/001`); how `EdgeConnectedBitmask`/`VertexCorners` use the canonical form once computed (see `format/005`).

### Invariant Statement

For every edge shared by two hex cells A and B: exactly one of the two possible encodings — `(A, dir_A→B)` or `(B, dir_B→A)` — is canonical, chosen as the pair whose hex has the lexicographically smaller `(q, r)`. A conforming renderer emits each physical edge exactly once per frame, using only its canonical encoding. Symmetrically, for every vertex shared by the hex grid's dual-mesh cells (3 cells per vertex on a hex grid): the corner-cell tuple has exactly one canonical ordering — the terrain ids at the corners sorted lexicographically, plus a `rotation` integer (∈ {0, 1, 2} for hex) recording which cyclic permutation of the physical corners produced that sorted order — and a conforming renderer emits each physical vertex exactly once per frame.

Both rules exist because an edge is discoverable independently from each of its two adjacent cells, and a vertex independently from each of its three adjacent cells — without a canonical form, a naive per-cell walk would emit every edge twice and every vertex three times.

### Enforcement Mechanism

`src/compile/edges.rs` provides `canonical_edge()`, which reduces both possible `(hex, direction)` encodings of a physical edge to the same `(canonical_hex, dir)` key — the per-cell frame walk (`algorithm/002`) that would otherwise visit an edge from both adjacent cells collapses onto this one key, so `EdgeConnectedBitmask` sampling and edge-instance emission (see `format/005`, `format/008`) happen exactly once. `src/compile/vertex.rs` performs the analogous reduction for vertex corner tuples, computing the sorted-corners-plus-rotation canonical form `VertexCorners` matching (`format/005`) reads against.

Unlike `invariant/001`'s referential-integrity rules, this invariant is enforced structurally by the compile-layer functions themselves (every caller that needs a canonical edge/vertex goes through `canonical_edge()`/its vertex counterpart — there is no alternate, non-canonicalizing path to reach an edge or vertex draw call) rather than by a separate load-time validation pass. There is no `ValidationError` variant for "duplicate edge/vertex emission" because the canonicalization functions make the violation structurally unreachable from `format/005`/`algorithm/002`'s own call sites, not because a check was deferred.

### Violation Consequences

If a future code path bypassed `canonical_edge()`/the vertex equivalent and worked from a raw, non-canonicalized `(hex, direction)` pair or corner tuple instead, the practical consequence would be double- or triple-emission: the same physical edge drawn once per adjacent cell that names it (up to 2×), or the same physical vertex drawn once per adjacent cell (up to 3×) — visually a doubled/tripled sprite at that location, and for `EdgeConnectedBitmask`/`VertexCorners` autotile lookups, a bitmask computed from the wrong endpoint's perspective producing a mismatched or rotated sprite. This is a correctness property of the compile layer's own call graph, not a spec-author-triggerable failure mode — a spec cannot itself construct a non-canonical edge/vertex reference, since `format/008`'s `edges[]` list is keyed by canonical `EdgePosition` already.

### Formats

| File | Relationship |
|------|--------------|
| [format/003_anchor_placement_types.md](../format/003_anchor_placement_types.md) | States the canonicalization rule as part of the `Edge`/`Vertex` anchor definitions |
| [format/005_sprite_sources.md](../format/005_sprite_sources.md) | `EdgeConnectedBitmask`/`VertexCorners` read neighbour state through this canonical form |

### Sources

| File | Relationship |
|------|--------------|
| `src/compile/edges.rs` | `canonical_edge()` |
| `src/compile/vertex.rs` | Vertex canonicalization |

### Tests

| File | Relationship |
|------|--------------|
| `src/compile/edges.rs` | Inline `#[cfg(test)]`: `canonical_picks_smaller_hex` and 6 others |
