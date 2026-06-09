# 📏 line_tools

**High-performance line rendering for WebGL 2.0**

`line_tools` renders thick, anti-aliased polylines in WebGL 2.0 using instanced
drawing. It provides two independent line implementations:

- **2D lines** (`d2::Line`) with configurable **caps** (Butt, Round, Square) and
  **joins** (Miter, Round, Bevel).
- **3D lines** (`d3::Line`) drawn as camera-facing quads, with optional
  per-vertex colors, world- or screen-space width, alpha-to-coverage
  anti-aliasing, and dashing.

Both build on `minwebgl` for the WebGL context and `ndarray_cg` for vector math.

## ✨ Features

- **2D caps & joins** — `Cap::{ Butt, Round, Square }` and
  `Join::{ Miter, Round, Bevel }`, with adjustable triangulation precision.
- **3D screen- or world-space width** — constant pixel width on screen, or a
  width measured in world units that shrinks with distance from the camera
  (`world_units_use`).
- **Per-vertex colors** — assign a color per point and interpolate along the
  line (`vertex_color_use`).
- **Dashing** *(requires the `distance` feature)* — selectable `DashPattern`
  variants (`V1`–`V4`) with an adjustable `dash_offset`.
- **Anti-aliasing** — alpha testing by default, or MSAA `alpha-to-coverage`
  (`alpha_to_coverage_use`).
- **Custom fragment shaders** — pass your own fragment shader to `mesh_create`.
- **Incremental editing** — add, remove, and overwrite individual points;
  meshes only re-upload the buffers that changed.
- **Serialization** *(requires the `serialization` feature)* — `Cap` and `Join`
  implement `serde::{ Serialize, Deserialize }`.

## 📦 Feature Flags

| Feature | Default | Description |
|---------|---------|-------------|
| `enabled` | ✅ | Core line rendering. Enables the `minwebgl` / `ndarray_cg` / `mod_interface` / `web-sys` dependencies. |
| `distance` | | Tracks cumulative arc length per point; required for dashing and distance-based effects. |
| `serialization` | | `serde` support for `Cap` and `Join`. |
| `full` | | Convenience: `enabled` + `distance` + `serialization`. |

## 🚀 Installation

```toml
[dependencies]
line_tools = { workspace = true }
# or, to enable everything:
line_tools = { workspace = true, features = [ "full" ] }
```

## 💡 Quick Start — 3D Line

A 3D line is built from a sequence of points, turned into a mesh, supplied with
its uniforms, and drawn each frame.

```rust,ignore
use line_tools::d3;
use minwebgl as gl;

fn setup( gl : &gl::WebGl2RenderingContext ) -> Result< d3::Line, gl::WebglError >
{
  let mut line = d3::Line::default();

  // Configure render state *before* creating the mesh.
  line.vertex_color_use( true );
  line.world_units_use( false );        // constant screen-space width
  line.alpha_to_coverage_use( true );

  // Add the points (and a color per point, since vertex colors are enabled).
  line.point_add_back( &[ 0.0, 0.0, 0.0 ] );
  line.color_add_back( gl::F32x3::new( 1.0, 0.0, 0.0 ) );
  line.point_add_back( &[ 1.0, 1.0, 0.0 ] );
  line.color_add_back( gl::F32x3::new( 0.0, 0.0, 1.0 ) );

  // Compile shaders and build the GPU buffers. Pass `None` for the default
  // fragment shader, or `Some( source )` for a custom one.
  line.mesh_create( gl, None )?;

  // Upload the uniforms the built-in shaders expect.
  let mesh = line.mesh_get_mut()?;
  mesh.upload( gl, "u_width", &4.0_f32 )?;
  mesh.upload( gl, "u_resolution", &gl::F32x2::from( [ 800.0, 600.0 ] ) )?;
  mesh.upload( gl, "u_projection_matrix", &projection_matrix )?;
  mesh.upload( gl, "u_world_matrix", &world_matrix )?;
  mesh.upload( gl, "u_view_matrix", &view_matrix )?;

  Ok( line )
}

// Each frame: append/trim points as needed, then draw. `draw` re-uploads any
// changed buffers automatically.
fn render( gl : &gl::WebGl2RenderingContext, line : &mut d3::Line ) -> Result< (), gl::WebglError >
{
  line.draw( gl )
}
```

### Dashed 3D Lines

Dashing requires the `distance` feature, which tracks each point's cumulative
arc length.

```rust,ignore
use line_tools::d3::{ Line, DashPattern };

let mut line = Line::default();
line.dash_use( true );
line.dash_pattern_set( DashPattern::V2( [ 0.3, 0.1 ] ) ); // dash, gap
line.dash_offset_set( 0.0 );
line.mesh_create( gl, None )?;

// The dash pattern and offset are uploaded as the `u_dash_pattern` and
// `u_dash_offset` uniforms; `draw` keeps them in sync each frame.
```

