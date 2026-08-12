//! This module provides single-shot render pass helpers that encode, run, and
//! submit a complete pass in one call, for offscreen single-frame rendering.

use mingl::mod_interface;

mod private
{
  /// Encodes and submits one render pass that clears `view` to `clear` and draws the
  /// bufferless triangle ( `draw( 0..3, 0..1 )` ) with `pipeline`.
  ///
  /// Bind groups are assigned to consecutive slots starting at 0 in the order given.
  /// The pass targets `view` as its single color attachment and uses no depth. The
  /// commands are submitted immediately on `queue`.
  pub fn draw_fullscreen
  (
    device : &wgpu::Device,
    queue : &wgpu::Queue,
    view : &wgpu::TextureView,
    clear : wgpu::Color,
    pipeline : &wgpu::RenderPipeline,
    groups : &[ &wgpu::BindGroup ]
  )
  {
    let mut encoder = device.create_command_encoder
    (
      &wgpu::CommandEncoderDescriptor { label : Some( "draw_fullscreen_encoder" ) }
    );

    {
      let mut render_pass = encoder.begin_render_pass
      (
        &wgpu::RenderPassDescriptor
        {
          label : Some( "draw_fullscreen_pass" ),
          color_attachments :
          &[
            Some
            (
              wgpu::RenderPassColorAttachment
              {
                view,
                resolve_target : None,
                ops : wgpu::Operations
                {
                  load : wgpu::LoadOp::Clear( clear ),
                  store : wgpu::StoreOp::Store,
                },
                depth_slice : None,
              }
            )
          ],
          depth_stencil_attachment : None,
          timestamp_writes : None,
          occlusion_query_set : None,
          multiview_mask : None,
        }
      );
      render_pass.set_pipeline( pipeline );
      for ( index, group ) in groups.iter().enumerate()
      {
        render_pass.set_bind_group( index as u32, *group, &[] );
      }
      render_pass.draw( 0..3, 0..1 );
    }

    queue.submit( Some( encoder.finish() ) );
  }
}

mod_interface!
{
  own use draw_fullscreen;
}
