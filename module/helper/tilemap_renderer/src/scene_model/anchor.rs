//! Anchor — how an object is attached to the world.
//!
//! See SPEC §3. The anchor determines what the object's "position" means, what
//! neighbour / context inputs its sprite sources can use, and how it is culled
//! and sorted. Anchor is a *schema* property of the object class, not of a
//! specific instance — per-instance positions live in the scene file.

mod private
{
  use serde::{ Deserialize, Serialize };

  /// Attachment mode for an [`crate::scene_model::object::Object`]. See SPEC §3.
  #[ derive( Debug, Clone, Serialize, Deserialize ) ]
  #[ non_exhaustive ]
  pub enum Anchor
  {
    /// One grid cell. Position = one `( q, r )`. See SPEC §3.1.
    Hex,
    /// An edge between two cells. Position = `( hex, direction )`. See SPEC §3.2.
    Edge,
    /// A vertex of the dual mesh. Position = tuple of adjacent cells. See SPEC §3.3.
    Vertex,
    /// Multiple cells in a fixed shape (castle, bridge). See SPEC §3.4.
    Multihex
    {
      /// Shape as a list of `( q, r )` offsets relative to the anchor cell.
      shape : Vec< ( i32, i32 ) >,
    },
    /// A free world-space pixel point (projectiles, particles, damage numbers).
    /// See SPEC §3.5.
    FreePos,
    /// Screen-space — drawn relative to the viewport. Fixed positioning plus
    /// optional parallax against the camera. See SPEC §3.6.
    Viewport,
  }

  /// Direction enum for hex neighbour sides. See SPEC §2.3.
  ///
  /// Lists all six directions for flat-top hex (`N, NE, SE, S, SW, NW`) and
  /// the `E` / `W` directions used by pointy-top hex. The active subset is
  /// determined by the pipeline's tiling strategy.
  #[ derive( Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize ) ]
  #[ non_exhaustive ]
  pub enum EdgeDirection
  {
    /// North (flat-top only).
    N,
    /// North-east.
    NE,
    /// East (pointy-top only).
    E,
    /// South-east.
    SE,
    /// South (flat-top only).
    S,
    /// South-west.
    SW,
    /// West (pointy-top only).
    W,
    /// North-west.
    NW,
  }

  /// Which Y coordinate a [`Anchor::Multihex`] object uses for Y-sorting.
  ///
  /// The default `Anchor` matches the simple case where the anchor cell is at
  /// the object's visual top; `BottomOfShape` suits objects whose base line
  /// is at the bottom of the bounding box (e.g. a multi-cell castle whose
  /// footprint extends south from the anchor).
  #[ derive( Debug, Clone, Copy, Serialize, Deserialize, Default ) ]
  #[ non_exhaustive ]
  pub enum SortYSource
  {
    /// Use the anchor cell's Y.
    #[ default ]
    Anchor,
    /// Use the bottom-most Y across all cells in the shape.
    BottomOfShape,
  }
}

mod_interface::mod_interface!
{
  own use Anchor;
  own use EdgeDirection;
  own use SortYSource;
}
