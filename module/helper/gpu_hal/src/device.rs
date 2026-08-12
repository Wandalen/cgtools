mod private
{
  #[ cfg( all( feature = "webgpu", target_arch = "wasm32" ) ) ]
  use minwebgpu as gl;
  #[ cfg( all( feature = "webgpu", target_arch = "wasm32" ) ) ]
  use gl::web_sys;
  #[ cfg( all( feature = "webgl", target_arch = "wasm32" ) ) ]
  use minwebgl as glw;
  #[ cfg( all( feature = "webgl", target_arch = "wasm32", not( feature = "webgpu" ) ) ) ]
  use glw::web_sys;
  #[ cfg( all( feature = "webgl", target_arch = "wasm32" ) ) ]
  use std::rc::Rc;
  #[ cfg( all( feature = "webgl", target_arch = "wasm32" ) ) ]
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
  #[ cfg( all( feature = "webgpu", target_arch = "wasm32" ) ) ]
  use crate::
  {
    TextureUsage,
    ShaderStages
  };
  // The native arms reach these enums through their `to_wgpu` methods on
  // field values, never by name — inside this layer, wasm32 implies at
  // least one browser backend ( see the layer gates in `lib.rs` ).
  #[ cfg( target_arch = "wasm32" ) ]
  use crate::
  {
    FilterMode,
    AddressMode,
    BindingType
  };
  #[ cfg( all( feature = "native", not( target_arch = "wasm32" ) ) ) ]
  use wgpu::util::DeviceExt;
  #[ cfg( all( feature = "native", not( target_arch = "wasm32" ) ) ) ]
  use crate::native::read_texture_rgba8;
  use crate::
  {
    Error,
    BufferUsage,
    TextureFormat,
    TextureDesc,
    SamplerDesc,
    ShaderSource,
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
    #[ cfg( all( feature = "webgpu", target_arch = "wasm32" ) ) ]
    WebGpu( web_sys::GpuDevice ),
    /// WebGL backend device — the GL context itself.
    #[ cfg( all( feature = "webgl", target_arch = "wasm32" ) ) ]
    WebGl( glw::GL ),
    /// Native backend device.
    #[ cfg( all( feature = "native", not( target_arch = "wasm32" ) ) ) ]
    Native( wgpu::Device )
  }

  /// The command queue of a device.
  #[ derive( Debug ) ]
  pub enum Queue
  {
    /// WebGPU backend queue.
    #[ cfg( all( feature = "webgpu", target_arch = "wasm32" ) ) ]
    WebGpu( web_sys::GpuQueue ),
    /// WebGL backend queue — the GL context executes commands eagerly.
    #[ cfg( all( feature = "webgl", target_arch = "wasm32" ) ) ]
    WebGl( glw::GL ),
    /// Native backend queue.
    #[ cfg( all( feature = "native", not( target_arch = "wasm32" ) ) ) ]
    Native( wgpu::Queue )
  }

  /// The canvas presentation surface of a device.
  #[ derive( Debug ) ]
  pub enum Surface
  {
    /// WebGPU backend surface: a configured canvas context.
    #[ cfg( all( feature = "webgpu", target_arch = "wasm32" ) ) ]
    WebGpu
    {
      /// Configured canvas presentation context.
      context : gl::GL,
      /// Format the canvas is configured with.
      format : TextureFormat
    },
    /// WebGL backend surface — the canvas backbuffer of the GL context.
    #[ cfg( all( feature = "webgl", target_arch = "wasm32" ) ) ]
    WebGl
    {
      /// The GL context whose canvas the surface presents to.
      context : glw::GL
    },
    /// Native backend surface : an offscreen render target, readable
    /// through `read_pixels` — there is no window to present to.
    #[ cfg( all( feature = "native", not( target_arch = "wasm32" ) ) ) ]
    Native
    {
      /// Offscreen color target of the surface.
      texture : wgpu::Texture,
      /// Format the target is created with.
      format : TextureFormat
    }
  }

  impl Device
  {
    /// Requests a WebGPU adapter and device, then configures `canvas` for
    /// presentation in the browser's preferred canvas format.
    ///
    /// # Errors
    ///
    /// Returns [`Error::WebGpu`] if creating the canvas context or
    /// configuring it for presentation fails, or [`Error::Unsupported`] if
    /// the canvas's preferred format has no HAL equivalent.
    #[ cfg( all( feature = "webgpu", target_arch = "wasm32" ) ) ]
    pub async fn new_webgpu
    (
      canvas : &web_sys::HtmlCanvasElement
    ) -> Result< ( Device, Queue, Surface ), Error >
    {
      let context = gl::context::from_canvas( canvas ).map_err( gl::WebGPUError::from )?;
      let adapter = gl::context::adapter_request().await;
      let device = gl::context::device_request( &adapter ).await;
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
    ///
    /// # Errors
    ///
    /// Returns [`Error::WebGl`] if creating the WebGL2 context or querying
    /// `EXT_color_buffer_float` fails, or [`Error::Unsupported`] if the
    /// extension is unavailable.
    #[ cfg( all( feature = "webgl", target_arch = "wasm32" ) ) ]
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

    /// Requests a wgpu adapter and device ( default options — any adapter
    /// qualifies, software rasterizers included ) and builds an offscreen
    /// rgba8 surface of the given size, readable through
    /// `Surface::read_pixels`.
    ///
    /// Synchronous : `minwgpu` blocks on the async requests internally,
    /// which is the natural shape off the browser event loop.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Native`] if requesting a wgpu adapter or finishing
    /// the device context fails.
    #[ cfg( all( feature = "native", not( target_arch = "wasm32" ) ) ) ]
    pub fn new_native( width : u32, height : u32 ) -> Result< ( Device, Queue, Surface ), Error >
    {
      let context = minwgpu::context::Context::builder()
      .instance_make()
      .adapter_request()?
      .context_finish()?;
      let device = context.device_get().clone();
      let queue = context.queue_get().clone();
      let format = TextureFormat::Rgba8Unorm;
      let texture = device.create_texture( &wgpu::TextureDescriptor
      {
        label : Some( "gpu_hal offscreen surface" ),
        size : wgpu::Extent3d { width, height, depth_or_array_layers : 1 },
        mip_level_count : 1,
        sample_count : 1,
        dimension : wgpu::TextureDimension::D2,
        format : format.to_wgpu(),
        usage : wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats : &[]
      } );

      Ok
      ((
        Device::Native( device ),
        Queue::Native( queue ),
        Surface::Native { texture, format }
      ))
    }

    /// Clip-space depth range the backend's projection matrices must target.
    #[must_use]
    pub fn depth_range( &self ) -> DepthRange
    {
      match self
      {
        #[ cfg( all( feature = "webgpu", target_arch = "wasm32" ) ) ]
        Self::WebGpu( _ ) => DepthRange::ZeroToOne,
        #[ cfg( all( feature = "webgl", target_arch = "wasm32" ) ) ]
        Self::WebGl( _ ) => DepthRange::NegOneToOne,
        #[ cfg( all( feature = "native", not( target_arch = "wasm32" ) ) ) ]
        Self::Native( _ ) => DepthRange::ZeroToOne
      }
    }

    /// Creates an uninitialized buffer of `size` bytes.
    ///
    /// # Errors
    ///
    /// Returns [`Error::WebGpu`] if the underlying WebGPU buffer-creation
    /// call fails, or [`Error::WebGl`] if the WebGL context fails to
    /// allocate the buffer. The native backend never fails this call.
    pub fn create_buffer( &self, size : u64, usage : BufferUsage ) -> Result< Buffer, Error >
    {
      // Browser buffer allocations sit far below f64's exact integer
      // range, so the cast is lossless in practice.
      #[ cfg( target_arch = "wasm32" ) ]
      let size_f64 = size as f64;
      match self
      {
        #[ cfg( all( feature = "webgpu", target_arch = "wasm32" ) ) ]
        Self::WebGpu( device ) =>
        {
          let raw = gl::BufferDescriptor::new( usage.bits() )
          .size_from_value( size_f64 )
          .create( device )?;
          Ok( Buffer::WebGpu( raw ) )
        }
        #[ cfg( all( feature = "webgl", target_arch = "wasm32" ) ) ]
        Self::WebGl( context ) =>
        {
          let target = webgl_buffer_target( usage );
          let buffer = context.create_buffer()
          .ok_or_else( || Error::WebGl( "failed to allocate buffer".to_string() ) )?;
          context.bind_buffer( target, Some( &buffer ) );
          context.buffer_data_with_f64( target, size_f64, webgl_buffer_hint( usage ) );
          Ok( Buffer::WebGl( BufferWebGl { buffer, target } ) )
        }
        #[ cfg( all( feature = "native", not( target_arch = "wasm32" ) ) ) ]
        Self::Native( device ) =>
        {
          Ok( Buffer::Native( device.create_buffer( &wgpu::BufferDescriptor
          {
            label : None,
            size,
            usage : usage.to_wgpu(),
            mapped_at_creation : false
          } ) ) )
        }
      }
    }

    /// Creates a buffer initialized with `data`.
    ///
    /// # Errors
    ///
    /// Returns [`Error::WebGpu`] if the underlying WebGPU buffer-creation
    /// call fails, or [`Error::WebGl`] if the WebGL context fails to
    /// allocate the buffer. The native backend never fails this call.
    pub fn create_buffer_init( &self, data : &[ u8 ], usage : BufferUsage ) -> Result< Buffer, Error >
    {
      match self
      {
        #[ cfg( all( feature = "webgpu", target_arch = "wasm32" ) ) ]
        Self::WebGpu( device ) =>
        {
          // v0 tradeoff : the init descriptor needs a sized value, so the
          // byte slice is copied once on upload.
          let data = data.to_vec();
          let raw = gl::BufferInitDescriptor::new( &data, usage.bits() ).create( device )?;
          Ok( Buffer::WebGpu( raw ) )
        }
        #[ cfg( all( feature = "webgl", target_arch = "wasm32" ) ) ]
        Self::WebGl( context ) =>
        {
          let target = webgl_buffer_target( usage );
          let buffer = context.create_buffer()
          .ok_or_else( || Error::WebGl( "failed to allocate buffer".to_string() ) )?;
          context.bind_buffer( target, Some( &buffer ) );
          context.buffer_data_with_u8_array( target, data, webgl_buffer_hint( usage ) );
          Ok( Buffer::WebGl( BufferWebGl { buffer, target } ) )
        }
        #[ cfg( all( feature = "native", not( target_arch = "wasm32" ) ) ) ]
        Self::Native( device ) =>
        {
          // DeviceExt::create_buffer_init pads to wgpu's copy alignment,
          // which a hand-rolled mapped_at_creation path would have to redo.
          Ok( Buffer::Native( device.create_buffer_init( &wgpu::util::BufferInitDescriptor
          {
            label : None,
            contents : data,
            usage : usage.to_wgpu()
          } ) ) )
        }
      }
    }

    /// Creates a 2d texture ( one mip, one sample ).
    ///
    /// # Errors
    ///
    /// Returns [`Error::WebGpu`] if the underlying WebGPU texture-creation
    /// call fails. Returns [`Error::WebGl`] if `desc.format` has no WebGL
    /// internal-format mapping, or if the WebGL context fails to allocate
    /// the texture. The native backend never fails this call.
    pub fn create_texture( &self, desc : &TextureDesc ) -> Result< Texture, Error >
    {
      match self
      {
        #[ cfg( all( feature = "webgpu", target_arch = "wasm32" ) ) ]
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
        #[ cfg( all( feature = "webgl", target_arch = "wasm32" ) ) ]
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
        #[ cfg( all( feature = "native", not( target_arch = "wasm32" ) ) ) ]
        Self::Native( device ) =>
        {
          Ok( Texture::Native( device.create_texture( &wgpu::TextureDescriptor
          {
            label : None,
            size : wgpu::Extent3d
            {
              width : desc.size[ 0 ],
              height : desc.size[ 1 ],
              depth_or_array_layers : desc.size[ 2 ]
            },
            mip_level_count : 1,
            sample_count : 1,
            dimension : wgpu::TextureDimension::D2,
            format : desc.format.to_wgpu(),
            usage : desc.usage.to_wgpu(),
            view_formats : &[]
          } ) ) )
        }
      }
    }

    /// Creates a sampler.
    ///
    /// # Errors
    ///
    /// Returns [`Error::WebGl`] if the WebGL context fails to allocate the
    /// sampler. The WebGPU and native backends never fail this call.
    #[ allow( clippy::unnecessary_wraps, reason = "fires only in single-backend builds where the surviving arm is infallible; the other backend's arm fails for real, so the signature stays fallible" ) ]
    pub fn create_sampler( &self, desc : SamplerDesc ) -> Result< Sampler, Error >
    {
      match self
      {
        #[ cfg( all( feature = "webgpu", target_arch = "wasm32" ) ) ]
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
        #[ cfg( all( feature = "webgl", target_arch = "wasm32" ) ) ]
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
        #[ cfg( all( feature = "native", not( target_arch = "wasm32" ) ) ) ]
        Self::Native( device ) =>
        {
          Ok( Sampler::Native( device.create_sampler( &wgpu::SamplerDescriptor
          {
            label : None,
            address_mode_u : desc.address.to_wgpu(),
            address_mode_v : desc.address.to_wgpu(),
            address_mode_w : desc.address.to_wgpu(),
            mag_filter : desc.filter.to_wgpu(),
            min_filter : desc.filter.to_wgpu(),
            ..wgpu::SamplerDescriptor::default()
          } ) ) )
        }
      }
    }

    /// Compiles a shader module from `source`. The WebGPU backend consumes
    /// the canonical WGSL and ignores the GLSL override slots; the WebGL
    /// backend requires both GLSL overrides and defers compilation to
    /// pipeline creation, where GL links per program.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Unsupported`] on the WebGL backend if `source` is
    /// missing either GLSL override slot. The WebGPU and native backends
    /// never fail this call.
    #[ allow( clippy::unnecessary_wraps, reason = "fires only in the webgpu-only build, whose infallibility is incidental; the WebGL arm fails for real, so the signature stays fallible" ) ]
    pub fn create_shader_module( &self, source : &ShaderSource< '_ > ) -> Result< ShaderModule, Error >
    {
      match self
      {
        #[ cfg( all( feature = "webgpu", target_arch = "wasm32" ) ) ]
        Self::WebGpu( device ) =>
        {
          Ok( ShaderModule::WebGpu( gl::ShaderModule::new( source.wgsl ).create( device ) ) )
        }
        #[ cfg( all( feature = "webgl", target_arch = "wasm32" ) ) ]
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
        #[ cfg( all( feature = "native", not( target_arch = "wasm32" ) ) ) ]
        Self::Native( device ) =>
        {
          Ok( ShaderModule::Native( device.create_shader_module( wgpu::ShaderModuleDescriptor
          {
            label : None,
            source : wgpu::ShaderSource::Wgsl( source.wgsl.into() )
          } ) ) )
        }
      }
    }

    /// Creates a bind group layout; binding indices follow entry order.
    ///
    /// # Errors
    ///
    /// Returns [`Error::WebGpu`] if the underlying WebGPU layout-entry or
    /// layout-creation call fails. The WebGL and native backends never
    /// fail this call.
    #[ allow( clippy::unnecessary_wraps, reason = "fires only in single-backend builds where the surviving arm is infallible; the other backend's arm fails for real, so the signature stays fallible" ) ]
    pub fn create_bind_group_layout
    (
      &self,
      entries : &[ BindGroupLayoutEntry ]
    ) -> Result< BindGroupLayout, Error >
    {
      match self
      {
        #[ cfg( all( feature = "webgpu", target_arch = "wasm32" ) ) ]
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
            // Fix(BUG-051): `BindGroupLayoutDescriptor::entry` became fallible (returns
            // `Result<Self, WebGPUError>`) once its underlying `TryFrom` conversion stopped
            // panicking on an unset binding type — propagate with `?` instead of a plain
            // assignment.
            // Root cause: caller written against the old infallible `entry` signature.
            // Pitfall: a downstream `From`-to-`TryFrom` signature change is a silent type
            // error at every call site, not a panic — the compiler catches it immediately,
            // but only once this crate is actually rebuilt against the new minwebgpu.
            builder = builder.entry( raw_entry )?;
          }
          Ok( BindGroupLayout::WebGpu( builder.create( device )? ) )
        }
        #[ cfg( all( feature = "webgl", target_arch = "wasm32" ) ) ]
        Self::WebGl( _ ) =>
        {
          // GL has no layout objects — the entry list is the layout, consumed
          // by pipeline creation for binding introspection.
          Ok( BindGroupLayout::WebGl( BindGroupLayoutWebGl { entries : entries.to_vec() } ) )
        }
        #[ cfg( all( feature = "native", not( target_arch = "wasm32" ) ) ) ]
        Self::Native( device ) =>
        {
          let raw_entries : Vec< wgpu::BindGroupLayoutEntry > = entries.iter().enumerate()
          .map
          (
            | ( index, entry ) |
            wgpu::BindGroupLayoutEntry
            {
              binding : u32::try_from( index ).unwrap_or( u32::MAX ),
              visibility : entry.visibility.to_wgpu(),
              ty : entry.ty.to_wgpu(),
              count : None
            }
          )
          .collect();
          Ok( BindGroupLayout::Native( device.create_bind_group_layout( &wgpu::BindGroupLayoutDescriptor
          {
            label : None,
            entries : &raw_entries
          } ) ) )
        }
      }
    }

    /// Creates a bind group; `resources` follow the layout's entry order.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Unsupported`] on the WebGL backend if `resources`
    /// includes the canvas backbuffer as a sampled texture view — the
    /// backbuffer cannot be sampled. The WebGPU and native backends never
    /// fail this call.
    #[ allow( clippy::unnecessary_wraps, reason = "fires only in single-backend builds where the surviving arm is infallible; the other backend's arm fails for real, so the signature stays fallible" ) ]
    pub fn create_bind_group
    (
      &self,
      layout : &BindGroupLayout,
      resources : &[ BindingResource< '_ > ]
    ) -> Result< BindGroup, Error >
    {
      match self
      {
        #[ cfg( all( feature = "webgpu", target_arch = "wasm32" ) ) ]
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
        #[ cfg( all( feature = "webgl", target_arch = "wasm32" ) ) ]
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
        #[ cfg( all( feature = "native", not( target_arch = "wasm32" ) ) ) ]
        Self::Native( device ) =>
        {
          let raw_entries : Vec< wgpu::BindGroupEntry< '_ > > = resources.iter().enumerate()
          .map
          (
            | ( index, resource ) |
            wgpu::BindGroupEntry
            {
              binding : u32::try_from( index ).unwrap_or( u32::MAX ),
              resource : match resource
              {
                BindingResource::Buffer( buffer ) => buffer.expect_native().as_entire_binding(),
                BindingResource::TextureView( view ) =>
                {
                  wgpu::BindingResource::TextureView( view.expect_native() )
                }
                BindingResource::Sampler( sampler ) =>
                {
                  wgpu::BindingResource::Sampler( sampler.expect_native() )
                }
              }
            }
          )
          .collect();
          Ok( BindGroup::Native( device.create_bind_group( &wgpu::BindGroupDescriptor
          {
            label : None,
            layout : layout.expect_native(),
            entries : &raw_entries
          } ) ) )
        }
      }
    }

    /// Creates a render pipeline.
    ///
    /// # Errors
    ///
    /// Returns [`Error::WebGpu`] if the underlying WebGPU pipeline-creation
    /// call fails, or [`Error::WebGl`] if the vertex/fragment shader pair
    /// fails to compile and link. The native backend never fails this
    /// call.
    pub fn create_render_pipeline( &self, desc : &RenderPipelineDesc< '_ > ) -> Result< RenderPipeline, Error >
    {
      match self
      {
        #[ cfg( all( feature = "webgpu", target_arch = "wasm32" ) ) ]
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
        #[ cfg( all( feature = "webgl", target_arch = "wasm32" ) ) ]
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
        #[ cfg( all( feature = "native", not( target_arch = "wasm32" ) ) ) ]
        Self::Native( device ) => Ok( native_create_render_pipeline( device, desc ) )
      }
    }

    /// Creates a command encoder for one frame's passes.
    #[must_use]
    pub fn create_command_encoder( &self ) -> CommandEncoder
    {
      match self
      {
        #[ cfg( all( feature = "webgpu", target_arch = "wasm32" ) ) ]
        Self::WebGpu( device ) => CommandEncoder::WebGpu( device.create_command_encoder() ),
        #[ cfg( all( feature = "webgl", target_arch = "wasm32" ) ) ]
        Self::WebGl( context ) => CommandEncoder::WebGl( context.clone() ),
        #[ cfg( all( feature = "native", not( target_arch = "wasm32" ) ) ) ]
        Self::Native( device ) =>
        {
          CommandEncoder::Native
          (
            device.create_command_encoder( &wgpu::CommandEncoderDescriptor::default() )
          )
        }
      }
    }

    /// The raw WebGPU object, when the handle belongs to the WebGPU backend.
    #[ cfg( all( feature = "webgpu", target_arch = "wasm32" ) ) ]
    #[must_use]
    pub fn as_webgpu( &self ) -> Option< &web_sys::GpuDevice >
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
        #[ cfg( all( feature = "webgpu", target_arch = "wasm32" ) ) ]
        Self::WebGpu( _ ) => None,
        Self::WebGl( raw ) => Some( raw )
      }
    }

    /// The raw wgpu object, when the handle belongs to the native backend.
    #[ cfg( all( feature = "native", not( target_arch = "wasm32" ) ) ) ]
    #[must_use]
    pub fn as_native( &self ) -> Option< &wgpu::Device >
    {
      match self
      {
        Self::Native( raw ) => Some( raw )
      }
    }

    #[ cfg( all( feature = "native", not( target_arch = "wasm32" ) ) ) ]
    pub( crate ) fn expect_native( &self ) -> &wgpu::Device
    {
      match self
      {
        Self::Native( raw ) => raw
      }
    }
  }

  impl Queue
  {
    /// Writes `data` into `buffer` at offset zero.
    ///
    /// # Errors
    ///
    /// Returns [`Error::WebGpu`] if the underlying WebGPU write call
    /// fails. The WebGL and native backends never fail this call.
    #[ allow( clippy::unnecessary_wraps, reason = "fires only in single-backend builds where the surviving arm is infallible; the other backend's arm fails for real, so the signature stays fallible" ) ]
    pub fn write_buffer( &self, buffer : &Buffer, data : &[ u8 ] ) -> Result< (), Error >
    {
      match self
      {
        #[ cfg( all( feature = "webgpu", target_arch = "wasm32" ) ) ]
        Self::WebGpu( queue ) =>
        {
          gl::queue::buffer_write( queue, buffer.expect_webgpu(), data )?;
          Ok( () )
        }
        #[ cfg( all( feature = "webgl", target_arch = "wasm32" ) ) ]
        Self::WebGl( context ) =>
        {
          let raw = buffer.expect_webgl();
          context.bind_buffer( raw.target, Some( &raw.buffer ) );
          context.buffer_sub_data_with_i32_and_u8_array( raw.target, 0, data );
          Ok( () )
        }
        #[ cfg( all( feature = "native", not( target_arch = "wasm32" ) ) ) ]
        Self::Native( queue ) =>
        {
          queue.write_buffer( buffer.expect_native(), 0, data );
          Ok( () )
        }
      }
    }

    /// Writes `data` into `texture` at the base mip level, covering the
    /// texture's full extent. `data` must be tightly packed ( no row
    /// padding ) — the WebGPU and native arms derive their own internal
    /// `bytes_per_row` from the texture's format; WebGL never needs one.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Unsupported`] if the texture's format has no
    /// portable CPU-side texel layout ( e.g. `Depth24Plus` ), or a
    /// backend-specific error if the underlying write call fails.
    pub fn write_texture( &self, texture : &Texture, data : &[ u8 ] ) -> Result< (), Error >
    {
      match self
      {
        #[ cfg( all( feature = "webgpu", target_arch = "wasm32" ) ) ]
        Self::WebGpu( queue ) =>
        {
          let raw = texture.expect_webgpu();
          let width = raw.width();
          let height = raw.height();
          let depth_or_array_layers = raw.depth_or_array_layers();
          let format = TextureFormat::from_webgpu( raw.format() )?;
          let bytes_per_row = width * format.bytes_per_texel()?;

          let data_layout = web_sys::GpuTexelCopyBufferLayout::new();
          data_layout.set_bytes_per_row( bytes_per_row );
          data_layout.set_rows_per_image( height );

          let size = web_sys::GpuExtent3dDict::new( width );
          size.set_height( height );
          size.set_depth_or_array_layers( depth_or_array_layers );

          gl::queue::texture_write
          (
            queue,
            &web_sys::GpuTexelCopyTextureInfo::new( raw ),
            data,
            &data_layout,
            &size
          )?;
          Ok( () )
        }
        #[ cfg( all( feature = "webgl", target_arch = "wasm32" ) ) ]
        Self::WebGl( context ) =>
        {
          let raw = texture.expect_webgl();
          let ( format, type_ ) = raw.format.webgl_format_and_type()?;
          context.bind_texture( glw::GL::TEXTURE_2D, Some( &raw.texture ) );
          context.tex_sub_image_2d_with_i32_and_i32_and_u32_and_type_and_opt_u8_array
          (
            glw::GL::TEXTURE_2D,
            0,
            0,
            0,
            to_i32( raw.size[ 0 ] ),
            to_i32( raw.size[ 1 ] ),
            format,
            type_,
            Some( data )
          )
          .map_err( | e | Error::WebGl( format!( "{e:?}" ) ) )?;
          Ok( () )
        }
        #[ cfg( all( feature = "native", not( target_arch = "wasm32" ) ) ) ]
        Self::Native( queue ) =>
        {
          let raw = texture.expect_native();
          let width = raw.width();
          let height = raw.height();
          // Native queries wgpu's own authoritative format-size table
          // directly, rather than routing through `TextureFormat::
          // bytes_per_texel` — `raw.format()` is already a
          // `wgpu::TextureFormat`, so a round trip through the HAL's own
          // enum would just re-derive what wgpu already knows.
          let bytes_per_row = width * raw.format().block_copy_size( None )
          .ok_or_else( || Error::Unsupported( format!( "{:?} has no portable CPU-side texel layout", raw.format() ) ) )?;

          queue.write_texture
          (
            wgpu::TexelCopyTextureInfo
            {
              texture : raw,
              mip_level : 0,
              origin : wgpu::Origin3d::ZERO,
              aspect : wgpu::TextureAspect::All
            },
            data,
            wgpu::TexelCopyBufferLayout
            {
              offset : 0,
              bytes_per_row : Some( bytes_per_row ),
              rows_per_image : Some( height )
            },
            wgpu::Extent3d { width, height, depth_or_array_layers : raw.depth_or_array_layers() }
          );
          Ok( () )
        }
      }
    }

    /// Finishes `encoder` and submits its command buffer.
    #[ allow( clippy::needless_pass_by_value, reason = "submitting consumes the encoder -- WebGPU's and wgpu's finish() both take ownership, and a submitted encoder must not be reusable afterward" ) ]
    pub fn submit( &self, encoder : CommandEncoder )
    {
      match self
      {
        #[ cfg( all( feature = "webgpu", target_arch = "wasm32" ) ) ]
        Self::WebGpu( queue ) =>
        {
          gl::queue::submit( queue, encoder.expect_webgpu().finish() );
        }
        #[ cfg( all( feature = "webgl", target_arch = "wasm32" ) ) ]
        Self::WebGl( context ) =>
        {
          // GL executed the commands eagerly as the pass recorded them;
          // flushing pushes them to the driver.
          let _ = encoder;
          context.flush();
        }
        #[ cfg( all( feature = "native", not( target_arch = "wasm32" ) ) ) ]
        Self::Native( queue ) =>
        {
          // Finishing needs ownership of the raw encoder, so the drill-down
          // happens by value here rather than through `expect_native`.
          match encoder
          {
            CommandEncoder::Native( raw ) =>
            {
              queue.submit( core::iter::once( raw.finish() ) );
            }
          }
        }
      }
    }

    /// The raw WebGPU object, when the handle belongs to the WebGPU backend.
    #[ cfg( all( feature = "webgpu", target_arch = "wasm32" ) ) ]
    #[must_use]
    pub fn as_webgpu( &self ) -> Option< &web_sys::GpuQueue >
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
        #[ cfg( all( feature = "webgpu", target_arch = "wasm32" ) ) ]
        Self::WebGpu( _ ) => None,
        Self::WebGl( raw ) => Some( raw )
      }
    }

    /// The raw wgpu object, when the handle belongs to the native backend.
    #[ cfg( all( feature = "native", not( target_arch = "wasm32" ) ) ) ]
    #[must_use]
    pub fn as_native( &self ) -> Option< &wgpu::Queue >
    {
      match self
      {
        Self::Native( raw ) => Some( raw )
      }
    }

    #[ cfg( all( feature = "native", not( target_arch = "wasm32" ) ) ) ]
    pub( crate ) fn expect_native( &self ) -> &wgpu::Queue
    {
      match self
      {
        Self::Native( raw ) => raw
      }
    }
  }

  impl Surface
  {
    /// Format the surface is configured with.
    #[must_use]
    pub fn format( &self ) -> TextureFormat
    {
      match self
      {
        #[ cfg( all( feature = "webgpu", target_arch = "wasm32" ) ) ]
        Self::WebGpu { format, .. } => *format,
        // The GL canvas backbuffer is 8-bit rgba; this is the nearest name
        // the v0 surface has for it.
        #[ cfg( all( feature = "webgl", target_arch = "wasm32" ) ) ]
        Self::WebGl { .. } => TextureFormat::Rgba8Unorm,
        #[ cfg( all( feature = "native", not( target_arch = "wasm32" ) ) ) ]
        Self::Native { format, .. } => *format
      }
    }

    /// A view of the texture the canvas presents next.
    ///
    /// Valid for the current frame only — request a fresh view every frame.
    ///
    /// # Errors
    ///
    /// Returns [`Error::WebGpu`] if retrieving the current WebGPU canvas
    /// texture or creating its view fails. The WebGL and native backends
    /// never fail this call.
    #[ allow( clippy::unnecessary_wraps, reason = "fires only in single-backend builds where the surviving arm is infallible; the other backend's arm fails for real, so the signature stays fallible" ) ]
    pub fn current_view( &self ) -> Result< TextureView, Error >
    {
      match self
      {
        #[ cfg( all( feature = "webgpu", target_arch = "wasm32" ) ) ]
        Self::WebGpu { context, .. } =>
        {
          let texture = gl::context::current_texture( context )?;
          Ok( TextureView::WebGpu( gl::texture::view( &texture )? ) )
        }
        #[ cfg( all( feature = "webgl", target_arch = "wasm32" ) ) ]
        Self::WebGl { .. } => Ok( TextureView::WebGl( TextureViewWebGl::CanvasBackbuffer ) ),
        #[ cfg( all( feature = "native", not( target_arch = "wasm32" ) ) ) ]
        Self::Native { texture, .. } =>
        {
          Ok( TextureView::Native( texture.create_view( &wgpu::TextureViewDescriptor::default() ) ) )
        }
      }
    }

    /// The raw WebGPU canvas context, when the handle belongs to the WebGPU
    /// backend.
    #[ cfg( all( feature = "webgpu", target_arch = "wasm32" ) ) ]
    #[must_use]
    pub fn as_webgpu( &self ) -> Option< &gl::GL >
    {
      match self
      {
        Self::WebGpu { context, .. } => Some( context ),
        #[ cfg( all( feature = "webgl", target_arch = "wasm32" ) ) ]
        Self::WebGl { .. } => None
      }
    }

    /// The raw GL context, when the handle belongs to the WebGL backend.
    #[ cfg( all( feature = "webgl", target_arch = "wasm32" ) ) ]
    #[ must_use ]
    pub fn as_webgl( &self ) -> Option< &glw::GL >
    {
      match self
      {
        #[ cfg( all( feature = "webgpu", target_arch = "wasm32" ) ) ]
        Self::WebGpu { .. } => None,
        Self::WebGl { context } => Some( context )
      }
    }

    /// Reads the surface's pixels back as tightly-packed rgba8 bytes, top
    /// row first — the native backend's counterpart of presenting to a
    /// canvas, and the ground truth a pixel-asserting test reads.
    ///
    /// The browser surfaces present to their canvas instead and return
    /// `Unsupported` — read the canvas from the embedding page there.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Unsupported`] on the WebGPU and WebGL backends —
    /// browser surfaces present to their canvas and cannot be read back
    /// through this call. On the native backend, propagates
    /// `read_texture_rgba8`'s errors: [`Error::Unsupported`] if the
    /// surface's texture format is not `Rgba8Unorm`, or [`Error::Native`]
    /// if the GPU readback fails.
    #[ cfg_attr( all( feature = "webgpu", feature = "webgl", target_arch = "wasm32" ), expect( clippy::match_same_arms, reason = "the WebGpu and WebGl arms are gated by independent features and cannot be merged into an or-pattern without breaking single-feature builds" ) ) ]
    pub fn read_pixels( &self, device : &Device, queue : &Queue ) -> Result< Vec< u8 >, Error >
    {
      match self
      {
        #[ cfg( all( feature = "webgpu", target_arch = "wasm32" ) ) ]
        Self::WebGpu { .. } =>
        {
          let _ = ( device, queue );
          Err( Error::Unsupported
          (
            "read_pixels is a native-backend operation; browser surfaces present to their canvas".to_string()
          ) )
        }
        #[ cfg( all( feature = "webgl", target_arch = "wasm32" ) ) ]
        Self::WebGl { .. } =>
        {
          let _ = ( device, queue );
          Err( Error::Unsupported
          (
            "read_pixels is a native-backend operation; browser surfaces present to their canvas".to_string()
          ) )
        }
        #[ cfg( all( feature = "native", not( target_arch = "wasm32" ) ) ) ]
        Self::Native { texture, .. } =>
        {
          read_texture_rgba8( device.expect_native(), queue.expect_native(), texture )
        }
      }
    }

    /// The raw wgpu texture the surface renders into, when the handle
    /// belongs to the native backend.
    #[ cfg( all( feature = "native", not( target_arch = "wasm32" ) ) ) ]
    #[must_use]
    pub fn as_native( &self ) -> Option< &wgpu::Texture >
    {
      match self
      {
        Self::Native { texture, .. } => Some( texture )
      }
    }
  }

  /// Resolves the binding name convention of a linked program : uniform
  /// blocks named `ub_{group}_{binding}` and sampler uniforms named
  /// `tex_{group}_{binding}` receive sequential binding points and texture
  /// units; names the linker pruned are skipped, matching GL practice for
  /// optimized-out uniforms.
  #[ cfg( all( feature = "webgl", target_arch = "wasm32" ) ) ]
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
  #[ cfg( all( feature = "webgl", target_arch = "wasm32" ) ) ]
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
  #[ cfg( all( feature = "webgl", target_arch = "wasm32" ) ) ]
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

  /// Builds the v0 fixed function pipeline over a raw wgpu device : triangle
  /// list, one color target without blending, optional always-on depth.
  #[ cfg( all( feature = "native", not( target_arch = "wasm32" ) ) ) ]
  fn native_create_render_pipeline
  (
    device : &wgpu::Device,
    desc : &RenderPipelineDesc< '_ >
  ) -> RenderPipeline
  {
    let shader = desc.shader.expect_native();

    let raw_layouts : Vec< &wgpu::BindGroupLayout > = desc.bind_group_layouts.iter()
    .map( | layout | layout.expect_native() )
    .collect();
    let pipeline_layout = device.create_pipeline_layout( &wgpu::PipelineLayoutDescriptor
    {
      label : None,
      bind_group_layouts : &raw_layouts,
      push_constant_ranges : &[]
    } );

    // Two passes because wgpu's slot layout borrows its attribute
    // slice — the attributes must outlive the layouts referencing them.
    let attributes : Vec< Vec< wgpu::VertexAttribute > > = desc.vertex_buffers.iter()
    .map
    (
      | layout |
      layout.attributes.iter()
      .map
      (
        | attribute |
        wgpu::VertexAttribute
        {
          format : attribute.format.to_wgpu(),
          offset : u64::from( attribute.offset ),
          shader_location : attribute.location
        }
      )
      .collect()
    )
    .collect();
    let vertex_buffers : Vec< wgpu::VertexBufferLayout< '_ > > = desc.vertex_buffers.iter()
    .zip( &attributes )
    .map
    (
      | ( layout, attributes ) |
      wgpu::VertexBufferLayout
      {
        array_stride : u64::from( layout.stride ),
        step_mode : wgpu::VertexStepMode::Vertex,
        attributes
      }
    )
    .collect();

    let pipeline = device.create_render_pipeline( &wgpu::RenderPipelineDescriptor
    {
      label : None,
      layout : Some( &pipeline_layout ),
      vertex : wgpu::VertexState
      {
        module : shader,
        entry_point : Some( desc.vertex_entry ),
        compilation_options : wgpu::PipelineCompilationOptions::default(),
        buffers : &vertex_buffers
      },
      primitive : wgpu::PrimitiveState
      {
        cull_mode : if desc.cull_back { Some( wgpu::Face::Back ) } else { None },
        ..wgpu::PrimitiveState::default()
      },
      depth_stencil : desc.depth.map
      (
        | depth |
        wgpu::DepthStencilState
        {
          format : depth.format.to_wgpu(),
          depth_write_enabled : true,
          depth_compare : wgpu::CompareFunction::Less,
          stencil : wgpu::StencilState::default(),
          bias : wgpu::DepthBiasState::default()
        }
      ),
      multisample : wgpu::MultisampleState::default(),
      fragment : Some( wgpu::FragmentState
      {
        module : shader,
        entry_point : Some( desc.fragment_entry ),
        compilation_options : wgpu::PipelineCompilationOptions::default(),
        targets : &[ Some( wgpu::ColorTargetState
        {
          format : desc.color_format.to_wgpu(),
          blend : None,
          write_mask : wgpu::ColorWrites::ALL
        } ) ]
      } ),
      multiview : None,
      cache : None
    } );
    RenderPipeline::Native( pipeline )
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
