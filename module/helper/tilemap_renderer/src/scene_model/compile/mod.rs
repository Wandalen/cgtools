//! Compile a parsed scene-model spec + scene into a backend `RenderCommand` stream.
//!
//! This is Phase-2 glue between the declarative [`crate::scene_model::RenderSpec`] /
//! [`crate::scene_model::Scene`] format and the existing backend-agnostic command
//! pipeline (`tilemap_renderer::commands`, `tilemap_renderer::backend::Backend`).
//!
//! The compile layer owns two boundaries:
//!
//! - **Coordinate system flip** — `tiles_tools` pixel conversions are Y-down
//!   (see `tiles_tools/src/coordinates/pixel.rs:8`); this crate's render
//!   backends are Y-up (see `lib.rs:22`). The flip happens exactly once, in
//!   [`coords::hex_to_world_pixel_flat`] / [`coords::hex_to_world_pixel_pointy`].
//! - **Asset resolution** — the spec declares asset paths; who loads the bytes
//!   (disk, in-memory, browser fetch) is a caller concern. The
//!   [`AssetResolver`] trait exposes that choice.
//!
//! # Slice 1 — what's supported
//!
//! This first slice ships the minimum viable path end-to-end:
//!
//! - [`crate::scene_model::Anchor::Hex`] objects only.
//! - [`crate::scene_model::SpriteSource::Static`] sources in the `"default"` animation only.
//! - [`crate::scene_model::AssetKind::Atlas`] assets only.
//! - Camera as translate + uniform zoom (no rotation, no parallax).
//!
//! Unsupported features trigger [`CompileError::UnsupportedAnchor`] /
//! `UnsupportedSource` / `UnsupportedAssetKind` at compile time rather than
//! silently dropping draw calls. Follow-up slices add variants / animations /
//! autotile / dual-mesh / edge-anchored objects / viewport parallax / runtime
//! mutation API.

pub mod animation;
pub mod assets;
pub mod camera;
pub mod conditions;
pub mod coords;
pub mod error;
pub mod frame;
pub mod ids;
pub mod neighbors;
pub mod resolver;
pub mod vertex;

pub use animation::resolve_animation_frame;
pub use assets::{ CompiledAssets, compile_assets };
pub use camera::Camera;
pub use coords::{ hex_to_world_pixel_flat, hex_to_world_pixel_pointy };
pub use error::CompileError;
pub use frame::compile_frame;
pub use ids::IdMap;
pub use resolver::{ AssetResolver, PathResolver };
