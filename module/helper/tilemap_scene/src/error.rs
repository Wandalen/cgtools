//! Error types for scene-model loading and validation.

mod private
{
  use core::fmt;

  /// Error returned by [`crate::spec::RenderSpec::load`] /
  /// [`crate::scene::Scene::load`] and their `from_ron_str`
  /// counterparts.
  ///
  /// Wraps I/O, RON parsing, and post-parse validation failures under a single
  /// type so callers can handle them uniformly.
  #[ derive( Debug ) ]
  #[ non_exhaustive ]
  pub enum LoadError
  {
    /// The file could not be read from disk.
    Io( std::io::Error ),
    /// The RON payload could not be parsed into scene-model types.
    Ron( ron::error::SpannedError ),
    /// Parsing succeeded but validation reported one or more errors.
    Validation( Vec< ValidationError > ),
  }

  impl fmt::Display for LoadError
  {
    #[ inline ]
    fn fmt( &self, f : &mut fmt::Formatter< '_ > ) -> fmt::Result
    {
      match self
      {
        Self::Io( e ) => write!( f, "scene-model io error: {e}" ),
        Self::Ron( e ) => write!( f, "scene-model ron error: {e}" ),
        Self::Validation( errs ) =>
        {
          writeln!( f, "scene-model validation failed with {} error(s):", errs.len() )?;
          for e in errs
          {
            writeln!( f, "  - {e}" )?;
          }
          Ok( () )
        },
      }
    }
  }

  impl core::error::Error for LoadError
  {
    #[ inline ]
    fn source( &self ) -> Option< &( dyn core::error::Error + 'static ) >
    {
      match self
      {
        Self::Io( e ) => Some( e ),
        Self::Ron( e ) => Some( e ),
        Self::Validation( _ ) => None,
      }
    }
  }

  impl From< std::io::Error > for LoadError
  {
    #[ inline ]
    fn from( value : std::io::Error ) -> Self { Self::Io( value ) }
  }

  impl From< ron::error::SpannedError > for LoadError
  {
    #[ inline ]
    fn from( value : ron::error::SpannedError ) -> Self { Self::Ron( value ) }
  }

  /// A single violation of a validation rule.
  ///
  /// Validation collects all violations and reports them together (SPEC §16);
  /// it does not stop at the first. This enum enumerates the distinct shapes a
  /// violation can take.
  #[ derive( Debug, Clone ) ]
  #[ non_exhaustive ]
  pub enum ValidationError
  {
    /// Two items of the same declaration kind share an id.
    DuplicateId
    {
      /// Kind of item that has duplicate ids (e.g. `"asset"`, `"object"`).
      kind : &'static str,
      /// The duplicated id.
      id : String,
    },
    /// A reference points to an id that was not declared in this spec.
    UnresolvedRef
    {
      /// Kind of reference (e.g. `"asset"`, `"tint"`, `"animation"`, `"object"`).
      kind : &'static str,
      /// The referenced id that could not be resolved.
      id : String,
      /// Human-readable location where the reference appeared.
      context : String,
    },
    /// A composite sprite source was nested inside another composite source.
    ///
    /// SPEC §5 forbids `NeighborBitmask` inside `VertexCorners`, etc. —
    /// composites accept only leaf sources (`Static` / `Variant` / `Animation`
    /// / `External`) as inner values.
    IllegalSourceNesting
    {
      /// The outer composite source kind.
      outer : &'static str,
      /// The inner composite source kind found in its leaf slot.
      inner : &'static str,
    },
    /// The spec requests a tiling strategy not supported by this implementation.
    ///
    /// Version 0.2.0 implements `HexFlatTop` and `HexPointyTop` only; the
    /// `Square4` / `Square8` values are reserved but rejected at load time.
    UnsupportedTiling( String ),
    /// A sprite source is not valid for the declaring object's anchor type.
    ///
    /// For example, `NeighborBitmask` only works on `Hex` anchors;
    /// `EdgeConnectedBitmask` only on `Edge`. See SPEC §3 and §5.
    AnchorSourceMismatch
    {
      /// The anchor kind (`"Hex"`, `"Edge"`, …).
      anchor : &'static str,
      /// The source kind attempted on it.
      source_kind : &'static str,
    },
    /// The object's `default_state` is not present in its `states` map.
    MissingDefaultState
    {
      /// Owning object's id.
      object : String,
      /// The state name that was set as default but not declared.
      state : String,
    },
    /// A reserved id was used in a user declaration.
    ///
    /// Currently the only reserved id is `"void"` (SPEC §15.1).
    ReservedId
    {
      /// The reserved id that was illegally declared.
      id : String,
    },
  }

  impl fmt::Display for ValidationError
  {
    #[ inline ]
    fn fmt( &self, f : &mut fmt::Formatter< '_ > ) -> fmt::Result
    {
      match self
      {
        Self::DuplicateId { kind, id } => write!( f, "duplicate {kind} id: {id:?}" ),
        Self::UnresolvedRef { kind, id, context } =>
          write!( f, "unresolved {kind} reference {id:?} in {context}" ),
        Self::IllegalSourceNesting { outer, inner } =>
          write!( f, "composite source {inner} cannot be nested inside {outer}" ),
        Self::UnsupportedTiling( name ) =>
          write!( f, "unsupported tiling strategy: {name}" ),
        Self::AnchorSourceMismatch { anchor, source_kind } =>
          write!( f, "sprite source {source_kind} is not valid for anchor {anchor}" ),
        Self::MissingDefaultState { object, state } =>
          write!( f, "object {object:?} declares default_state {state:?} but has no such entry" ),
        Self::ReservedId { id } => write!( f, "reserved id used in declaration: {id:?}" ),
      }
    }
  }

  impl core::error::Error for ValidationError {}
}

mod_interface::mod_interface!
{
  exposed use LoadError;
  exposed use ValidationError;
}
