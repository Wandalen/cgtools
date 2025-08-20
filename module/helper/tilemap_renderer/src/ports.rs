//! Port traits defining the rendering abstraction layer.
//!
//! This module defines the core traits that backend adapters must implement
//! to provide rendering capabilities. The Ports & Adapters architecture
//! decouples the core rendering logic from specific backend implementations.

#[ cfg( feature = "enabled" ) ]
mod private
{

  // Allow certain clippy warnings for trait definitions
  #![ allow( clippy::missing_inline_in_public_items ) ]
  #![ allow( clippy::implicit_return ) ]
  #![ allow( clippy::exhaustive_enums ) ]

  use crate::scene::Scene;
  use crate::commands::{ Point2D, RenderCommand, LineCommand, CurveCommand, TextCommand, TilemapCommand, ParticleEmitterCommand };

  /// Rendering capability information returned by backends.
  #[ derive( Debug, Clone, PartialEq ) ]
  #[ non_exhaustive ]
  #[ allow( clippy::struct_excessive_bools ) ]
  pub struct RendererCapabilities
  {
    /// Backend name (e.g., "SVG", "Terminal", "WebGL").
    pub backend_name : String,
    /// Backend version information.
    pub backend_version : String,
    /// Maximum texture size supported (0 if not applicable).
    pub max_texture_size : u32,
    /// Whether the backend supports transparency/alpha blending.
    pub supports_transparency : bool,
    /// Whether the backend supports antialiasing.
    pub supports_antialiasing : bool,
    /// Whether the backend supports custom fonts.
    pub supports_custom_fonts : bool,
    /// Whether the backend supports particle effects.
    pub supports_particles : bool,
    /// Whether the backend supports real-time rendering.
    pub supports_realtime : bool,
    /// Maximum scene complexity (estimated render commands).
    pub max_scene_complexity : usize,
  }

  /// Rendering context information for frame rendering.
  #[ derive( Debug, Clone, Copy, PartialEq ) ]
  #[ non_exhaustive ]
  pub struct RenderContext
  {
    /// Output width in pixels.
    pub width : u32,
    /// Output height in pixels.
    pub height : u32,
    /// Background color (RGBA, 0.0-1.0).
    pub background_color : [ f32; 4 ],
    /// Whether to clear the background before rendering.
    pub clear_background : bool,
    /// Optional viewport offset.
    pub viewport_offset : Point2D,
    /// Optional viewport scale factor.
    pub viewport_scale : f32,
  }

  impl RenderContext
  {
    pub fn new
    (
      width : u32,
      height : u32,
      background_color : [ f32; 4 ],
      clear_background : bool,
      viewport_offset : Point2D,
      viewport_scale : f32
    ) -> Self
    {
      Self { width, height, background_color, clear_background, viewport_offset, viewport_scale }
    }
  }

  /// Rendering error types that can occur during rendering operations.
  #[ derive( Debug, Clone, PartialEq ) ]
  pub enum RenderError
  {
    /// Generic rendering failure with message.
    RenderFailed( String ),
    /// Unsupported command type for this backend.
    UnsupportedCommand( String ),
    /// Invalid rendering context or parameters.
    InvalidContext( String ),
    /// Backend-specific initialization failure.
    InitializationFailed( String ),
    /// Resource allocation failure (memory, GPU, etc.).
    ResourceAllocationFailed( String ),
    /// IO error during output generation.
    OutputError( String ),
    /// Scene complexity exceeds backend limits.
    ComplexityLimitExceeded,
    /// Backend feature not implemented.
    FeatureNotImplemented( String ),
  }

  impl core::fmt::Display for RenderError
  {
    #[ allow( clippy::min_ident_chars ) ]
    #[ allow( clippy::uninlined_format_args ) ]
    fn fmt( &self, formatter: &mut core::fmt::Formatter< '_ > ) -> core::fmt::Result
    {
      match self
      {
        RenderError::RenderFailed( msg ) => write!( formatter, "Render failed: {msg}" ),
        RenderError::UnsupportedCommand( cmd ) => write!( formatter, "Unsupported command: {cmd}" ),
        RenderError::InvalidContext( msg ) => write!( formatter, "Invalid context: {msg}" ),
        RenderError::InitializationFailed( msg ) => write!( formatter, "Initialization failed: {msg}" ),
        RenderError::ResourceAllocationFailed( msg ) => write!( formatter, "Resource allocation failed: {msg}" ),
        RenderError::OutputError( msg ) => write!( formatter, "Output error: {msg}" ),
        RenderError::ComplexityLimitExceeded => write!( formatter, "Scene complexity limit exceeded" ),
        RenderError::FeatureNotImplemented( feature ) => write!( formatter, "Feature not implemented: {feature}" ),
      }
    }
  }

