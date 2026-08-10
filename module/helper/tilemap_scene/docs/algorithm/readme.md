# Algorithm Doc Definition

An **algorithm** instance documents one nontrivial computation this crate performs, worked through precisely enough to reimplement or audit. In `tilemap_scene`, that means the deterministic, cross-context-reusable procedures — such as animation phase/frame resolution and the per-frame scene rendering pass — whose step-by-step computation and correctness properties need to be pinned down. This collection holds one instance per algorithm; the table below is the index into them.

### Scope

- **Purpose**: Navigational hub for `tilemap_scene`'s deterministic, cross-context-reusable procedures.
- **Responsibility**: Document each algorithm's step-by-step computation and correctness properties.
- **In Scope**: Animation phase/frame resolution, the per-frame scene rendering pass.
- **Out of Scope**: The data these algorithms operate on (see `format/`), the API surface that triggers them (see `api/`).

### Overview Table

| ID | Name | Purpose | Status |
|----|------|---------|--------|
| 001 | [Animation Phase & Frame Selection](001_animation_phase_and_frame_selection.md) | `t_local` computation and per-mode frame-index selection | ✅ |
| 002 | [Scene Rendering Pass](002_scene_rendering_pass.md) | Per-bucket gather/sort/submit walk, tint composition, cache replay | ✅ |
