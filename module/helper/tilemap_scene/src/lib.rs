#![ allow( clippy::exhaustive_structs ) ]
#![ allow( clippy::exhaustive_enums ) ]
#![ allow( clippy::wildcard_imports ) ]
#![ allow( clippy::min_ident_chars ) ]
#![ allow( clippy::missing_inline_in_public_items ) ]
#![ allow( clippy::cast_possible_truncation ) ]
#![ allow( clippy::cast_precision_loss ) ]
#![ allow( clippy::cast_possible_wrap ) ]
#![ allow( clippy::cast_sign_loss ) ]
#![ allow( clippy::too_many_lines ) ]
#![ allow( clippy::too_many_arguments ) ]
#![ allow( clippy::module_name_repetitions ) ]

//! Compositional declarative scene format for 2D tile-based games.
//!
//! Implements the format described in `spec.md` (v0.2.0). Provides
//! serde-compatible data types for describing a render spec and a scene,
//! plus a compile layer that turns them into a stream of
//! [`tilemap_renderer::commands::RenderCommand`]s consumable by existing
//! backends.
//!
//! Three primitives form the entire vocabulary:
//!
//! - [`Object`] — a renderable class with an [`Anchor`] and named state stacks.
//! - [`ObjectLayer`] — one textured strip, combining a [`SpriteSource`]
//!   (what to draw) with a [`LayerBehaviour`] (tint, blend, effects).
//! - [`Anchor`] — hex cell / edge / triangle vertex / multihex / world pixel
//!   / viewport — decides what "position" means and what neighbour context
//!   is visible to the layer.
//!
//! See `spec.md` for the normative specification.

mod private {}

mod_interface::mod_interface!
{
  layer anchor;
  layer compile;
  layer coords;
  layer error;
  layer hash;
  layer layer;
  layer load;
  layer object;
  layer pipeline;
  layer resource;
  layer scene;
  layer source;
  layer spec;
  layer validate;
}
