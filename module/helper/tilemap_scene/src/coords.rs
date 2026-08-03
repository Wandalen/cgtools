//! Coordinate type aliases over `tiles_tools`.
//!
//! The scene-model format uses axial `( q, r )` coordinates for hex cells and
//! tri-axial `( a, b, c )` coordinates for dual-mesh triangles (see SPEC §2, §3).
//! Rather than redefine these, this module aliases the corresponding types from
//! `tiles_tools`. Serde / hash / arithmetic come along from the upstream types.
//!
//! Coordinates inside RON scene data may appear as either strongly-typed
//! `HexCoord` / `TriCoord` values or as raw `( i32, i32 )` tuples — the latter
//! keeps RON literals terse and orientation-agnostic. Conversion happens at
//! use sites.

mod private
{
  pub use tiles_tools::coordinates::hexagonal::{ Axial, Flat, Pointy, Offset, Odd, Even };
  pub use tiles_tools::coordinates::triangular::{ FlatSided, FlatTopped };

  /// Hex coordinate in axial `( q, r )` form.
  ///
  /// Default orientation is [`Flat`]-top; pointy-top is available via
  /// `HexCoord< Pointy >`. See SPEC §2.3 for direction ordering.
  pub type HexCoord< Orientation = Flat > =
    tiles_tools::coordinates::hexagonal::Coordinate< Axial, Orientation >;

  /// Triangle coordinate in the dual mesh of a hex grid.
  ///
  /// Each triangle touches exactly 3 hexes (its corners). See SPEC §3.3 and
  /// `tiles_tools::coordinates::triangular` for the `a + b + c ∈ { 1, 2 }`
  /// invariant.
  pub type TriCoord< Orientation = FlatSided > =
    tiles_tools::coordinates::triangular::Coordinate< Orientation >;

  /// Pixel coordinate in world or screen space.
  ///
  /// Used by the `FreePos` anchor (world pixels) and by viewport layout
  /// helpers (screen pixels — same type, different semantic space).
  pub type Pixel = tiles_tools::coordinates::pixel::Pixel;
}

mod_interface::mod_interface!
{
  exposed use Axial;
  exposed use Flat;
  exposed use Pointy;
  exposed use Offset;
  exposed use Odd;
  exposed use Even;
  exposed use FlatSided;
  exposed use FlatTopped;
  exposed use HexCoord;
  exposed use TriCoord;
  exposed use Pixel;
}