  impl core::error::Error for RenderError {}

  /// Primary renderer trait defining the rendering lifecycle.
  ///
  /// This trait defines the main interface that backend adapters must implement.
  /// It provides the core rendering lifecycle: initialization, frame rendering,
  /// and cleanup operations.
  pub trait Renderer
  {
    /// Backend-specific output type (e.g., SVG string, image buffer, etc.).
    type Output;

    /// Returns the capabilities of this renderer backend.
    fn capabilities( &self ) -> RendererCapabilities;

    /// Initializes the renderer with the given context.
    ///
    /// This method should prepare the renderer for rendering operations,
    /// allocate necessary resources, and validate the rendering context.
    ///
    /// # Errors
    /// Returns `InitializationFailed` if the renderer cannot be initialized.
    fn initialize( &mut self, context: &RenderContext ) -> core::result::Result< (), RenderError >;

    /// Begins a new rendering frame.
    ///
    /// This method should prepare for rendering a new frame, typically
    /// clearing the output surface and setting up initial state.
    ///
    /// # Errors
    /// Returns `InvalidContext` if the context is invalid or `RenderFailed` if frame setup fails.
    fn begin_frame( &mut self, context: &RenderContext ) -> core::result::Result< (), RenderError >;

    /// Renders a complete scene to the output.
    ///
    /// This is the main rendering method that processes all commands in the
    /// scene and produces the final output.
    ///
    /// # Errors
    /// Returns various errors including `UnsupportedCommand`, `ComplexityLimitExceeded`, or `RenderFailed`.
    fn render_scene( &mut self, scene: &Scene ) -> core::result::Result< (), RenderError >;

    /// Ends the current rendering frame and finalizes the output.
    ///
    /// This method should complete any pending operations and prepare
    /// the final output for retrieval.
    ///
    /// # Errors
    /// Returns `RenderFailed` if frame finalization fails.
    fn end_frame( &mut self ) -> core::result::Result< (), RenderError >;

    /// Retrieves the rendered output.
    ///
    /// This method should return the final rendered output in the
    /// backend-specific format.
    ///
    /// # Errors
    /// Returns `OutputError` if the output cannot be retrieved.
    fn output( &self ) -> core::result::Result< Self::Output, RenderError >;

    /// Performs cleanup and releases resources.
    ///
    /// This method should clean up any allocated resources and
    /// reset the renderer state.
    ///
    /// # Errors
    /// Returns `RenderFailed` if cleanup operations fail.
    fn cleanup( &mut self ) -> core::result::Result< (), RenderError >;

    /// Checks if a specific command type is supported by this renderer.
    fn supports_command( &self, command: &RenderCommand ) -> bool
    {
      match command
      {
        RenderCommand::Line( _ ) => self.supports_lines(),
        RenderCommand::Curve( _ ) => self.supports_curves(),
        RenderCommand::Text( _ ) => self.supports_text(),
        RenderCommand::Tilemap( _ ) => self.supports_tilemaps(),
        RenderCommand::ParticleEmitter( _ ) => self.supports_particles(),
        RenderCommand::Geometry2DCommand( _ ) => self.supports_geometry2d(),
        RenderCommand::SpriteCommand( _ ) => self.supports_sprite(),
      }
    }

    /// Returns whether this renderer supports line primitives.
    fn supports_lines( &self ) -> bool
    {
      true // Most backends support lines
    }

    /// Returns whether this renderer supports curve primitives.
    fn supports_curves( &self ) -> bool
    {
      true // Most backends support curves
    }

    /// Returns whether this renderer supports text primitives.
    fn supports_text( &self ) -> bool
    {
      true // Most backends support text
    }

    /// Returns whether this renderer supports tilemap primitives.
    fn supports_tilemaps( &self ) -> bool
    {
      false // Not all backends support tilemaps
    }

    /// Returns whether this renderer supports particle emitter primitives.
    fn supports_particles( &self ) -> bool
    {
      false // Not all backends support particles
    }

    fn supports_geometry2d( &self ) -> bool
    {
      false
    }

    fn supports_sprite( &self ) -> bool
    {
      false
    }

