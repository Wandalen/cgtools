mod private
{
  #[ cfg( all( feature = "webgpu", target_arch = "wasm32" ) ) ]
  use minwebgpu as gl;
  #[ cfg( all( feature = "webgpu", target_arch = "wasm32" ) ) ]
  use gl::web_sys;
  #[ cfg( all( feature = "webgl", target_arch = "wasm32" ) ) ]
  use minwebgl as glw;
  #[ cfg( all( feature = "webgl", target_arch = "wasm32" ) ) ]
  use std::rc::Rc;
  #[ cfg( all( feature = "webgl", target_arch = "wasm32" ) ) ]
  use crate::
  {
    TextureViewWebGl,
    BindGroupEntryWebGl,
    RenderPassWebGl,
    RenderPipelineWebGl,
    webgl::to_i32,
    webgl::to_u32
  };
  #[ cfg( all( feature = "vulkan", not( target_arch = "wasm32" ) ) ) ]
  use crate::vulkan::
  {
    CommandEncoderVulkan,
    RenderPassVulkan,
    render_pass_begin as vulkan_render_pass_begin,
    pipeline_set as vulkan_pipeline_set,
    bind_group_set as vulkan_bind_group_set,
    vertex_buffer_set as vulkan_vertex_buffer_set,
    index_buffer_set as vulkan_index_buffer_set,
    draw as vulkan_draw,
    draw_indexed as vulkan_draw_indexed,
    render_pass_end as vulkan_render_pass_end
  };
  use crate::
  {
    Error,
    IndexFormat,
    TextureView,
    Buffer,
    BindGroup,
    RenderPipeline
  };

  /// Color attachment of a render pass; always cleared on load, stored on
  /// end — the v0 fixed function set.
  #[ derive( Debug ) ]
  pub struct ColorAttachmentDesc< 'a >
  {
    /// Target view.
    pub view : &'a TextureView,
    /// Clear color.
    pub clear : [ f32; 4 ]
  }

  /// Depth attachment of a render pass; always cleared to 1.0 on load,
  /// stored on end — the v0 fixed function set.
  #[ derive( Debug ) ]
  pub struct DepthAttachmentDesc< 'a >
  {
    /// Target view.
    pub view : &'a TextureView
  }

  /// Records commands of one frame of the active backend.
  #[ derive( Debug ) ]
  pub enum CommandEncoder
  {
    /// WebGPU backend command encoder.
    #[ cfg( all( feature = "webgpu", target_arch = "wasm32" ) ) ]
    WebGpu( web_sys::GpuCommandEncoder ),
    /// WebGL backend command encoder — the GL context executes eagerly, so
    /// the encoder is the context itself.
    #[ cfg( all( feature = "webgl", target_arch = "wasm32" ) ) ]
    WebGl( glw::GL ),
    /// Native backend command encoder.
    #[ cfg( all( feature = "native", not( target_arch = "wasm32" ) ) ) ]
    Native( wgpu::CommandEncoder ),
    /// Native Vulkan backend command encoder.
    #[ cfg( all( feature = "vulkan", not( target_arch = "wasm32" ) ) ) ]
    Vulkan( Box< CommandEncoderVulkan > )
  }

  impl CommandEncoder
  {
    /// Begins a render pass with one color attachment and an optional depth
    /// attachment.
    ///
    /// Takes `&mut self` because the native backend records into its raw
    /// encoder mutably; the browser backends share the signature.
    ///
    /// On the WebGL backend the canvas backbuffer accepts no depth
    /// attachment, and attachments must be texture views of matching size.
    ///
    /// # Errors
    ///
    /// Returns [`Error::WebGpu`] if the underlying WebGPU pass-creation
    /// call fails. On the WebGL backend, returns [`Error::Unsupported`] if
    /// a depth attachment is paired with the canvas backbuffer (as the
    /// color target or as the depth view itself), or [`Error::WebGl`] if
    /// the backing framebuffer fails to allocate. The native backend
    /// never fails this call. On Vulkan, returns [`Error::Vulkan`] if the
    /// underlying render pass or framebuffer creation fails.
    pub fn render_pass_begin
    (
      &mut self,
      color : &ColorAttachmentDesc< '_ >,
      depth : Option< &DepthAttachmentDesc< '_ > >
    ) -> Result< RenderPass, Error >
    {
      match self
      {
        #[ cfg( all( feature = "webgpu", target_arch = "wasm32" ) ) ]
        Self::WebGpu( encoder ) => webgpu_render_pass_begin( encoder, color, depth ),
        #[ cfg( all( feature = "webgl", target_arch = "wasm32" ) ) ]
        Self::WebGl( context ) =>
        {
          match color.view.expect_webgl()
          {
            TextureViewWebGl::CanvasBackbuffer =>
            {
              webgl_canvas_pass_begin( context, color.clear, depth.is_some() )
            }
            TextureViewWebGl::Texture { texture, size, .. } =>
            {
              webgl_texture_pass_begin( context, texture, *size, color.clear, depth )
            }
          }
        }
        #[ cfg( all( feature = "native", not( target_arch = "wasm32" ) ) ) ]
        Self::Native( encoder ) => Ok( native_render_pass_begin( encoder, color, depth ) ),
        #[ cfg( all( feature = "vulkan", not( target_arch = "wasm32" ) ) ) ]
        Self::Vulkan( encoder ) =>
        {
          Ok( RenderPass::Vulkan( Box::new( vulkan_render_pass_begin( encoder, color, depth )? ) ) )
        }
      }
    }

    /// The raw WebGPU object, when the handle belongs to the WebGPU backend.
    #[ cfg( all( feature = "webgpu", target_arch = "wasm32" ) ) ]
    #[must_use]
    pub fn as_webgpu( &self ) -> Option< &web_sys::GpuCommandEncoder >
    {
      match self
      {
        Self::WebGpu( raw ) => Some( raw ),
        #[ cfg( all( feature = "webgl", target_arch = "wasm32" ) ) ]
        Self::WebGl( _ ) => None
      }
    }

    /// The raw GL context, when the handle belongs to the WebGL backend.
    #[ cfg( all( feature = "webgl", target_arch = "wasm32" ) ) ]
    #[ must_use ]
    pub fn as_webgl( &self ) -> Option< &glw::GL >
    {
      match self
      {
        Self::WebGl( raw ) => Some( raw ),
        #[ cfg( all( feature = "webgpu", target_arch = "wasm32" ) ) ]
        Self::WebGpu( _ ) => None
      }
    }

    #[ cfg( all( feature = "webgpu", target_arch = "wasm32" ) ) ]
    pub( crate ) fn expect_webgpu( &self ) -> &web_sys::GpuCommandEncoder
    {
      match self
      {
        Self::WebGpu( raw ) => raw,
        #[ cfg( all( feature = "webgl", target_arch = "wasm32" ) ) ]
        Self::WebGl( _ ) => panic!( "backend mismatch : expected a WebGPU command encoder" )
      }
    }

    /// The raw wgpu object, when the handle belongs to the native backend.
    #[ cfg( all( feature = "native", not( target_arch = "wasm32" ) ) ) ]
    #[must_use]
    pub fn as_native( &self ) -> Option< &wgpu::CommandEncoder >
    {
      match self
      {
        Self::Native( raw ) => Some( raw ),
        #[ cfg( all( feature = "vulkan", not( target_arch = "wasm32" ) ) ) ]
        Self::Vulkan( _ ) => None
      }
    }

    /// The raw Vulkan object, when the handle belongs to the Vulkan backend.
    #[ cfg( all( feature = "vulkan", not( target_arch = "wasm32" ) ) ) ]
    #[must_use]
    pub fn as_vulkan( &self ) -> Option< &CommandEncoderVulkan >
    {
      match self
      {
        Self::Vulkan( raw ) => Some( raw ),
        #[ cfg( all( feature = "native", not( target_arch = "wasm32" ) ) ) ]
        Self::Native( _ ) => None
      }
    }
  }

  /// Records draws of one render pass of the active backend.
  ///
  /// Recording methods take `&mut self` because the native backend records
  /// into its raw pass mutably; the browser backends share the signature.
  ///
  /// The WebGL backend applies state eagerly, which imposes one ordering
  /// requirement WebGPU shares in spirit : `pipeline_set` must precede
  /// `bind_group_set` and `vertex_buffer_set`, as both resolve through the
  /// active pipeline's introspected binding maps.
  #[ derive( Debug ) ]
  pub enum RenderPass
  {
    /// WebGPU backend render pass encoder.
    #[ cfg( all( feature = "webgpu", target_arch = "wasm32" ) ) ]
    WebGpu( web_sys::GpuRenderPassEncoder ),
    /// WebGL backend render pass state.
    #[ cfg( all( feature = "webgl", target_arch = "wasm32" ) ) ]
    WebGl( RenderPassWebGl ),
    /// Native backend render pass, untied from its encoder's borrow.
    #[ cfg( all( feature = "native", not( target_arch = "wasm32" ) ) ) ]
    Native( Box< wgpu::RenderPass< 'static > > ),
    /// Native Vulkan backend render pass.
    #[ cfg( all( feature = "vulkan", not( target_arch = "wasm32" ) ) ) ]
    Vulkan( Box< RenderPassVulkan > )
  }

  impl RenderPass
  {
    /// Sets the active render pipeline.
    pub fn pipeline_set( &mut self, pipeline : &RenderPipeline )
    {
      match self
      {
        #[ cfg( all( feature = "webgpu", target_arch = "wasm32" ) ) ]
        Self::WebGpu( pass ) =>
        {
          pass.set_pipeline( pipeline.expect_webgpu() );
        }
        #[ cfg( all( feature = "webgl", target_arch = "wasm32" ) ) ]
        Self::WebGl( pass ) => webgl_pipeline_set( pass, Rc::clone( pipeline.expect_webgl() ) ),
        #[ cfg( all( feature = "native", not( target_arch = "wasm32" ) ) ) ]
        Self::Native( pass ) =>
        {
          pass.set_pipeline( pipeline.expect_native() );
        }
        #[ cfg( all( feature = "vulkan", not( target_arch = "wasm32" ) ) ) ]
        Self::Vulkan( pass ) =>
        {
          vulkan_pipeline_set( pass, pipeline.expect_vulkan() );
        }
      }
    }

    /// Binds `group` at group `index`.
    ///
    /// On the WebGL backend the bindings apply through the active pipeline's
    /// introspected maps; entries the shader does not reference are skipped,
    /// and the call is a no-op before `pipeline_set`.
    pub fn bind_group_set( &mut self, index : u32, group : &BindGroup )
    {
      match self
      {
        #[ cfg( all( feature = "webgpu", target_arch = "wasm32" ) ) ]
        Self::WebGpu( pass ) =>
        {
          pass.set_bind_group( index, Some( group.expect_webgpu() ) );
        }
        #[ cfg( all( feature = "webgl", target_arch = "wasm32" ) ) ]
        Self::WebGl( pass ) => webgl_bind_group_set( pass, index, group ),
        #[ cfg( all( feature = "native", not( target_arch = "wasm32" ) ) ) ]
        Self::Native( pass ) =>
        {
          pass.set_bind_group( index, group.expect_native(), &[] );
        }
        #[ cfg( all( feature = "vulkan", not( target_arch = "wasm32" ) ) ) ]
        Self::Vulkan( pass ) =>
        {
          vulkan_bind_group_set( pass, index, group.expect_vulkan() );
        }
      }
    }

    /// Binds `buffer` at vertex buffer `slot`.
    ///
    /// On the WebGL backend the attribute pointers of the pipeline's slot
    /// layout apply immediately; the call is a no-op before `pipeline_set`.
    pub fn vertex_buffer_set( &mut self, slot : u32, buffer : &Buffer )
    {
      match self
      {
        #[ cfg( all( feature = "webgpu", target_arch = "wasm32" ) ) ]
        Self::WebGpu( pass ) =>
        {
          pass.set_vertex_buffer( slot, Some( buffer.expect_webgpu() ) );
        }
        #[ cfg( all( feature = "webgl", target_arch = "wasm32" ) ) ]
        Self::WebGl( pass ) =>
        {
          let Some( pipeline ) = pass.current_pipeline()
          else
          {
            return;
          };
          let Some( layout ) = pipeline.vertex_buffers.get( slot as usize )
          else
          {
            return;
          };
          pass.gl.bind_buffer( glw::GL::ARRAY_BUFFER, Some( &buffer.expect_webgl().buffer ) );
          for attribute in &layout.attributes
          {
            pass.gl.enable_vertex_attrib_array( attribute.location );
            pass.gl.vertex_attrib_pointer_with_i32
            (
              attribute.location,
              attribute.format.webgl_component_count(),
              glw::GL::FLOAT,
              false,
              to_i32( layout.stride ),
              to_i32( attribute.offset )
            );
          }
        }
        #[ cfg( all( feature = "native", not( target_arch = "wasm32" ) ) ) ]
        Self::Native( pass ) => native_vertex_buffer_set( pass, slot, buffer ),
        #[ cfg( all( feature = "vulkan", not( target_arch = "wasm32" ) ) ) ]
        Self::Vulkan( pass ) =>
        {
          vulkan_vertex_buffer_set( pass, slot, buffer.expect_vulkan() );
        }
      }
    }

    /// Binds `buffer` as the index buffer.
    pub fn index_buffer_set( &mut self, buffer : &Buffer, format : IndexFormat )
    {
      match self
      {
        #[ cfg( all( feature = "webgpu", target_arch = "wasm32" ) ) ]
        Self::WebGpu( pass ) =>
        {
          pass.set_index_buffer( buffer.expect_webgpu(), gl::GpuIndexFormat::from( format ) );
        }
        #[ cfg( all( feature = "webgl", target_arch = "wasm32" ) ) ]
        Self::WebGl( pass ) =>
        {
          // v0 has a single index format; draw_indexed hardcodes the
          // matching GL element type.
          let IndexFormat::Uint32 = format;
          pass.gl.bind_buffer( glw::GL::ELEMENT_ARRAY_BUFFER, Some( &buffer.expect_webgl().buffer ) );
        }
        #[ cfg( all( feature = "native", not( target_arch = "wasm32" ) ) ) ]
        Self::Native( pass ) =>
        {
          let raw = buffer.expect_native();
          // Fix(BUG-208): see vertex_buffer_set's Native arm above -- same
          // wgpu zero-size BufferSlice panic ( size_expect_nonzero() ),
          // same root cause, same no-op-when-empty fix shape.
          if raw.size() > 0
          {
            pass.set_index_buffer( raw.slice( .. ), wgpu::IndexFormat::from( format ) );
          }
        }
        #[ cfg( all( feature = "vulkan", not( target_arch = "wasm32" ) ) ) ]
        Self::Vulkan( pass ) =>
        {
          vulkan_index_buffer_set( pass, buffer.expect_vulkan(), format );
        }
      }
    }

    /// Draws `vertex_count` vertices.
    pub fn draw( &mut self, vertex_count : u32 )
    {
      match self
      {
        #[ cfg( all( feature = "webgpu", target_arch = "wasm32" ) ) ]
        Self::WebGpu( pass ) => pass.draw( vertex_count ),
        #[ cfg( all( feature = "webgl", target_arch = "wasm32" ) ) ]
        Self::WebGl( pass ) =>
        {
          pass.gl.draw_arrays( glw::GL::TRIANGLES, 0, to_i32( vertex_count ) );
        }
        #[ cfg( all( feature = "native", not( target_arch = "wasm32" ) ) ) ]
        Self::Native( pass ) => pass.draw( 0..vertex_count, 0..1 ),
        #[ cfg( all( feature = "vulkan", not( target_arch = "wasm32" ) ) ) ]
        Self::Vulkan( pass ) => vulkan_draw( pass, vertex_count )
      }
    }

    /// Draws `index_count` indices from the bound index buffer.
    pub fn draw_indexed( &mut self, index_count : u32 )
    {
      match self
      {
        #[ cfg( all( feature = "webgpu", target_arch = "wasm32" ) ) ]
        Self::WebGpu( pass ) => pass.draw_indexed( index_count ),
        #[ cfg( all( feature = "webgl", target_arch = "wasm32" ) ) ]
        Self::WebGl( pass ) =>
        {
          pass.gl.draw_elements_with_i32
          (
            glw::GL::TRIANGLES,
            to_i32( index_count ),
            glw::GL::UNSIGNED_INT,
            0
          );
        }
        #[ cfg( all( feature = "native", not( target_arch = "wasm32" ) ) ) ]
        Self::Native( pass ) => pass.draw_indexed( 0..index_count, 0, 0..1 ),
        #[ cfg( all( feature = "vulkan", not( target_arch = "wasm32" ) ) ) ]
        Self::Vulkan( pass ) => vulkan_draw_indexed( pass, index_count )
      }
    }

    /// Ends the pass, consuming the recorder.
    pub fn end( self )
    {
      match self
      {
        #[ cfg( all( feature = "webgpu", target_arch = "wasm32" ) ) ]
        Self::WebGpu( pass ) => pass.end(),
        #[ cfg( all( feature = "webgl", target_arch = "wasm32" ) ) ]
        Self::WebGl( pass ) =>
        {
          pass.gl.bind_framebuffer( glw::GL::FRAMEBUFFER, None );
          if let Some( fbo ) = &pass.fbo
          {
            pass.gl.delete_framebuffer( Some( fbo ) );
          }
        }
        // Dropping the raw pass is wgpu's own end-of-pass signal.
        #[ cfg( all( feature = "native", not( target_arch = "wasm32" ) ) ) ]
        Self::Native( pass ) => drop( pass ),
        #[ cfg( all( feature = "vulkan", not( target_arch = "wasm32" ) ) ) ]
        // `Box<RenderPassVulkan>`'s own compiler-blessed deref-move lets `*pass`
        // hand `vulkan_render_pass_end` the owned value its by-value signature
        // needs, straight out of the box, with no extra clone.
        Self::Vulkan( pass ) => vulkan_render_pass_end( *pass )
      }
    }

    /// The raw WebGPU object, when the handle belongs to the WebGPU backend.
    #[ cfg( all( feature = "webgpu", target_arch = "wasm32" ) ) ]
    #[must_use]
    pub fn as_webgpu( &self ) -> Option< &web_sys::GpuRenderPassEncoder >
    {
      match self
      {
        Self::WebGpu( raw ) => Some( raw ),
        #[ cfg( all( feature = "webgl", target_arch = "wasm32" ) ) ]
        Self::WebGl( _ ) => None
      }
    }

    /// The WebGL backend data, when the handle belongs to the WebGL backend.
    #[ cfg( all( feature = "webgl", target_arch = "wasm32" ) ) ]
    #[ must_use ]
    pub fn as_webgl( &self ) -> Option< &RenderPassWebGl >
    {
      match self
      {
        Self::WebGl( raw ) => Some( raw ),
        #[ cfg( all( feature = "webgpu", target_arch = "wasm32" ) ) ]
        Self::WebGpu( _ ) => None
      }
    }

    /// The raw wgpu object, when the handle belongs to the native backend.
    #[ cfg( all( feature = "native", not( target_arch = "wasm32" ) ) ) ]
    #[must_use]
    pub fn as_native( &self ) -> Option< &wgpu::RenderPass< 'static > >
    {
      match self
      {
        Self::Native( raw ) => Some( raw ),
        #[ cfg( all( feature = "vulkan", not( target_arch = "wasm32" ) ) ) ]
        Self::Vulkan( _ ) => None
      }
    }

    /// The Vulkan backend data, when the handle belongs to the Vulkan backend.
    #[ cfg( all( feature = "vulkan", not( target_arch = "wasm32" ) ) ) ]
    #[ must_use ]
    pub fn as_vulkan( &self ) -> Option< &RenderPassVulkan >
    {
      match self
      {
        Self::Vulkan( raw ) => Some( raw ),
        #[ cfg( all( feature = "native", not( target_arch = "wasm32" ) ) ) ]
        Self::Native( _ ) => None
      }
    }
  }

  /// Binds `raw`'s program, resets the attribute-array/depth/cull-face GL
  /// state for it, and records it as `pass`'s active pipeline.
  #[ cfg( all( feature = "webgl", target_arch = "wasm32" ) ) ]
  fn webgl_pipeline_set( pass : &mut RenderPassWebGl, raw : Rc< RenderPipelineWebGl > )
  {
    pass.gl.use_program( Some( &raw.program ) );
    // Attribute arrays enabled by a previous pipeline would otherwise leak
    // into this one ( GL state is global ), breaking attributeless draws
    // such as a fullscreen triangle. 16 is the WebGL2 MAX_VERTEX_ATTRIBS
    // floor — every location a layout could enable.
    for location in 0u32..16
    {
      pass.gl.disable_vertex_attrib_array( location );
    }
    if raw.depth.is_some()
    {
      pass.gl.enable( glw::GL::DEPTH_TEST );
      pass.gl.depth_func( glw::GL::LESS );
      pass.gl.depth_mask( true );
    }
    else
    {
      pass.gl.disable( glw::GL::DEPTH_TEST );
    }
    if raw.cull_back
    {
      pass.gl.enable( glw::GL::CULL_FACE );
      pass.gl.cull_face( glw::GL::BACK );
    }
    else
    {
      pass.gl.disable( glw::GL::CULL_FACE );
    }
    pass.current_pipeline_set( raw );
  }

  /// Begins a pass on the canvas backbuffer : the default framebuffer,
  /// which accepts no depth attachment.
  #[ cfg( all( feature = "webgl", target_arch = "wasm32" ) ) ]
  fn webgl_canvas_pass_begin
  (
    context : &glw::GL,
    clear : [ f32; 4 ],
    has_depth : bool
  ) -> Result< RenderPass, Error >
  {
    if has_depth
    {
      return Err( Error::Unsupported
      (
        "the WebGL backend cannot attach depth to the canvas backbuffer".to_string()
      ) );
    }
    context.bind_framebuffer( glw::GL::FRAMEBUFFER, None );
    context.viewport( 0, 0, context.drawing_buffer_width(), context.drawing_buffer_height() );
    context.color_mask( true, true, true, true );
    context.clear_color( clear[ 0 ], clear[ 1 ], clear[ 2 ], clear[ 3 ] );
    context.clear( glw::GL::COLOR_BUFFER_BIT );
    Ok( RenderPass::WebGl( RenderPassWebGl::new( context.clone(), None ) ) )
  }

  /// Begins a pass on a texture view : builds a framebuffer around the
  /// attachments, owned by the pass and deleted when it ends.
  #[ cfg( all( feature = "webgl", target_arch = "wasm32" ) ) ]
  fn webgl_texture_pass_begin
  (
    context : &glw::GL,
    texture : &glw::web_sys::WebGlTexture,
    size : [ u32; 2 ],
    clear : [ f32; 4 ],
    depth : Option< &DepthAttachmentDesc< '_ > >
  ) -> Result< RenderPass, Error >
  {
    let fbo = context.create_framebuffer()
    .ok_or_else( || Error::WebGl( "failed to allocate framebuffer".to_string() ) )?;
    context.bind_framebuffer( glw::GL::FRAMEBUFFER, Some( &fbo ) );
    context.framebuffer_texture_2d
    (
      glw::GL::FRAMEBUFFER,
      glw::GL::COLOR_ATTACHMENT0,
      glw::GL::TEXTURE_2D,
      Some( texture ),
      0
    );
    let clear_bits = glw::GL::COLOR_BUFFER_BIT | webgl_texture_pass_depth_attach( context, &fbo, depth )?;
    let status = context.check_framebuffer_status( glw::GL::FRAMEBUFFER );
    if status != glw::GL::FRAMEBUFFER_COMPLETE
    {
      context.bind_framebuffer( glw::GL::FRAMEBUFFER, None );
      context.delete_framebuffer( Some( &fbo ) );
      return Err( Error::WebGl( format!( "framebuffer incomplete : status {status}" ) ) );
    }
    context.viewport( 0, 0, to_i32( size[ 0 ] ), to_i32( size[ 1 ] ) );
    context.color_mask( true, true, true, true );
    context.clear_color( clear[ 0 ], clear[ 1 ], clear[ 2 ], clear[ 3 ] );
    context.clear( clear_bits );
    Ok( RenderPass::WebGl( RenderPassWebGl::new( context.clone(), Some( fbo ) ) ) )
  }

  /// Attaches `depth`'s texture to `fbo`'s depth slot, if present, and
  /// returns the additional clear-bit mask to OR into the pass's clear
  /// call. On failure, unbinds and deletes `fbo` before returning the
  /// error — the caller has no other cleanup path for it at that point.
  #[ cfg( all( feature = "webgl", target_arch = "wasm32" ) ) ]
  fn webgl_texture_pass_depth_attach
  (
    context : &glw::GL,
    fbo : &glw::web_sys::WebGlFramebuffer,
    depth : Option< &DepthAttachmentDesc< '_ > >
  ) -> Result< u32, Error >
  {
    let Some( depth ) = depth
    else
    {
      return Ok( 0 );
    };
    let TextureViewWebGl::Texture { texture : depth_texture, .. } = depth.view.expect_webgl()
    else
    {
      context.bind_framebuffer( glw::GL::FRAMEBUFFER, None );
      context.delete_framebuffer( Some( fbo ) );
      return Err( Error::Unsupported
      (
        "the canvas backbuffer cannot serve as a depth attachment".to_string()
      ) );
    };
    context.framebuffer_texture_2d
    (
      glw::GL::FRAMEBUFFER,
      glw::GL::DEPTH_ATTACHMENT,
      glw::GL::TEXTURE_2D,
      Some( depth_texture ),
      0
    );
    context.depth_mask( true );
    context.clear_depth( 1.0 );
    Ok( glw::GL::DEPTH_BUFFER_BIT )
  }

  /// Begins a WebGPU render pass with `color`'s view attached, and
  /// `depth`'s view attached if present.
  #[ cfg( all( feature = "webgpu", target_arch = "wasm32" ) ) ]
  fn webgpu_render_pass_begin
  (
    encoder : &mut web_sys::GpuCommandEncoder,
    color : &ColorAttachmentDesc< '_ >,
    depth : Option< &DepthAttachmentDesc< '_ > >
  ) -> Result< RenderPass, Error >
  {
    let mut desc = gl::render_pass::desc()
    .color_attachment
    (
      gl::ColorAttachment::new( color.view.expect_webgpu() ).clear_value( color.clear )
    );
    if let Some( depth ) = depth
    {
      desc = desc.depth_stencil_attachment
      (
        gl::DepthStencilAttachment::new( depth.view.expect_webgpu() )
      );
    }
    let pass = encoder.begin_render_pass( &desc.into() )
    .map_err( | e | Error::WebGpu( format!( "failed to begin render pass : {e:?}" ) ) )?;
    Ok( RenderPass::WebGpu( pass ) )
  }

  /// Builds `depth`'s wgpu depth-stencil attachment description, if
  /// present.
  #[ cfg( all( feature = "native", not( target_arch = "wasm32" ) ) ) ]
  fn native_depth_stencil_attachment_build< 'a >
  (
    depth : Option< &DepthAttachmentDesc< 'a > >
  ) -> Option< wgpu::RenderPassDepthStencilAttachment< 'a > >
  {
    depth.map
    (
      | depth |
      wgpu::RenderPassDepthStencilAttachment
      {
        view : depth.view.expect_native(),
        depth_ops : Some( wgpu::Operations
        {
          load : wgpu::LoadOp::Clear( 1.0 ),
          store : wgpu::StoreOp::Store
        } ),
        stencil_ops : None
      }
    )
  }

  /// Begins a native render pass with `color`'s view attached, and
  /// `depth`'s view attached if present.
  #[ cfg( all( feature = "native", not( target_arch = "wasm32" ) ) ) ]
  fn native_render_pass_begin
  (
    encoder : &mut wgpu::CommandEncoder,
    color : &ColorAttachmentDesc< '_ >,
    depth : Option< &DepthAttachmentDesc< '_ > >
  ) -> RenderPass
  {
    let depth_stencil_attachment = native_depth_stencil_attachment_build( depth );
    let pass = encoder.begin_render_pass( &wgpu::RenderPassDescriptor
    {
      label : None,
      color_attachments : &[ Some( wgpu::RenderPassColorAttachment
      {
        view : color.view.expect_native(),
        depth_slice : None,
        resolve_target : None,
        ops : wgpu::Operations
        {
          load : wgpu::LoadOp::Clear( wgpu::Color
          {
            r : f64::from( color.clear[ 0 ] ),
            g : f64::from( color.clear[ 1 ] ),
            b : f64::from( color.clear[ 2 ] ),
            a : f64::from( color.clear[ 3 ] )
          } ),
          store : wgpu::StoreOp::Store
        }
      } ) ],
      depth_stencil_attachment,
      timestamp_writes : None,
      occlusion_query_set : None,
      multiview_mask : None
    } );
    // Untying the pass from the encoder borrow lets the HAL hand it
    // out as a plain value, like the browser passes; wgpu then checks
    // the encode-before-finish ordering at runtime instead.
    RenderPass::Native( Box::new( pass.forget_lifetime() ) )
  }

  /// Binds `buffer` at vertex buffer `slot` on the native backend, skipping
  /// the bind for a zero-size buffer.
  #[ cfg( all( feature = "native", not( target_arch = "wasm32" ) ) ) ]
  fn native_vertex_buffer_set( pass : &mut wgpu::RenderPass< 'static >, slot : u32, buffer : &Buffer )
  {
    let raw = buffer.expect_native();
    // Fix(BUG-208): wgpu::RenderPass::set_vertex_buffer panics via
    // BufferSlice::size_expect_nonzero()'s
    // .expect("buffer slice can not be empty") whenever the bound
    // buffer's own allocated size is 0 -- reachable with ordinary
    // input (e.g. an all-empty Geometry, traced end-to-end from
    // renderer::webgpu::Geometry::new through renderer.rs's per-slot
    // vertex_buffer_set loop, which applies no vertex_count>0 guard).
    // Root cause: no size check existed between the caller's buffer
    // and wgpu's own panicking slice() call.
    // A zero-size buffer has nothing to read regardless of whether
    // it's bound, so skipping the native bind call is a safe no-op --
    // mirroring this same function's WebGL arm above, which already
    // no-ops when there's nothing meaningful to bind yet.
    if raw.size() > 0
    {
      pass.set_vertex_buffer( slot, raw.slice( .. ) );
    }
  }

  /// Binds `group`'s entries into `pass`'s active pipeline's introspected
  /// uniform-block/texture-unit maps; a no-op before `pipeline_set`.
  #[ cfg( all( feature = "webgl", target_arch = "wasm32" ) ) ]
  fn webgl_bind_group_set( pass : &mut RenderPassWebGl, index : u32, group : &BindGroup )
  {
    let Some( pipeline ) = pass.current_pipeline()
    else
    {
      return;
    };
    let mut last_unit = None;
    for ( binding_index, entry ) in group.expect_webgl().entries.iter().enumerate()
    {
      let key = ( index, to_u32( binding_index ) );
      match entry
      {
        BindGroupEntryWebGl::Buffer( buffer ) =>
        {
          if let Some( ( _, point ) ) = pipeline.ubo_points.iter().find( | ( k, _ ) | *k == key )
          {
            pass.gl.bind_buffer_base( glw::GL::UNIFORM_BUFFER, *point, Some( buffer ) );
          }
        }
        BindGroupEntryWebGl::Texture( texture ) =>
        {
          if let Some( ( _, unit ) ) = pipeline.texture_units.iter().find( | ( k, _ ) | *k == key )
          {
            pass.gl.active_texture( glw::GL::TEXTURE0 + *unit );
            pass.gl.bind_texture( glw::GL::TEXTURE_2D, Some( texture ) );
            last_unit = Some( *unit );
          }
          else
          {
            last_unit = None;
          }
        }
        BindGroupEntryWebGl::Sampler( sampler ) =>
        {
          // A sampler pairs with the nearest preceding texture entry.
          if let Some( unit ) = last_unit
          {
            pass.gl.bind_sampler( unit, Some( sampler ) );
          }
        }
      }
    }
  }
}

crate::mod_interface!
{
  orphan use
  {
    ColorAttachmentDesc,
    DepthAttachmentDesc,
    CommandEncoder,
    RenderPass
  };
}
