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
  use crate::native::texture_rgba8_read;
  #[ cfg( all( feature = "vulkan", not( target_arch = "wasm32" ) ) ) ]
  use crate::vulkan::
  {
    DeviceVulkan,
    QueueVulkan,
    SurfaceVulkan,
    TextureViewVulkan,
    surface_create,
    buffer_create as vulkan_buffer_create,
    buffer_init_create as vulkan_buffer_init_create,
    buffer_write as vulkan_buffer_write,
    texture_create as vulkan_texture_create,
    texture_write as vulkan_texture_write,
    sampler_create as vulkan_sampler_create,
    shader_module_create as vulkan_shader_module_create,
    bind_group_layout_create as vulkan_bind_group_layout_create,
    bind_group_create as vulkan_bind_group_create,
    render_pipeline_create as vulkan_render_pipeline_create,
    command_encoder_create as vulkan_command_encoder_create,
    pixels_read as vulkan_pixels_read,
    submit as vulkan_submit
  };
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
    Native( wgpu::Device ),
    /// Native Vulkan backend device — see docs/adr/004_native_vulkan_hal_backend.md.
    /// Boxed : `DeviceVulkan` embeds the instance/physical-device/logical-device
    /// handles directly, dwarfing every other variant ( `large_enum_variant` )
    /// -- unboxed, every WebGPU/WebGL/native `Device` would pay that size in
    /// padding regardless of which backend is active.
    #[ cfg( all( feature = "vulkan", not( target_arch = "wasm32" ) ) ) ]
    Vulkan( Box< DeviceVulkan > )
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
    Native( wgpu::Queue ),
    /// Native Vulkan backend queue. Boxed alongside `Device::Vulkan` -- see
    /// its doc comment ( `large_enum_variant` ); `QueueVulkan` carries a full
    /// `DeviceVulkan` clone.
    #[ cfg( all( feature = "vulkan", not( target_arch = "wasm32" ) ) ) ]
    Vulkan( Box< QueueVulkan > )
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
    /// through `pixels_read` — there is no window to present to.
    #[ cfg( all( feature = "native", not( target_arch = "wasm32" ) ) ) ]
    Native
    {
      /// Offscreen color target of the surface.
      texture : wgpu::Texture,
      /// Format the target is created with.
      format : TextureFormat
    },
    /// Native Vulkan backend surface : an offscreen render target, readable
    /// through `pixels_read` — there is no window to present to.
    #[ cfg( all( feature = "vulkan", not( target_arch = "wasm32" ) ) ) ]
    Vulkan( SurfaceVulkan )
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
      let adapter = gl::context::adapter_request().await?;
      let device = gl::context::device_request( &adapter ).await?;
      let queue = device.queue();
      let raw_format = gl::context::preferred_format()?;
      gl::context::configure( &device, &context, raw_format )?;
      let format = TextureFormat::try_from( raw_format )?;

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
    /// `Surface::pixels_read`.
    ///
    /// Synchronous : `minwgpu` blocks on the async requests internally,
    /// which is the natural shape off the browser event loop.
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidInput`] if `width` or `height` is `0`.
    /// Returns [`Error::Native`] if requesting a wgpu adapter or finishing
    /// the device context fails.
    #[ cfg( all( feature = "native", not( target_arch = "wasm32" ) ) ) ]
    pub fn new_native( width : u32, height : u32 ) -> Result< ( Device, Queue, Surface ), Error >
    {
      // Fix(BUG-199): `width`/`height` reached `wgpu::Device::create_texture`
      // unguarded -- a zero-component `Extent3d` panics outright, the same
      // defect class already fixed for `Surface::configure` (BUG-165) and
      // `Device::texture_create` (BUG-176) in this same crate, just missed
      // at this third call site. `width`/`height` are plain public `u32`
      // parameters with no caller-side guarantee of non-zero, so this is
      // reachable with entirely ordinary caller input (e.g. a size derived
      // from a not-yet-laid-out viewport or an unloaded image).
      // Root cause: no validation existed between the caller and
      // `create_texture`, unlike this file's other two texture-creation
      // paths.
      if width == 0 || height == 0
      {
        return Err( Error::InvalidInput( format!
        (
          "new_native: width and height must be non-zero, got ( {width}, {height} )"
        ) ) );
      }

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
        format : wgpu::TextureFormat::from( format ),
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

    /// Requests a native Vulkan context ( instance, physical device, logical
    /// device, one graphics queue ) directly via `minvulkan` + `ash`, and
    /// builds an offscreen rgba8 surface of the given size, readable through
    /// `Surface::pixels_read` — the Vulkan counterpart of `new_native`,
    /// deliberately independent of `wgpu`/`minwgpu` ( see
    /// docs/adr/004_native_vulkan_hal_backend.md ).
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidInput`] if `width` or `height` is `0`.
    /// Returns [`Error::Vulkan`] if creating the Vulkan instance/device or
    /// the offscreen surface fails.
    #[ cfg( all( feature = "vulkan", not( target_arch = "wasm32" ) ) ) ]
    pub fn new_vulkan( width : u32, height : u32 ) -> Result< ( Device, Queue, Surface ), Error >
    {
      // Mirrors `new_native`'s BUG-199 zero-size guard : `width`/`height`
      // are plain public `u32` parameters with no caller-side non-zero
      // guarantee, and `image_allocate`'s `VkExtent3D` carries the same
      // zero-size hazard `wgpu::Device::create_texture` panics on natively.
      if width == 0 || height == 0
      {
        return Err( Error::InvalidInput( format!
        (
          "new_vulkan: width and height must be non-zero, got ( {width}, {height} )"
        ) ) );
      }

      let ( device_vulkan, queue_vulkan, surface_vulkan ) = vulkan_handles_create( width, height )?;

      Ok
      ((
        Device::Vulkan( Box::new( device_vulkan ) ),
        Queue::Vulkan( Box::new( queue_vulkan ) ),
        Surface::Vulkan( surface_vulkan )
      ))
    }

    /// Clip-space depth range the backend's projection matrices must target.
    #[must_use]
    // `native` and `vulkan` are independent, orthogonal features -- an
    // `Self::Native( _ ) | Self::Vulkan( _ )` merged arm would fail to
    // compile whenever exactly one is enabled, since the other variant
    // would not exist on `Self` at all. The identical bodies reflect that
    // both backends target the same Vulkan/D3D/Metal-family [0,1] NDC depth
    // convention, not duplicated logic.
    #[ allow( clippy::match_same_arms, reason = "native/vulkan are independently feature-gated -- \