`DashPattern` variants carry an increasing number of segment lengths:
`V1( f32 )`, `V2( [ f32; 2 ] )`, `V3( [ f32; 3 ] )`, `V4( [ f32; 4 ] )`.

## 💡 Quick Start — 2D Line

```rust,ignore
use line_tools::{ d2, Cap, Join };
use minwebgl as gl;

let mut line = d2::Line::default();
line.cap_set( Cap::Round( 16 ) );
line.join_set( Join::Miter( 7, 7 ) );

line.point_add( [ 0.0,   0.0 ] );
line.point_add( [ 100.0, 50.0 ] );
line.point_add( [ 200.0, 0.0 ] );

// The 2D line requires an explicit fragment shader source.
line.mesh_create( gl, fragment_shader_source )?;

// Per-program uniforms (body, body_terminal, join, cap) plus shared matrices.
let mesh = line.mesh_get_mut();
mesh.upload( gl, "u_projection_matrix", &projection_matrix )?;
mesh.upload( gl, "u_world_matrix", &world_matrix )?;
mesh.upload( gl, "u_view_matrix", &view_matrix )?;
mesh.upload( gl, "u_width", &50.0_f32 )?;
mesh.upload_to( gl, "body", "u_color", &[ 1.0, 1.0, 1.0 ] )?;

// Each frame:
line.draw( gl )?;
```

## 📚 API Overview

### Types

| Type | Description |
|------|-------------|
| `d3::Line` | 3D camera-facing line. Methods use noun-verb order (`mesh_create`, `point_add_back`). |
| `d2::Line` | 2D line with caps and joins. Methods use noun-verb order (`mesh_create`, `point_add`). |
| `Cap` | `Butt` (default), `Round( segments )`, `Square`. |
| `Join` | `Miter( h, v )`, `Round( h, v )`, `Bevel( h, v )` — `h`/`v` are triangulation precision. |
| `d3::DashPattern` | `V1`–`V4` repeating dash/gap patterns. Defaults to `V1( 0.5 )`. |
| `Mesh` | Owns the WebGL buffers, VAOs, and shader programs; exposes `upload` / `upload_to`. |

### Common `d3::Line` methods

| Method | Purpose |
|--------|---------|
| `point_add_back` / `point_add_front` | Append/prepend a single point. |
| `points_add_back` / `points_add_front` | Append/prepend a slice of points. |
| `color_add_back` / `color_add_front` | Add a per-point color. |
| `point_set` / `color_set` | Overwrite an existing point/color by index. |
| `point_remove` / `point_remove_back` / `point_remove_front` | Remove points. |
| `point_get` / `points_get` / `colors_get` | Read back geometry. |
| `num_points` | Number of points in the line. |
| `vertex_color_use` / `world_units_use` / `alpha_to_coverage_use` / `dash_use` | Toggle render-state flags (call before `mesh_create`). |
| `mesh_create` / `mesh_update` / `mesh_get` / `mesh_get_mut` | Manage the GPU mesh. |
| `draw` | Sync changed buffers and issue the draw call. |
| `clear` | Remove all points without freeing memory. |

With the `distance` feature, `d3::Line` and `d2::Line` additionally expose
`total_distance_get` / `total_distance_get`, `distances_get`,
`distances_update`, and the dash controls (`dash_pattern_set`,
`dash_offset_set` / `dash_offset_get`).

### Uniforms expected by the built-in shaders

| Uniform | Used by | Meaning |
|---------|---------|---------|
| `u_width` | both | Line width (pixels in screen space, world units with `world_units_use`). |
| `u_color` | both | Base line color (when vertex colors are disabled). |
| `u_resolution` | 3D | Viewport size in pixels, for screen-space width. |
| `u_projection_matrix` / `u_view_matrix` / `u_world_matrix` | both | Standard transform matrices. |
| `u_dash_pattern` / `u_dash_offset` | 3D (dash) | Active dash pattern and its starting offset. |

## 🧩 Integration

`line_tools` is part of the **cgtools** ecosystem and composes with:

- **minwebgl** — WebGL 2.0 context, shader compilation, buffer helpers.
- **ndarray_cg** — vector and matrix math (`F32x2`, `F32x3`, matrices).

See the runnable examples in the repository for complete applications:

- `examples/minwebgl/2d_line` — interactive caps and joins.
- `examples/minwebgl/3d_line` — animated N-body trails with dashing and
  world-space width.

## 📖 Documentation

```bash
cargo doc --open
```
