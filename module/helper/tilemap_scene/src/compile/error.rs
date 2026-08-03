//! `CompileError` — all failure modes of the scene-model compile layer.

mod private
{
  use core::fmt;
  use error_tools::Error;

  /// Every way compilation can fail.
  ///
  /// Variants split by root cause so callers can react programmatically —
  /// e.g. an editor GUI can highlight "unsupported source" differently from
  /// "unresolved asset reference".
  ///
  /// Uses `#[derive(error_tools::Error)]` for the `core::error::Error` impl
  /// (which lights up the `?`-operator and `Error::source` chain). Display
  /// is intentionally manual — see the impl below.
  #[ derive( Debug, Clone, Error ) ]
  #[ non_exhaustive ]
  pub enum CompileError
  {
    /// An [`crate::compile::AssetResolver`] returned a failure
    /// (file missing, fetch failed, format unrecognised).
    AssetResolution
    {
      /// The asset id the resolver was asked about.
      id : String,
      /// Resolver-supplied explanation.
      reason : String,
    },
    /// A reference by id could not be resolved against the spec.
    UnresolvedRef
    {
      /// The kind of reference (`"asset"`, `"object"`, `"animation"`, `"tint"`, …).
      kind : &'static str,
      /// The id that could not be resolved.
      id : String,
      /// Human-readable context (e.g. `"layer sprite_source in object \"grass\""`).
      context : String,
    },
    /// An object uses an anchor not yet supported by the current slice.
    ///
    /// Slice 1 supports `Hex` only. `Edge`, `Vertex`, `Multihex`, `FreePos`,
    /// `Viewport` land in follow-up slices.
    UnsupportedAnchor
    {
      /// Owning object id.
      object : String,
      /// Anchor kind encountered.
      anchor : &'static str,
    },
    /// A layer uses a sprite source not yet supported by the current slice.
    ///
    /// Slice 1 supports `Static` only.
    UnsupportedSource
    {
      /// Owning object id.
      object : String,
      /// Source kind encountered. Named `source_kind` (not `source`) so
      /// `error_tools::Error` / thiserror does not auto-treat this `&str`
      /// as the error chain's source.
      source_kind : &'static str,
    },
    /// An asset declares a kind not supported by the current slice.
    ///
    /// Slice 1 supports `Atlas` only.
    UnsupportedAssetKind
    {
      /// The asset id.
      asset : String,
      /// Asset kind encountered.
      kind : &'static str,
    },
    /// An `Atlas` `SpriteRef` references a frame name that isn't a non-negative integer.
    InvalidFrameName
    {
      /// Host asset id.
      asset : String,
      /// The offending frame name.
      frame : String,
    },
    /// An object's `default_state` isn't present in its `states` map.
    MissingDefaultState
    {
      /// Owning object id.
      object : String,
    },
    /// A numeric reference is out of range for its source (e.g. atlas frame index
    /// past the last row/column).
    OutOfRange
    {
      /// The owning asset or resource id for the offending reference.
      owner : String,
      /// The requested index.
      index : u32,
      /// The exclusive upper bound.
      max : u32,
    },
    /// An `Atlas` frame's resolved pixel rect lies outside the
    /// declared `image_size`.
    ///
    /// Indicates a mismatch between authored `columns` (or
    /// `frames`/named-cell entries) and the actual image. Surfaces
    /// the bug where `columns` is treated as the image's column count
    /// instead of the addressable column count of the auto-numbering
    /// scheme.
    FrameOutOfBounds
    {
      /// The asset id whose frame was out of bounds.
      asset : String,
      /// The frame name (numeric index or named-cell id) that failed.
      frame : String,
      /// The frame's resolved `( col, row )` in the grid.
      cell : ( u32, u32 ),
      /// The frame's resolved pixel rect `[ x, y, w, h ]`.
      rect : [ u32; 4 ],
      /// Declared `image_size` the rect was checked against.
      image_size : ( u32, u32 ),
    },
  }

  impl fmt::Display for CompileError
  {
    #[ inline ]
    fn fmt( &self, f : &mut fmt::Formatter< '_ > ) -> fmt::Result
    {
      match self
      {
        Self::AssetResolution { id, reason } =>
          write!( f, "asset resolution failed for {id:?}: {reason}" ),
        Self::UnresolvedRef { kind, id, context } =>
          write!( f, "unresolved {kind} reference {id:?} in {context}" ),
        Self::UnsupportedAnchor { object, anchor } =>
          write!( f, "object {object:?} uses anchor {anchor} which is not supported in this slice" ),
        Self::UnsupportedSource { object, source_kind } =>
          write!( f, "object {object:?} uses sprite source {source_kind} which is not supported in this slice" ),
        Self::UnsupportedAssetKind { asset, kind } =>
          write!( f, "asset {asset:?} uses kind {kind} which is not supported in this slice" ),
        Self::InvalidFrameName { asset, frame } =>
          write!( f, "asset {asset:?} received non-numeric frame name {frame:?}" ),
        Self::MissingDefaultState { object } =>
          write!( f, "object {object:?} has no state matching its declared default" ),
        Self::OutOfRange { owner, index, max } =>
          write!( f, "index {index} out of range for {owner:?} (max {max})" ),
        Self::FrameOutOfBounds { asset, frame, cell : ( col, row ), rect, image_size : ( iw, ih ) } =>
          write!
          (
            f,
            "atlas {asset:?}: frame {frame:?} resolves to col={col} row={row} \
             (rect [x={}, y={}, w={}, h={}]) which extends past image_size ({iw} × {ih}). \
             To use this region, either bump `columns`, set `image_size` to the actual \
             image dimensions, or add an explicit `frame_rects` entry.",
            rect[ 0 ], rect[ 1 ], rect[ 2 ], rect[ 3 ],
          ),
      }
    }
  }
}

mod_interface::mod_interface!
{
  exposed use CompileError;
}