merging via `|` would not compile with exactly one enabled ; the shared [0,1] NDC convention is \
coincidental to the two backend families, not duplicated logic" ) ]
    pub fn depth_range( &self ) -> DepthRange
    {
      match self
      {
        #[ cfg( all( feature = "webgpu", target_arch = "wasm32" ) ) ]
        Self::WebGpu( _ ) => DepthRange::ZeroToOne,
        #[ cfg( all( feature = "webgl", target_arch = "wasm32" ) ) ]
        Self::WebGl( _ ) => DepthRange::NegOneToOne,
        #[ cfg( all( feature = "native", not( target_arch = "wasm32" ) ) ) ]
        Self::Native( _ ) => DepthRange::ZeroToOne,
        #[ cfg( all( feature = "vulkan", not( target_arch = "wasm32" ) ) ) ]
        Self::Vulkan( _ ) => DepthRange::ZeroToOne
      }
    }

    /// Creates an uninitialized buffer of `size` bytes.
    ///
    /// # Errors
    ///
    /// Returns [`Error::WebGpu`] if the underlying WebGPU buffer-creation
    /// call fails, or [`Error::WebGl`] if the WebGL context fails to
    /// allocate the buffer. The native backend never fails this call.
    pub fn buffer_create( &self, size : u64, usage : BufferUsage ) -> Result< Buffer, Error >
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
          Ok( Buffer::WebGl( BufferWebGl { buffer, target, size } ) )
        }
        #[ cfg( all( feature = "native", not( target_arch = "wasm32" ) ) ) ]
        Self::Native( device ) =>
        {
          Ok( Buffer::Native( device.create_buffer( &wgpu::BufferDescriptor
          {
            label : None,
            size,
            usage : wgpu::BufferUsages::from( usage ),
            mapped_at_creation : false
          } ) ) )
        }
        #[ cfg( all( feature = "vulkan", not( target_arch = "wasm32" ) ) ) ]
        Self::Vulkan( device_vulkan ) =>
        {
          Ok( Buffer::Vulkan( vulkan_buffer_create( device_vulkan, size, usage )? ) )
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
    pub fn buffer_init_create( &self, data : &[ u8 ], usage : BufferUsage ) -> Result< Buffer, Error >
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
          Ok( Buffer::WebGl( BufferWebGl { buffer, target, size : data.len() as u64 } ) )
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
            usage : wgpu::BufferUsages::from( usage )
          } ) ) )
        }
        #[ cfg( all( feature = "vulkan", not( target_arch = "wasm32" ) ) ) ]
        Self::Vulkan( device_vulkan ) =>
        {
          Ok( Buffer::Vulkan( vulkan_buffer_init_create( device_vulkan, data, usage )? ) )
        }
      }
    }

    /// Creates a 2d texture ( one mip, one sample ).
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidInput`] if any of `desc.size`'s three
    /// components is `0`. Returns [`Error::WebGpu`] if the underlying WebGPU
    /// texture-creation call fails. Returns [`Error::WebGl`] if
    /// `desc.format` has no WebGL internal-format mapping, or if the WebGL
    /// context fails to allocate the texture. The native backend never
    /// fails this call for reasons other than an invalid `desc.size`.
    pub fn texture_create( &self, desc : &TextureDesc ) -> Result< Texture, Error >
    {
      // Fix(BUG-176): `desc.size` reached every backend unguarded. WebGPU
      // rejects a zero-sized descriptor via an uncaught validation error;
      // WebGL's `tex_storage_2d` silently no-ops on `INVALID_VALUE` ( its
      // error is never surfaced through this Result, so the function
      // returned `Ok` for a texture that was never actually allocated );
      // native `wgpu::Device::create_texture` panics outright — the same
      // zero-size validation panic already fixed for `Surface::configure`
      // ( BUG-165 ). A live canvas can transiently report `width()`/
      // `height()` as `0` ( hidden tab, not yet laid out ), so this is
      // reachable with no malformed caller input at all.
      // Root cause: no validation existed between the caller and any of
      // the three backend-specific texture-creation calls.
      // Pitfall: this function's own doc comment claimed "the native
      // backend never fails this call" — true in the narrow sense that it
      // never returns `Err`, false in the sense that mattered: it panics.
      if desc.size[ 0 ] == 0 || desc.size[ 1 ] == 0 || desc.size[ 2 ] == 0
      {
        return Err( Error::InvalidInput( format!
        (
          "texture_create: size must be non-zero on all 3 components, got {:?}", desc.size
        ) ) );
      }

      match self
      {
        #[ cfg( all( feature = "webgpu", target_arch = "wasm32" ) ) ]
        Self::WebGpu( device ) =>
        {
          let mut builder = gl::texture::desc()
          .size( desc.size )
          .format( gl::GpuTextureFormat::from( desc.format ) );
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
            format : wgpu::TextureFormat::from( desc.format ),
            usage : wgpu::TextureUsages::from( desc.usage ),
            view_formats : &[]
          } ) ) )
        }
        #[ cfg( all( feature = "vulkan", not( target_arch = "wasm32" ) ) ) ]
        Self::Vulkan( device_vulkan ) =>
        {
          Ok( Texture::Vulkan( Box::new( vulkan_texture_create( device_vulkan, desc )? ) ) )
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
    pub fn sampler_create( &self, desc : SamplerDesc ) -> Result< Sampler, Error >
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
            address_mode_u : wgpu::AddressMode::from( desc.address ),
            address_mode_v : wgpu::AddressMode::from( desc.address ),
            address_mode_w : wgpu::AddressMode::from( desc.address ),
            mag_filter : wgpu::FilterMode::from( desc.filter ),
            min_filter : wgpu::FilterMode::from( desc.filter ),
            ..wgpu::SamplerDescriptor::default()
          } ) ) )
        }
        #[ cfg( all( feature = "vulkan", not( target_arch = "wasm32" ) ) ) ]
        Self::Vulkan( device_vulkan ) =>
        {
          Ok( Sampler::Vulkan( vulkan_sampler_create( device_vulkan, desc )? ) )
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
    pub fn shader_module_create( &self, source : &ShaderSource< '_ > ) -> Result< ShaderModule, Error >
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
        #[ cfg( all( feature = "vulkan", not( target_arch = "wasm32" ) ) ) ]
        Self::Vulkan( device_vulkan ) =>
        {
          Ok( ShaderModule::Vulkan( vulkan_shader_module_create( device_vulkan, source.wgsl )? ) )
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
    pub fn bind_group_layout_create
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
              visibility : wgpu::ShaderStages::from( entry.visibility ),
              ty : wgpu::BindingType::from( entry.ty ),
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
        #[ cfg( all( feature = "vulkan", not( target_arch = "wasm32" ) ) ) ]
        Self::Vulkan( device_vulkan ) =>
        {
          Ok( BindGroupLayout::Vulkan( vulkan_bind_group_layout_create( device_vulkan, entries )? ) )
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
    #[ allow( clippy::too_many_lines, reason = "one match arm per backend, each performing genuinely distinct resource-creation calls; splitting would scatter closely-related backend-dispatch logic across helper functions with no comprehension benefit" ) ]
    pub fn bind_group_create
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
        #[ cfg( all( feature = "vulkan", not( target_arch = "wasm32" ) ) ) ]
        Self::Vulkan( device_vulkan ) =>
        {
          Ok( BindGroup::Vulkan( vulkan_bind_group_create( device_vulkan, layout.expect_vulkan(), resources )? ) )
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
    pub fn render_pipeline_create( &self, desc : &RenderPipelineDesc< '_ > ) -> Result< RenderPipeline, Error >
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
                .format( gl::GpuVertexFormat::from( attribute.format ) )
                .offset_from_value( f64::from( attribute.offset ) )
              );
            }
            let web_layout : web_sys::GpuVertexBufferLayout = raw_layout.into();
            vertex_state = vertex_state.buffer( &web_layout );
          }

          let fragment_state = gl::FragmentState::new( shader )
          .entry_point( desc.fragment_entry )
          .target( gl::ColorTargetState::new().format( gl::GpuTextureFormat::from( desc.color_format ) ) );

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
            .depth_stencil( gl::DepthStencilState::new().format( gl::GpuTextureFormat::from( depth.format ) ) );
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
          webgl_bindings_introspect( context, &program, desc.bind_group_layouts );

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
        Self::Native( device ) => Ok( native_render_pipeline_create( device, desc ) ),
        #[ cfg( all( feature = "vulkan", not( target_arch = "wasm32" ) ) ) ]
        Self::Vulkan( device_vulkan ) =>
        {
          Ok( RenderPipeline::Vulkan( vulkan_render_pipeline_create( device_vulkan, desc )? ) )
        }
      }
    }

    /// Creates a command encoder for one frame's passes.
    ///
    /// # Panics
    ///
    /// On the Vulkan backend, panics if the underlying `vkCreateCommandPool`/
    /// `vkAllocateCommandBuffers`/`vkBeginCommandBuffer` calls fail — every
    /// other backend's own `create_command_encoder` equivalent is already
    /// infallible, so this method's signature stays infallible too rather
    /// than rippling a `Result` through every caller across the crate; see
    /// `vulkan::submit`'s own doc comment for the same tradeoff.
    #[must_use]
    pub fn command_encoder_create( &self ) -> CommandEncoder
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
        #[ cfg( all( feature = "vulkan", not( target_arch = "wasm32" ) ) ) ]
        Self::Vulkan( device_vulkan ) =>
        {
          CommandEncoder::Vulkan
          (
            Box::new
            (
              vulkan_command_encoder_create( device_vulkan )
              .unwrap_or_else( | e | panic!( "command_encoder_create: Vulkan backend failed :: {e}" ) )
            )
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
        Self::Native( raw ) => Some( raw ),
        #[ cfg( all( feature = "vulkan", not( target_arch = "wasm32" ) ) ) ]
        Self::Vulkan( _ ) => None
      }
    }

    #[ cfg( all( feature = "native", not( target_arch = "wasm32" ) ) ) ]
    pub( crate ) fn expect_native( &self ) -> &wgpu::Device
    {
      match self
      {
        Self::Native( raw ) => raw,
        #[ cfg( all( feature = "vulkan", not( target_arch = "wasm32" ) ) ) ]
        Self::Vulkan( _ ) => panic!( "expect_native called on a Device::Vulkan handle" )
      }
    }

    /// The raw Vulkan device, when the handle belongs to the Vulkan backend.
    #[ cfg( all( feature = "vulkan", not( target_arch = "wasm32" ) ) ) ]
    #[must_use]
    pub fn as_vulkan( &self ) -> Option< &DeviceVulkan >
    {
      match self
      {
        Self::Vulkan( raw ) => Some( raw ),
        #[ cfg( all( feature = "native", not( target_arch = "wasm32" ) ) ) ]
        Self::Native( _ ) => None
      }
    }

    #[ cfg( all( feature = "vulkan", not( target_arch = "wasm32" ) ) ) ]
    pub( crate ) fn expect_vulkan( &self ) -> &DeviceVulkan
    {
      match self
      {
        Self::Vulkan( raw ) => raw,
        #[ cfg( all( feature = "native", not( target_arch = "wasm32" ) ) ) ]
        Self::Native( _ ) => panic!( "expect_vulkan called on a Device::Native handle" )
      }
    }
  }

  // `expect_vulkan` is `pub( crate )`, unreachable from `tests/` ( which only
  // ever sees the public API ) — this is the one test in this crate that
  // must live beside its target instead of in `tests/`, since it exercises a
  // crate-private panic contract no external caller can reach. See task
  // 202's T04. Requires both backends compiled in, since constructing a
  // "non-vulkan-constructed `Device`" needs `Device::new_native`.
  #[ cfg( all( test, feature = "native", feature = "vulkan", not( target_arch = "wasm32" ) ) ) ]
  mod device_expect_vulkan_tests
  {
    use super::Device;

    #[ test ]
    #[ should_panic( expected = "expect_vulkan called on a Device::Native handle" ) ]
    fn expect_vulkan_panics_on_native_device()
    {
      let ( device, _queue, _surface ) = Device::new_native( 4, 4 )
      .expect( "no native wgpu adapter available" );
      let _ = device.expect_vulkan();
    }
  }

  impl Queue
  {
    /// Writes `data` into `buffer` at offset zero.
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidInput`] if `data` is longer than the WebGL
    /// backend's `buffer` was allocated with. Returns [`Error::WebGpu`] if
    /// the underlying WebGPU write call fails. The native backend never
    /// fails this call.
    #[ allow( clippy::unnecessary_wraps, reason = "fires only in single-backend builds where the surviving arm is infallible; the other backend's arm fails for real, so the signature stays fallible" ) ]
    pub fn buffer_write( &self, buffer : &Buffer, data : &[ u8 ] ) -> Result< (), Error >
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
        Self::WebGl( context ) => webgl_buffer_write( context, buffer.expect_webgl(), data ),
        #[ cfg( all( feature = "native", not( target_arch = "wasm32" ) ) ) ]
        Self::Native( queue ) =>
        {
          queue.write_buffer( buffer.expect_native(), 0, data );
          Ok( () )
        }
        #[ cfg( all( feature = "vulkan", not( target_arch = "wasm32" ) ) ) ]
        Self::Vulkan( queue_vulkan ) =>
        {
          vulkan_buffer_write( &queue_vulkan.device, buffer.expect_vulkan(), data )
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
    pub fn texture_write( &self, texture : &Texture, data : &[ u8 ] ) -> Result< (), Error >
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
          let format = TextureFormat::try_from( raw.format() )?;
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
          let depth_or_array_layers = raw.depth_or_array_layers();

          // Fix(BUG-204): `wgpu::Queue::write_texture` is itself documented
          // ( `wgpu-30.0.0/src/api/queue.rs` ) to "fail... if `data` is too
          // short" -- but the method returns `()`, not `Result`, so that
          // failure has nowhere to go except wgpu's own error sink. This
          // crate's `Device`/`Queue` never install a custom
          // `on_uncaptured_error` handler, so an undersized `data` here
          // reaches wgpu-core's `default_error_handler`
          // ( `wgpu-core-30.0.0/src/backend/wgpu_core.rs` ), which
          // unconditionally panics: "Handling wgpu errors as fatal by
          // default". Confirmed by direct inspection of both crates'
          // sources, not assumed from the panic message alone. This is the
          // same "unguarded native panic on bad input" class already fixed
          // at `Surface::configure` ( BUG-165 ), `texture_create`
          // ( BUG-176 ) and `new_native` ( BUG-199 ) in this file -- just
          // reached through a 4th call site.
          // Root cause: no validation existed between the caller's `data`
          // and wgpu's own fallible ( but `()`-returning ) write call.
          let required = u64::from( bytes_per_row ) * u64::from( height ) * u64::from( depth_or_array_layers );
          if ( data.len() as u64 ) < required
          {
            return Err( Error::InvalidInput( format!
            (
              "texture_write: data is {} bytes, but the {width}×{height} region ( {depth_or_array_layers} layer(s), \
               {bytes_per_row} bytes/row ) requires {required} bytes",
              data.len()
            ) ) );
          }

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
            wgpu::Extent3d { width, height, depth_or_array_layers }
          );
          Ok( () )
        }
        #[ cfg( all( feature = "vulkan", not( target_arch = "wasm32" ) ) ) ]
        Self::Vulkan( queue_vulkan ) =>
        {
          vulkan_texture_write( &queue_vulkan.device, queue_vulkan.queue, texture.expect_vulkan(), data )
        }
      }
    }

    /// Finishes `encoder` and submits its command buffer.
    ///
    /// # Panics
    ///
    /// Panics on a cross-backend mismatch : an `encoder` produced by one
    /// backend's `Device` ( e.g. Vulkan ) submitted through a different
    /// backend's `Queue` ( e.g. native ). Callers that always pair a
    /// `Device`/`Queue`/`CommandEncoder` from the same `Device::new_*` call
    /// never hit this.
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
        // Finishing needs ownership of the raw encoder, so each drill-down
        // happens by value here rather than through `expect_native`/
        // `expect_vulkan`.
        #[ cfg( all( feature = "native", not( target_arch = "wasm32" ) ) ) ]
        Self::Native( queue ) => native_submit( queue, encoder ),
        #[ cfg( all( feature = "vulkan", not( target_arch = "wasm32" ) ) ) ]
        Self::Vulkan( queue_vulkan ) => vulkan_queue_submit( queue_vulkan, encoder )
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
        Self::Native( raw ) => Some( raw ),
        #[ cfg( all( feature = "vulkan", not( target_arch = "wasm32" ) ) ) ]
        Self::Vulkan( _ ) => None
      }
    }

    #[ cfg( all( feature = "native", not( target_arch = "wasm32" ) ) ) ]
    pub( crate ) fn expect_native( &self ) -> &wgpu::Queue
    {
      match self
      {
        Self::Native( raw ) => raw,
        #[ cfg( all( feature = "vulkan", not( target_arch = "wasm32" ) ) ) ]
        Self::Vulkan( _ ) => panic!( "expect_native called on a Queue::Vulkan handle" )
      }
    }

    /// The raw Vulkan queue, when the handle belongs to the Vulkan backend.
    #[ cfg( all( feature = "vulkan", not( target_arch = "wasm32" ) ) ) ]
    #[must_use]
    pub fn as_vulkan( &self ) -> Option< &QueueVulkan >
    {
      match self
      {
        Self::Vulkan( raw ) => Some( raw ),
        #[ cfg( all( feature = "native", not( target_arch = "wasm32" ) ) ) ]
        Self::Native( _ ) => None
      }
    }

    #[ cfg( all( feature = "vulkan", not( target_arch = "wasm32" ) ) ) ]
    pub( crate ) fn expect_vulkan( &self ) -> &QueueVulkan
    {
      match self
      {
        Self::Vulkan( raw ) => raw,
        #[ cfg( all( feature = "native", not( target_arch = "wasm32" ) ) ) ]
        Self::Native( _ ) => panic!( "expect_vulkan called on a Queue::Native handle" )
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
        Self::Native { format, .. } => *format,
        #[ cfg( all( feature = "vulkan", not( target_arch = "wasm32" ) ) ) ]
        Self::Vulkan( surface ) => surface.format
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
        #[ cfg( all( feature = "vulkan", not( target_arch = "wasm32" ) ) ) ]
        Self::Vulkan( surface ) =>
        {
          Ok( TextureView::Vulkan( TextureViewVulkan
          {
            view : surface.view,
            format : surface.format,
            vulkan_format : surface.vulkan_format,
            size : surface.size
          } ) )
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
    /// `texture_rgba8_read`'s errors: [`Error::Unsupported`] if the
    /// surface's texture format is not `Rgba8Unorm`, or [`Error::Native`]
    /// if the GPU readback fails.
    #[ cfg_attr( all( feature = "webgpu", feature = "webgl", target_arch = "wasm32" ), expect( clippy::match_same_arms, reason = "the WebGpu and WebGl arms are gated by independent features and cannot be merged into an or-pattern without breaking single-feature builds" ) ) ]
    pub fn pixels_read( &self, device : &Device, queue : &Queue ) -> Result< Vec< u8 >, Error >
    {
      match self
      {
        #[ cfg( all( feature = "webgpu", target_arch = "wasm32" ) ) ]
        Self::WebGpu { .. } =>
        {
          let _ = ( device, queue );
          Err( Error::Unsupported
          (
            "pixels_read is a native-backend operation; browser surfaces present to their canvas".to_string()
          ) )
        }
        #[ cfg( all( feature = "webgl", target_arch = "wasm32" ) ) ]
        Self::WebGl { .. } =>
        {
          let _ = ( device, queue );
          Err( Error::Unsupported
          (
            "pixels_read is a native-backend operation; browser surfaces present to their canvas".to_string()
          ) )
        }
        #[ cfg( all( feature = "native", not( target_arch = "wasm32" ) ) ) ]
        Self::Native { texture, .. } =>
        {
          texture_rgba8_read( device.expect_native(), queue.expect_native(), texture )
        }
        #[ cfg( all( feature = "vulkan", not( target_arch = "wasm32" ) ) ) ]
        Self::Vulkan( surface ) =>
        {
          vulkan_pixels_read( device.expect_vulkan(), queue.expect_vulkan().queue, surface )
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
        Self::Native { texture, .. } => Some( texture ),
        #[ cfg( all( feature = "vulkan", not( target_arch = "wasm32" ) ) ) ]
        Self::Vulkan( _ ) => None
      }
    }

    /// The raw Vulkan surface, when the handle belongs to the Vulkan
    /// backend.
    #[ cfg( all( feature = "vulkan", not( target_arch = "wasm32" ) ) ) ]
    #[must_use]
    pub fn as_vulkan( &self ) -> Option< &SurfaceVulkan >
    {
      match self
      {
        Self::Vulkan( raw ) => Some( raw ),
        #[ cfg( all( feature = "native", not( target_arch = "wasm32" ) ) ) ]
        Self::Native { .. } => None
      }
    }
  }

  /// Resolves the binding name convention of a linked program : uniform
  /// blocks named `ub_{group}_{binding}` and sampler uniforms named
  /// `tex_{group}_{binding}` receive sequential binding points and texture
  /// units; names the linker pruned are skipped, matching GL practice for
  /// optimized-out uniforms.
  #[ cfg( all( feature = "webgl", target_arch = "wasm32" ) ) ]
  fn webgl_bindings_introspect
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
  fn native_render_pipeline_create
  (
    device : &wgpu::Device,
    desc : &RenderPipelineDesc< '_ >
  ) -> RenderPipeline
  {
    let shader = desc.shader.expect_native();

    let raw_layouts : Vec< Option< &wgpu::BindGroupLayout > > = desc.bind_group_layouts.iter()
    .map( | layout | Some( layout.expect_native() ) )
    .collect();
    let pipeline_layout = device.create_pipeline_layout( &wgpu::PipelineLayoutDescriptor
    {
      label : None,
      bind_group_layouts : &raw_layouts,
      immediate_size : 0
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
          format : wgpu::VertexFormat::from( attribute.format ),
          offset : u64::from( attribute.offset ),
          shader_location : attribute.location
        }
      )
      .collect()
    )
    .collect();
    let vertex_buffers : Vec< Option< wgpu::VertexBufferLayout< '_ > > > = desc.vertex_buffers.iter()
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
    .map( Some )
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
          format : wgpu::TextureFormat::from( depth.format ),
          depth_write_enabled : Some( true ),
          depth_compare : Some( wgpu::CompareFunction::Less ),
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
          format : wgpu::TextureFormat::from( desc.color_format ),
          blend : None,
          write_mask : wgpu::ColorWrites::ALL
        } ) ]
      } ),
      multiview_mask : None,
      cache : None
    } );
    RenderPipeline::Native( pipeline )
  }

  /// Builds a fresh Vulkan instance/device/queue/offscreen-surface set.
  #[ cfg( all( feature = "vulkan", not( target_arch = "wasm32" ) ) ) ]
  fn vulkan_handles_create( width : u32, height : u32 ) -> Result< ( DeviceVulkan, QueueVulkan, SurfaceVulkan ), Error >
  {
    let context = minvulkan::context::Context::builder()
    .instance_make()?
    .context_finish()?;

    let device_vulkan = DeviceVulkan
    {
      instance : context.instance_get().clone(),
      physical_device : context.physical_device_get(),
      device : context.device_get().clone(),
      queue_family_index : context.queue_family_index_get()
    };
    let queue_vulkan = QueueVulkan
    {
      device : device_vulkan.clone(),
      queue : context.queue_get()
    };
    let surface_vulkan = surface_create( &device_vulkan, width, height )?;

    // SAFETY: `device_vulkan`/`queue_vulkan` hold clones of `context`'s own
    // `ash::Instance`/`ash::Device` handles, not new Vulkan objects — letting
    // `context`'s `Drop` destroy the originals here would leave those clones
    // dangling. Forgetting `context` leaks it deliberately, matching every
    // other long-lived Vulkan handle this backend hands back ( see the v0
    // "no Drop-based cleanup" tradeoff documented in `vulkan.rs`'s module
    // doc comment ).
    std::mem::forget( context );

    Ok( ( device_vulkan, queue_vulkan, surface_vulkan ) )
  }

  /// Validates `data` against `raw`'s allocated size, then writes it via `bufferSubData`.
  #[ cfg( all( feature = "webgl", target_arch = "wasm32" ) ) ]
  fn webgl_buffer_write( context : &glw::GL, raw : &BufferWebGl, data : &[ u8 ] ) -> Result< (), Error >
  {
    // Fix(BUG-200): `bufferSubData` ( called via
    // `buffer_sub_data_with_i32_and_u8_array`, which returns `()` and has no
    // way to surface a GL error ) silently no-ops per the WebGL2 spec when
    // `data` would overflow the destination's allocated size — the buffer
    // keeps its old contents while this still returned `Ok(())`.
    // Root cause: no validation existed between `data` and the buffer's own
    // allocated size, and the underlying WebGL call cannot report overflow.
    if data.len() as u64 > raw.size
    {
      return Err( Error::InvalidInput( format!
      (
        "buffer_write: data is {} bytes, buffer was allocated with {} bytes",
        data.len(), raw.size
      ) ) );
    }
    context.bind_buffer( raw.target, Some( &raw.buffer ) );
    context.buffer_sub_data_with_i32_and_u8_array( raw.target, 0, data );
    Ok( () )
  }

  /// Finishes `encoder`'s raw native command buffer and submits it.
  #[ cfg( all( feature = "native", not( target_arch = "wasm32" ) ) ) ]
  fn native_submit( queue : &wgpu::Queue, encoder : CommandEncoder )
  {
    match encoder
    {
      CommandEncoder::Native( raw ) =>
      {
        queue.submit( core::iter::once( raw.finish() ) );
      }
      #[ cfg( all( feature = "vulkan", not( target_arch = "wasm32" ) ) ) ]
      CommandEncoder::Vulkan( _ ) =>
      panic!( "backend mismatch : Queue::Native received a Device::Vulkan CommandEncoder" )
    }
  }

  /// Finishes `encoder`'s raw Vulkan command buffer and submits it.
  #[ cfg( all( feature = "vulkan", not( target_arch = "wasm32" ) ) ) ]
  fn vulkan_queue_submit( queue_vulkan : &QueueVulkan, encoder : CommandEncoder )
  {
    match encoder
    {
      CommandEncoder::Vulkan( raw ) =>
      {
        // `Box<CommandEncoderVulkan>`'s deref-move lets `*raw` hand
        // `vulkan_submit` the owned value its by-value signature needs.
        vulkan_submit( &queue_vulkan.device, queue_vulkan.queue, *raw );
      }
      #[ cfg( all( feature = "native", not( target_arch = "wasm32" ) ) ) ]
      CommandEncoder::Native( _ ) =>
      panic!( "backend mismatch : Queue::Vulkan received a Device::Native CommandEncoder" )
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
