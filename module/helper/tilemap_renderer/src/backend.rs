//! Backend trait — the port that all adapters implement.
//!
//! Creation is backend-specific (each has its own `new()`).
//! Usage is uniform through the `Backend` trait.

use crate::types::{ ResourceId, Batch };
use crate::commands::{ RenderCommand, SpriteInstance };
use crate::assets::Assets;

// ============================================================================
// Error type
// ============================================================================

/// Errors that can occur during rendering.
#[ derive( Debug ) ]
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
  fn fmt( &self, f : &mut core::fmt::Formatter< '_ > ) -> core::fmt::Result
  {
    match self
    {
      RenderError::MissingAsset( idx ) => write!( f, "missing asset: {}", idx ),
      RenderError::Unsupported( what ) => write!( f, "unsupported: {}", what ),
      RenderError::BackendError( msg ) => write!( f, "backend error: {}", msg ),
    }
  }
}

impl std::error::Error for RenderError {}

// ============================================================================
// Output type
// ============================================================================

/// The result of rendering.
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
pub struct Bitmap
{
  pub bytes : Vec< u8 >,
  pub width : u32,
  pub height : u32,
  /// Bytes per pixel (3 = RGB, 4 = RGBA).
  pub channels : u8,
}

// ============================================================================
// Capabilities
// ============================================================================

/// What a backend supports. Caller can check before submitting commands.
#[ derive( Debug, Clone, Copy ) ]
pub struct Capabilities
{
  pub paths : bool,
  pub text : bool,
  pub meshes : bool,
  pub sprites : bool,
  pub instancing : bool,
  pub gradients : bool,
  pub patterns : bool,
  pub clip_masks : bool,
  pub effects : bool,
  pub blend_modes : bool,
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
  fn load_assets( &mut self, assets : &Assets ) -> Result< (), RenderError >;

  /// Process a command queue. This is the main render call.
  /// Backend iterates commands sequentially, maintaining internal state
  /// for streaming commands (BeginPath..EndPath, etc.).
  fn submit( &mut self, commands : &[ RenderCommand ] ) -> Result< (), RenderError >;

  /// Retrieve the rendered output.
  fn output( &self ) -> Result< Output, RenderError >;

  /// Resize the output surface.
  /// GPU: recreates swapchain / framebuffer.
  /// SVG: updates viewBox dimensions.
  /// Terminal: reallocates character buffer.
  fn resize( &mut self, width : u32, height : u32 );

  /// Query backend capabilities.
  fn capabilities( &self ) -> Capabilities;
}

// ============================================================================
// Batch extension trait — persistent instance buffer
// ============================================================================

/// Extension trait for backends that support persistent batches.
///
/// A batch is a pre-recorded instance buffer that lives on the backend.
/// It is created via streaming commands (BeginRecordBatch..EndRecordBatch),
/// then drawn and updated via methods on this trait.
///
/// GPU: instance buffer lives on GPU, `update_instance` does a sub-buffer write.
/// SVG (browser/DOM): each instance is a `<use>` element, updates modify DOM attributes.
///
/// ```ignore
/// // Record once — via command stream
/// commands.push( BeginRecordBatch( BeginRecordBatch { batch: TILEMAP_BATCH, sheet: tileset, ... } ) );
/// commands.push( SpriteInstance( SpriteInstance { transform: ..., sprite: grass, tint: WHITE } ) );
/// commands.push( SpriteInstance( SpriteInstance { transform: ..., sprite: water, tint: WHITE } ) );
/// commands.push( EndRecordBatch( EndRecordBatch ) );
/// gpu.submit( &commands )?;
///
/// // Every frame — draw without re-upload
/// gpu.draw_batch( TILEMAP_BATCH )?;
///
/// // Update one tile that changed
/// gpu.update_instance( TILEMAP_BATCH, 42, &new_sprite_instance )?;
/// ```
pub trait BatchBackend : Backend
{
  /// Draw a previously recorded batch.
  /// GPU: single instanced draw call using the stored buffer.
  /// SVG: emits `<use>` elements from stored data.
  fn draw_batch( &mut self, batch : ResourceId< Batch > ) -> Result< (), RenderError >;

  /// Update a single instance within a batch by index.
  /// GPU: sub-buffer write (cheap, no full re-upload).
  /// SVG (browser): update DOM attributes on the corresponding `<use>` element.
  fn update_instance( &mut self, batch : ResourceId< Batch >, index : u32, instance : &SpriteInstance ) -> Result< (), RenderError >;

  /// Delete a batch and free its resources.
  fn delete_batch( &mut self, batch : ResourceId< Batch > ) -> Result< (), RenderError >;
}
