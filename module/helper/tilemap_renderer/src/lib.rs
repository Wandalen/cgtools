//! Agnostic 2D rendering engine.
//!
//! Backend-agnostic scene definition with POD commands.
//! Define commands once, render to any backend (SVG, GPU, terminal).
//!
//! ```ignore
//! use tilemap_renderer::{ commands::*, types::*, assets::*, backend::* };
//! use tilemap_renderer::adapters::SvgBackend;
//!
//! let mut svg = SvgBackend::new( 800, 600 );
//! svg.load_assets( &assets )?;
//! svg.submit( &commands )?;
//! let Output::String( doc ) = svg.output()? else { unreachable!() };
//! ```

pub mod types;
pub mod commands;
pub mod assets;
pub mod backend;

#[ cfg( any
(
  feature = "adapter-svg",
  feature = "adapter-terminal",
  feature = "adapter-webgl",
) ) ]
pub mod adapters;
