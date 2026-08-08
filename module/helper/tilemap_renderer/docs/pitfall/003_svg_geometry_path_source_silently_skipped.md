# Pitfall: SVG Geometry Path Source Silently Skipped

### Scope

- **Purpose**: Record that `GeometryAsset` and `ImageAsset` handle their respective `Path` source variant inconsistently in the SVG backend.
- **Responsibility**: Document the trap, the full silent-failure chain, and the asymmetry with the equivalent image-asset case.
- **In Scope**: `SvgBackend::load_geometries` and the downstream `generate_mesh_def` / `cmd_mesh` path for `GeometryAsset { positions: Source::Path(..), .. }`.
- **Out of Scope**: `ImageSource::Path` handling for images/sprites, which is the *non*-silent counterpart described below (see [feature/001_svg_backend_adapter.md](../feature/001_svg_backend_adapter.md)).

### Trap

Assuming that because `ImageSource::Path` (images referenced by sprites) produces a visible diagnostic when the SVG backend can't resolve it without file I/O, the exactly analogous `Source::Path` on `GeometryAsset.positions` behaves the same way.

### Failure

`SvgBackend::load_geometries` (`src/adapters/svg.rs`) only matches `Source::Bytes(bytes)`; a `GeometryAsset` whose `positions` is `Source::Path` falls through the `if let` with no `else` branch — no `eprintln!`, no comment, nothing — and no entry is ever stored for that geometry's `ResourceId` in `SvgResources`. Any later `Mesh` command referencing that ID reaches `cmd_mesh`, which looks up a cached `<symbol>` def, misses, and calls `generate_mesh_def`; `generate_mesh_def` does `self.resources.geometry(geom_id)?` — the `?` on the `None` returned by the missing lookup makes `generate_mesh_def` itself return `None`, and `cmd_mesh` matches that with `None => return`. The command is dropped end-to-end: no `<symbol>` definition, no `<use>` reference, no stderr warning, no diagnostic comment in the output — the mesh is simply absent from the rendered SVG with no trail explaining why.

This is the **opposite** of how the SVG backend treats the equivalent case for images: an `ImageSource::Path` sheet referenced by a `Sprite` command *is* caught — the backend detects the zero-width/zero-height placeholder it stored, emits an `eprintln!` warning to stderr, and writes a diagnostic HTML comment (`<!-- sprite_N skipped: image_N has unknown dimensions ... -->`) into the SVG in place of the sprite. `GeometryAsset`'s `Source::Path` has no equivalent placeholder-and-detect step — it is dropped one stage earlier, before any code path exists that could flag it.

### Mitigation

None currently. The only trace in source is a `// TODO: Source::Path geometries are silently skipped for now.` comment in `load_geometries` describing the intended future fix (load via `std::fs` on native or `fetch()` on `wasm32`, then re-invoke `store_geometry`) and stating that callers must resolve paths to `Source::Bytes` themselves before calling `load_assets`. As of this migration there is no warning, diagnostic, or `RenderError` surfaced for this case.

### Features

| File | Relationship |
|------|--------------|
| [feature/001_svg_backend_adapter.md](../feature/001_svg_backend_adapter.md) | `load_geometries` and `cmd_mesh` are both part of the SVG backend's asset/command handling |

### Sources

| File | Relationship |
|------|--------------|
| `src/adapters/svg.rs` | `load_geometries` (silent skip), `generate_mesh_def` (propagates the miss via `?`), `cmd_mesh` (silently returns on `None`) |

### Tests

| File | Relationship |
|------|--------------|
| — | No test in `src/adapters/svg.rs` or `tests/` currently exercises `Source::Path` geometry handling; the analogous `ImageSource::Path` *image* case is covered by `sprite_on_path_sheet_is_skipped_with_comment`, but that test does not extend to geometries |
