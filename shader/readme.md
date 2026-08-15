# shader

Repo-root collection of reusable WGSL **shader chunks** — small, composable
pieces of shader source, one per directory, bundled at compile time and
composed by [`shader_chunks_core`](../module/shader/shader_chunks_core/readme.md)
and inspected/composed from the terminal by the
[`shader_chunks`](../module/shader/shader_chunks/readme.md) (`sch`) CLI.

Each chunk lives in its own directory: `shader/<name>/<name>.wgsl` (the
chunk's WGSL source, opening with a `//@`-prefixed manifest header) plus a
`readme.md` (visualization, parameters, and links to related chunks) and a
`preview.png` (a generated visualization of what the chunk actually
produces). See any chunk's own `readme.md` for the manifest-field
conventions (`name`/`description`/`tags`/`stage`/`depends_on`/`export`).

| Chunk | Responsibility | Depends On |
|-------|-----------------|------------|
| [hash21/](hash21/readme.md) | Hash a 2D point to a single pseudo-random value | — |
| [value_noise/](value_noise/readme.md) | Bilinear-interpolated smooth noise over `hash21` | `hash21` |
| [fbm3/](fbm3/readme.md) | 3-octave fractal Brownian motion over `value_noise` | `value_noise` |
| [fullscreen_triangle/](fullscreen_triangle/readme.md) | Big-triangle vertex stage covering the viewport | — |
| [hash22/](hash22/readme.md) | Hash a 2D point to two pseudo-random channels | — |
| [hash13/](hash13/readme.md) | Hash a 3D point to a single pseudo-random value | — |
| [hash33/](hash33/readme.md) | Hash a 3D point to three pseudo-random channels | — |
| [value_noise3/](value_noise3/readme.md) | Trilinear-interpolated smooth noise over `hash13` | `hash13` |
| [gradient_noise/](gradient_noise/readme.md) | Quintic-faded gradient (Perlin) noise over `hash22` | `hash22` |
| [voronoi/](voronoi/readme.md) | Cellular F1 distance and cell id over `hash22` | `hash22` |
| [domain_warp/](domain_warp/readme.md) | Warp a 2D point by centered `fbm3` offsets | `fbm3` |
| [d2_sdf_circle/](d2_sdf_circle/readme.md) | Signed distance to a circle | — |
| [d2_sdf_ring/](d2_sdf_ring/readme.md) | Unsigned distance to a circle line | — |
| [d2_sdf_box/](d2_sdf_box/readme.md) | Signed distance to an axis-aligned box | — |
| [d2_sdf_round_box/](d2_sdf_round_box/readme.md) | Signed distance to a rounded-corner box | `d2_sdf_box` |
| [d2_sdf_segment/](d2_sdf_segment/readme.md) | Unsigned distance to a line segment | — |
| [d2_sdf_equilateral_triangle/](d2_sdf_equilateral_triangle/readme.md) | Signed distance to an equilateral triangle | — |
| [d2_sdf_hexagon/](d2_sdf_hexagon/readme.md) | Signed distance to a regular hexagon | — |
| [d2_sdf_arc/](d2_sdf_arc/readme.md) | Unsigned distance to a ring arc | — |
| [d2_sdf_pie/](d2_sdf_pie/readme.md) | Signed distance to a pie/wedge slice | — |
| [d2_sdf_vesica/](d2_sdf_vesica/readme.md) | Signed distance to a lens/vesica shape | — |
| [d2_sdf_star5/](d2_sdf_star5/readme.md) | Signed distance to a 5-pointed star | — |
| [d2_sdf_cross/](d2_sdf_cross/readme.md) | Signed distance to a plus/cross shape | — |
| [d3_sdf_sphere/](d3_sdf_sphere/readme.md) | Signed distance to a sphere | — |
| [d3_sdf_box/](d3_sdf_box/readme.md) | Signed distance to an axis-aligned box | — |
| [d3_sdf_round_box/](d3_sdf_round_box/readme.md) | Signed distance to a rounded-edge box | `d3_sdf_box` |
| [d3_sdf_torus/](d3_sdf_torus/readme.md) | Signed distance to a torus | — |
| [d3_sdf_capsule/](d3_sdf_capsule/readme.md) | Signed distance to a capsule (swept sphere) | — |
| [d3_sdf_capped_cylinder/](d3_sdf_capped_cylinder/readme.md) | Signed distance to a flat-capped cylinder | — |
| [d3_sdf_capped_cone/](d3_sdf_capped_cone/readme.md) | Signed distance to a capped (frustum) cone | — |
| [d3_sdf_plane/](d3_sdf_plane/readme.md) | Signed distance to an infinite plane | — |
| [d3_sdf_octahedron/](d3_sdf_octahedron/readme.md) | Signed distance to an octahedron | — |
| [d3_sdf_ellipsoid/](d3_sdf_ellipsoid/readme.md) | Signed distance bound to an ellipsoid | — |
| [d3_sdf_hex_prism/](d3_sdf_hex_prism/readme.md) | Signed distance to a hexagonal prism | — |
| [d3_sdf_round_cone/](d3_sdf_round_cone/readme.md) | Signed distance to a round cone (swept sphere) | — |
| [sdf_op_union/](sdf_op_union/readme.md) | Sharp union of two distances | — |
| [sdf_op_subtract/](sdf_op_subtract/readme.md) | Sharp subtraction of two distances | — |
| [sdf_op_intersect/](sdf_op_intersect/readme.md) | Sharp intersection of two distances | — |
| [sdf_op_union_smooth/](sdf_op_union_smooth/readme.md) | Smoothly blended union | — |
| [sdf_op_subtract_smooth/](sdf_op_subtract_smooth/readme.md) | Smoothly blended subtraction | — |
| [sdf_op_intersect_smooth/](sdf_op_intersect_smooth/readme.md) | Smoothly blended intersection | — |
| [sdf_op_round/](sdf_op_round/readme.md) | Rounds a distance field's corners | — |
| [sdf_op_onion/](sdf_op_onion/readme.md) | Hollows a distance field into a shell | — |
| [glow/](glow/readme.md) | Radial falloff from distance to intensity | — |
| [aa_step/](aa_step/readme.md) | Antialiased threshold via screen-space derivative | — |
| [rot2/](rot2/readme.md) | 2D rotation matrix from an angle | — |
| [palette_cosine/](palette_cosine/readme.md) | Cosine 4-parameter color gradient | — |
| [srgb/](srgb/readme.md) | Linear/sRGB color conversions, both directions | — |
| [tonemap_aces/](tonemap_aces/readme.md) | ACES filmic HDR-to-LDR tone map | — |
| [gaussian_weight/](gaussian_weight/readme.md) | Unnormalized 1D Gaussian blur weight | — |

Row order above is load-bearing: `shader_chunks_core`'s `build.rs` takes
`CHUNKS` registry order from this table — and fails the build if a chunk
directory and a row here ever disagree — so reordering rows reorders every
registry consumer's default output (e.g. the `sch` CLI's `sort::input`).

Dependency order is derived live by `sch tree` from each chunk's
`//@ depends_on:` header — not duplicated here.
