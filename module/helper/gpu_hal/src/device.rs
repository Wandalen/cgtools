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
    SurfaceVulkanWindow,
    TextureViewVulkan,
    surface_create,
    present_transition as vulkan_present_transition,
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
    buffer_destroy as vulkan_buffer_destroy,
    texture_destroy as vulkan_texture_destroy,
    texture_view_destroy as vulkan_texture_view_destroy,
    sampler_destroy as vulkan_sampler_destroy,
    shader_module_destroy as vulkan_shader_module_destroy,
    bind_group_layout_destroy as vulkan_bind_group_layout_destroy,
    bind_group_destroy as vulkan_bind_group_destroy,
    render_pipeline_destroy as vulkan_render_pipeline_destroy,
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
    /// Native backend surface presenting to a window, via a real swapchain.
    ///
    /// The windowed counterpart of [`Surface::Native`] : where that one renders
    /// offscreen and is read back with `pixels_read`, this one acquires a frame
    /// per tick through `current_view` and shows it with `present`.
    #[ cfg( all( feature = "native", not( target_arch = "wasm32" ) ) ) ]
    NativeWindow
    {
      /// Context, swapchain surface and its current configuration.
      ///
      /// Boxed for the same reason [`Surface::VulkanWindow`] is : unboxed, this
      /// one variant is several times the size of every other and would set the
      /// size of every `Surface` value, including the browser ones that carry
      /// only a handful of handles.
      windowed : Box< minwgpu::surface::Windowed< 'static > >,
      /// The frame acquired by the most recent `current_view`, awaiting
      /// `present`.
      ///
      /// A swapchain frame must be held from acquisition until presentation,
      /// but `current_view` takes `&self` on every other backend — where the
      /// canvas or offscreen texture holds itself. Interior mutability keeps
      /// that shared signature rather than forcing `&mut self` across all four
      /// backends for the one that needs it.
      acquired : core::cell::RefCell< Option< wgpu::SurfaceTexture > >,
      /// Presentation format the swapchain selected, resolved to the HAL's own
      /// vocabulary once at construction so `format` stays infallible.
      format : TextureFormat
    },
    /// Native Vulkan backend surface : an offscreen render target, readable
    /// through `pixels_read` — there is no window to present to.
    #[ cfg( all( feature = "vulkan", not( target_arch = "wasm32" ) ) ) ]
    Vulkan( SurfaceVulkan ),
    /// Native Vulkan backend surface presenting to a window, via a real
    /// `VK_KHR_swapchain`.
    ///
    /// Stands to [`Surface::Vulkan`] exactly as [`Surface::NativeWindow`]
    /// stands to [`Surface::Native`] — acquire a frame per tick through
    /// `current_view`, show it with `present` — but reaches the swapchain
    /// through `minvulkan` rather than `wgpu`, so a process using it links no
    /// `wgpu` at all.
    ///
    /// Boxed because its `minvulkan::surface::Windowed` carries whole `ash`
    /// dispatch tables inline, which would otherwise set the size of every
    /// `Surface` value including the browser ones.
    #[ cfg( all( feature = "vulkan", not( target_arch = "wasm32" ) ) ) ]
    VulkanWindow( Box< SurfaceVulkanWindow > )
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

    /// Creates a device for whichever browser backend feature is active —
    /// `webgpu` if enabled, `webgl` otherwise ( see `new_webgpu`/`new_webgl`
    /// above for what each backend requires ). Callers that don't care
    /// which backend actually ran should prefer this over naming a specific
    /// backend constructor directly.
    ///
    /// # Errors
    ///
    /// Returns whichever [`Error`] the selected backend's own constructor
    /// reports — see `new_webgpu`/`new_webgl`.
    #[ cfg( all( feature = "webgpu", target_arch = "wasm32" ) ) ]
    pub async fn new
    (
      canvas : &web_sys::HtmlCanvasElement
    ) -> Result< ( Device, Queue, Surface ), Error >
    {
      Self::new_webgpu( canvas ).await
    }

    /// See the `webgpu` arm above.
    ///
    /// # Errors
    ///
    /// Returns whichever [`Error`] `new_webgl` reports.
    #[ cfg( all( feature = "webgl", target_arch = "wasm32", not( feature = "webgpu" ) ) ) ]
    #[ allow( clippy::unused_async, reason = "keeps the same async call shape as the webgpu arm above, so callers never have to branch on which backend is active" ) ]
    pub async fn new
    (
      canvas : &web_sys::HtmlCanvasElement
    ) -> Result< ( Device, Queue, Surface ), Error >
    {
      Self::new_webgl( canvas )
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

    /// Requests a native `wgpu` context and builds a swapchain surface that
    /// presents to `window` — the windowed counterpart of [`Device::new_native`].
    ///
    /// `window` is anything `wgpu` accepts as a surface target : any type
    /// implementing both `raw_window_handle::HasWindowHandle` and
    /// `HasDisplayHandle`, such as an `Arc< winit::window::Window >`. Taking the
    /// handle traits rather than a concrete window type keeps this crate — like
    /// `minwgpu` beneath it — independent of any particular windowing library.
    ///
    /// The `'static` bound is what lets the resulting [`Surface`] stay free of a
    /// lifetime parameter, as its other three variants are; pass a shared handle
    /// ( `Arc< Window > ` ) rather than a borrow.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Native`] if surface creation, adapter selection, device
    /// request, or the initial surface configuration fails — including when
    /// `size` is zero in either dimension.
    #[ cfg( all( feature = "native", not( target_arch = "wasm32" ) ) ) ]
    pub fn new_native_windowed
    (
      window : impl Into< wgpu::SurfaceTarget< 'static > >,
      size : ( u32, u32 )
    ) -> Result< ( Device, Queue, Surface ), Error >
    {
      // `minwgpu::surface::Windowed` already validates the size, orders
      // instance -> surface -> compatible adapter -> device correctly, and picks
      // an sRGB presentation format; the zero-size guard `new_native` spells out
      // for BUG-199 lives there rather than being repeated here.
      let windowed = minwgpu::surface::Windowed::new( window, size )
      .map_err( | error | Error::Native( error.to_string() ) )?;

      let device = windowed.device_get().clone();
      let queue = windowed.queue_get().clone();
      // The swapchain picks its own presentation format ( commonly
      // `Bgra8UnormSrgb` on desktop ) -- resolve it into the HAL's vocabulary
      // once, here, so `Surface::format` can stay infallible like its siblings.
      let format = TextureFormat::try_from( windowed.format() )?;

      Ok
      ((
        Device::Native( device ),
        Queue::Native( queue ),
        Surface::NativeWindow { windowed : Box::new( windowed ), acquired : core::cell::RefCell::new( None ), format }
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

    /// Requests a native Vulkan context directly via `minvulkan` + `ash` and
    /// builds a real `VK_KHR_swapchain` over `window` at `size` — the Vulkan
    /// counterpart of `new_native_windowed`, and the only constructor whose
    /// process links no `wgpu` at all.
    ///
    /// `window` is anything implementing the `raw_window_handle` traits
    /// ( notably `winit::window::Window` ), re-exported as
    /// `minvulkan::raw_window_handle` so a caller need not name that crate in
    /// its own manifest. Taking handle traits rather than a windowing type is
    /// what keeps cgtools free of any windowing dependency — see
    /// docs/adr/005_windowed_native_presentation.md.
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidInput`] if either dimension of `size` is `0`.
    /// Returns [`Error::Vulkan`] if the window yields no usable handle, if no
    /// physical device can both render and present to the resulting surface,
    /// or if instance/device/swapchain creation fails. Returns
    /// [`Error::Unsupported`] if the swapchain's chosen presentation format
    /// has no HAL equivalent.
    #[ cfg( all( feature = "vulkan", not( target_arch = "wasm32" ) ) ) ]
    pub fn new_vulkan_windowed
    (
      window : &( impl minvulkan::raw_window_handle::HasDisplayHandle + minvulkan::raw_window_handle::HasWindowHandle ),
      size : ( u32, u32 )
    ) -> Result< ( Device, Queue, Surface ), Error >
    {
      // Mirrors `new_vulkan`'s BUG-199 zero-size guard : a zero extent reaches
      // `vkCreateSwapchainKHR` as a validation error rather than a diagnosable
      // one, and a minimized window reports exactly that.
      if size.0 == 0 || size.1 == 0
      {
        return Err( Error::InvalidInput( format!
        (
          "new_vulkan_windowed: width and height must be non-zero, got {size:?}"
        ) ) );
      }

      let windowed = minvulkan::surface::Windowed::new( window, size )
      .map_err( | error | Error::Vulkan( error.to_string() ) )?;

      let context = windowed.context_get();
      let device_vulkan = DeviceVulkan
      {
        instance : context.instance_get().clone(),
        physical_device : context.physical_device_get(),
        device : context.device_get().clone(),
        queue_family_index : context.queue_family_index_get()
      };
      let queue = context.queue_get();
      let queue_vulkan = QueueVulkan { device : device_vulkan.clone(), queue };

      // The swapchain picks its own presentation format ( commonly
      // `B8G8R8A8_SRGB` on desktop ) -- resolve it into the HAL's vocabulary
      // once, here, so `Surface::format` can stay infallible like its siblings.
      let vulkan_format = windowed.format();
      let format = TextureFormat::try_from( vulkan_format )?;

      let surface = SurfaceVulkanWindow
      {
        windowed : core::mem::ManuallyDrop::new( windowed ),
        device : device_vulkan.clone(),
        queue,
        acquired : core::cell::RefCell::new( None ),
        format,
        vulkan_format
      };

      Ok
      ((
        Device::Vulkan( Box::new( device_vulkan ) ),
        Queue::Vulkan( Box::new( queue_vulkan ) ),
        Surface::VulkanWindow( Box::new( surface ) )
      ))
    }

    /// Creates a device for whichever native backend feature is active —
    /// `native` ( wgpu ) if enabled, `vulkan` otherwise ( see
    /// `new_native`/`new_vulkan` above for what each backend requires ).
    /// Callers that don't care which backend actually ran should prefer
    /// this over naming a specific backend constructor directly.
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidInput`] if `width` or `height` is `0`.
    /// Returns whichever other [`Error`] the selected backend's own
    /// constructor reports — see `new_native`/`new_vulkan`.
    #[ cfg( all( feature = "native", not( target_arch = "wasm32" ) ) ) ]
    pub fn new( width : u32, height : u32 ) -> Result< ( Device, Queue, Surface ), Error >
    {
      Self::new_native( width, height )
    }

    /// See the `native` arm above.
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidInput`] if `width` or `height` is `0`.
    /// Returns whichever other [`Error`] `new_vulkan` reports.
    #[ cfg( all( feature = "vulkan", not( target_arch = "wasm32" ), not( feature = "native" ) ) ) ]
    pub fn new( width : u32, height : u32 ) -> Result< ( Device, Queue, Surface ), Error >
    {
      Self::new_vulkan( width, height )
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

    /// Lowercase name of the active backend — `"webgpu"`, `"webgl"`,
    /// `"native"`, or `"vulkan"`.
    #[must_use]
    pub fn backend_name( &self ) -> &'static str
    {
      match self
      {
        #[ cfg( all( feature = "webgpu", target_arch = "wasm32" ) ) ]
        Self::WebGpu( _ ) => "webgpu",
        #[ cfg( all( feature = "webgl", target_arch = "wasm32" ) ) ]
        Self::WebGl( _ ) => "webgl",
        #[ cfg( all( feature = "native", not( target_arch = "wasm32" ) ) ) ]
        Self::Native( _ ) => "native",
        #[ cfg( all( feature = "vulkan", not( target_arch = "wasm32" ) ) ) ]
        Self::Vulkan( _ ) => "vulkan"
      }
    }

    /// Creates an uninitialized buffer of `size` bytes.
    ///
    /// # Errors
    ///
    /// Returns [`Error::WebGpu`] if the underlying WebGPU buffer-creation
    /// call fails, or [`Error::WebGl`] if the WebGL context fails to
    /// allocate the buffer, or [`Error::Vulkan`] if the underlying Vulkan
    /// memory allocation or buffer creation fails. The native backend never
    /// fails this call.
    ///
    /// # Resource Lifetime
    ///
    /// The returned [`Buffer`] is never freed automatically. Call
    /// [`Device::buffer_destroy`] once it is no longer needed — on the
    /// WebGL and Vulkan backends the underlying GPU allocation otherwise
    /// leaks for the process's lifetime; on WebGPU and native ( wgpu ) it is
    /// eventually reclaimed once the handle is dropped ( JS garbage
    /// collection / Rust `Drop`, respectively ), but calling
    /// [`Device::buffer_destroy`] still releases it earlier and
    /// deterministically there too.
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
    /// allocate the buffer, or on Vulkan whichever [`Error`] the underlying
    /// `buffer_create` or `buffer_write` call reports. The native backend
    /// never fails this call.
    ///
    /// # Resource Lifetime
    ///
    /// Same contract as [`Device::buffer_create`] — call
    /// [`Device::buffer_destroy`] once the returned [`Buffer`] is no longer
    /// needed.
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
    /// context fails to allocate the texture. Returns [`Error::Vulkan`] if
    /// Vulkan format resolution or the underlying image allocation fails.
    /// The native backend never fails this call for reasons other than an
    /// invalid `desc.size`.
    ///
    /// # Resource Lifetime
    ///
    /// The returned [`Texture`] is never freed automatically. Call
    /// [`Device::texture_destroy`] once it is no longer needed — on the
    /// WebGL and Vulkan backends the underlying GPU allocation otherwise
    /// leaks for the process's lifetime; on WebGPU and native ( wgpu ) it is
    /// eventually reclaimed once the handle is dropped, but calling
    /// [`Device::texture_destroy`] still releases it earlier and
    /// deterministically there too. A [`TextureView`] built from this
    /// texture via [`Texture::view`] must be dropped ( or destroyed via
    /// [`Device::texture_view_destroy`] ) before or independently of the
    /// texture itself — see [`Texture::view`]'s own doc comment.
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
        Self::WebGpu( device ) => webgpu_texture_create( device, desc ),
        #[ cfg( all( feature = "webgl", target_arch = "wasm32" ) ) ]
        Self::WebGl( context ) => webgl_texture_create( context, desc ),
        #[ cfg( all( feature = "native", not( target_arch = "wasm32" ) ) ) ]
        Self::Native( device ) => Ok( native_texture_create( device, desc ) ),
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
    /// sampler, or [`Error::Vulkan`] if the underlying `vkCreateSampler`
    /// call fails. The WebGPU and native backends never fail this call.
    ///
    /// # Resource Lifetime
    ///
    /// The returned [`Sampler`] is never freed automatically. On the WebGL
    /// and Vulkan backends, call [`Device::sampler_destroy`] once it is no
    /// longer needed, or the underlying GPU object leaks for the process's
    /// lifetime. On WebGPU and native, destroying it is a no-op — those
    /// backends reclaim it once the handle is dropped, so
    /// [`Device::sampler_destroy`] is safe to call unconditionally.
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
        Self::WebGl( context ) => webgl_sampler_create( context, desc ),
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
    /// missing either GLSL override slot, or [`Error::Vulkan`] on the
    /// Vulkan backend if WGSL-to-SPIR-V compilation or the underlying
    /// `vkCreateShaderModule` call fails. The WebGPU and native backends
    /// never fail this call.
    ///
    /// # Resource Lifetime
    ///
    /// The returned [`ShaderModule`] is never freed automatically. On the
    /// Vulkan backend, call [`Device::shader_module_destroy`] once every
    /// [`RenderPipeline`] built from it exists, or the underlying
    /// `VkShaderModule` leaks for the process's lifetime. On WebGPU and
    /// native, destroying it is a no-op; on WebGL it compiles no GPU object
    /// at all until pipeline creation links it, so there is nothing to leak
    /// either — [`Device::shader_module_destroy`] is safe to call
    /// unconditionally on every backend.
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
    /// layout-creation call fails, or [`Error::Vulkan`] if the underlying
    /// `vkCreateDescriptorSetLayout` call fails. The WebGL and native
    /// backends never fail this call.
    ///
    /// # Resource Lifetime
    ///
    /// The returned [`BindGroupLayout`] is never freed automatically. On
    /// the Vulkan backend, call [`Device::bind_group_layout_destroy`] once
    /// every [`BindGroup`]/[`RenderPipeline`] built from it exists, or the
    /// underlying `VkDescriptorSetLayout` leaks for the process's lifetime.
    /// On WebGPU and native, destroying it is a no-op; on WebGL it is a
    /// plain CPU-side entry list with no GPU object at all — safe to call
    /// [`Device::bind_group_layout_destroy`] unconditionally on every
    /// backend.
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
        Self::WebGpu( device ) => webgpu_bind_group_layout_create( device, entries ),
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
    /// backbuffer cannot be sampled, or [`Error::Vulkan`] if the underlying
    /// descriptor pool creation or descriptor set allocation fails. The
    /// WebGPU and native backends never fail this call.
    ///
    /// # Resource Lifetime
    ///
    /// The returned [`BindGroup`] is never freed automatically. On the
    /// Vulkan backend, call [`Device::bind_group_destroy`] once it is no
    /// longer needed, or the underlying `VkDescriptorPool` leaks for the
    /// process's lifetime. On WebGPU and native, destroying it is a no-op;
    /// on WebGL its entries are clones of handles owned by the original
    /// `Buffer`/`Texture`/`Sampler` and there is nothing of its own to
    /// free — safe to call [`Device::bind_group_destroy`] unconditionally
    /// on every backend.
    #[ allow( clippy::unnecessary_wraps, reason = "fires only in single-backend builds where the surviving arm is infallible; the other backend's arm fails for real, so the signature stays fallible" ) ]
    #[ allow( unused_variables, reason = "fires only in a webgl-only build -- the WebGl arm ignores `layout` ( the GL context takes no separate layout object ), but the webgpu/native/vulkan arms all read it" ) ]
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
        Self::WebGpu( device ) => Ok( BindGroup::WebGpu( webgpu_bind_group_create( device, layout.expect_webgpu(), resources ) ) ),
        #[ cfg( all( feature = "webgl", target_arch = "wasm32" ) ) ]
        Self::WebGl( _ ) => webgl_bind_group_create( resources ),
        #[ cfg( all( feature = "native", not( target_arch = "wasm32" ) ) ) ]
        Self::Native( device ) => Ok( BindGroup::Native( native_bind_group_create( device, layout.expect_native(), resources ) ) ),
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
    /// fails to compile and link, or [`Error::Vulkan`] if an entry point
    /// name contains an interior nul byte, or if the underlying pipeline
    /// layout, render pass, or graphics pipeline creation fails. The
    /// native backend never fails this call.
    ///
    /// # Resource Lifetime
    ///
    /// The returned [`RenderPipeline`] is never freed automatically. On the
    /// Vulkan backend, call [`Device::render_pipeline_destroy`] once it is
    /// no longer needed, or the underlying `VkPipeline`/`VkPipelineLayout`
    /// leak for the process's lifetime. On WebGPU and native, destroying it
    /// is a no-op. On WebGL the pipeline shares its compiled GL program via
    /// `Rc` with any [`RenderPass`](crate::RenderPass) currently using it as
    /// the bound draw state — [`Device::render_pipeline_destroy`] only
    /// deletes the GL program once this was the last surviving reference,
    /// otherwise it just drops this one and leaves the program alive for
    /// the pass still holding it. Safe to call unconditionally on every
    /// backend.
    pub fn render_pipeline_create( &self, desc : &RenderPipelineDesc< '_ > ) -> Result< RenderPipeline, Error >
    {
      match self
      {
        #[ cfg( all( feature = "webgpu", target_arch = "wasm32" ) ) ]
        Self::WebGpu( device ) => webgpu_render_pipeline_create( device, desc ),
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

    // Fix(BUG-430): `gpu_hal` had a full resource-CREATE API ( `buffer_create`,
    // `texture_create`, `sampler_create`, `shader_module_create`,
    // `bind_group_layout_create`, `bind_group_create`, `render_pipeline_create`
    // above ) but no destroy/free counterpart on any backend -- every
    // `Buffer`/`Texture`/`TextureView`/`Sampler`/`ShaderModule`/
    // `BindGroupLayout`/`BindGroup`/`RenderPipeline` leaked unconditionally on
    // the WebGL and Vulkan backends ( Vulkan's leak was at least acknowledged
    // in `vulkan.rs`'s own module doc comment; WebGL's was completely
    // undocumented ). A real-time app allocating per-frame transient
    // resources through either backend had no way to free anything by hand
    // and would exhaust GPU memory. The 8 `Device::*_destroy` methods below
    // close that gap.
    // Root cause: the crate's v0 scope shipped resource creation before
    // resource destruction ever became a concrete, exercised need -- the
    // existing test suite is `cargo nextest`-isolated ( one process per
    // test ), so the leak never accumulated across a run and stayed
    // invisible to it.
    // Pitfall: an API surface that lets a caller allocate GPU resources but
    // never free them looks complete from the type signatures alone --
    // `Result< Buffer, Error >` gives no hint that the only way to release
    // the allocation was dropping the whole `Device`. Any future resource
    // type added to `resource.rs` needs its own `Device::*_destroy` method
    // added in the same change, not as a follow-up.
    //
    // Design: `Device::*_destroy( &self, resource : T )` methods returning
    // `()`, consuming `resource` by value -- not `impl Drop` on the resource
    // types themselves. Considered and rejected: most Vulkan wrapper structs
    // ( `BufferVulkan`, the raw `Sampler`/`ShaderModule` handles,
    // `BindGroupLayoutVulkan`, `BindGroupVulkan`, `RenderPipelineVulkan`,
    // `TextureViewVulkan` ) and most WebGL wrapper structs ( `BufferWebGl`,
    // `TextureWebGl`, the raw `WebGlSampler` handle, `RenderPipelineWebGl` )
    // carry no device/context handle of their own -- only `TextureVulkan`
    // does. A working `Drop` impl would need a device/context clone field
    // added to roughly 10 structs across `vulkan.rs`/`webgl.rs`, widening
    // every one of them and touching every construction call site, purely to
    // support a destructor. Dispatching through `Device` instead needs zero
    // struct changes : `self` already holds the backend context every
    // destroy call needs, exactly mirroring this crate's own pre-existing
    // `Queue::buffer_write`/`texture_write` dispatch idiom. Every backend's
    // destroy operation is also provably infallible per its own spec
    // ( `vkDestroy*`/`vkFree*` are void-returning; wgpu's and WebGPU's
    // `.destroy()` return `()`/`undefined`; WebGL's `gl.delete*` calls
    // return `undefined` ), so these methods return `()` rather than
    // `Result< (), Error >` -- no new `Error` variant needed. Consuming
    // `resource` by value is a deliberate safety margin beyond what any
    // backend strictly requires : since none of these types carry a `Drop`
    // impl, a caller holding onto a stale handle after an explicit destroy
    // could otherwise pass it to another HAL call and reach the driver with
    // an already-freed handle -- taking ownership here makes that a compile
    // error instead.

    /// Destroys `buffer`'s underlying GPU allocation.
    ///
    /// See [`Device::buffer_create`]'s "Resource Lifetime" section for which
    /// backends actually need this call.
    ///
    /// # Panics
    ///
    /// Panics on a cross-backend mismatch : a `buffer` produced by one
    /// backend's `Device` used to call `buffer_destroy` on a different
    /// backend's `Device`. Callers that always pair a resource with the
    /// `Device` that created it never hit this, matching `Queue::submit`'s
    /// identical guarantee.
    #[ allow( clippy::needless_pass_by_value, reason = "consuming `buffer` deliberately -- it prevents a caller from reusing a handle whose backing GPU allocation this call may have just freed, since none of the 4 resource wrapper types carry a `Drop` impl to catch that misuse at runtime instead" ) ]
    pub fn buffer_destroy( &self, buffer : Buffer )
    {
      match self
      {
        #[ cfg( all( feature = "webgpu", target_arch = "wasm32" ) ) ]
        Self::WebGpu( _ ) => buffer.expect_webgpu().destroy(),
        #[ cfg( all( feature = "webgl", target_arch = "wasm32" ) ) ]
        Self::WebGl( context ) => context.delete_buffer( Some( &buffer.expect_webgl().buffer ) ),
        #[ cfg( all( feature = "native", not( target_arch = "wasm32" ) ) ) ]
        Self::Native( _ ) => buffer.expect_native().destroy(),
        #[ cfg( all( feature = "vulkan", not( target_arch = "wasm32" ) ) ) ]
        Self::Vulkan( device_vulkan ) =>
        {
          match buffer
          {
            Buffer::Vulkan( raw ) => vulkan_buffer_destroy( device_vulkan, raw ),
            #[ cfg( all( feature = "native", not( target_arch = "wasm32" ) ) ) ]
            Buffer::Native( _ ) => panic!( "backend mismatch : Device::Vulkan received a Device::Native Buffer" )
          }
        }
      }
    }

    /// Destroys `texture`'s underlying GPU allocation.
    ///
    /// See [`Device::texture_create`]'s "Resource Lifetime" section for
    /// which backends actually need this call. Does not touch any
    /// [`TextureView`] built from `texture` via [`Texture::view`] -- those
    /// are freed independently, through [`Device::texture_view_destroy`].
    ///
    /// # Panics
    ///
    /// Panics on a cross-backend mismatch : a `texture` produced by one
    /// backend's `Device` used to call `texture_destroy` on a different
    /// backend's `Device`. Callers that always pair a resource with the
    /// `Device` that created it never hit this, matching `Queue::submit`'s
    /// identical guarantee.
    #[ allow( clippy::needless_pass_by_value, reason = "consuming `texture` deliberately -- it prevents a caller from reusing a handle whose backing GPU allocation this call may have just freed, since none of the 4 resource wrapper types carry a `Drop` impl to catch that misuse at runtime instead" ) ]
    pub fn texture_destroy( &self, texture : Texture )
    {
      match self
      {
        #[ cfg( all( feature = "webgpu", target_arch = "wasm32" ) ) ]
        Self::WebGpu( _ ) => texture.expect_webgpu().destroy(),
        #[ cfg( all( feature = "webgl", target_arch = "wasm32" ) ) ]
        Self::WebGl( context ) => context.delete_texture( Some( &texture.expect_webgl().texture ) ),
        #[ cfg( all( feature = "native", not( target_arch = "wasm32" ) ) ) ]
        Self::Native( _ ) => texture.expect_native().destroy(),
        #[ cfg( all( feature = "vulkan", not( target_arch = "wasm32" ) ) ) ]
        Self::Vulkan( _ ) =>
        {
          match texture
          {
            // `Box<TextureVulkan>`'s own compiler-blessed deref-move lets `*raw`
            // hand `vulkan_texture_destroy` the owned value its by-value
            // signature needs, straight out of the box, with no extra clone --
            // mirrors `Queue::submit`'s identical `Box<CommandEncoderVulkan>`
            // deref-move.
            Texture::Vulkan( raw ) => vulkan_texture_destroy( *raw ),
            #[ cfg( all( feature = "native", not( target_arch = "wasm32" ) ) ) ]
            Texture::Native( _ ) => panic!( "backend mismatch : Device::Vulkan received a Device::Native Texture" )
          }
        }
      }
    }

    /// Destroys `view`'s underlying GPU view object, when the backend has
    /// one of its own. Never touches the source texture or its memory.
    ///
    /// See [`Texture::view`]'s doc comment for which backends actually need
    /// this call.
    ///
    /// # Panics
    ///
    /// Panics on a cross-backend mismatch : a `view` produced by one
    /// backend's `Device` used to call `texture_view_destroy` on a
    /// different backend's `Device`. Callers that always pair a resource
    /// with the `Device` that created it never hit this -- every arm below
    /// checks `view` against its own backend even where there is nothing
    /// else to free, matching `Queue::submit`'s identical guarantee.
    #[ allow( clippy::match_same_arms, reason = "every backend but Vulkan has no separate view object to free -- WebGPU/native views are GC/Drop-managed, and the WebGL view is either a non-owning alias of the source texture's own handle or the canvas backbuffer, neither ever independently deletable; each arm is independently feature-gated, so merging them into one or-pattern would not compile whenever only some of those features are enabled" ) ]
    #[ allow( clippy::needless_pass_by_value, reason = "consumes `view` by value deliberately -- ownership-transfer is what makes an already-destroyed handle a compile error instead of a runtime hazard on reuse (see BUG-430's design rationale); the non-Vulkan arms only borrow it via `expect_*` for the cross-backend mismatch check and have nothing else to free, but every arm must still consume the same value uniformly" ) ]
    pub fn texture_view_destroy( &self, view : TextureView )
    {
      match self
      {
        #[ cfg( all( feature = "webgpu", target_arch = "wasm32" ) ) ]
        Self::WebGpu( _ ) => { view.expect_webgpu(); }
        #[ cfg( all( feature = "webgl", target_arch = "wasm32" ) ) ]
        Self::WebGl( _ ) => { view.expect_webgl(); }
        #[ cfg( all( feature = "native", not( target_arch = "wasm32" ) ) ) ]
        Self::Native( _ ) => { view.expect_native(); }
        #[ cfg( all( feature = "vulkan", not( target_arch = "wasm32" ) ) ) ]
        Self::Vulkan( device_vulkan ) => vulkan_texture_view_destroy( device_vulkan, *view.expect_vulkan() )
      }
    }

    /// Destroys `sampler`'s underlying GPU object.
    ///
    /// See [`Device::sampler_create`]'s "Resource Lifetime" section for
    /// which backends actually need this call.
    ///
    /// # Panics
    ///
    /// Panics on a cross-backend mismatch : a `sampler` produced by one
    /// backend's `Device` used to call `sampler_destroy` on a different
    /// backend's `Device`. Callers that always pair a resource with the
    /// `Device` that created it never hit this -- every arm below checks
    /// `sampler` against its own backend even where there is nothing else
    /// to free, matching `Queue::submit`'s identical guarantee.
    #[ allow( clippy::match_same_arms, reason = "WebGPU and native samplers are GC/Drop-managed with no explicit destroy in either API -- only WebGL and Vulkan actually free anything here; each arm is independently feature-gated, so merging them into one or-pattern would not compile whenever only some of those features are enabled" ) ]
    #[ allow( clippy::needless_pass_by_value, reason = "consumes `sampler` by value deliberately -- ownership-transfer is what makes an already-destroyed handle a compile error instead of a runtime hazard on reuse (see BUG-430's design rationale); the WebGPU/native arms only borrow it via `expect_*` for the cross-backend mismatch check and have nothing else to free, but every arm must still consume the same value uniformly" ) ]
    pub fn sampler_destroy( &self, sampler : Sampler )
    {
      match self
      {
        #[ cfg( all( feature = "webgpu", target_arch = "wasm32" ) ) ]
        Self::WebGpu( _ ) => { sampler.expect_webgpu(); }
        #[ cfg( all( feature = "webgl", target_arch = "wasm32" ) ) ]
        Self::WebGl( context ) => context.delete_sampler( Some( sampler.expect_webgl() ) ),
        #[ cfg( all( feature = "native", not( target_arch = "wasm32" ) ) ) ]
        Self::Native( _ ) => { sampler.expect_native(); }
        #[ cfg( all( feature = "vulkan", not( target_arch = "wasm32" ) ) ) ]
        Self::Vulkan( device_vulkan ) => vulkan_sampler_destroy( device_vulkan, *sampler.expect_vulkan() )
      }
    }

    /// Destroys `module`'s underlying GPU object, when the backend compiled
    /// one at all.
    ///
    /// See [`Device::shader_module_create`]'s "Resource Lifetime" section
    /// for which backends actually need this call.
    ///
    /// # Panics
    ///
    /// Panics on a cross-backend mismatch : a `module` produced by one
    /// backend's `Device` used to call `shader_module_destroy` on a
    /// different backend's `Device`. Callers that always pair a resource
    /// with the `Device` that created it never hit this -- every arm below
    /// checks `module` against its own backend even where there is nothing
    /// else to free, matching `Queue::submit`'s identical guarantee.
    #[ allow( clippy::match_same_arms, reason = "WebGPU/native shader modules are GC/Drop-managed, and WebGL compiles no GL object until pipeline creation links one -- only Vulkan actually frees anything here; each arm is independently feature-gated, so merging them into one or-pattern would not compile whenever only some of those features are enabled" ) ]
    #[ allow( clippy::needless_pass_by_value, reason = "consumes `module` by value deliberately -- ownership-transfer is what makes an already-destroyed handle a compile error instead of a runtime hazard on reuse (see BUG-430's design rationale); the non-Vulkan arms only borrow it via `expect_*` for the cross-backend mismatch check and have nothing else to free, but every arm must still consume the same value uniformly" ) ]
    pub fn shader_module_destroy( &self, module : ShaderModule )
    {
      match self
      {
        #[ cfg( all( feature = "webgpu", target_arch = "wasm32" ) ) ]
        Self::WebGpu( _ ) => { module.expect_webgpu(); }
        #[ cfg( all( feature = "webgl", target_arch = "wasm32" ) ) ]
        Self::WebGl( _ ) => { module.expect_webgl(); }
        #[ cfg( all( feature = "native", not( target_arch = "wasm32" ) ) ) ]
        Self::Native( _ ) => { module.expect_native(); }
        #[ cfg( all( feature = "vulkan", not( target_arch = "wasm32" ) ) ) ]
        Self::Vulkan( device_vulkan ) => vulkan_shader_module_destroy( device_vulkan, *module.expect_vulkan() )
      }
    }

    /// Destroys `layout`'s underlying GPU object, when the backend has one
    /// at all.
    ///
    /// See [`Device::bind_group_layout_create`]'s "Resource Lifetime"
    /// section for which backends actually need this call.
    ///
    /// # Panics
    ///
    /// Panics on a cross-backend mismatch : a `layout` produced by one
    /// backend's `Device` used to call `bind_group_layout_destroy` on a
    /// different backend's `Device`. Callers that always pair a resource
    /// with the `Device` that created it never hit this -- every arm below
    /// checks `layout` against its own backend even where there is nothing
    /// else to free, matching `Queue::submit`'s identical guarantee.
    #[ allow( clippy::match_same_arms, reason = "WebGPU/native bind group layouts are GC/Drop-managed, and WebGL's is a plain CPU-side entry list with no GPU object at all -- only Vulkan actually frees anything here; each arm is independently feature-gated, so merging them into one or-pattern would not compile whenever only some of those features are enabled" ) ]
    #[ allow( clippy::needless_pass_by_value, reason = "consuming `layout` deliberately -- it prevents a caller from reusing a handle whose backing GPU object this call may have just freed, since `BindGroupLayoutVulkan` carries no `Drop` impl to catch that misuse at runtime instead" ) ]
    pub fn bind_group_layout_destroy( &self, layout : BindGroupLayout )
    {
      match self
      {
        #[ cfg( all( feature = "webgpu", target_arch = "wasm32" ) ) ]
        Self::WebGpu( _ ) => { layout.expect_webgpu(); }
        #[ cfg( all( feature = "webgl", target_arch = "wasm32" ) ) ]
        Self::WebGl( _ ) => { layout.expect_webgl(); }
        #[ cfg( all( feature = "native", not( target_arch = "wasm32" ) ) ) ]
        Self::Native( _ ) => { layout.expect_native(); }
        #[ cfg( all( feature = "vulkan", not( target_arch = "wasm32" ) ) ) ]
        Self::Vulkan( device_vulkan ) =>
        {
          match layout
          {
            BindGroupLayout::Vulkan( raw ) => vulkan_bind_group_layout_destroy( device_vulkan, raw ),
            #[ cfg( all( feature = "native", not( target_arch = "wasm32" ) ) ) ]
            BindGroupLayout::Native( _ ) =>
            panic!( "backend mismatch : Device::Vulkan received a Device::Native BindGroupLayout" )
          }
        }
      }
    }

    /// Destroys `group`'s underlying GPU object, when the backend has one
    /// at all.
    ///
    /// See [`Device::bind_group_create`]'s "Resource Lifetime" section for
    /// which backends actually need this call.
    ///
    /// # Panics
    ///
    /// Panics on a cross-backend mismatch : a `group` produced by one
    /// backend's `Device` used to call `bind_group_destroy` on a different
    /// backend's `Device`. Callers that always pair a resource with the
    /// `Device` that created it never hit this -- every arm below checks
    /// `group` against its own backend even where there is nothing else to
    /// free, matching `Queue::submit`'s identical guarantee.
    #[ allow( clippy::match_same_arms, reason = "WebGPU/native bind groups are GC/Drop-managed, and WebGL's entries are clones of handles owned by the original Buffer/Texture/Sampler with nothing of their own to free -- only Vulkan actually frees anything here; each arm is independently feature-gated, so merging them into one or-pattern would not compile whenever only some of those features are enabled" ) ]
    #[ allow( clippy::needless_pass_by_value, reason = "consuming `group` deliberately -- it prevents a caller from reusing a handle whose backing GPU object this call may have just freed, since `BindGroupVulkan` carries no `Drop` impl to catch that misuse at runtime instead" ) ]
    pub fn bind_group_destroy( &self, group : BindGroup )
    {
      match self
      {
        #[ cfg( all( feature = "webgpu", target_arch = "wasm32" ) ) ]
        Self::WebGpu( _ ) => { group.expect_webgpu(); }
        #[ cfg( all( feature = "webgl", target_arch = "wasm32" ) ) ]
        Self::WebGl( _ ) => { group.expect_webgl(); }
        #[ cfg( all( feature = "native", not( target_arch = "wasm32" ) ) ) ]
        Self::Native( _ ) => { group.expect_native(); }
        #[ cfg( all( feature = "vulkan", not( target_arch = "wasm32" ) ) ) ]
        Self::Vulkan( device_vulkan ) =>
        {
          match group
          {
            BindGroup::Vulkan( raw ) => vulkan_bind_group_destroy( device_vulkan, raw ),
            #[ cfg( all( feature = "native", not( target_arch = "wasm32" ) ) ) ]
            BindGroup::Native( _ ) => panic!( "backend mismatch : Device::Vulkan received a Device::Native BindGroup" )
          }
        }
      }
    }

    /// Destroys `pipeline`'s underlying GPU object, when the backend has
    /// one at all.
    ///
    /// See [`Device::render_pipeline_create`]'s "Resource Lifetime" section
    /// for which backends actually need this call, including the WebGL
    /// `Rc`-shared-ownership caveat.
    ///
    /// # Panics
    ///
    /// Panics on a cross-backend mismatch : a `pipeline` produced by one
    /// backend's `Device` used to call `render_pipeline_destroy` on a
    /// different backend's `Device`. Callers that always pair a resource
    /// with the `Device` that created it never hit this -- every arm below
    /// checks `pipeline` against its own backend even where there is
    /// nothing else to free, matching `Queue::submit`'s identical
    /// guarantee.
    #[ allow( clippy::match_same_arms, reason = "WebGPU/native render pipelines are GC/Drop-managed -- only WebGL ( conditionally, per its Rc refcount ) and Vulkan actually free anything here; each arm is independently feature-gated, so merging them into one or-pattern would not compile whenever only some of those features are enabled" ) ]
    #[ allow( clippy::needless_pass_by_value, reason = "consuming `pipeline` deliberately -- it prevents a caller from reusing a handle whose backing GPU object this call may have just freed, since `RenderPipelineVulkan` carries no `Drop` impl to catch that misuse at runtime instead" ) ]
    pub fn render_pipeline_destroy( &self, pipeline : RenderPipeline )
    {
      match self
      {
        #[ cfg( all( feature = "webgpu", target_arch = "wasm32" ) ) ]
        Self::WebGpu( _ ) => { pipeline.expect_webgpu(); }
        #[ cfg( all( feature = "webgl", target_arch = "wasm32" ) ) ]
        Self::WebGl( context ) =>
        {
          // `RenderPipeline::WebGl` shares its compiled GL program via `Rc`
          // with any `RenderPass` currently using it as the bound draw
          // state ( see `RenderPipelineWebGl`'s own doc comment ). Deleting
          // the GL program out from under a pass still holding a clone would
          // corrupt that pass's rendering, so this only issues the real GL
          // delete when this was the last surviving reference; otherwise it
          // just drops this one clone and leaves the program alive for
          // whichever pass still holds the other one. `expect_webgl()`
          // ( not `if let` ) so a cross-backend `RenderPipeline` panics here
          // exactly like every other arm in this file, instead of silently
          // doing nothing.
          let raw = pipeline.expect_webgl();
          if std::rc::Rc::strong_count( raw ) == 1
          {
            context.delete_program( Some( &raw.program ) );
          }
        }
        #[ cfg( all( feature = "native", not( target_arch = "wasm32" ) ) ) ]
        Self::Native( _ ) => { pipeline.expect_native(); }
        #[ cfg( all( feature = "vulkan", not( target_arch = "wasm32" ) ) ) ]
        Self::Vulkan( device_vulkan ) =>
        {
          match pipeline
          {
            RenderPipeline::Vulkan( raw ) => vulkan_render_pipeline_destroy( device_vulkan, raw ),
            #[ cfg( all( feature = "native", not( target_arch = "wasm32" ) ) ) ]
            RenderPipeline::Native( _ ) =>
            panic!( "backend mismatch : Device::Vulkan received a Device::Native RenderPipeline" )
          }
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
    /// backend's `buffer` was allocated with, or — on the native backend —
    /// if `data`'s length isn't a multiple of wgpu's `COPY_BUFFER_ALIGNMENT`
    /// or overruns the buffer's own allocated size ( BUG-207 ), or — on
    /// Vulkan — if `data` is longer than the buffer's allocated size.
    /// Returns [`Error::WebGpu`] if the underlying WebGPU write call
    /// fails, or [`Error::Vulkan`] if Vulkan's underlying `vkMapMemory`
    /// call fails.
    #[ allow( clippy::unnecessary_wraps, reason = "fires only in single-backend builds where the surviving arm is infallible; the other backends' arms fail for real, so the signature stays fallible" ) ]
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
          let raw = buffer.expect_native();
          native_buffer_write_len_validate( data, raw.size() )?;
          queue.write_buffer( raw, 0, data );
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
        Self::WebGpu( queue ) => webgpu_texture_write( queue, texture, data ),
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
        Self::Native( queue ) => native_texture_write( queue, texture.expect_native(), data ),
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
    /// never hit this. On the Vulkan backend specifically, also panics if
    /// the underlying `vkEndCommandBuffer`/`vkQueueSubmit`/`vkQueueWaitIdle`
    /// calls fail — see `vulkan::submit`'s own doc comment for the same
    /// infallible-signature tradeoff `command_encoder_create` documents.
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
        Self::Native { format, .. } | Self::NativeWindow { format, .. } => *format,
        #[ cfg( all( feature = "vulkan", not( target_arch = "wasm32" ) ) ) ]
        Self::Vulkan( surface ) => surface.format,
        #[ cfg( all( feature = "vulkan", not( target_arch = "wasm32" ) ) ) ]
        Self::VulkanWindow( surface ) => surface.format
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
        #[ cfg( all( feature = "native", not( target_arch = "wasm32" ) ) ) ]
        Self::NativeWindow { windowed, acquired, .. } =>
        {
          // Reconfiguration needs `&mut`, which this shared-signature call does
          // not have, so a stale swapchain is reported rather than repaired
          // here -- `Surface::resize` is the repair, driven by the window
          // resize event a caller already handles.
          match windowed.frame_acquire().map_err( | error | Error::Native( error.to_string() ) )?
          {
            minwgpu::surface::Frame::Ready { texture, view } =>
            {
              *acquired.borrow_mut() = Some( texture );
              Ok( TextureView::Native( view ) )
            }
            minwgpu::surface::Frame::Skip | minwgpu::surface::Frame::Reconfigure =>
            {
              Err( Error::SurfaceNotReady )
            }
          }
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
        #[ cfg( all( feature = "vulkan", not( target_arch = "wasm32" ) ) ) ]
        Self::VulkanWindow( surface ) =>
        {
          // As the `NativeWindow` arm above : an out-of-date chain is reported
          // rather than repaired here, because rebuilding it needs `&mut` and
          // this shared signature has only `&self`. `Surface::resize` is the
          // repair.
          match surface.windowed.frame_acquire().map_err( | error | Error::Vulkan( error.to_string() ) )?
          {
            minvulkan::swapchain::Frame::Ready { index, view, extent, .. } =>
            {
              *surface.acquired.borrow_mut() = Some( index );
              Ok( TextureView::Vulkan( TextureViewVulkan
              {
                view,
                format : surface.format,
                vulkan_format : surface.vulkan_format,
                size : [ extent.width, extent.height ]
              } ) )
            }
            minvulkan::swapchain::Frame::Reconfigure => Err( Error::SurfaceNotReady )
          }
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
    #[ cfg_attr( all( feature = "native", feature = "vulkan", not( target_arch = "wasm32" ) ), expect( clippy::match_same_arms, reason = "the NativeWindow and VulkanWindow arms are gated by independent features and cannot be merged into an or-pattern without breaking single-feature builds" ) ) ]
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
        #[ cfg( all( feature = "native", not( target_arch = "wasm32" ) ) ) ]
        Self::NativeWindow { .. } =>
        {
          let _ = ( device, queue );
          Err( Error::Unsupported
          (
            "pixels_read is an offscreen operation; a windowed surface presents to its window".to_string()
          ) )
        }
        #[ cfg( all( feature = "vulkan", not( target_arch = "wasm32" ) ) ) ]
        Self::Vulkan( surface ) =>
        {
          vulkan_pixels_read( device.expect_vulkan(), queue.expect_vulkan().queue, surface )
        }
        #[ cfg( all( feature = "vulkan", not( target_arch = "wasm32" ) ) ) ]
        Self::VulkanWindow { .. } =>
        {
          let _ = ( device, queue );
          Err( Error::Unsupported
          (
            "pixels_read is an offscreen operation; a windowed surface presents to its window".to_string()
          ) )
        }
      }
    }

    /// Presents the frame most recently returned by [`Surface::current_view`].
    ///
    /// A no-op on every backend except the two windowed ones
    /// ( [`Surface::NativeWindow`], [`Surface::VulkanWindow`] ) : canvas and
    /// offscreen surfaces have nothing to present. Calling it without a frame
    /// in flight is also a no-op, so a render loop that skipped a tick on
    /// [`Error::SurfaceNotReady`] may call it unconditionally.
    ///
    /// # Panics
    ///
    /// On [`Surface::VulkanWindow`], panics if the present-layout transition or
    /// `vkQueuePresentKHR` reports a genuine driver failure — matching
    /// `Queue::submit`, whose signature is infallible for the same
    /// cross-backend reason and which likewise refuses to lose a driver error
    /// silently. An out-of-date swapchain is not such a failure : it is
    /// expected, ignored here, and repaired through [`Surface::resize`] once
    /// [`Surface::current_view`] reports [`Error::SurfaceNotReady`].
    #[ allow( clippy::match_same_arms, reason = "every non-windowed backend has nothing to \
present, and each arm is independently feature-gated -- merging them into one or-pattern would \
not compile whenever only some of those features are enabled" ) ]
    pub fn present( &self )
    {
      match self
      {
        #[ cfg( all( feature = "native", not( target_arch = "wasm32" ) ) ) ]
        Self::NativeWindow { windowed, acquired, .. } =>
        {
          if let Some( texture ) = acquired.borrow_mut().take()
          {
            windowed.frame_present( texture );
          }
        }
        #[ cfg( all( feature = "vulkan", not( target_arch = "wasm32" ) ) ) ]
        Self::VulkanWindow( surface ) =>
        {
          if let Some( index ) = surface.acquired.borrow_mut().take()
          {
            let image = surface.windowed.swapchain_get().images_get()[ index as usize ];
            vulkan_present_transition( &surface.device, surface.queue, image )
            .unwrap_or_else( | e | panic!( "present-layout transition failed :: {e}" ) );
            // The `bool` says the chain is out of date. Discarded deliberately :
            // the next `current_view` reports it as `SurfaceNotReady` anyway, and
            // that is the one path a caller already handles.
            surface.windowed.frame_present( index )
            .unwrap_or_else( | e | panic!( "vkQueuePresentKHR failed :: {e}" ) );
          }
        }
        #[ cfg( all( feature = "webgpu", target_arch = "wasm32" ) ) ]
        Self::WebGpu { .. } => {}
        #[ cfg( all( feature = "webgl", target_arch = "wasm32" ) ) ]
        Self::WebGl { .. } => {}
        #[ cfg( all( feature = "native", not( target_arch = "wasm32" ) ) ) ]
        Self::Native { .. } => {}
        #[ cfg( all( feature = "vulkan", not( target_arch = "wasm32" ) ) ) ]
        Self::Vulkan( _ ) => {}
      }
    }

    /// Re-applies the surface configuration at a new drawable size.
    ///
    /// Only the windowed surfaces ( [`Surface::NativeWindow`],
    /// [`Surface::VulkanWindow`] ) have a swapchain to reconfigure; every other
    /// backend ignores this. Drive it from the window resize event, and also
    /// when [`Surface::current_view`] reports [`Error::SurfaceNotReady`]
    /// persistently — that is how a lost or outdated swapchain is repaired.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Native`] or [`Error::Vulkan`] when the new size is zero
    /// in either dimension ( e.g. reported while the window is minimized ),
    /// leaving the existing configuration untouched so rendering resumes once
    /// the window returns.
    #[ allow( unused_variables, reason = "size is unused in builds without a windowed backend" ) ]
    #[ allow( clippy::match_same_arms, reason = "every non-windowed backend has nothing to \
reconfigure, and each arm is independently feature-gated -- merging them into one or-pattern \
would not compile whenever only some of those features are enabled" ) ]
    pub fn resize( &mut self, size : ( u32, u32 ) ) -> Result< (), Error >
    {
      match self
      {
        #[ cfg( all( feature = "native", not( target_arch = "wasm32" ) ) ) ]
        Self::NativeWindow { windowed, .. } =>
        {
          windowed.resize( size ).map_err( | error | Error::Native( error.to_string() ) )
        }
        #[ cfg( all( feature = "vulkan", not( target_arch = "wasm32" ) ) ) ]
        Self::VulkanWindow( surface ) =>
        {
          surface.windowed.resize( size ).map_err( | error | Error::Vulkan( error.to_string() ) )
        }
        #[ cfg( all( feature = "webgpu", target_arch = "wasm32" ) ) ]
        Self::WebGpu { .. } => Ok( () ),
        #[ cfg( all( feature = "webgl", target_arch = "wasm32" ) ) ]
        Self::WebGl { .. } => Ok( () ),
        #[ cfg( all( feature = "native", not( target_arch = "wasm32" ) ) ) ]
        Self::Native { .. } => Ok( () ),
        #[ cfg( all( feature = "vulkan", not( target_arch = "wasm32" ) ) ) ]
        Self::Vulkan( _ ) => Ok( () )
      }
    }

    /// The raw wgpu texture the surface renders into, when the handle
    /// belongs to the offscreen native backend.
    ///
    /// A [`Surface::NativeWindow`] returns `None` : its color target is a
    /// swapchain frame that exists only between acquire and present, not a
    /// persistent texture.
    #[ cfg( all( feature = "native", not( target_arch = "wasm32" ) ) ) ]
    #[must_use]
    #[ allow( clippy::match_same_arms, reason = "every non-offscreen-wgpu backend has no such \
texture, and the vulkan arms are independently feature-gated -- merging them into one or-pattern \
would not compile without the `vulkan` feature" ) ]
    pub fn as_native( &self ) -> Option< &wgpu::Texture >
    {
      match self
      {
        Self::Native { texture, .. } => Some( texture ),
        Self::NativeWindow { .. } => None,
        #[ cfg( all( feature = "vulkan", not( target_arch = "wasm32" ) ) ) ]
        Self::Vulkan( _ ) | Self::VulkanWindow( _ ) => None
      }
    }

    /// The raw offscreen Vulkan surface, when the handle belongs to the Vulkan
    /// backend.
    ///
    /// A [`Surface::VulkanWindow`] returns `None`, for the same reason
    /// [`Surface::as_native`] returns `None` for a [`Surface::NativeWindow`] :
    /// its color target is a swapchain image that exists only between acquire
    /// and present, not a persistent one. Reach its swapchain through
    /// [`Surface::as_vulkan_windowed`] instead.
    #[ cfg( all( feature = "vulkan", not( target_arch = "wasm32" ) ) ) ]
    #[must_use]
    #[ allow( clippy::match_same_arms, reason = "every non-offscreen-Vulkan backend has no such \
surface, and the wgpu arms are independently feature-gated -- merging them into one or-pattern \
would not compile without the `native` feature" ) ]
    pub fn as_vulkan( &self ) -> Option< &SurfaceVulkan >
    {
      match self
      {
        Self::Vulkan( raw ) => Some( raw ),
        Self::VulkanWindow( _ ) => None,
        #[ cfg( all( feature = "native", not( target_arch = "wasm32" ) ) ) ]
        Self::Native { .. } | Self::NativeWindow { .. } => None
      }
    }

    /// The raw windowed Vulkan surface — context, window surface and
    /// swapchain — when the handle belongs to the windowed Vulkan backend.
    #[ cfg( all( feature = "vulkan", not( target_arch = "wasm32" ) ) ) ]
    #[must_use]
    #[ allow( clippy::match_same_arms, reason = "every non-windowed-Vulkan backend has no such \
surface, and the wgpu arms are independently feature-gated -- merging them into one or-pattern \
would not compile without the `native` feature" ) ]
    pub fn as_vulkan_windowed( &self ) -> Option< &SurfaceVulkanWindow >
    {
      match self
      {
        Self::VulkanWindow( raw ) => Some( raw ),
        Self::Vulkan( _ ) => None,
        #[ cfg( all( feature = "native", not( target_arch = "wasm32" ) ) ) ]
        Self::Native { .. } | Self::NativeWindow { .. } => None
      }
    }
  }

  /// Creates a WebGPU texture from `desc`.
  #[ cfg( all( feature = "webgpu", target_arch = "wasm32" ) ) ]
  fn webgpu_texture_create( device : &web_sys::GpuDevice, desc : &TextureDesc ) -> Result< Texture, Error >
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

  /// Creates a WebGL texture from `desc`.
  #[ cfg( all( feature = "webgl", target_arch = "wasm32" ) ) ]
  fn webgl_texture_create( context : &glw::GL, desc : &TextureDesc ) -> Result< Texture, Error >
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

  /// Creates a native wgpu texture from `desc`.
  #[ cfg( all( feature = "native", not( target_arch = "wasm32" ) ) ) ]
  fn native_texture_create( device : &wgpu::Device, desc : &TextureDesc ) -> Texture
  {
    Texture::Native( device.create_texture( &wgpu::TextureDescriptor
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
    } ) )
  }

  /// Creates a WebGL sampler from `desc`.
  #[ cfg( all( feature = "webgl", target_arch = "wasm32" ) ) ]
  fn webgl_sampler_create( context : &glw::GL, desc : SamplerDesc ) -> Result< Sampler, Error >
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

  /// Creates a WebGPU bind group layout from `entries`.
  #[ cfg( all( feature = "webgpu", target_arch = "wasm32" ) ) ]
  fn webgpu_bind_group_layout_create( device : &web_sys::GpuDevice, entries : &[ BindGroupLayoutEntry ] ) -> Result< BindGroupLayout, Error >
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

  /// Creates a WebGPU bind group from `resources`, in layout entry order.
  #[ cfg( all( feature = "webgpu", target_arch = "wasm32" ) ) ]
  fn webgpu_bind_group_create( device : &web_sys::GpuDevice, raw_layout : &web_sys::GpuBindGroupLayout, resources : &[ BindingResource< '_ > ] ) -> web_sys::GpuBindGroup
  {
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
    builder.create( device )
  }

  /// Creates a WebGL bind group from `resources`. GL has no layout objects,
  /// so unlike the other backends this ignores the layout entirely.
  #[ cfg( all( feature = "webgl", target_arch = "wasm32" ) ) ]
  fn webgl_bind_group_create( resources : &[ BindingResource< '_ > ] ) -> Result< BindGroup, Error >
  {
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

  /// Creates a native wgpu bind group from `resources`, in layout entry order.
  #[ cfg( all( feature = "native", not( target_arch = "wasm32" ) ) ) ]
  fn native_bind_group_create( device : &wgpu::Device, layout : &wgpu::BindGroupLayout, resources : &[ BindingResource< '_ > ] ) -> wgpu::BindGroup
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
    device.create_bind_group( &wgpu::BindGroupDescriptor
    {
      label : None,
      layout,
      entries : &raw_entries
    } )
  }

  /// Creates a WebGPU render pipeline from `desc`.
  #[ cfg( all( feature = "webgpu", target_arch = "wasm32" ) ) ]
  fn webgpu_render_pipeline_create( device : &web_sys::GpuDevice, desc : &RenderPipelineDesc< '_ > ) -> Result< RenderPipeline, Error >
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
      raw_layout = match layout.step_mode
      {
        mingl::StepMode::Vertex => raw_layout.vertex(),
        mingl::StepMode::Instance => raw_layout.instance()
      };
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

  /// Writes `data` into a WebGPU `texture`.
  #[ cfg( all( feature = "webgpu", target_arch = "wasm32" ) ) ]
  fn webgpu_texture_write( queue : &web_sys::GpuQueue, texture : &Texture, data : &[ u8 ] ) -> Result< (), Error >
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

  /// Validates `data`'s length against wgpu's alignment requirement and
  /// `buffer_size` before a native `write_buffer` call.
  #[ cfg( all( feature = "native", not( target_arch = "wasm32" ) ) ) ]
  fn native_buffer_write_len_validate( data : &[ u8 ], buffer_size : u64 ) -> Result< (), Error >
  {
    // Fix(BUG-207): `wgpu::Queue::write_buffer` is documented
    // ( `wgpu-30.0.0/src/api/queue.rs` ) to require "written fully
    // in-bounds, that is, `offset + data.len() <= buffer.len()`" --
    // but the method returns `()`, not `Result`, so a violation has
    // nowhere to go except wgpu's own error sink. As already
    // confirmed for `texture_write` ( BUG-204 ), this crate installs
    // no custom `on_uncaptured_error` handler, so wgpu-core's
    // `validate_write_buffer_impl`
    // ( `wgpu-core-30.0.0/src/device/queue.rs` ) rejecting an
    // oversized or misaligned write reaches wgpu-core's
    // `default_error_handler`
    // ( `wgpu-core-30.0.0/src/backend/wgpu_core.rs` ), which
    // unconditionally panics -- the same "unguarded native panic on
    // bad input" class already fixed at `Surface::configure`
    // ( BUG-165 ), `texture_create` ( BUG-176 ), `new_native`
    // ( BUG-199 ) and `texture_write` ( BUG-204 ) in this file -- just
    // reached through a 5th call site.
    // Root cause: no validation existed between the caller's `data`
    // and wgpu's own fallible ( but `()`-returning ) write call.
    let len = data.len() as u64;
    if len % wgpu::COPY_BUFFER_ALIGNMENT != 0
    {
      return Err( Error::InvalidInput( format!
      (
        "buffer_write: data is {len} bytes, not a multiple of wgpu's {}-byte COPY_BUFFER_ALIGNMENT",
        wgpu::COPY_BUFFER_ALIGNMENT
      ) ) );
    }
    if len > buffer_size
    {
      return Err( Error::InvalidInput( format!
      (
        "buffer_write: data is {len} bytes, but the buffer was only allocated with {buffer_size} bytes"
      ) ) );
    }
    Ok( () )
  }

  /// Validates that `data` is large enough for a `width`×`height` native
  /// texture write with `depth_or_array_layers` layers at `bytes_per_row`
  /// stride.
  #[ cfg( all( feature = "native", not( target_arch = "wasm32" ) ) ) ]
  fn native_texture_write_data_len_validate
  (
    width : u32,
    height : u32,
    depth_or_array_layers : u32,
    bytes_per_row : u32,
    data : &[ u8 ]
  ) -> Result< (), Error >
  {
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
    Ok( () )
  }

  /// Writes `data` into a native wgpu `texture`.
  #[ cfg( all( feature = "native", not( target_arch = "wasm32" ) ) ) ]
  fn native_texture_write( queue : &wgpu::Queue, texture : &wgpu::Texture, data : &[ u8 ] ) -> Result< (), Error >
  {
    let width = texture.width();
    let height = texture.height();
    // Native queries wgpu's own authoritative format-size table
    // directly, rather than routing through `TextureFormat::
    // bytes_per_texel` — `texture.format()` is already a
    // `wgpu::TextureFormat`, so a round trip through the HAL's own
    // enum would just re-derive what wgpu already knows.
    let bytes_per_row = width * texture.format().block_copy_size( None )
    .ok_or_else( || Error::Unsupported( format!( "{:?} has no portable CPU-side texel layout", texture.format() ) ) )?;
    let depth_or_array_layers = texture.depth_or_array_layers();

    native_texture_write_data_len_validate( width, height, depth_or_array_layers, bytes_per_row, data )?;

    queue.write_texture
    (
      wgpu::TexelCopyTextureInfo
      {
        texture,
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
    let pipeline_layout = native_pipeline_layout_create( device, desc );
    // Two passes because wgpu's slot layout borrows its attribute
    // slice — the attributes must outlive the layouts referencing them.
    let attributes = native_vertex_attributes_build( desc );
    let vertex_buffers = native_vertex_buffers_build( desc, &attributes );
    let depth_stencil = native_depth_stencil_state( desc );
    let pipeline = native_pipeline_create_raw( device, desc, shader, &pipeline_layout, &vertex_buffers, depth_stencil );
    RenderPipeline::Native( pipeline )
  }

  /// Builds the pipeline layout from `desc`'s bind group layouts.
  #[ cfg( all( feature = "native", not( target_arch = "wasm32" ) ) ) ]
  fn native_pipeline_layout_create( device : &wgpu::Device, desc : &RenderPipelineDesc< '_ > ) -> wgpu::PipelineLayout
  {
    let raw_layouts : Vec< Option< &wgpu::BindGroupLayout > > = desc.bind_group_layouts.iter()
    .map( | layout | Some( layout.expect_native() ) )
    .collect();
    device.create_pipeline_layout( &wgpu::PipelineLayoutDescriptor
    {
      label : None,
      bind_group_layouts : &raw_layouts,
      immediate_size : 0
    } )
  }

  /// Converts `desc`'s per-buffer vertex attributes into wgpu's own type,
  /// one `Vec` per vertex buffer layout.
  #[ cfg( all( feature = "native", not( target_arch = "wasm32" ) ) ) ]
  fn native_vertex_attributes_build( desc : &RenderPipelineDesc< '_ > ) -> Vec< Vec< wgpu::VertexAttribute > >
  {
    desc.vertex_buffers.iter()
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
    .collect()
  }

  /// Pairs each vertex buffer layout with its own attribute slice, borrowed
  /// from `attributes` so the layouts can outlive this call.
  #[ cfg( all( feature = "native", not( target_arch = "wasm32" ) ) ) ]
  fn native_vertex_buffers_build< 'a >
  (
    desc : &RenderPipelineDesc< '_ >,
    attributes : &'a [ Vec< wgpu::VertexAttribute > ]
  ) -> Vec< Option< wgpu::VertexBufferLayout< 'a > > >
  {
    desc.vertex_buffers.iter()
    .zip( attributes )
    .map
    (
      | ( layout, attributes ) |
      wgpu::VertexBufferLayout
      {
        array_stride : u64::from( layout.stride ),
        step_mode : match layout.step_mode
        {
          mingl::StepMode::Vertex => wgpu::VertexStepMode::Vertex,
          mingl::StepMode::Instance => wgpu::VertexStepMode::Instance
        },
        attributes
      }
    )
    .map( Some )
    .collect()
  }

  /// Builds the always-on depth/stencil state, when `desc` requests a depth
  /// attachment.
  #[ cfg( all( feature = "native", not( target_arch = "wasm32" ) ) ) ]
  fn native_depth_stencil_state( desc : &RenderPipelineDesc< '_ > ) -> Option< wgpu::DepthStencilState >
  {
    desc.depth.map
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
    )
  }

  /// Assembles the final `wgpu::RenderPipeline` from its already-built parts.
  #[ cfg( all( feature = "native", not( target_arch = "wasm32" ) ) ) ]
  fn native_pipeline_create_raw
  (
    device : &wgpu::Device,
    desc : &RenderPipelineDesc< '_ >,
    shader : &wgpu::ShaderModule,
    pipeline_layout : &wgpu::PipelineLayout,
    vertex_buffers : &[ Option< wgpu::VertexBufferLayout< '_ > > ],
    depth_stencil : Option< wgpu::DepthStencilState >
  ) -> wgpu::RenderPipeline
  {
    device.create_render_pipeline( &wgpu::RenderPipelineDescriptor
    {
      label : None,
      layout : Some( pipeline_layout ),
      vertex : wgpu::VertexState
      {
        module : shader,
        entry_point : Some( desc.vertex_entry ),
        compilation_options : wgpu::PipelineCompilationOptions::default(),
        buffers : vertex_buffers
      },
      primitive : wgpu::PrimitiveState
      {
        cull_mode : if desc.cull_back { Some( wgpu::Face::Back ) } else { None },
        ..wgpu::PrimitiveState::default()
      },
      depth_stencil,
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
    } )
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
