//! Loading API for `render_spec.ron` and `scene.ron` files.
//!
//! Parsing uses `ron`; validation runs as a second pass (see
//! [`crate::validate`]) and errors are collected so the user sees
//! all issues, not just the first.

mod private
{
  use std::path::Path;
  use crate::error::LoadError;
  use crate::scene::Scene;
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
    /// # Errors
    ///
    /// - [`LoadError::Io`] if the file cannot be read.
    /// - [`LoadError::Ron`] if parsing fails.
    /// - [`LoadError::Validation`] if the parsed spec violates SPEC §16 rules.
    #[ inline ]
    pub fn load( path : impl AsRef< Path > ) -> Result< Self, LoadError >
    {
      let text = std::fs::read_to_string( path )?;
      let spec = Self::from_ron_str( &text )?;
      spec.validate().map_err( LoadError::Validation )?;
      Ok( spec )
    }
  }

  impl Scene
  {
    /// Parses a scene from a RON-formatted string.
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
    /// # Errors
    ///
    /// Same as [`RenderSpec::load`], plus scene-specific validation failures.
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
