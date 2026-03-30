//! Backend trait — the port that all adapters implement.
//!
//! Creation is backend-specific (each has its own `new()`).
//! Usage is uniform through the `Backend` trait.

mod private
{
  use crate::commands::RenderCommand;
  use crate::assets::Assets;
  use error_tools::Error;

  // ============================================================================
  // Error type
  // ============================================================================

  /// Errors that can occur during rendering.
  #[ derive( Debug, Error ) ]
  #[ non_exhaustive ]
  pub enum RenderError
  {
    /// A command references a resource not present in Assets.
    MissingAsset( u32 ),
    /// Backend does not support this command.
    Unsupported( &'static str ),
    /// Backend-specific error.
    BackendError( String ),
  }

  impl core::fmt::Display for RenderError
  {
    #[ inline ]
    fn fmt( &self, f : &mut core::fmt::Formatter< '_ > ) -> core::fmt::Result
    {
      match self
      {
        RenderError::MissingAsset( idx ) => write!( f, "missing asset: {idx}" ),
        RenderError::Unsupported( what ) => write!( f, "unsupported: {what}" ),
        RenderError::BackendError( msg ) => write!( f, "backend error: {msg}" ),
      }
    }
  }

  // ============================================================================
  // Output type
  // ============================================================================

  /// The result of rendering.
  #[ derive( Debug ) ]
  #[ non_exhaustive ]
  pub enum Output
  {
    /// SVG string, terminal text, or other string-based output.
    String( String ),
    /// Pixel data from offscreen GPU render.
    Bitmap( Bitmap ),
    /// Already presented to screen (GPU realtime). No data to retrieve.
    Presented,
  }

  /// Raw pixel data.
  #[ derive( Debug ) ]
  pub struct Bitmap
  {
    /// Raw pixel bytes.
    pub bytes : Vec< u8 >,
    /// Image width in pixels.
    pub width : u32,
    /// Image height in pixels.
    pub height : u32,
    /// Bytes per pixel (3 = RGB, 4 = RGBA).
    pub channels : u8,
  }

  // ============================================================================
  // Capabilities
  // ============================================================================

  /// What a backend supports. Caller can check before submitting commands.
  #[ derive( Debug, Clone, Copy ) ]
  #[ allow( clippy::struct_excessive_bools ) ]
  #[ non_exhaustive ]
  pub struct Capabilities
  {
    /// Supports path drawing.
    pub paths : bool,
    /// Supports text rendering.
    pub text : bool,
    /// Supports mesh/geometry rendering.
    pub meshes : bool,
    /// Supports sprite rendering.
    pub sprites : bool,
    /// Supports batched draw calls.
    pub batches : bool,
    /// Supports gradient fills.
    pub gradients : bool,
    /// Supports tiling patterns.
    pub patterns : bool,
    /// Supports clip masks.
    pub clip_masks : bool,
    /// Supports visual effects.
    pub effects : bool,
    /// Supports blend modes.
    pub blend_modes : bool,
    /// Supports text on a path.
    pub text_on_path : bool,
    /// Maximum texture/image dimension. 0 = unlimited (e.g. SVG).
    pub max_texture_size : u32,
  }

  // ============================================================================
  // The Backend trait
  // ============================================================================

  /// The core trait that all rendering backends implement.
  ///
  /// ```ignore
  /// let config = RenderConfig { width: 800, height: 600, ..Default::default() };
  ///
  /// // SVG
  /// let mut svg = SvgBackend::new( config );
  /// svg.load_assets( &assets )?;
  /// svg.submit( &commands )?;
  /// let Output::String( doc ) = svg.output()? else { unreachable!() };
  ///
  /// // GPU (realtime) — may take extra backend-specific params
  /// let mut gpu = WgpuBackend::new( config, &window );
  /// gpu.load_assets( &assets )?;
  /// gpu.submit( &commands )?; // presents to screen
  /// ```
  pub trait Backend
  {
    /// Upload / prepare assets for this backend.
    /// Called once (or when assets change).
    ///
    /// - SVG: generates `<defs>` (symbols, gradients, patterns, clipPaths)
    /// - GPU: uploads textures, creates samplers, builds vertex buffers
    ///
    /// # Errors
    ///
    /// Returns [`RenderError::MissingAsset`] if a referenced resource cannot be resolved,
    /// or [`RenderError::BackendError`] if asset upload fails.
    fn load_assets( &mut self, assets : &Assets ) -> Result< (), RenderError >;

    /// Process a command queue. This is the main render call.
    /// Backend iterates commands sequentially, maintaining internal state
    /// for streaming commands (BeginPath..EndPath, etc.).
    ///
    /// # Errors
    ///
    /// Returns [`RenderError::MissingAsset`] if a command references an unloaded resource,
    /// [`RenderError::Unsupported`] if the backend cannot handle a command type,
    /// or [`RenderError::BackendError`] on a backend-specific failure.
    fn submit( &mut self, commands : &[ RenderCommand ] ) -> Result< (), RenderError >;

    /// Retrieve the rendered output.
    ///
    /// # Errors
    ///
    /// Returns [`RenderError::BackendError`] if the output cannot be retrieved
    /// (e.g. GPU readback failure).
    fn output( &self ) -> Result< Output, RenderError >;

    /// Resize the output surface.
    /// GPU: recreates swapchain / framebuffer.
    /// SVG: updates viewBox dimensions.
    /// Terminal: reallocates character buffer.
    fn resize( &mut self, width : u32, height : u32 );

    /// Query backend capabilities.
    fn capabilities( &self ) -> Capabilities;
  }
}

mod_interface::mod_interface!
{
  own use RenderError;
  own use Output;
  own use Bitmap;
  own use Capabilities;
  own use Backend;
}
