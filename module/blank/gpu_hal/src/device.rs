mod private
{
  #[ cfg( feature = "webgpu" ) ]
  use minwebgpu as gl;
  #[ cfg( feature = "webgpu" ) ]
  use gl::web_sys;
  #[ cfg( feature = "webgl" ) ]
  use minwebgl as glw;
  #[ cfg( all( feature = "webgl", not( feature = "webgpu" ) ) ) ]
  use glw::web_sys;
  #[ cfg( feature = "webgl" ) ]
  use std::rc::Rc;
  #[ cfg( feature = "webgl" ) ]
  use crate::
  {
    BufferWebGl,
    TextureWebGl,
    TextureViewWebGl,
    ShaderModuleWebGl,
    BindGroupLayoutWebGl,
    BindGroupWebGl,
    BindGroupEntryWebGl,
    RenderPipelineWebGl,
    BindingMap,
    webgl::to_i32,
    webgl::to_u32
  };
  #[ cfg( feature = "webgpu" ) ]
  use crate::
  {
    TextureUsage,
    ShaderStages
  };
  use crate::
  {
    Error,
    BufferUsage,
    TextureFormat,
    TextureDesc,
    SamplerDesc,
    FilterMode,
    AddressMode,
    ShaderSource,
    BindingType,
    BindGroupLayoutEntry,
    DepthRange,
    Buffer,
    Texture,
    Sampler,
    ShaderModule,
    BindGroupLayout,
    BindGroup,
    RenderPipeline,
    BindingResource,
    RenderPipelineDesc,
    TextureView,
    CommandEncoder
  };

  /// A logical GPU device of the active backend; creates every resource.
  #[ derive( Debug ) ]
  pub enum Device
  {
    /// WebGPU backend device.
    #[ cfg( feature = "webgpu" ) ]
    WebGpu( web_sys::GpuDevice ),
    /// WebGL backend device — the GL context itself.
    #[ cfg( feature = "webgl" ) ]
    WebGl( glw::GL )
  }

  /// The command queue of a device.
  #[ derive( Debug ) ]
  pub enum Queue
  {
    /// WebGPU backend queue.
    #[ cfg( feature = "webgpu" ) ]
    WebGpu( web_sys::GpuQueue ),
    /// WebGL backend queue — the GL context executes commands eagerly.
    #[ cfg( feature = "webgl" ) ]
    WebGl( glw::GL )
  }

  /// The canvas presentation surface of a device.
  #[ derive( Debug ) ]
  pub enum Surface
  {
    /// WebGPU backend surface: a configured canvas context.
    #[ cfg( feature = "webgpu" ) ]
    WebGpu
    {
      /// Configured canvas presentation context.
      context : gl::GL,
      /// Format the canvas is configured with.
      format : TextureFormat
    },
    /// WebGL backend surface — the canvas backbuffer of the GL context.
    #[ cfg( feature = "webgl" ) ]
    WebGl
    {
      /// The GL context whose canvas the surface presents to.
      context : glw::GL
    }
  }

  impl Device
  {
    /// Requests a WebGPU adapter and device, then configures `canvas` for
    /// presentation in the browser's preferred canvas format.
    #[ cfg( feature = "webgpu" ) ]
    pub async fn new_webgpu
    (
      canvas : &web_sys::HtmlCanvasElement
    ) -> Result< ( Device, Queue, Surface ), Error >
    {
      let context = gl::context::from_canvas( canvas ).map_err( gl::WebGPUError::from )?;
      let adapter = gl::context::request_adapter().await;
      let device = gl::context::request_device( &adapter ).await;
      let queue = device.queue();
      let raw_format = gl::context::preferred_format();
      gl::context::configure( &device, &context, raw_format )?;
      let format = TextureFormat::from_webgpu( raw_format )?;

      Ok
      ((
        Device::WebGpu( device ),
        Queue::WebGpu( queue ),
        Surface::WebGpu { context, format }
      ))
    }

    /// Creates a WebGL2 context on `canvas`.
    ///
    /// Requires `EXT_color_buffer_float`, so float color targets are
    /// renderable on this backend too.
    #[ cfg( feature = "webgl" ) ]
    pub fn new_webgl
    (
      canvas : &web_sys::HtmlCanvasElement
    ) -> Result< ( Device, Queue, Surface ), Error >
    {
      let context = glw::context::from_canvas( canvas ).map_err( glw::WebglError::from )?;
      let extension = context.get_extension( "EXT_color_buffer_float" )
      .map_err( | e | Error::WebGl( format!( "failed to query EXT_color_buffer_float : {e:?}" ) ) )?;
      if extension.is_none()
      {
        return Err( Error::Unsupported( "EXT_color_buffer_float is unavailable".to_string() ) );
      }

      Ok
      ((
        Device::WebGl( context.clone() ),
        Queue::WebGl( context.clone() ),
        Surface::WebGl { context }
      ))
    }

    /// Clip-space depth range the backend's projection matrices must target.
    pub fn depth_range( &self ) -> DepthRange
    {
      match self
      {
        #[ cfg( feature = "webgpu" ) ]
        Self::WebGpu( _ ) => DepthRange::ZeroToOne,
        #[ cfg( feature = "webgl" ) ]
        Self::WebGl( _ ) => DepthRange::NegOneToOne
      }
    }

    /// Creates an uninitialized buffer of `size` bytes.
    pub fn create_buffer( &self, size : u64, usage : BufferUsage ) -> Result< Buffer, Error >
    {
      // Browser buffer allocations sit far below f64's exact integer
      // range, so the cast is lossless in practice.
      #[ allow( clippy::cast_precision_loss ) ]
      let size_f64 = size as f64;
      match self
      {
        #[ cfg( feature = "webgpu" ) ]
        Self::WebGpu( device ) =>
        {
          let raw = gl::BufferDescriptor::new( usage.bits() )
          .size_from_value( size_f64 )
          .create( device )?;
          Ok( Buffer::WebGpu( raw ) )
        }
        #[ cfg( feature = "webgl" ) ]
        Self::WebGl( context ) =>
        {
          let target = webgl_buffer_target( usage );
          let buffer = context.create_buffer()
          .ok_or_else( || Error::WebGl( "failed to allocate buffer".to_string() ) )?;
          context.bind_buffer( target, Some( &buffer ) );
          context.buffer_data_with_f64( target, size_f64, webgl_buffer_hint( usage ) );
          Ok( Buffer::WebGl( BufferWebGl { buffer, target } ) )
        }
      }
    }

    /// Creates a buffer initialized with `data`.
    pub fn create_buffer_init( &self, data : &[ u8 ], usage : BufferUsage ) -> Result< Buffer, Error >
    {
      match self
      {
        #[ cfg( feature = "webgpu" ) ]
        Self::WebGpu( device ) =>
        {
          // v0 tradeoff : the init descriptor needs a sized value, so the
          // byte slice is copied once on upload.
          let data = data.to_vec();
          let raw = gl::BufferInitDescriptor::new( &data, usage.bits() ).create( device )?;
          Ok( Buffer::WebGpu( raw ) )
        }
        #[ cfg( feature = "webgl" ) ]
        Self::WebGl( context ) =>
        {
          let target = webgl_buffer_target( usage );
          let buffer = context.create_buffer()
          .ok_or_else( || Error::WebGl( "failed to allocate buffer".to_string() ) )?;
          context.bind_buffer( target, Some( &buffer ) );
          context.buffer_data_with_u8_array( target, data, webgl_buffer_hint( usage ) );
          Ok( Buffer::WebGl( BufferWebGl { buffer, target } ) )
        }
      }
    }

    /// Creates a 2d texture ( one mip, one sample ).
    pub fn create_texture( &self, desc : &TextureDesc ) -> Result< Texture, Error >
    {
      match self
      {
        #[ cfg( feature = "webgpu" ) ]
        Self::WebGpu( device ) =>
        {
          let mut builder = gl::texture::desc()
          .size( desc.size )
          .format( desc.format.to_webgpu() );
          if desc.usage.contains( TextureUsage::COPY_DST )
          {
            builder = builder.copy_dst();
          }
          if desc.usage.contains( TextureUsage::TEXTURE_BINDING )
          {
            builder = builder.texture_binding();
          }
          if desc.usage.contains( TextureUsage::RENDER_ATTACHMENT )
          {
            builder = builder.render_attachment();
          }
          Ok( Texture::WebGpu( builder.create( device )? ) )
        }
        #[ cfg( feature = "webgl" ) ]
        Self::WebGl( context ) =>
        {
          let internal_format = desc.format.webgl_internal_format()?;
          let texture = context.create_texture()
          .ok_or_else( || Error::WebGl( "failed to allocate texture".to_string() ) )?;
          context.bind_texture( glw::GL::TEXTURE_2D, Some( &texture ) );
          context.tex_storage_2d
          (
            glw::GL::TEXTURE_2D,
            1,
            internal_format,
            to_i32( desc.size[ 0 ] ),
            to_i32( desc.size[ 1 ] )
          );
          // Sampler-less binds ( e.g. texelFetch passes ) must not depend on
          // the mipmap-filtering defaults of a single-level texture.
          context.tex_parameteri( glw::GL::TEXTURE_2D, glw::GL::TEXTURE_MIN_FILTER, to_i32( glw::GL::NEAREST ) );
          context.tex_parameteri( glw::GL::TEXTURE_2D, glw::GL::TEXTURE_MAG_FILTER, to_i32( glw::GL::NEAREST ) );
          Ok( Texture::WebGl( TextureWebGl
          {
            texture,
            size : desc.size,
            format : desc.format
          } ) )
        }
      }
    }

    /// Creates a sampler.
    // A single-backend build can make the surviving arm infallible; the
    // other backend's arm fails for real, so the signature stays fallible.
    #[ allow( clippy::unnecessary_wraps ) ]
    pub fn create_sampler( &self, desc : SamplerDesc ) -> Result< Sampler, Error >
    {
      match self
      {
        #[ cfg( feature = "webgpu" ) ]
        Self::WebGpu( device ) =>
        {
          let mut builder = gl::sampler::desc();
          if desc.filter == FilterMode::Linear
          {
            builder = builder.linear();
          }
          if desc.address == AddressMode::Repeat
          {
            builder = builder.repeat();
          }
          Ok( Sampler::WebGpu( gl::sampler::create( device, builder ) ) )
        }
        #[ cfg( feature = "webgl" ) ]
        Self::WebGl( context ) =>
        {
          let sampler = context.create_sampler()
          .ok_or_else( || Error::WebGl( "failed to allocate sampler".to_string() ) )?;
          let filter = match desc.filter
          {
            FilterMode::Nearest => glw::GL::NEAREST,
            FilterMode::Linear => glw::GL::LINEAR
          };
          let address = match desc.address
          {
            AddressMode::ClampToEdge => glw::GL::CLAMP_TO_EDGE,
            AddressMode::Repeat => glw::GL::REPEAT
          };
          context.sampler_parameteri( &sampler, glw::GL::TEXTURE_MIN_FILTER, to_i32( filter ) );
          context.sampler_parameteri( &sampler, glw::GL::TEXTURE_MAG_FILTER, to_i32( filter ) );
          context.sampler_parameteri( &sampler, glw::GL::TEXTURE_WRAP_S, to_i32( address ) );
          context.sampler_parameteri( &sampler, glw::GL::TEXTURE_WRAP_T, to_i32( address ) );
          Ok( Sampler::WebGl( sampler ) )
        }
      }
    }

    /// Compiles a shader module from `source`. The WebGPU backend consumes
    /// the canonical WGSL and ignores the GLSL override slots; the WebGL
    /// backend requires both GLSL overrides and defers compilation to
    /// pipeline creation, where GL links per program.
    // Infallibility of the webgpu-only build is incidental : the WebGL arm
    // fails for real, so the signature stays fallible.
    #[ allow( clippy::unnecessary_wraps ) ]
    pub fn create_shader_module( &self, source : &ShaderSource< '_ > ) -> Result< ShaderModule, Error >
    {
      match self
      {
        #[ cfg( feature = "webgpu" ) ]
        Self::WebGpu( device ) =>
        {
          Ok( ShaderModule::WebGpu( gl::ShaderModule::new( source.wgsl ).create( device ) ) )
        }
        #[ cfg( feature = "webgl" ) ]
        Self::WebGl( _ ) =>
        {
          let ( Some( vertex ), Some( fragment ) ) = ( source.glsl_vertex, source.glsl_fragment )
          else
          {
            return Err( Error::Unsupported
            (
              "the WebGL backend requires both GLSL override slots of ShaderSource".to_string()
            ) );
          };
          Ok( ShaderModule::WebGl( ShaderModuleWebGl
          {
            vertex : vertex.to_string(),
            fragment : fragment.to_string()
          } ) )
        }
      }
    }

    /// Creates a bind group layout; binding indices follow entry order.
    // A single-backend build can make the surviving arm infallible; the
    // other backend's arm fails for real, so the signature stays fallible.
    #[ allow( clippy::unnecessary_wraps ) ]
    pub fn create_bind_group_layout
    (
      &self,
      entries : &[ BindGroupLayoutEntry ]
    ) -> Result< BindGroupLayout, Error >
    {
      match self
      {
        #[ cfg( feature = "webgpu" ) ]
        Self::WebGpu( device ) =>
        {
          let mut builder = gl::BindGroupLayoutDescriptor::new().auto_bindings();
          for entry in entries
          {
            let mut raw_entry = gl::BindGroupLayoutEntry::new();
            if entry.visibility.contains( ShaderStages::VERTEX )
            {
              raw_entry = raw_entry.vertex();
            }
            if entry.visibility.contains( ShaderStages::FRAGMENT )
            {
              raw_entry = raw_entry.fragment();
            }
            let raw_entry = match entry.ty
            {
              BindingType::UniformBuffer => raw_entry.ty( gl::binding_type::buffer_type() ),
              BindingType::Texture => raw_entry.ty( gl::binding_type::texture_type() ),
              BindingType::Sampler => raw_entry.ty( gl::binding_type::sampler_type() )
            };
            builder = builder.entry( raw_entry );
          }
          Ok( BindGroupLayout::WebGpu( builder.create( device )? ) )
        }
        #[ cfg( feature = "webgl" ) ]
        Self::WebGl( _ ) =>
        {
          // GL has no layout objects — the entry list is the layout, consumed
          // by pipeline creation for binding introspection.
          Ok( BindGroupLayout::WebGl( BindGroupLayoutWebGl { entries : entries.to_vec() } ) )
        }
      }
    }

    /// Creates a bind group; `resources` follow the layout's entry order.
    // A single-backend build can make the surviving arm infallible; the
    // other backend's arm fails for real, so the signature stays fallible.
    #[ allow( clippy::unnecessary_wraps ) ]
    pub fn create_bind_group
    (
      &self,
      layout : &BindGroupLayout,
      resources : &[ BindingResource< '_ > ]
    ) -> Result< BindGroup, Error >
    {
      match self
      {
        #[ cfg( feature = "webgpu" ) ]
        Self::WebGpu( device ) =>
        {
          let raw_layout = layout.expect_webgpu();
          let mut builder = gl::BindGroupDescriptor::new( raw_layout ).auto_bindings();
          for resource in resources
          {
            builder = match resource
            {
              BindingResource::Buffer( buffer ) =>
              {
                builder.entry_from_resource( &gl::BufferBinding::new( buffer.expect_webgpu() ) )
              }
              BindingResource::TextureView( view ) =>
              {
                builder.entry_from_resource( view.expect_webgpu() )
              }
              BindingResource::Sampler( sampler ) =>
              {
                builder.entry_from_resource( sampler.expect_webgpu() )
              }
            };
          }
          Ok( BindGroup::WebGpu( builder.create( device ) ) )
        }
        #[ cfg( feature = "webgl" ) ]
        Self::WebGl( _ ) =>
        {
          let _ = layout;
          let mut entries = Vec::with_capacity( resources.len() );
          for resource in resources
          {
            match resource
            {
              BindingResource::Buffer( buffer ) =>
              {
                entries.push( BindGroupEntryWebGl::Buffer( buffer.expect_webgl().buffer.clone() ) );
              }
              BindingResource::TextureView( view ) =>
              {
                match view.expect_webgl()
                {
                  TextureViewWebGl::Texture { texture, .. } =>
                  {
                    entries.push( BindGroupEntryWebGl::Texture( texture.clone() ) );
                  }
                  TextureViewWebGl::CanvasBackbuffer =>
                  {
                    return Err( Error::Unsupported
                    (
                      "the canvas backbuffer cannot be sampled on the WebGL backend".to_string()
                    ) );
                  }
                }
              }
              BindingResource::Sampler( sampler ) =>
              {
                entries.push( BindGroupEntryWebGl::Sampler( sampler.expect_webgl().clone() ) );
              }
            }
          }
          Ok( BindGroup::WebGl( BindGroupWebGl { entries } ) )
        }
      }
    }

    /// Creates a render pipeline.
    pub fn create_render_pipeline( &self, desc : &RenderPipelineDesc< '_ > ) -> Result< RenderPipeline, Error >
    {
      match self
      {
        #[ cfg( feature = "webgpu" ) ]
        Self::WebGpu( device ) =>
        {
          let shader = desc.shader.expect_webgpu();

          let mut layout_builder = gl::layout::pipeline::desc();
          for layout in desc.bind_group_layouts
          {
            layout_builder = layout_builder.bind_group( layout.expect_webgpu() );
          }
          let pipeline_layout = layout_builder.create( device );

          let mut vertex_state = gl::VertexState::new( shader ).entry_point( desc.vertex_entry );
          for layout in desc.vertex_buffers
          {
            let mut raw_layout = gl::VertexBufferLayout::new()
            .stride_from_value( f64::from( layout.stride ) );
            for attribute in &layout.attributes
            {
              raw_layout = raw_layout.attribute
              (
                gl::VertexAttribute::new()
                .location( attribute.location )
                .format( attribute.format.to_webgpu() )
                .offset_from_value( f64::from( attribute.offset ) )
              );
            }
            let web_layout : web_sys::GpuVertexBufferLayout = raw_layout.into();
            vertex_state = vertex_state.buffer( &web_layout );
          }

          let fragment_state = gl::FragmentState::new( shader )
          .entry_point( desc.fragment_entry )
          .target( gl::ColorTargetState::new().format( desc.color_format.to_webgpu() ) );

          let mut pipeline_desc = gl::render_pipeline::desc( vertex_state )
          .layout( &pipeline_layout )
          .fragment( fragment_state );
          if desc.cull_back
          {
            pipeline_desc = pipeline_desc.primitive( gl::PrimitiveState::new().cull_back() );
          }
          if let Some( depth ) = desc.depth
          {
            pipeline_desc = pipeline_desc
            .depth_stencil( gl::DepthStencilState::new().format( depth.format.to_webgpu() ) );
          }
          Ok( RenderPipeline::WebGpu( pipeline_desc.create( device )? ) )
        }
        #[ cfg( feature = "webgl" ) ]
        Self::WebGl( context ) =>
        {
          let module = desc.shader.expect_webgl();
          let program = glw::ProgramFromSources::new( &module.vertex, &module.fragment )
          .compile_and_link( context )
          .map_err( glw::WebglError::from )?;
          let ( ubo_points, texture_units ) =
          webgl_introspect_bindings( context, &program, desc.bind_group_layouts );

          Ok( RenderPipeline::WebGl( Rc::new( RenderPipelineWebGl
          {
            program,
            vertex_buffers : desc.vertex_buffers.to_vec(),
            depth : desc.depth,
            cull_back : desc.cull_back,
            ubo_points,
            texture_units
          } ) ) )
        }
      }
    }

    /// Creates a command encoder for one frame's passes.
    pub fn create_command_encoder( &self ) -> CommandEncoder
    {
      match self
      {
        #[ cfg( feature = "webgpu" ) ]
        Self::WebGpu( device ) => CommandEncoder::WebGpu( device.create_command_encoder() ),
        #[ cfg( feature = "webgl" ) ]
        Self::WebGl( context ) => CommandEncoder::WebGl( context.clone() )
      }
    }

    /// The raw WebGPU object, when the handle belongs to the WebGPU backend.
    #[ cfg( feature = "webgpu" ) ]
    pub fn as_webgpu( &self ) -> Option< &web_sys::GpuDevice >
    {
      match self
      {
        Self::WebGpu( raw ) => Some( raw ),
        #[ cfg( feature = "webgl" ) ]
        Self::WebGl( _ ) => None
      }
    }

    /// The raw GL context, when the handle belongs to the WebGL backend.
    #[ cfg( feature = "webgl" ) ]
    pub fn as_webgl( &self ) -> Option< &glw::GL >
    {
      match self
      {
        #[ cfg( feature = "webgpu" ) ]
        Self::WebGpu( _ ) => None,
        Self::WebGl( raw ) => Some( raw )
      }
    }
  }

  impl Queue
  {
    /// Writes `data` into `buffer` at offset zero.
    // A single-backend build can make the surviving arm infallible; the
    // other backend's arm fails for real, so the signature stays fallible.
    #[ allow( clippy::unnecessary_wraps ) ]
    pub fn write_buffer( &self, buffer : &Buffer, data : &[ u8 ] ) -> Result< (), Error >
    {
      match self
      {
        #[ cfg( feature = "webgpu" ) ]
        Self::WebGpu( queue ) =>
        {
          gl::queue::write_buffer( queue, buffer.expect_webgpu(), data )?;
          Ok( () )
        }
        #[ cfg( feature = "webgl" ) ]
        Self::WebGl( context ) =>
        {
          let raw = buffer.expect_webgl();
          context.bind_buffer( raw.target, Some( &raw.buffer ) );
          context.buffer_sub_data_with_i32_and_u8_array( raw.target, 0, data );
          Ok( () )
        }
      }
    }

    /// Finishes `encoder` and submits its command buffer.
    // Consuming the encoder forecloses reuse after submission, which WebGPU
    // rejects at runtime.
    #[ allow( clippy::needless_pass_by_value ) ]
    pub fn submit( &self, encoder : CommandEncoder )
    {
      match self
      {
        #[ cfg( feature = "webgpu" ) ]
        Self::WebGpu( queue ) =>
        {
          gl::queue::submit( queue, encoder.expect_webgpu().finish() );
        }
        #[ cfg( feature = "webgl" ) ]
        Self::WebGl( context ) =>
        {
          // GL executed the commands eagerly as the pass recorded them;
          // flushing pushes them to the driver.
          let _ = encoder;
          context.flush();
        }
      }
    }

    /// The raw WebGPU object, when the handle belongs to the WebGPU backend.
    #[ cfg( feature = "webgpu" ) ]
    pub fn as_webgpu( &self ) -> Option< &web_sys::GpuQueue >
    {
      match self
      {
        Self::WebGpu( raw ) => Some( raw ),
        #[ cfg( feature = "webgl" ) ]
        Self::WebGl( _ ) => None
      }
    }

    /// The raw GL context, when the handle belongs to the WebGL backend.
    #[ cfg( feature = "webgl" ) ]
    pub fn as_webgl( &self ) -> Option< &glw::GL >
    {
      match self
      {
        #[ cfg( feature = "webgpu" ) ]
        Self::WebGpu( _ ) => None,
        Self::WebGl( raw ) => Some( raw )
      }
    }
  }

  impl Surface
  {
    /// Format the surface is configured with.
    pub fn format( &self ) -> TextureFormat
    {
      match self
      {
        #[ cfg( feature = "webgpu" ) ]
        Self::WebGpu { format, .. } => *format,
        // The GL canvas backbuffer is 8-bit rgba; this is the nearest name
        // the v0 surface has for it.
        #[ cfg( feature = "webgl" ) ]
        Self::WebGl { .. } => TextureFormat::Rgba8Unorm
      }
    }

    /// A view of the texture the canvas presents next.
    ///
    /// Valid for the current frame only — request a fresh view every frame.
    // A single-backend build can make the surviving arm infallible; the
    // other backend's arm fails for real, so the signature stays fallible.
    #[ allow( clippy::unnecessary_wraps ) ]
    pub fn current_view( &self ) -> Result< TextureView, Error >
    {
      match self
      {
        #[ cfg( feature = "webgpu" ) ]
        Self::WebGpu { context, .. } =>
        {
          let texture = gl::context::current_texture( context )?;
          Ok( TextureView::WebGpu( gl::texture::view( &texture )? ) )
        }
        #[ cfg( feature = "webgl" ) ]
        Self::WebGl { .. } => Ok( TextureView::WebGl( TextureViewWebGl::CanvasBackbuffer ) )
      }
    }

    /// The raw WebGPU canvas context, when the handle belongs to the WebGPU
    /// backend.
    #[ cfg( feature = "webgpu" ) ]
    pub fn as_webgpu( &self ) -> Option< &gl::GL >
    {
      match self
      {
        Self::WebGpu { context, .. } => Some( context ),
        #[ cfg( feature = "webgl" ) ]
        Self::WebGl { .. } => None
      }
    }

    /// The raw GL context, when the handle belongs to the WebGL backend.
    #[ cfg( feature = "webgl" ) ]
    pub fn as_webgl( &self ) -> Option< &glw::GL >
    {
      match self
      {
        #[ cfg( feature = "webgpu" ) ]
        Self::WebGpu { .. } => None,
        Self::WebGl { context } => Some( context )
      }
    }
  }

  /// Resolves the binding name convention of a linked program : uniform
  /// blocks named `ub_{group}_{binding}` and sampler uniforms named
  /// `tex_{group}_{binding}` receive sequential binding points and texture
  /// units; names the linker pruned are skipped, matching GL practice for
  /// optimized-out uniforms.
  #[ cfg( feature = "webgl" ) ]
  fn webgl_introspect_bindings
  (
    context : &glw::GL,
    program : &glw::web_sys::WebGlProgram,
    bind_group_layouts : &[ &BindGroupLayout ]
  ) -> ( BindingMap, BindingMap )
  {
    context.use_program( Some( program ) );
    let mut ubo_points = Vec::new();
    let mut texture_units = Vec::new();
    let mut next_point : u32 = 0;
    let mut next_unit : u32 = 0;
    for ( group_index, layout ) in bind_group_layouts.iter().enumerate()
    {
      let group_index = to_u32( group_index );
      for ( binding_index, entry ) in layout.expect_webgl().entries.iter().enumerate()
      {
        let binding_index = to_u32( binding_index );
        match entry.ty
        {
          BindingType::UniformBuffer =>
          {
            let name = format!( "ub_{group_index}_{binding_index}" );
            let block_index = context.get_uniform_block_index( program, &name );
            if block_index != glw::GL::INVALID_INDEX
            {
              context.uniform_block_binding( program, block_index, next_point );
              ubo_points.push( ( ( group_index, binding_index ), next_point ) );
            }
            next_point += 1;
          }
          BindingType::Texture =>
          {
            let name = format!( "tex_{group_index}_{binding_index}" );
            if let Some( location ) = context.get_uniform_location( program, &name )
            {
              context.uniform1i( Some( &location ), to_i32( next_unit ) );
              texture_units.push( ( ( group_index, binding_index ), next_unit ) );
            }
            next_unit += 1;
          }
          BindingType::Sampler => {}
        }
      }
    }
    ( ubo_points, texture_units )
  }

  /// GL bind target a buffer is created against, from its usage flags.
  #[ cfg( feature = "webgl" ) ]
  fn webgl_buffer_target( usage : BufferUsage ) -> u32
  {
    if usage.contains( BufferUsage::INDEX )
    {
      glw::GL::ELEMENT_ARRAY_BUFFER
    }
    else if usage.contains( BufferUsage::UNIFORM )
    {
      glw::GL::UNIFORM_BUFFER
    }
    else
    {
      glw::GL::ARRAY_BUFFER
    }
  }

  /// GL data-store usage hint, from the buffer's usage flags — uniforms are
  /// rewritten per frame, everything else is uploaded once.
  #[ cfg( feature = "webgl" ) ]
  fn webgl_buffer_hint( usage : BufferUsage ) -> u32
  {
    if usage.contains( BufferUsage::UNIFORM )
    {
      glw::GL::DYNAMIC_DRAW
    }
    else
    {
      glw::GL::STATIC_DRAW
    }
  }
}

crate::mod_interface!
{
  orphan use
  {
    Device,
    Queue,
    Surface
  };
}
