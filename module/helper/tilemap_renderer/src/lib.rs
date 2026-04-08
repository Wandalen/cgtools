#![ allow( clippy::exhaustive_structs ) ]
#![ allow( clippy::exhaustive_enums ) ]
#![ allow( clippy::wildcard_imports ) ]
#![ allow( clippy::min_ident_chars ) ]

//! Agnostic 2D rendering engine.
//!
//! Backend-agnostic rendering with POD commands and Y-up coordinate system.
//! Define commands once, render to any backend (SVG, WebGL, terminal).
//!
//! ## Coordinate system
//!
//! All backends use a **Y-up** convention:
//! - `(0, 0)` is the **bottom-left** corner
//! - Positive Y points **up**
//! - Positive rotation is **counter-clockwise**
//!
//! ## Usage
//!
//! ```ignore
//! use tilemap_renderer::{ commands::*, types::*, assets::*, backend::* };
//! use tilemap_renderer::adapters::SvgBackend;
//!
//! let config = RenderConfig { width : 800, height : 600, ..Default::default() };
//! let mut svg = SvgBackend::new( config );
//! svg.load_assets( &assets )?;
//! svg.submit( &commands )?;
//! let Output::String( doc ) = svg.output()? else { unreachable!() };
//! ```

mod private {}

#[ cfg( feature = "enabled" ) ]
mod_interface::mod_interface!
{
  layer types;
  layer commands;
  layer assets;
  layer backend;

  #[ cfg( any
  (
    feature = "adapter-svg",
    feature = "adapter-terminal",
    feature = "adapter-webgl",
  ) ) ]
  layer adapters;
}
