//! Renders a [`shader_chunks_preview_core::PreviewBundle`] to raw RGBA
//! pixels on a headless GPU — one static frame of exactly what the
//! browser runner shows live, with every slider at its initial value.
//! The whole graphics path is `minwgpu`'s offscreen toolkit: headless
//! context, one uniform buffer laid out by the preview convention
//! ( `time`, then the parameters, then `resolution : vec4f` at the next
//! 16-byte boundary — [`uniform_floats`] packs it, reusing
//! [`shader_chunks_preview_core::resolution_index`] so this crate and the
//! browser runner can never disagree on the layout ), the bufferless
//! fullscreen-triangle pipeline, a single clear-and-draw pass, and a
//! row-padding-aware readback.
//!
//! The render target is `Rgba8Unorm`, deliberately not `Rgba8UnormSrgb`:
//! chunks write display-ready values ( the `srgb` chunk exists precisely
//! because encoding is the shader author's explicit move ), so an sRGB
//! target would double-encode them. No I/O and no image encoding happen
//! here — the crate returns pixels; writing a PNG is the CLI layer's job
//! ( `shader_chunks_render` ).

mod private
{
  use core::fmt;
  use shader_chunks_preview_core::{ PreviewBundle, resolution_index };

  /// One rendered frame: tightly packed RGBA8 pixels, top row first,
  /// together with the `( width, height )` size they were rendered at.
  #[ derive( Debug, Clone, PartialEq, Eq ) ]
  pub struct RenderedImage
  {
    /// Tightly packed RGBA8 pixels, `width * height * 4` bytes.
    pub pixels : Vec< u8 >,
    /// The `( width, height )` render size in pixels.
    pub size : ( u32, u32 ),
  }

  /// Error returned by [`render`].
  #[ derive( Debug, Clone, PartialEq, Eq ) ]
  pub enum RenderError
  {
    /// The requested size has a zero dimension — nothing to render.
    ZeroSize,
    /// Acquiring the headless GPU context failed ( no usable
    /// adapter/device on this machine ).
    Context( String ),
    /// The GPU rejected the bundle's WGSL or the render setup — a
    /// validation error caught by the device's error scope.
    Gpu( String ),
    /// GPU→host readback of the rendered pixels failed.
    Readback( String ),
  }

  impl fmt::Display for RenderError
  {
    fn fmt( &self, f : &mut fmt::Formatter< '_ > ) -> fmt::Result
    {
      match self
      {
        Self::ZeroSize => write!( f, "render size must be at least 1x1 pixel" ),
        Self::Context( msg ) => write!( f, "no usable headless GPU context: {msg}" ),
        Self::Gpu( msg ) => write!( f, "GPU rejected the render: {msg}" ),
        Self::Readback( msg ) => write!( f, "reading rendered pixels back failed: {msg}" ),
      }
    }
  }

  impl std::error::Error for RenderError {}

  /// Packs the uniform buffer content for one frame of `bundle` at the
  /// given render `size` and `time`, following the preview uniform layout
  /// convention: index 0 is `time`, indices `1..=N` are the N parameters'
  /// initial values in declaration order, zero padding runs up to
  /// [`resolution_index`], and the final four floats are
  /// `( width, height, 0, 0 )` — so the buffer is always a whole number
  /// of 16-byte rows, as WGSL's struct rules require.
  #[ must_use ]
  pub fn uniform_floats( bundle : &PreviewBundle, size : ( u32, u32 ), time : f32 ) -> Vec< f32 >
  {
    let resolution_at = resolution_index( bundle.parameters.len() );
    let mut floats = vec![ 0.0_f32; resolution_at + 4 ];
    floats[ 0 ] = time;
    for ( index, parameter ) in bundle.parameters.iter().enumerate()
    {
      floats[ index + 1 ] = parameter.value as f32;
    }
    floats[ resolution_at ] = size.0 as f32;
    floats[ resolution_at + 1 ] = size.1 as f32;
    floats
  }

  /// Renders one frame of `bundle` at `size` pixels and `time` seconds on
  /// a headless GPU, returning the frame as tightly packed RGBA8 pixels.
  /// Every bundle parameter takes its initial ( slider-start ) value —
  /// the frame matches what the browser preview shows before anyone
  /// touches a slider, at the same `time`.
  ///
  /// The bundle's WGSL is compiled by `wgpu` itself here; callers wanting
  /// friendlier diagnostics should naga-validate first ( as
  /// `shader_chunks_render`'s `bundle_prepare` reuse does ) — a shader
  /// rejected by the GPU still fails loudly as [`RenderError::Gpu`], never
  /// a panic, because the whole setup runs inside a validation error
  /// scope.
  ///
  /// # Errors
  ///
  /// - [`RenderError::ZeroSize`] — `size` has a zero dimension; checked
  ///   before any GPU work.
  /// - [`RenderError::Context`] — no usable headless adapter/device.
  /// - [`RenderError::Gpu`] — the WGSL, texture, or pipeline failed GPU
  ///   validation ( e.g. a size beyond the device's texture limit ).
  /// - [`RenderError::Readback`] — the GPU→host copy failed.
  pub fn render( bundle : &PreviewBundle, size : ( u32, u32 ), time : f32 ) -> Result< RenderedImage, RenderError >
  {
    if size.0 == 0 || size.1 == 0
    {
      return Err( RenderError::ZeroSize );
    }

    let context = minwgpu::context::headless()
    .map_err( | err | RenderError::Context( err.to_string() ) )?;
    let device = context.device_get();
    let queue = context.queue_get();

    let error_scope = device.push_error_scope( wgpu::ErrorFilter::Validation );

    let floats = uniform_floats( bundle, size, time );
    let uniform = minwgpu::buffer::buffer( wgpu::BufferUsages::UNIFORM )
    .label( "render_params" )
    .data( &floats )
    .build( device );
    let ( layout, group ) = minwgpu::bind::single_uniform( device, &uniform, wgpu::ShaderStages::VERTEX_FRAGMENT );

    let format = wgpu::TextureFormat::Rgba8Unorm;
    let target = minwgpu::texture::render_target_2d( device, size, format );
    let pipeline = minwgpu::pipeline::fullscreen( device, &bundle.wgsl, format, &[ &layout ] );
    minwgpu::pass::draw_fullscreen( device, queue, &target.view, wgpu::Color::BLACK, &pipeline, &[ &group ] );

    if let Some( error ) = pollster::block_on( error_scope.pop() )
    {
      return Err( RenderError::Gpu( error.to_string() ) );
    }

    let ( pixels, size ) = minwgpu::readback::rgba8( device, queue, &target.texture )
    .map_err( | err | RenderError::Readback( err.to_string() ) )?;

    Ok( RenderedImage { pixels, size } )
  }
}

::mod_interface::mod_interface!
{
  own use RenderedImage;
  own use RenderError;
  own use uniform_floats;
  own use render;
}
