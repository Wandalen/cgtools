
//! Agnostic 2D rendering engine.
//!
//! Backend-agnostic rendering with POD commands and Y-up coordinate system.
//! Define commands once, render to any backend (SVG and WebGL today; terminal planned).
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
    feature = "adapter-webgpu",
    feature = "adapter-native",
    feature = "adapter-none",
  ) ) ]
  layer adapters;
}

// Scene-model has been extracted into its own crate: `tilemap_scene`. The
// `scene-model` feature now only gates the serde derives on the sampler
// types (`SamplerFilter`, `MipmapMode`, `WrapMode`) that `tilemap_scene`
// needs to serialize / deserialize alongside its own declaration types.
