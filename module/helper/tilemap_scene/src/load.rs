//! Loading API for `render_spec.ron` and `scene.ron` files.
//!
//! Parsing uses `ron`; validation runs as a second pass (see
//! [`crate::validate`]) and errors are collected so the user sees
//! all issues, not just the first.

mod private
{
  use std::path::Path;
  use crate::error::LoadError;
  use crate::snapshot::SceneSnapshot;
  use crate::spec::RenderSpec;
  use crate::validate::Validate;

  impl RenderSpec
  {
    /// Parses a render spec from a RON-formatted string.
    ///
    /// Does **not** run validation — tests use this to exercise the parser
    /// in isolation. Production callers should prefer [`Self::load`].
    ///
    /// # Errors
    ///
    /// Returns [`LoadError::Ron`] if the input is not valid RON or does not
    /// match the expected schema.
    #[ inline ]
    pub fn from_ron_str( s : &str ) -> Result< Self, LoadError >
    {
      let spec = ron::from_str::< Self >( s )?;
      Ok( spec )
    }

    /// Loads, parses, and validates a render spec from a file on disk.
    ///
    /// Validation enforces a partial subset of SPEC §16 — see [`Validate`]
    /// for the per-type list of rules currently checked. Rules not yet
    /// implemented stay as `// TODO SPEC §16` markers in
    /// `tilemap_scene/src/validate.rs`; a successful load proves only the
    /// implemented rules pass.
    ///
    /// # Errors
    ///
    /// - [`LoadError::Io`] if the file cannot be read.
    /// - [`LoadError::Ron`] if parsing fails.
    /// - [`LoadError::Validation`] when one or more enforced SPEC §16 rules
    ///   reject the spec (see [`Validate`] for the current rule set).
    #[ inline ]
    pub fn load( path : impl AsRef< Path > ) -> Result< Self, LoadError >
    {
      let text = std::fs::read_to_string( path )?;
      let spec = Self::from_ron_str( &text )?;
      spec.validate().map_err( LoadError::Validation )?;
      Ok( spec )
    }
  }

  impl SceneSnapshot
  {
    /// Parses a scene snapshot from a RON-formatted string.
    ///
    /// Does **not** run validation; see [`Self::load`] for the full pipeline.
    ///
    /// # Errors
    ///
    /// Returns [`LoadError::Ron`] if the input is not valid RON or does not
    /// match the expected schema.
    #[ inline ]
    pub fn from_ron_str( s : &str ) -> Result< Self, LoadError >
    {
      let scene = ron::from_str::< Self >( s )?;
      Ok( scene )
    }

    /// Loads, parses, and validates a scene from a file on disk.
    ///
    /// Scene-internal validation is not yet implemented (see [`Validate`]
    /// for the trait-level note); cross-file Scene → `RenderSpec` checks run
    /// in a separate pass.
    ///
    /// # Errors
    ///
    /// Same as [`RenderSpec::load`], plus scene-specific validation failures
    /// once Scene-side SPEC §16 rules are enforced.
    #[ inline ]
    pub fn load( path : impl AsRef< Path > ) -> Result< Self, LoadError >
    {
      let text = std::fs::read_to_string( path )?;
      let scene = Self::from_ron_str( &text )?;
      scene.validate().map_err( LoadError::Validation )?;
      Ok( scene )
    }
  }
}

mod_interface::mod_interface!
{
}
