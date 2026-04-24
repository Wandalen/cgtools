//! `CompileError` — all failure modes of the scene-model compile layer.

mod private
{
  use core::fmt;

  /// Every way compilation can fail.
  ///
  /// Variants split by root cause so callers can react programmatically —
  /// e.g. an editor GUI can highlight "unsupported source" differently from
  /// "unresolved asset reference".
  #[ derive( Debug, Clone ) ]
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
      /// Source kind encountered.
      source : &'static str,
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
        Self::UnsupportedSource { object, source } =>
          write!( f, "object {object:?} uses sprite source {source} which is not supported in this slice" ),
        Self::UnsupportedAssetKind { asset, kind } =>
          write!( f, "asset {asset:?} uses kind {kind} which is not supported in this slice" ),
        Self::InvalidFrameName { asset, frame } =>
          write!( f, "asset {asset:?} received non-numeric frame name {frame:?}" ),
        Self::MissingDefaultState { object } =>
          write!( f, "object {object:?} has no state matching its declared default" ),
        Self::OutOfRange { owner, index, max } =>
          write!( f, "index {index} out of range for {owner:?} (max {max})" ),
      }
    }
  }

  impl core::error::Error for CompileError {}
}

mod_interface::mod_interface!
{
  exposed use CompileError;
}
