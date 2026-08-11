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
    webgl::to_i32,
    webgl::to_u32
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
    Native( wgpu::CommandEncoder )
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
    /// never fails this call.
    pub fn begin_render_pass
    (
      &mut self,
      color : &ColorAttachmentDesc< '_ >,
      depth : Option< &DepthAttachmentDesc< '_ > >
    ) -> Result< RenderPass, Error >
    {
      match self
      {
        #[ cfg( all( feature = "webgpu", target_arch = "wasm32" ) ) ]
        Self::WebGpu( encoder ) =>
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
        #[ cfg( all( feature = "webgl", target_arch = "wasm32" ) ) ]
        Self::WebGl( context ) =>
        {
          match color.view.expect_webgl()
          {
            TextureViewWebGl::CanvasBackbuffer =>
            {
              webgl_begin_canvas_pass( context, color.clear, depth.is_some() )
            }
            TextureViewWebGl::Texture { texture, size, .. } =>
            {
              webgl_begin_texture_pass( context, texture, *size, color.clear, depth )
            }
          }
        }
        #[ cfg( all( feature = "native", not( target_arch = "wasm32" ) ) ) ]
        Self::Native( encoder ) =>
        {
          let depth_stencil_attachment = depth.map
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
          );
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
            occlusion_query_set : None
          } );
          // Untying the pass from the encoder borrow lets the HAL hand it
          // out as a plain value, like the browser passes; wgpu then checks
          // the encode-before-finish ordering at runtime instead.
          Ok( RenderPass::Native( pass.forget_lifetime() ) )
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
        Self::Native( raw ) => Some( raw )
      }
    }
  }

  /// Records draws of one render pass of the active backend.
  ///
  /// Recording methods take `&mut self` because the native backend records
  /// into its raw pass mutably; the browser backends share the signature.
  ///
  /// The WebGL backend applies state eagerly, which imposes one ordering
  /// requirement WebGPU shares in spirit : `set_pipeline` must precede
  /// `set_bind_group` and `set_vertex_buffer`, as both resolve through the
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
    Native( wgpu::RenderPass< 'static > )
  }

  impl RenderPass
  {
    /// Sets the active render pipeline.
    pub fn set_pipeline( &mut self, pipeline : &RenderPipeline )
    {
      match self
      {
        #[ cfg( all( feature = "webgpu", target_arch = "wasm32" ) ) ]
        Self::WebGpu( pass ) =>
        {
          pass.set_pipeline( pipeline.expect_webgpu() );
        }
        #[ cfg( all( feature = "webgl", target_arch = "wasm32" ) ) ]
        Self::WebGl( pass ) =>
        {
          let raw = Rc::clone( pipeline.expect_webgl() );
          pass.gl.use_program( Some( &raw.program ) );
          // Attribute arrays enabled by a previous pipeline would otherwise
          // leak into this one ( GL state is global ), breaking attributeless
          // draws such as a fullscreen triangle. 16 is the WebGL2
          // MAX_VERTEX_ATTRIBS floor — every location a layout could enable.
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
          pass.set_current_pipeline( raw );
        }
        #[ cfg( all( feature = "native", not( target_arch = "wasm32" ) ) ) ]
        Self::Native( pass ) =>
        {
          pass.set_pipeline( pipeline.expect_native() );
        }
      }
    }

    /// Binds `group` at group `index`.
    ///
    /// On the WebGL backend the bindings apply through the active pipeline's
    /// introspected maps; entries the shader does not reference are skipped,
    /// and the call is a no-op before `set_pipeline`.
    pub fn set_bind_group( &mut self, index : u32, group : &BindGroup )
    {
      match self
      {
        #[ cfg( all( feature = "webgpu", target_arch = "wasm32" ) ) ]
        Self::WebGpu( pass ) =>
        {
          pass.set_bind_group( index, Some( group.expect_webgpu() ) );
        }
        #[ cfg( all( feature = "webgl", target_arch = "wasm32" ) ) ]
        Self::WebGl( pass ) =>
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
        #[ cfg( all( feature = "native", not( target_arch = "wasm32" ) ) ) ]
        Self::Native( pass ) =>
        {
          pass.set_bind_group( index, group.expect_native(), &[] );
        }
      }
    }

    /// Binds `buffer` at vertex buffer `slot`.
    ///
    /// On the WebGL backend the attribute pointers of the pipeline's slot
    /// layout apply immediately; the call is a no-op before `set_pipeline`.
    pub fn set_vertex_buffer( &mut self, slot : u32, buffer : &Buffer )
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
        Self::Native( pass ) =>
        {
          pass.set_vertex_buffer( slot, buffer.expect_native().slice( .. ) );
        }
      }
    }

    /// Binds `buffer` as the index buffer.
    pub fn set_index_buffer( &mut self, buffer : &Buffer, format : IndexFormat )
    {
      match self
      {
        #[ cfg( all( feature = "webgpu", target_arch = "wasm32" ) ) ]
        Self::WebGpu( pass ) =>
        {
          pass.set_index_buffer( buffer.expect_webgpu(), format.to_webgpu() );
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
          pass.set_index_buffer( buffer.expect_native().slice( .. ), format.to_wgpu() );
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
        Self::Native( pass ) => pass.draw( 0..vertex_count, 0..1 )
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
        Self::Native( pass ) => pass.draw_indexed( 0..index_count, 0, 0..1 )
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
        Self::Native( pass ) => drop( pass )
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
        Self::Native( raw ) => Some( raw )
      }
    }
  }

  /// Begins a pass on the canvas backbuffer : the default framebuffer,
  /// which accepts no depth attachment.
  #[ cfg( all( feature = "webgl", target_arch = "wasm32" ) ) ]
  fn webgl_begin_canvas_pass
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
  fn webgl_begin_texture_pass
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
    let mut clear_bits = glw::GL::COLOR_BUFFER_BIT;
    if let Some( depth ) = depth
    {
      let TextureViewWebGl::Texture { texture : depth_texture, .. } = depth.view.expect_webgl()
      else
      {
        context.bind_framebuffer( glw::GL::FRAMEBUFFER, None );
        context.delete_framebuffer( Some( &fbo ) );
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
      clear_bits |= glw::GL::DEPTH_BUFFER_BIT;
    }
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