    /// Validates that the renderer can handle the given scene.
    ///
    /// # Errors
    /// Returns `ComplexityLimitExceeded` or `UnsupportedCommand` if validation fails.
    fn validate_scene( &self, scene: &Scene ) -> core::result::Result< (), RenderError >
    {
      // Check scene complexity
      if scene.len() > self.capabilities().max_scene_complexity
      {
        return Err( RenderError::ComplexityLimitExceeded );
      }

      // Check command support
      for command in scene.commands()
      {
        if !self.supports_command( command )
        {
          let cmd_name = match command
          {
            RenderCommand::Line( _ ) => "Line",
            RenderCommand::Curve( _ ) => "Curve",
            RenderCommand::Text( _ ) => "Text",
            RenderCommand::Tilemap( _ ) => "Tilemap",
            RenderCommand::ParticleEmitter( _ ) => "ParticleEmitter",
            RenderCommand::Geometry2DCommand( _ ) => "Geometry2DCommand",
            RenderCommand::SpriteCommand( _ ) => "SpriteCommand",
          };
          return Err( RenderError::UnsupportedCommand( cmd_name.to_string() ) );
        }
      }

      Ok( () )
    }
  }

  /// Trait for rendering individual primitive commands.
  ///
  /// This trait provides granular control over how individual rendering
  /// commands are processed. Backends can implement this trait to handle
  /// specific command types with custom logic.
  pub trait PrimitiveRenderer
  {
    /// Renders a line command.
    ///
    /// # Errors
    /// Returns `RenderFailed` if line rendering fails.
    fn render_line( &mut self, command: &LineCommand ) -> core::result::Result< (), RenderError >;

    /// Renders a curve command.
    ///
    /// # Errors
    /// Returns `RenderFailed` if curve rendering fails.
    fn render_curve( &mut self, command: &CurveCommand ) -> core::result::Result< (), RenderError >;

    /// Renders a text command.
    ///
    /// # Errors
    /// Returns `RenderFailed` if text rendering fails.
    fn render_text( &mut self, command: &TextCommand ) -> core::result::Result< (), RenderError >;

    /// Renders a tilemap command.
    ///
    /// # Errors
    /// Returns `FeatureNotImplemented` if tilemap rendering is not supported.
    fn render_tilemap( &mut self, command: &TilemapCommand ) -> core::result::Result< (), RenderError >;

    /// Renders a particle emitter command.
    ///
    /// # Errors
    /// Returns `FeatureNotImplemented` if particle rendering is not supported.
    fn render_particle_emitter( &mut self, command: &ParticleEmitterCommand ) -> core::result::Result< (), RenderError >;

    /// Dispatches a render command to the appropriate primitive renderer method.
    ///
    /// # Errors
    /// Returns errors from the underlying primitive rendering methods.
    fn render_command( &mut self, command: &RenderCommand ) -> core::result::Result< (), RenderError >
    {
      match command
      {
        RenderCommand::Line( cmd ) => self.render_line( cmd ),
        RenderCommand::Curve( cmd ) => self.render_curve( cmd ),
        RenderCommand::Text( cmd ) => self.render_text( cmd ),
        RenderCommand::Tilemap( cmd ) => self.render_tilemap( cmd ),
        RenderCommand::ParticleEmitter( cmd ) => self.render_particle_emitter( cmd ),
        RenderCommand::Geometry2DCommand( _cmd ) => todo!(),
        RenderCommand::SpriteCommand( _cmd ) => todo!(),
      }
    }
  }

  /// Trait for async rendering operations.
  ///
  /// This trait extends the basic Renderer trait with async capabilities
  /// for backends that benefit from asynchronous processing.
  pub trait AsyncRenderer : Renderer + Send + Sync
  {
    /// Asynchronously renders a scene.
    fn render_scene_async( &mut self, scene: &Scene ) -> impl core::future::Future< Output = core::result::Result< (), RenderError > > + Send
    where
      Self: Send,
    {
      async move
      {
        self.render_scene( scene )
      }
    }

    /// Asynchronously retrieves the rendered output.
    fn output_async( &self ) -> impl core::future::Future< Output = core::result::Result< Self::Output, RenderError > > + Send
    where
      Self: Sync,
    {
      async move
      {
        self.output()
      }
    }
  }

  impl Default for RenderContext
  {
    fn default() -> Self
    {
      Self
      {
        width: 800,
        height: 600,
        background_color: [ 1.0, 1.0, 1.0, 1.0 ], // White background
        clear_background: true,
        viewport_offset: Point2D::default(),
        viewport_scale: 1.0,
      }
    }
  }

  impl Default for RendererCapabilities
  {
    fn default() -> Self
    {
      Self
      {
        backend_name: "Unknown".to_string(),
        backend_version: "0.0.0".to_string(),
        max_texture_size: 0,
        supports_transparency: false,
        supports_antialiasing: false,
        supports_custom_fonts: false,
        supports_particles: false,
        supports_realtime: false,
        max_scene_complexity: 1000,
      }
    }
  }

}

#[ cfg( feature = "enabled" ) ]
pub use private::*;
