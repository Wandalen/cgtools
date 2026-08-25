//! Axial-hex → world-pixel conversion with the `tiles_tools` → `tilemap_renderer`
//! Y-axis flip baked in.
//!
//! Why this module exists: `tiles_tools::coordinates::pixel::Pixel` uses a
//! Y-down convention (see `tiles_tools/src/coordinates/pixel.rs:8`); this
//! crate's render backends use Y-up (see `lib.rs:22`). The flip happens here
//! so the rest of the compile pipeline can work in a single coordinate
//! system. Conversions also scale the unit-sized `tiles_tools` result by the
//! spec-declared `grid_stride`.
//!
//! Note: `grid_stride` is the **inter-centre spacing**, not a cell bounding
//! box. For equilateral sprites the two coincide; for stylised art they
//! diverge, and the compile layer always treats the value as spacing.
//!
//! The output is a **world**-space pixel coordinate — camera transform still
//! has to be applied before handing to a backend [`tilemap_renderer::commands::Sprite`].

mod private
{
  use tiles_tools::coordinates::hexagonal::{ Axial, Coordinate, Flat, Pointy };
  use tiles_tools::coordinates::pixel::Pixel;

  /// Axial `( q, r )` on a flat-top hex grid → world-pixel centre with Y-up.
  ///
  /// `grid_stride` is the pixel spacing between centres of adjacent cells
  /// along the primary axes (see [`crate::HexConfig::grid_stride`]).
  #[ inline ]
  #[ must_use ]
  pub fn hex_to_world_pixel_flat( q : i32, r : i32, grid_stride : ( u32, u32 ) ) -> ( f32, f32 )
  {
    // tiles_tools returns unit-scale, Y-down coordinates. Its flat-top formula:
    //   x = 1.5 * q
    //   y = sqrt(3) / 2 * q + sqrt(3) * r   (Y-down)
    //
    // We first ask tiles_tools for the unit-scale position, then scale by
    // grid_stride and negate Y to produce Y-up world pixels.
    let pixel = Pixel::from( Coordinate::< Axial, Flat >::new( q, r ) );
    let cw = grid_stride.0 as f32;
    let ch = grid_stride.1 as f32;
    // tiles_tools' formula assumes size factors of (3/2) and sqrt(3); we want
    // the unit-size output to span `grid_stride` exactly. Since tiles_tools scales
    // by its own trigonometric constants (not by grid_stride), we compensate by
    // scaling the X/Y independently: X by (cw / 1.5) and Y by (ch / sqrt(3)).
    let sx = cw / 1.5;
    let sy = ch / 3.0_f32.sqrt();
    ( pixel.x() * sx, -pixel.y() * sy )
  }

  /// Axial `( q, r )` on a pointy-top hex grid → world-pixel centre with Y-up.
  ///
  /// `grid_stride` is the pixel spacing between centres of adjacent cells
  /// along the primary axes (see [`crate::HexConfig::grid_stride`]).
  #[ inline ]
  #[ must_use ]
  pub fn hex_to_world_pixel_pointy( q : i32, r : i32, grid_stride : ( u32, u32 ) ) -> ( f32, f32 )
  {
    // Pointy-top formula in tiles_tools:
    //   x = sqrt(3) * q + sqrt(3)/2 * r
    //   y = 1.5 * r   (Y-down)
    let pixel = Pixel::from( Coordinate::< Axial, Pointy >::new( q, r ) );
    let cw = grid_stride.0 as f32;
    let ch = grid_stride.1 as f32;
    let sx = cw / 3.0_f32.sqrt();
    let sy = ch / 1.5;
    ( pixel.x() * sx, -pixel.y() * sy )
  }
}

mod_interface::mod_interface!
{
  exposed use hex_to_world_pixel_flat;
  exposed use hex_to_world_pixel_pointy;
}
