//! This module provides single-shot render pass helpers that encode, run, and
//! submit a complete pass in one call — a general form covering vertex buffers and
//! instancing, plus the bufferless-triangle shortcut used by offscreen single-frame
//! rendering.

use mingl::mod_interface;

mod private
{
  /// What one [`render`] call draws.
  ///
  /// Built fluently rather than as a struct literal, matching this crate's other builders
  /// and leaving room to grow without breaking callers. Defaults draw nothing
  /// ( `vertices` is empty ) — set at least [`Draw::vertices`].
  #[ derive( Debug, Clone ) ]
  pub struct Draw< 'a >
  {
    pub( super ) pipeline : &'a wgpu::RenderPipeline,
    pub( super ) groups : &'a [ &'a wgpu::BindGroup ],
    pub( super ) vertex_buffers : &'a [ &'a wgpu::Buffer ],
    pub( super ) vertices : core::ops::Range< u32 >,
    pub( super ) instances : core::ops::Range< u32 >,
  }

  impl< 'a > Draw< 'a >
  {
    /// Creates a draw description for `pipeline`, drawing no vertices and one instance.
    #[ inline ]
    #[ must_use ]
    pub fn new( pipeline : &'a wgpu::RenderPipeline ) -> Self
    {
      Self
      {
        pipeline,
        groups : &[],
        vertex_buffers : &[],
        vertices : 0..0,
        instances : 0..1,
      }
    }

    /// Sets the bind groups, assigned to consecutive slots starting at 0 in the order given.
    #[ inline ]
    #[ must_use ]
    pub fn groups( mut self, value : &'a [ &'a wgpu::BindGroup ] ) -> Self
    {
      self.groups = value;
      self
    }

    /// Sets the vertex buffers, assigned to consecutive slots starting at 0 in the order
    /// given — matching the order the pipeline's own vertex buffer layouts were declared in.
    #[ inline ]
    #[ must_use ]
    pub fn vertex_buffers( mut self, value : &'a [ &'a wgpu::Buffer ] ) -> Self
    {
      self.vertex_buffers = value;
      self
    }

    /// Sets the range of vertices to draw.
    #[ inline ]
    #[ must_use ]
    pub fn vertices( mut self, value : core::ops::Range< u32 > ) -> Self
    {
      self.vertices = value;
      self
    }

    /// Sets the range of instances to draw ( default `0..1` ).
    ///
    /// Drawing more than one instance requires the pipeline to declare a vertex buffer
    /// layout with [`wgpu::VertexStepMode::Instance`]; see
    /// `crate::pipeline::RenderPipelineBuilder::buffer_layout`.
    #[ inline ]
    #[ must_use ]
    pub fn instances( mut self, value : core::ops::Range< u32 > ) -> Self
    {
      self.instances = value;
      self
    }

    /// Returns the pipeline this draw uses.
    #[ inline ]
    #[ must_use ]
    pub fn pipeline_get( &self ) -> &'a wgpu::RenderPipeline
    {
      self.pipeline
    }

    /// Returns the configured bind groups.
    #[ inline ]
    #[ must_use ]
    pub fn groups_get( &self ) -> &'a [ &'a wgpu::BindGroup ]
    {
      self.groups
    }

    /// Returns the configured vertex buffers.
    #[ inline ]
    #[ must_use ]
    pub fn vertex_buffers_get( &self ) -> &'a [ &'a wgpu::Buffer ]
    {
      self.vertex_buffers
    }

    /// Returns the configured vertex range.
    #[ inline ]
    #[ must_use ]
    pub fn vertices_get( &self ) -> core::ops::Range< u32 >
    {
      self.vertices.clone()
    }

    /// Returns the configured instance range.
    #[ inline ]
    #[ must_use ]
    pub fn instances_get( &self ) -> core::ops::Range< u32 >
    {
      self.instances.clone()
    }
  }

  /// Encodes and submits one render pass that clears `view` to `clear` and runs `draw`.
  ///
  /// The pass targets `view` as its single color attachment and uses no depth. Commands are
  /// submitted immediately on `queue`. For a windowed frame, `view` is the one carried by
  /// `crate::surface::Frame::Ready` — present it afterwards with
  /// `crate::surface::frame_present`.
  pub fn render
  (
    device : &wgpu::Device,
    queue : &wgpu::Queue,
    view : &wgpu::TextureView,
    clear : wgpu::Color,
    draw : &Draw< '_ >,
  )
  {
    let mut encoder = device.create_command_encoder
    (
      &wgpu::CommandEncoderDescriptor { label : Some( "render_encoder" ) }
    );

    {
      let mut render_pass = encoder.begin_render_pass
      (
        &wgpu::RenderPassDescriptor
        {
          label : Some( "render_pass" ),
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

      render_pass.set_pipeline( draw.pipeline );
      for ( index, group ) in draw.groups.iter().enumerate()
      {
        render_pass.set_bind_group( index as u32, *group, &[] );
      }
      for ( index, buffer ) in draw.vertex_buffers.iter().enumerate()
      {
        render_pass.set_vertex_buffer( index as u32, buffer.slice( .. ) );
      }
      render_pass.draw( draw.vertices.clone(), draw.instances.clone() );
    }

    queue.submit( Some( encoder.finish() ) );
  }

  /// Encodes and submits one render pass that clears `view` to `clear` and draws the
  /// bufferless triangle ( `draw( 0..3, 0..1 )` ) with `pipeline`.
  ///
  /// A named shortcut over [`render`] for the pipeline produced by
  /// `crate::pipeline::fullscreen`. Bind groups are assigned to consecutive slots starting
  /// at 0 in the order given. The pass targets `view` as its single color attachment and
  /// uses no depth. The commands are submitted immediately on `queue`.
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
    render( device, queue, view, clear, &Draw::new( pipeline ).groups( groups ).vertices( 0..3 ) );
  }
}

mod_interface!
{
  own use Draw;
  own use render;
  own use draw_fullscreen;
}
