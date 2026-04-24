//! Compile a parsed scene-model spec + scene into a backend `RenderCommand` stream.
//!
//! This is Phase-2 glue between the declarative [`crate::RenderSpec`] /
//! [`crate::Scene`] format and the existing backend-agnostic command
//! pipeline (`tilemap_renderer::commands`, `tilemap_renderer::backend::Backend`).
//!
//! The compile layer owns two boundaries:
//!
//! - **Coordinate system flip** — `tiles_tools` pixel conversions are Y-down
//!   (see `tiles_tools/src/coordinates/pixel.rs:8`); this crate's render
//!   backends are Y-up (see `tilemap_renderer/src/lib.rs:22`). The flip
//!   happens exactly once, in [`coords::hex_to_world_pixel_flat`] /
//!   [`coords::hex_to_world_pixel_pointy`].
//! - **Asset resolution** — the spec declares asset paths; who loads the bytes
//!   (disk, in-memory, browser fetch) is a caller concern. The
//!   [`AssetResolver`] trait exposes that choice.

mod private {}

mod_interface::mod_interface!
{
  layer animation;
  layer assets;
  layer camera;
  layer conditions;
  layer coords;
  layer edges;
  layer error;
  layer frame;
  layer ids;
  layer neighbors;
  layer resolver;
  layer vertex;
  layer viewport;
}
