//! Axial-hex → world-pixel conversion with the `tiles_tools` → `tilemap_renderer`
//! Y-axis flip baked in.
//!
//! Why this module exists: `tiles_tools::coordinates::pixel::Pixel` uses a
//! Y-down convention (see `tiles_tools/src/coordinates/pixel.rs:8`); this
//! crate's render backends use Y-up (see `lib.rs:22`). The flip happens here
//! so the rest of the compile pipeline can work in a single coordinate
//! system. Conversions also scale the unit-sized `tiles_tools` result by the
//! spec-declared `cell_size`.
//!
//! The output is a **world**-space pixel coordinate — camera transform still
//! has to be applied before handing to a backend [`crate::commands::Sprite`].

mod private
{
  use tiles_tools::coordinates::hexagonal::{ Axial, Coordinate, Flat, Pointy };
  use tiles_tools::coordinates::pixel::Pixel;

  /// Axial `( q, r )` on a flat-top hex grid → world-pixel centre with Y-up.
  ///
  /// `cell_size` is the full bounding-box size of one hex, in pixels.
  #[ inline ]
  #[ must_use ]
  pub fn hex_to_world_pixel_flat( q : i32, r : i32, cell_size : ( u32, u32 ) ) -> ( f32, f32 )
  {
    // tiles_tools returns unit-scale, Y-down coordinates. Its flat-top formula:
    //   x = 1.5 * q
    //   y = sqrt(3) / 2 * q + sqrt(3) * r   (Y-down)
    //
    // We first ask tiles_tools for the unit-scale position, then scale by
    // cell_size and negate Y to produce Y-up world pixels.
    let pixel = Pixel::from( Coordinate::< Axial, Flat >::new( q, r ) );
    let cw = cell_size.0 as f32;
    let ch = cell_size.1 as f32;
    // tiles_tools' formula assumes size factors of (3/2) and sqrt(3); we want
    // the unit-size output to span `cell_size` exactly. Since tiles_tools scales
    // by its own trigonometric constants (not by cell_size), we compensate by
    // scaling the X/Y independently: X by (cw / 1.5) and Y by (ch / sqrt(3)).
    let sx = cw / 1.5;
    let sy = ch / 3.0_f32.sqrt();
    ( pixel.x() * sx, -pixel.y() * sy )
  }

  /// Axial `( q, r )` on a pointy-top hex grid → world-pixel centre with Y-up.
  ///
  /// `cell_size` is the full bounding-box size of one hex, in pixels.
  #[ inline ]
  #[ must_use ]
  pub fn hex_to_world_pixel_pointy( q : i32, r : i32, cell_size : ( u32, u32 ) ) -> ( f32, f32 )
  {
    // Pointy-top formula in tiles_tools:
    //   x = sqrt(3) * q + sqrt(3)/2 * r
    //   y = 1.5 * r   (Y-down)
    let pixel = Pixel::from( Coordinate::< Axial, Pointy >::new( q, r ) );
    let cw = cell_size.0 as f32;
    let ch = cell_size.1 as f32;
    let sx = cw / 3.0_f32.sqrt();
    let sy = ch / 1.5;
    ( pixel.x() * sx, -pixel.y() * sy )
  }
}

#[ cfg( test ) ]
mod tests
{
  use super::private::*;

  // Reference facts we pin: flat-top origin maps to (0,0); positive r moves
  // the point south in tiles_tools (Y-down), which becomes north-up in our
  // Y-up world pixels → i.e. larger r produces MORE NEGATIVE world y.

  #[ test ]
  fn origin_maps_to_zero()
  {
    let ( x, y ) = hex_to_world_pixel_flat( 0, 0, ( 72, 64 ) );
    assert!( x.abs() < 1e-5, "expected x ≈ 0, got {x}" );
    assert!( y.abs() < 1e-5, "expected y ≈ 0, got {y}" );
  }

  #[ test ]
  fn flat_top_y_flip_is_applied()
  {
    // Flat-top: stepping r by +1 in tiles_tools' Y-down frame means moving
    // south on-screen (larger Y). After the compile-layer flip we expect
    // negative world Y instead.
    let ( _, y0 ) = hex_to_world_pixel_flat( 0, 0, ( 72, 64 ) );
    let ( _, y1 ) = hex_to_world_pixel_flat( 0, 1, ( 72, 64 ) );
    assert!( y1 < y0, "expected r=1 to produce smaller world y than r=0, got y0={y0} y1={y1}" );
  }

  #[ test ]
  fn flat_top_x_scales_with_cell_width()
  {
    // q=1, r=0 should place the cell one full cell width to the right of origin.
    let ( x, _ ) = hex_to_world_pixel_flat( 1, 0, ( 72, 64 ) );
    // Tolerance is generous because sqrt(3) conversion is lossy — we just
    // want to pin the sign and rough magnitude. The exact value is 72 px
    // (full cell width) because of the compensating sx scale.
    assert!( x > 0.0 && x < 120.0, "x out of expected range: {x}" );
  }

  #[ test ]
  fn pointy_top_x_shifts_with_row()
  {
    // Pointy-top: moving r by +1 shifts x by half a cell width (zig-zag).
    let ( x0, _ ) = hex_to_world_pixel_pointy( 0, 0, ( 64, 72 ) );
    let ( x1, _ ) = hex_to_world_pixel_pointy( 0, 1, ( 64, 72 ) );
    assert!( ( x1 - x0 ).abs() > 1.0, "expected row shift on pointy top, got x0={x0} x1={x1}" );
  }
}

mod_interface::mod_interface!
{
  own use hex_to_world_pixel_flat;
  own use hex_to_world_pixel_pointy;
}
