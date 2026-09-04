# Architectural Evaluation: ECS Library Selection

### Scope

- **Purpose**: Record why `hecs` was selected over `bevy_ecs` and `specs` as the ECS foundation for `tiles_tools`, migrated from the crate's original `docs/ecs_decision.md` ADR.
- **Responsibility**: Preserve the original evaluation's criteria, per-library ratings, and recommendation in `analysis.rulebook.md`-conformant Concern Legend / Trade-off Matrix / Verdict form.
- **In Scope**: `hecs` vs `bevy_ecs` vs `specs`, evaluated across the 7 concerns the original ADR named.
- **Out of Scope**: Whether the shipped `ecs::World` API actually preserves the original decision's planned abstraction layer — see `api/001`'s Compatibility Guarantees for that divergence.

### Concern Legend

| Concern | Meaning |
|---------|---------|
| Performance | Runtime performance for typical game operations |
| API Ergonomics | Developer experience and ease of use |
| Compile Times | Impact on build times |
| Feature Set | Advanced features such as change detection |
| Maintenance | Community support and active development |
| Dependency Overhead | Size of the dependency tree pulled in |
| Workspace Compatibility | Alignment with the existing cgtools ecosystem and its stated requirements |

### Trade-off Matrix

| Concern | Weight | hecs | bevy_ecs | specs |
|---------|--------|------|----------|-------|
| Performance | 🔶 High | 🟢🟢 | 🟢🟢 | 🟢 |
| API Ergonomics | 🔶 High | 🟢 | 🟢🟢 | 🔴 |
| Compile Times | 🔶 High | 🟢🟢 | 🔴🔴 | 🟢 |
| Feature Set | 🔷 Medium | 🟢 | 🟢🟢 | 🟢 |
| Maintenance | 🔷 Medium | 🟢 | 🟢🟢 | 🔴🔴 |
| Dependency Overhead | 🔶 High | 🟢🟢 | 🔴🔴 | 🔴 |
| Workspace Compatibility | 🔶 High | 🟢🟢 | 🔴🔴 | 🔴 |
| **Total Score** | | **11** | **2** | **−2** |

**Score Legend**: 🟢🟢 = +2 (Best) · 🟢 = +1 (Adequate) · 🔴 = −1 (Weakness) · 🔴🔴 = −2 (Disqualifying)
**Weight Legend**: 🔺 Critical · 🔶 High · 🔷 Medium · 🔹 Low

Total Score is an unweighted sum of the symbol values per alternative; the Weight column is retained for prioritization context when reading disagreements, matching the source ADR's own presentation. `bevy_ecs`'s Workspace Compatibility is 🔴🔴 (Disqualifying) rather than a milder Weakness because the source ADR itself frames it as an outright requirement conflict, not a mere preference — see Verdict below.

### Verdict

**Selected: hecs** — total score 11, clear of `bevy_ecs` (2) and `specs` (−2), matching the original ADR's own (differently-scaled) 23/17/16 ranking.

Decisive factors:
1. **Workspace Compatibility is disqualifying for `bevy_ecs`**: the original ADR states directly that avoiding `bevy_ecs` was an explicit user requirement — "**Conflicts with user requirement to avoid bevy**" — independent of `bevy_ecs`'s otherwise-strong ratings elsewhere.
2. **Compile times**: `hecs` rates 🟢🟢 against `bevy_ecs`'s 🔴🔴 — critical for cgtools development velocity given its minimal-dependency-tree philosophy.
3. **Dependency discipline**: `hecs`'s lightweight dependency tree (🟢🟢) aligns with cgtools' stated preference for minimal dependencies, where both alternatives score 🔴/🔴🔴.
4. **API simplicity over `specs`**: `hecs`'s clean, minimal API was judged easier to wrap in a thin abstraction layer than `specs`'s more verbose, maintenance-mode (🔴🔴) design.

**Reconsideration trigger**: the original ADR's own recorded weaknesses for `hecs` were "fewer advanced features compared to bevy_ecs" and "less extensive documentation ecosystem." If a future milestone needs a `hecs`-absent advanced feature (e.g. built-in change detection, complex system scheduling) badly enough that hand-rolling it becomes substantial work, re-evaluate against `bevy_ecs`'s richer built-in feature set — the Workspace Compatibility constraint that disqualified it here is a project policy, not a technical limitation, and could be revisited explicitly if circumstances change.

**Since this decision**: `hecs` was in fact adopted and is the real, working foundation of `ecs::World` (see `api/001`) — `World.hecs_world: hecs::World` and every `World` operation delegate to it directly. One part of this ADR's own Implementation Plan was not carried out as sketched, though: it proposed `Entity`/`World` as private-field newtypes (`pub struct Entity(hecs::Entity)`) for abstraction-layer opacity; the shipped code instead uses `hecs::Entity`/`hecs::World` directly throughout, with `hecs_world` itself a `pub` field — see `api/001`'s Compatibility Guarantees for what this means for callers.

### APIs

| File | Relationship |
|------|--------------|
| [api/001_ecs_world_runtime_api.md](../api/001_ecs_world_runtime_api.md) | The shipped API resulting from this decision, including where it diverges from the ADR's own sketched abstraction layer |

### Sources

| File | Relationship |
|------|--------------|
| `docs/ecs_decision.md` | Original ADR this instance migrates — superseded by this file; deleted as part of this migration |

### Tests

Not applicable — an architectural decision record has no executable test surface. Its outcome is verified indirectly through `api/001`'s Tests section (the doc-tests exercising the shipped `hecs`-based `World`).
