# Feature: SVG Backend Adapter

`adapters::SvgBackend` implements the core `Backend` trait to generate SVG 1.1 documents from a rendering command stream, behind the `adapter-svg` feature.

### Scope

- **Purpose**: Let a command stream produce a static, standards-compliant SVG 1.1 document.
- **Responsibility**: Cross-reference the SVG adapter's source, the invariants it must uphold, and its known pitfalls and gaps.
- **In Scope**: SVG document generation for all command families (paths, text, sprites, meshes, batches, groups/effects) and asset handling (images, geometries, gradients, patterns, clip masks).
- **Out of Scope**: WebGL2, Terminal, None, WebGPU, and Native adapters (see [002](002_webgl2_backend_adapter.md), [003](003_terminal_backend_adapter.md), [004](004_none_backend_adapter.md), [005](005_webgpu_backend_adapter.md), [006](006_native_backend_adapter.md)); the Y-up→Y-down conversion mechanics themselves (see [invariant/001](../invariant/001_y_up_coordinate_system.md)); the injection-safety guarantee mechanics themselves (see [invariant/002](../invariant/002_svg_injection_safe_output.md)).

### Design

Documents are built incrementally via an internal content manager that tracks byte offsets for the `<defs>` section, the body element list, and the top-level viewport `<g transform>` wrapper, so inserts and viewport updates are `O(1)` splice operations on one contiguous string rather than a full re-render — `viewport_offset_set`/`viewport_scale_set` rewrite only the wrapper's `transform` attribute via `replace_range`, without re-submitting any commands.

Coordinate conversion (Y-up → SVG's native Y-down) is applied per positioned element; see [invariant/001](../invariant/001_y_up_coordinate_system.md) for the mechanism. Batch instances are drawn with raw, unflipped local transforms inside an already-flipped `<g>` parent, avoiding a double conversion.

Effects (blur, drop shadow, color matrix, opacity) are emitted as SVG `<filter>` elements; sprite tint reuses the same mechanism via `feColorMatrix`. Mesh geometry is emitted as a `<symbol>` definition, generated lazily the first time a given `(geometry, topology)` pair is actually drawn, so only topologies actually used appear in `<defs>`; a `TriangleStrip` mesh alternates vertex order on its odd-indexed triangles (the standard OpenGL/Direct3D strip convention) so the emitted SVG polygon sequence keeps consistent counter-clockwise winding. Mesh texturing is approximated via an SVG `<pattern>` fill, since SVG has no native per-triangle UV-mapped texturing.

Bitmap images are re-encoded to PNG (via the `image` crate) and inlined as `data:image/png;base64` data URIs. Images supplied as already-encoded bytes (`ImageSource::Encoded`) have their MIME type auto-detected from magic bytes (PNG, JPEG, GIF, WebP, SVG, falling back to PNG). Dimension extraction for both paths goes through one general helper, `image_dimensions`, which uses the `image` crate's format-guessing reader (`ImageReader::with_guessed_format`) and works for any format the crate recognizes (PNG, JPEG, GIF, WebP, BMP, TIFF, etc.) — not a hand-rolled PNG-only IHDR-chunk parser. A hand-rolled IHDR reader (`png_dimensions`) does still exist in source, but it is `#[cfg(test)]`-only, kept as a sanity check on the `image` crate's own behavior for PNG inputs, not the production code path. (Earlier design notes described dimension extraction as reading the IHDR chunk directly; that description no longer matches the current source and is corrected here.) Colors are emitted in SVG 1.1's `rgb()` form with a separate `-opacity` attribute per element kind, rather than CSS Color Level 4 `rgba()`, since some SVG 1.1 parsers (e.g. Inkscape) reject the newer syntax; a fully opaque color omits the opacity attribute entirely. Arc commands store rotation in radians internally but emit degrees, matching the SVG 1.1 elliptical-arc path-command spec.

Injection-safety for caller-controlled text and path strings is a dedicated cross-cutting guarantee — see [invariant/002](../invariant/002_svg_injection_safe_output.md) for exactly what is and isn't covered.

**Known gap — font selection**: `Assets.fonts` is currently ignored by `assets_load`; no `@font-face`/`<font-face>` definitions are emitted and `<text>` elements carry no `font-family`, so all text renders in the SVG viewer's default font. Text *rendering* (positioning, anchoring, styling other than font) works; font *selection* does not. Confirmed directly in the `capabilities()` doc comment in `src/adapters/svg.rs` and listed under svg adapter gaps in `roadmap.md`.

**Known gap — image `Path` sources**: `ImageSource::Path` performs no file I/O at `assets_load` time — callers must supply `ImageSource::Bitmap`/`ImageSource::Encoded` when sprite regions are needed; sprites referencing a `Path` sheet are skipped with a stderr warning and an HTML diagnostic comment. `GeometryAsset`'s `Source::Path`, by contrast, *is* loaded, via a blocking `std::fs` read (implemented by task 064): on success the geometry behaves exactly like `Source::Bytes`; on read failure — missing file, or wasm32 where no filesystem exists — the geometry is skipped with the same stderr-warning + diagnostic-comment idiom, and a failed *index* source skips the whole geometry rather than silently falling back to unindexed topology. The former silent-drop trap (pitfall/003) is retired.

Given the font gap, this adapter's status is tracked as partial (⚠️) rather than fully complete, even though every other command and asset family is implemented.

### Invariants

| File | Relationship |
|------|--------------|
| [invariant/001_y_up_coordinate_system.md](../invariant/001_y_up_coordinate_system.md) | This adapter performs the only active Y-up → Y-down conversion among shipped backends |
| [invariant/002_svg_injection_safe_output.md](../invariant/002_svg_injection_safe_output.md) | This adapter is the only backend that emits caller-controlled strings into a markup document |
| [invariant/003_z_layer_draw_ordering.md](../invariant/003_z_layer_draw_ordering.md) | No depth buffer exists here; submission order is this backend's entire ordering contract |
| [invariant/004_vector_representability_of_commands.md](../invariant/004_vector_representability_of_commands.md) | This adapter's completeness across the full command set is the invariant's living proof |

### Patterns

| File | Relationship |
|------|--------------|
| [pattern/001_ports_and_adapters_backend_architecture.md](../pattern/001_ports_and_adapters_backend_architecture.md) | This adapter is one `Backend` implementation within the crate's hexagonal architecture |

### Sources

| File | Relationship |
|------|--------------|
| `src/adapters/svg.rs` | Full `SvgBackend` implementation — content manager, transform conversion, effects, asset loaders, escaping |

### Tests

| File | Relationship |
|------|--------------|
| `tests/svg_backend_test.rs` | Relocated from inline `src/adapters/svg.rs` by task 071 — no `#[cfg(test)]` block remains in the source file. Behavior tests drive `SvgBackend` through its public surface (`SvgBackend::new` / `viewport_scale_set` / `viewport_offset_set` plus the `Backend` trait), asserting on the rendered SVG string from `output()`; formatting/encoding helper tests cover the `#[doc(hidden)]`-exported helpers. Covers transform conversion, escaping, MIME detection, asset-skip diagnostics, geometry `Source::Path` loading (from-disk happy path, missing-file skip, missing-index whole-skip), blend modes, effects, and more |
| `tests/backend_test.rs` | `Backend` trait contract exercised generically (not SVG-specific) |
