//! Asset resolution — how the compile layer obtains bytes / paths for declared assets.
//!
//! The scene-model format declares each asset by `path` — a string relative to
//! some caller-defined base. The compile layer does not read files itself;
//! instead it asks an [`AssetResolver`] to produce a backend-ready
//! [`crate::assets::ImageSource`].
//!
//! Two common scenarios:
//!
//! - WebGL in a browser — files aren't read; the adapter loads images over the
//!   wire via `<img>` elements at backend load time. Use the default
//!   [`PathResolver`] which simply wraps the declared path in
//!   [`crate::assets::ImageSource::Path`].
//! - SVG or headless render — the caller has pre-decoded bytes or wants to
//!   read the file now. Implement [`AssetResolver`] to return
//!   [`crate::assets::ImageSource::Bitmap`] / `Encoded`.

mod private
{
  use std::path::PathBuf;
  use crate::assets::ImageSource;
  use crate::scene_model::compile::error::CompileError;
  use crate::scene_model::resource::AssetKind;

  /// Turns a declared asset path into a concrete [`ImageSource`] the backend
  /// can load.
  ///
  /// Called once per asset at `compile_assets` time.
  pub trait AssetResolver
  {
    /// Resolve one asset.
    ///
    /// # Arguments
    ///
    /// - `asset_id` — the asset's declared id (unique within the spec).
    /// - `path` — the declared path string, as authored in the render spec.
    /// - `kind` — the asset's layout (`Atlas` / `SpriteSheet` / `Single`),
    ///   provided so implementations that pre-load bitmaps can e.g. slice
    ///   an atlas into per-sprite bitmaps if they want to.
    ///
    /// # Errors
    ///
    /// Return [`CompileError::AssetResolution`] when the asset cannot be
    /// produced (I/O error, decoding failure, unsupported format). The
    /// compile layer aborts on the first resolver failure.
    fn resolve
    (
      &self,
      asset_id : &str,
      path : &str,
      kind : &AssetKind,
    ) -> Result< ImageSource, CompileError >;
  }

  /// Default resolver: wraps every asset's path in [`ImageSource::Path`].
  ///
  /// Appropriate for WebGL targets running in a browser — the adapter fetches
  /// the image over the wire when `Backend::load_assets` runs. Not appropriate
  /// for backends that need bytes synchronously (e.g. SVG, headless).
  #[ derive( Debug, Default, Clone, Copy ) ]
  pub struct PathResolver;

  impl AssetResolver for PathResolver
  {
    #[ inline ]
    fn resolve
    (
      &self,
      _asset_id : &str,
      path : &str,
      _kind : &AssetKind,
    ) -> Result< ImageSource, CompileError >
    {
      Ok( ImageSource::Path( PathBuf::from( path ) ) )
    }
  }
}

mod_interface::mod_interface!
{
  own use AssetResolver;
  own use PathResolver;
}
