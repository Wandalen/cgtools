#![ allow( clippy::exhaustive_structs ) ]          // POD command types are intentionally open; #[non_exhaustive] conflicts with Copy
#![ allow( clippy::exhaustive_enums ) ]            // Small, stable enums meant to be matched exhaustively by adapter authors
#![ allow( clippy::wildcard_imports ) ]            // mod_interface! generates glob re-exports; no per-item scope is available inside a proc-macro expansion
#![ allow( clippy::min_ident_chars ) ]             // Short names like x, y, m are idiomatic in math/graphics contexts throughout this crate
#![ allow( clippy::missing_inline_in_public_items ) ] // Inline decisions belong to the optimizer for this crate size
#![ allow( clippy::trivially_copy_pass_by_ref ) ]     // &u32 / &f32 params are idiomatic in GPU/math call sites throughout
#![ allow( clippy::cast_possible_wrap ) ]             // GPU sizes / counts are bounded; u32→i32 wrapping is unreachable in practice
#![ allow( clippy::cast_possible_truncation ) ]       // GPU values fit in their target types at all realistic sizes
#![ allow( clippy::cast_precision_loss ) ]            // f32 precision loss is expected and acceptable in graphics code
#![ allow( clippy::too_many_arguments ) ]             // GPU draw / setup functions inherently take many parameters
#![ allow( clippy::too_many_lines ) ]                 // Large match blocks in adapter implementations are expected
#![ allow( clippy::std_instead_of_alloc ) ]           // wasm32+std: alloc crate is not separately linked; std::rc/std::collections are correct here

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
//! // Note: SvgBackend and TerminalBackend are stubs —
//! // implementations arrive in follow-up PRs.
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
