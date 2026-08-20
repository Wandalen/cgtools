# gpu_picking

GPU id-buffer object picking for `minwebgl`-based WebGL2 apps.

Renders every pickable part's small integer id into an off-screen `R32I` texture
via [`IdProgram`]/[`PickBuffer`], then reads a single pixel back at a click
location to find out what's there — no CPU-side ray/AABB intersection math
needed, since the GPU already rasterized exactly what's visible at that pixel.

Callers implement the [`Pickable`] trait for whatever their own "one drawable
part" type already is (own VAO, index count, world transform, pick id); this
crate never needs to know anything else about it.

## Responsibility Table

| File | Responsibility |
|------|-----------------|
| `src/lib.rs` | `Pickable` trait, `IdProgram` (id-pass shader), `PickBuffer` (off-screen id texture + `read_pixels`) |
| `src/shaders/id.vert`, `src/shaders/id.frag` | Minimal id-pass shader pair |

## Usage

```rust,ignore
struct MyPart { vao: WebGlVertexArrayObject, index_count: i32, model: F32x4x4, id: i32 }

impl gpu_picking::Pickable for MyPart
{
  fn vao( &self ) -> &WebGlVertexArrayObject { &self.vao }
  fn index_count( &self ) -> i32 { self.index_count }
  fn model( &self ) -> F32x4x4 { self.model }
  fn pick_id( &self ) -> i32 { self.id }
}

let id_program = gpu_picking::IdProgram::new( &gl );
let mut pick_buffer = gpu_picking::PickBuffer::new( &gl, width, height );

// Before reading a pick (e.g. on pointerdown), or every frame if parts move:
pick_buffer.render( &gl, &id_program, view_proj, parts.iter(), None );
let picked_id = pick_buffer.pick( &gl, x, y );
```
