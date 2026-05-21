//! `HexConfig::from_hex_size` — closes Polish §11 in `tilemap_scene/roadmap.md`.
//!
//! Pins the `grid_stride` arithmetic for the common case where the caller
//! has the hex sprite's bounding box and wants neighbours to tile without
//! gap or overlap.

#![ allow( clippy::min_ident_chars ) ]

use tilemap_scene::{ HexConfig, TilingStrategy };

#[ test ]
fn from_hex_size_flat_top_three_quarters_w_for_x_stride()
{
  // 72×64 flat-top hex: q-axis (horizontal) stride is ¾·72 = 54;
  // r-axis (vertical) stride is the full height (sqrt(3)/2 factor
  // is applied inside `hex_to_world_pixel_flat`, not here).
  let cfg = HexConfig::from_hex_size( 72, 64, TilingStrategy::HexFlatTop );
  assert_eq!( cfg.tiling, TilingStrategy::HexFlatTop );
  assert_eq!( cfg.grid_stride, ( 54, 64 ) );
}

#[ test ]
fn from_hex_size_pointy_top_swaps_axes()
{
  // 72×64 pointy-top hex: r-axis (vertical) stride is ¾·64 = 48;
  // q-axis (horizontal) stride is the full width.
  let cfg = HexConfig::from_hex_size( 72, 64, TilingStrategy::HexPointyTop );
  assert_eq!( cfg.tiling, TilingStrategy::HexPointyTop );
  assert_eq!( cfg.grid_stride, ( 72, 48 ) );
}

#[ test ]
fn from_hex_size_square_returns_bounding_box_stride()
{
  // Square tilings are rejected by validation, but the constructor
  // returns a sane (w, h) default so exploratory callers don't trip
  // a panic.
  let cfg = HexConfig::from_hex_size( 32, 32, TilingStrategy::Square4 );
  assert_eq!( cfg.grid_stride, ( 32, 32 ) );
}
