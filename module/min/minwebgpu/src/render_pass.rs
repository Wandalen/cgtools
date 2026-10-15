/// Internal namespace.
mod private
{
  use crate::{ RenderPassDescriptor, web_sys, WebGPUError, Into, ColorAttachment, RenderPassError };

  /// Returns a new `RenderPassDescriptor` with default settings.
  #[ inline ]
  #[ must_use ]
  pub fn desc< 'a >() -> RenderPassDescriptor< 'a >
  {
    RenderPassDescriptor::new()
  }

  /// Runs a single render pass with one color attachment over `view`
  /// ( defaults from [`ColorAttachment`]: clear to `[ 0.0, 0.0, 0.0, 0.0 ]`,
  /// store ), then ends the pass and submits the encoder — the common
  /// "clear, draw, submit" ceremony collapsed into one call.
  ///
  /// `draw` receives the raw `web_sys::GpuRenderPassEncoder`: set the
  /// pipeline, bind groups, vertex/index buffers and issue draw calls on it
  /// exactly as with the manual encoder/pass/submit sequence — nothing about
  /// draw commands is hidden or defaulted.
  ///
  /// For multiple color attachments, a depth-stencil attachment, or several
  /// passes per submit, build the `RenderPassDescriptor` and command encoder
  /// by hand instead.
  ///
  /// # Errors
  /// Returns `error::RenderPassError::FailedToBegin` if the underlying
  /// `GPUCommandEncoder.beginRenderPass` call throws.
  #[ inline ]
  pub fn draw_to
  (
    device : &web_sys::GpuDevice,
    queue : &web_sys::GpuQueue,
    view : &web_sys::GpuTextureView,
    draw : impl FnOnce( &web_sys::GpuRenderPassEncoder )
  ) -> Result< (), WebGPUError >
  {
    let encoder = device.create_command_encoder();
    let pass = encoder.begin_render_pass( &desc().color_attachment( ColorAttachment::new( view ) ).into() )
    .map_err( | e | RenderPassError::FailedToBegin( format!( "{e:?}" ) ) )?;

    draw( &pass );
    pass.end();
    queue.submit( &[ encoder.finish() ] );

    Ok( () )
  }
}

crate::mod_interface!
{
  /// Color attachment related
  layer color_attachment;
  /// Depth stenctil attachment related
  layer depth_stencil_attachment;

  own use
  {
    desc,
    draw_to
  };
}
