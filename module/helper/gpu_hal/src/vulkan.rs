//!
//! Native Vulkan backend for `gpu_hal`, built directly on `ash` against a
//! `minvulkan::Context` — the `wgpu`-free counterpart to `native.rs`. Issues
//! raw Vulkan calls instead of delegating to a higher-level graphics crate;
//! see `docs/adr/004_native_vulkan_hal_backend.md`.
//!
//! Resources use dedicated ( non-suballocated ) memory: one `vkAllocateMemory`
//! call per buffer/image, no allocator crate. Buffers are uniformly
//! `HOST_VISIBLE | HOST_COHERENT` ( direct `vkMapMemory` writes, no staging
//! buffer ). Images are `DEVICE_LOCAL` with `OPTIMAL` tiling — the only
//! tiling Vulkan universally guarantees sampled-image support for — so CPU
//! upload/readback goes through a temporary staging buffer plus a one-shot
//! command buffer ( see `command_buffer_one_shot_submit` ).
//!
//! Long-lived handles handed back through the public HAL API ( buffers,
//! textures, pipelines, bind groups, the render pass/framebuffer a
//! `RenderPass` owns while recording ) carry no `Drop`-based cleanup —
//! `cargo nextest` isolates each test into its own process, so this v0
//! "minimum resource support" tradeoff never accumulates across a suite.
//! Short-lived temporaries whose GPU-completion is provably known within a
//! single call ( a one-shot command buffer's pool/fence, a staging buffer
//! after its one-shot submit returns, a throwaway compatibility render pass
//! right after pipeline creation ) are destroyed immediately instead, since
//! those genuinely could run in a tight loop within one test process.
//!

mod private
{
  // Raw Vulkan FFI backend module -- every `ash` call that touches the
  // Vulkan API is inherently unsafe ; each call site carries its own
  // `// SAFETY:` comment rather than repeating this justification at every
  // one of them. The `unsafe_code` lint allow for this module lives on the
  // `layer vulkan;` declaration in `lib.rs` -- a `#![allow]` inner attribute
  // nested this deep inside `mod_interface!`'s generated module tree does
  // not reach the lint machinery, so it must be attached as an outer
  // attribute on the layer declaration itself ( mirrors `minvulkan::lib`'s
  // identical crate-level allow ).

  use crate::
  {
    Error,
    BufferUsage,
    TextureFormat,
    TextureDesc,
    SamplerDesc,
    VertexFormat,
    IndexFormat,
    ShaderStages,
    BindingType,
    BindGroupLayout,
    BindGroupLayoutEntry,
    FilterMode,
    AddressMode,
    BindingResource,
    RenderPipelineDesc,
    VertexBufferLayout,
    ColorAttachmentDesc,
    DepthAttachmentDesc
  };

  // ============================================================================
  // Backend handle types
  // ============================================================================

  /// Raw Vulkan handles backing a `Device::Vulkan`: the instance/physical
  /// device/logical device from `minvulkan::Context`, plus the queue family
  /// index needed to create command pools.
  #[ derive( Clone ) ]
  pub struct DeviceVulkan
  {
    /// The Vulkan instance.
    pub instance : ash::Instance,
    /// The physical device the logical device was created against.
    pub physical_device : ash::vk::PhysicalDevice,
    /// The logical device every resource is created on.
    pub device : ash::Device,
    /// Queue family index the graphics queue was requested on.
    pub queue_family_index : u32
  }

  impl std::fmt::Debug for DeviceVulkan
  {
    fn fmt( &self, f : &mut std::fmt::Formatter< '_ > ) -> std::fmt::Result
    {
      f.debug_struct( "DeviceVulkan" ).finish_non_exhaustive()
    }
  }

  /// Raw Vulkan handles backing a `Queue::Vulkan`. Carries a full
  /// `DeviceVulkan` rather than just its `ash::Device` because
  /// `texture_write`'s staging-buffer allocation needs `instance`/
  /// `physical_device` too ( see `memory_type_index_find` ), and every
  /// `Queue::*` HAL method dispatches from `&self` alone, with no separate
  /// `&Device` parameter to draw one from.
  #[ derive( Clone ) ]
  pub struct QueueVulkan
  {
    /// The device this queue's commands run on.
    pub device : DeviceVulkan,
    /// The graphics queue every command buffer is submitted to.
    pub queue : ash::vk::Queue
  }

  impl std::fmt::Debug for QueueVulkan
  {
    fn fmt( &self, f : &mut std::fmt::Formatter< '_ > ) -> std::fmt::Result
    {
      f.debug_struct( "QueueVulkan" ).finish_non_exhaustive()
    }
  }

  /// Raw Vulkan handles backing a `Surface::Vulkan`: an offscreen
  /// `DEVICE_LOCAL` color image usable as both a render target and a
  /// `vkCmdCopyImageToBuffer` source ( see `pixels_read` ). The view is
  /// created once, up front — there is no swapchain, so the same view is
  /// valid for every frame. Carries the already-resolved `vulkan_format`
  /// for the same reason as `TextureViewVulkan`: `Surface::current_view` —
  /// the cross-backend method that reads it — takes no `&Device` parameter.
  #[ derive( Debug ) ]
  pub struct SurfaceVulkan
  {
    /// Offscreen color image.
    pub image : ash::vk::Image,
    /// Dedicated memory backing `image`.
    pub memory : ash::vk::DeviceMemory,
    /// Full view of `image`, valid for the surface's whole lifetime.
    pub view : ash::vk::ImageView,
    /// Format `image` was created with.
    pub format : TextureFormat,
    /// `format` already resolved to `VkFormat`.
    pub vulkan_format : ash::vk::Format,
    /// Width, height of `image`.
    pub size : [ u32 ; 2 ]
  }

  /// Raw Vulkan handles backing a `Surface::VulkanWindow`: a real
  /// `VK_KHR_swapchain` over a window, acquired per frame and presented on the
  /// context's own graphics queue.
  ///
  /// The windowed counterpart of [`SurfaceVulkan`] : where that one renders
  /// into one long-lived offscreen image read back with `pixels_read`, this
  /// one hands out a different swapchain image every frame.
  #[ derive( Debug ) ]
  pub struct SurfaceVulkanWindow
  {
    /// Context, window surface and swapchain, owned together.
    ///
    /// Held in `ManuallyDrop` so that dropping the surface never destroys the
    /// device or instance : `device` below, and the `Device`/`Queue` handed
    /// back beside this surface, hold clones of those same handles, and this
    /// backend's v0 tradeoff is to leak long-lived Vulkan objects rather than
    /// risk dangling them ( see this module's own doc comment, and
    /// `vulkan_handles_create`'s `mem::forget` for the offscreen path ).
    pub windowed : core::mem::ManuallyDrop< minvulkan::surface::Windowed >,
    /// Device handles, for the layout transition `present` records.
    pub device : DeviceVulkan,
    /// The graphics queue that transition is submitted on, and that presents.
    pub queue : ash::vk::Queue,
    /// Index of the swapchain image acquired by the most recent
    /// `current_view`, awaiting `present`.
    ///
    /// Interior mutability for the same reason as `Surface::NativeWindow`'s
    /// own `acquired` : `current_view` takes `&self` on every backend, because
    /// every other one has nothing to hold between acquire and present.
    pub acquired : core::cell::RefCell< Option< u32 > >,
    /// Presentation format the swapchain selected, in the HAL's vocabulary.
    pub format : TextureFormat,
    /// `format` still in its original `VkFormat` form.
    pub vulkan_format : ash::vk::Format
  }

  /// Raw Vulkan handles backing a `Buffer::Vulkan`: a dedicated
  /// `HOST_VISIBLE | HOST_COHERENT` allocation, written directly via
  /// `vkMapMemory` ( no staging buffer for CPU -> buffer uploads ).
  #[ derive( Debug ) ]
  pub struct BufferVulkan
  {
    /// The buffer object.
    pub buffer : ash::vk::Buffer,
    /// Dedicated memory backing `buffer`.
    pub memory : ash::vk::DeviceMemory,
    /// Size in bytes `buffer` was created with.
    pub size : u64
  }

  /// Raw Vulkan handles backing a `Texture::Vulkan`: a `DEVICE_LOCAL`,
  /// `OPTIMAL`-tiling image. Carries its own `device` and a pre-resolved
  /// `vulkan_format` because `Texture::view()` — the cross-backend method
  /// that builds a view from it — takes no `&Device` parameter of its own
  /// ( see `texture_view_create` ).
  pub struct TextureVulkan
  {
    /// The image object.
    pub image : ash::vk::Image,
    /// Dedicated memory backing `image`.
    pub memory : ash::vk::DeviceMemory,
    /// The logical device `image` was created on.
    pub device : ash::Device,
    /// Format `image` was created with.
    pub format : TextureFormat,
    /// `format` already resolved to `VkFormat` — avoids re-resolving
    /// `Depth24Plus` through `depth_format_select` on every view creation.
    pub vulkan_format : ash::vk::Format,
    /// Width, height, depth-or-layers of `image`.
    pub size : [ u32 ; 3 ]
  }

  impl std::fmt::Debug for TextureVulkan
  {
    fn fmt( &self, f : &mut std::fmt::Formatter< '_ > ) -> std::fmt::Result
    {
      f.debug_struct( "TextureVulkan" ).finish_non_exhaustive()
    }
  }

  /// Raw Vulkan handles backing a `TextureView::Vulkan`. Carries its own
  /// pixel size ( mirroring `TextureViewWebGl::Texture` ) because neither
  /// `RenderPipelineDesc` nor `RenderPass` otherwise exposes viewport
  /// dimensions — `render_pass_begin` reads it to drive
  /// `vkCmdSetViewport`/`vkCmdSetScissor` and the framebuffer's own extent.
  /// Also carries the already-resolved `vulkan_format`, since
  /// `CommandEncoder::render_pass_begin` — the cross-backend method that
  /// consumes this view — takes no `&Device` parameter to re-resolve it
  /// from `format` with.
  #[ derive( Debug, Clone, Copy ) ]
  pub struct TextureViewVulkan
  {
    /// The view object.
    pub view : ash::vk::ImageView,
    /// Format of the texture the view was created from.
    pub format : TextureFormat,
    /// `format` already resolved to `VkFormat`.
    pub vulkan_format : ash::vk::Format,
    /// Width, height of the texture the view was created from.
    pub size : [ u32 ; 2 ]
  }

  /// Raw Vulkan handles backing a `BindGroupLayout::Vulkan`. `entries` is
  /// retained because both `bind_group_create` ( building
  /// `VkWriteDescriptorSet`s ) and `render_pipeline_create` ( building
  /// `VkPipelineLayoutCreateInfo` ) need it again after layout creation.
  #[ derive( Debug, Clone ) ]
  pub struct BindGroupLayoutVulkan
  {
    /// The descriptor set layout object.
    pub layout : ash::vk::DescriptorSetLayout,
    /// Entries the layout was created from, in binding order.
    pub entries : Vec< BindGroupLayoutEntry >
  }

  /// Raw Vulkan handles backing a `BindGroup::Vulkan`: a dedicated
  /// `VkDescriptorPool` sized exactly for this group's own entries, and the
  /// one set allocated from it — avoids shared-pool growth bookkeeping.
  #[ derive( Debug ) ]
  pub struct BindGroupVulkan
  {
    /// Dedicated pool `set` was allocated from.
    pub pool : ash::vk::DescriptorPool,
    /// The descriptor set every binding was written into.
    pub set : ash::vk::DescriptorSet
  }

  /// Raw Vulkan handles backing a `RenderPipeline::Vulkan`.
  #[ derive( Debug ) ]
  pub struct RenderPipelineVulkan
  {
    /// The pipeline object.
    pub pipeline : ash::vk::Pipeline,
    /// Pipeline layout `bind_group_set` needs to bind descriptor sets.
    pub layout : ash::vk::PipelineLayout
  }

  /// Raw Vulkan handles backing a `CommandEncoder::Vulkan`: a dedicated
  /// command pool and the one primary command buffer allocated from it,
  /// already in the recording state ( `command_encoder_create` calls
  /// `vkBeginCommandBuffer` itself, so any number of render passes can be
  /// begun/ended into it before `Queue::submit` ends the recording ).
  #[ derive( Clone ) ]
  pub struct CommandEncoderVulkan
  {
    /// The logical device the encoder's pool was created on.
    pub device : ash::Device,
    /// Dedicated pool `command_buffer` was allocated from.
    pub pool : ash::vk::CommandPool,
    /// The one primary command buffer this encoder records into.
    pub command_buffer : ash::vk::CommandBuffer
  }

  impl std::fmt::Debug for CommandEncoderVulkan
  {
    fn fmt( &self, f : &mut std::fmt::Formatter< '_ > ) -> std::fmt::Result
    {
      f.debug_struct( "CommandEncoderVulkan" ).finish_non_exhaustive()
    }
  }

  /// Raw Vulkan handles backing a `RenderPass::Vulkan`: the command buffer
  /// being recorded into, the render pass/framebuffer this recording began
  /// with ( intentionally never destroyed — see the module doc comment ),
  /// and the layout of whichever pipeline `pipeline_set` bound most
  /// recently, needed by `bind_group_set`'s `vkCmdBindDescriptorSets` call.
  #[ derive( Clone ) ]
  pub struct RenderPassVulkan
  {
    /// The logical device the pass's command buffer belongs to.
    pub device : ash::Device,
    /// The command buffer this pass records into.
    pub command_buffer : ash::vk::CommandBuffer,
    /// The render pass this recording began with.
    pub render_pass : ash::vk::RenderPass,
    /// The framebuffer this recording began with.
    pub framebuffer : ash::vk::Framebuffer,
    /// Layout of the most recently bound pipeline, set by `pipeline_set`
    /// and read by `bind_group_set` — mirrors `RenderPassWebGl::
    /// current_pipeline`'s eager-state-tracking role for the same
    /// documented "`pipeline_set` precedes `bind_group_set`" ordering
    /// contract every backend of this HAL shares.
    pub current_pipeline_layout : Option< ash::vk::PipelineLayout >
  }

  impl std::fmt::Debug for RenderPassVulkan
  {
    fn fmt( &self, f : &mut std::fmt::Formatter< '_ > ) -> std::fmt::Result
    {
      f.debug_struct( "RenderPassVulkan" ).finish_non_exhaustive()
    }
  }

  // ============================================================================
  // Format / usage / flag mappings
  // ============================================================================

  /// Maps the HAL's backend-agnostic `BufferUsage` flags onto
  /// `VkBufferUsageFlags`. Unlike the native backend's `wgpu` mapping,
  /// Vulkan's bit values are not shared with WebGPU's, so this is an
  /// explicit per-bit translation rather than a bit-identical cast.
  fn buffer_usage_to_vulkan( usage : BufferUsage ) -> ash::vk::BufferUsageFlags
  {
    let mut flags = ash::vk::BufferUsageFlags::empty();
    if usage.contains( BufferUsage::COPY_DST ) { flags |= ash::vk::BufferUsageFlags::TRANSFER_DST; }
    if usage.contains( BufferUsage::INDEX ) { flags |= ash::vk::BufferUsageFlags::INDEX_BUFFER; }
    if usage.contains( BufferUsage::VERTEX ) { flags |= ash::vk::BufferUsageFlags::VERTEX_BUFFER; }
    if usage.contains( BufferUsage::UNIFORM ) { flags |= ash::vk::BufferUsageFlags::UNIFORM_BUFFER; }
    flags
  }

  /// Selects a depth format Vulkan guarantees `DEPTH_STENCIL_ATTACHMENT`
  /// optimal-tiling support for on every conformant implementation —
  /// `D32_SFLOAT` or the packed `X8_D24_UNORM_PACK32` alternative, per the
  /// spec's mandatory format support table ( neither alone is universally
  /// required, but at least one always is ).
  fn depth_format_select
  (
    instance : &ash::Instance,
    physical_device : ash::vk::PhysicalDevice
  ) -> Result< ash::vk::Format, Error >
  {
    for candidate in [ ash::vk::Format::D32_SFLOAT, ash::vk::Format::X8_D24_UNORM_PACK32 ]
    {
      // SAFETY: `physical_device` is a valid handle enumerated from this same
      // `instance` ( `DeviceVulkan` is only ever built from a live `minvulkan::Context` );
      // querying format properties performs no writes through caller-supplied pointers.
      let properties = unsafe { instance.get_physical_device_format_properties( physical_device, candidate ) };
      if properties.optimal_tiling_features.contains( ash::vk::FormatFeatureFlags::DEPTH_STENCIL_ATTACHMENT )
      {
        return Ok( candidate );
      }
    }
    Err( Error::Vulkan( "no depth format with DEPTH_STENCIL_ATTACHMENT support was found".to_string() ) )
  }

  /// Maps a HAL `TextureFormat` onto a `VkFormat`. `Depth24Plus` has no
  /// fixed Vulkan equivalent — it resolves through `depth_format_select`,
  /// which needs a live instance/physical device to query support, so this
  /// function takes them as parameters rather than folding depth into a
  /// context-free match.
  fn texture_format_to_vulkan
  (
    format : TextureFormat,
    instance : &ash::Instance,
    physical_device : ash::vk::PhysicalDevice
  ) -> Result< ash::vk::Format, Error >
  {
    match format
    {
      TextureFormat::Rgba8Unorm => Ok( ash::vk::Format::R8G8B8A8_UNORM ),
      TextureFormat::Rgba8UnormSrgb => Ok( ash::vk::Format::R8G8B8A8_SRGB ),
      TextureFormat::Bgra8Unorm => Ok( ash::vk::Format::B8G8R8A8_UNORM ),
      TextureFormat::Bgra8UnormSrgb => Ok( ash::vk::Format::B8G8R8A8_SRGB ),
      TextureFormat::Rgba16Float => Ok( ash::vk::Format::R16G16B16A16_SFLOAT ),
      TextureFormat::Depth24Plus => depth_format_select( instance, physical_device )
    }
  }

  impl TryFrom< ash::vk::Format > for TextureFormat
  {
    /// The error type returned if the conversion fails.
    type Error = Error;

    /// The HAL equivalent of a raw `VkFormat`, when the v0 surface has one.
    ///
    /// Reverse of `texture_format_to_vulkan`, minus its depth arm : depth
    /// formats are *selected* from what the device supports rather than named
    /// by a driver, so nothing ever needs converting back. Needed because a
    /// swapchain picks its own presentation format — the HAL must name
    /// whatever the driver chose, exactly as the `wgpu` backend's own
    /// `TryFrom< wgpu::TextureFormat >` does.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Unsupported`] when `format` has no equivalent in the
    /// v0 surface.
    fn try_from( format : ash::vk::Format ) -> Result< Self, Self::Error >
    {
      match format
      {
        ash::vk::Format::R8G8B8A8_UNORM => Ok( Self::Rgba8Unorm ),
        ash::vk::Format::R8G8B8A8_SRGB => Ok( Self::Rgba8UnormSrgb ),
        ash::vk::Format::B8G8R8A8_UNORM => Ok( Self::Bgra8Unorm ),
        ash::vk::Format::B8G8R8A8_SRGB => Ok( Self::Bgra8UnormSrgb ),
        ash::vk::Format::R16G16B16A16_SFLOAT => Ok( Self::Rgba16Float ),
        other => Err( Error::Unsupported( format!
        (
          "VkFormat {other:?} has no gpu_hal TextureFormat equivalent"
        ) ) )
      }
    }
  }

  /// Maps a HAL `VertexFormat` onto a `VkFormat`.
  fn vertex_format_to_vulkan( format : VertexFormat ) -> ash::vk::Format
  {
    match format
    {
      VertexFormat::Float32x2 => ash::vk::Format::R32G32_SFLOAT,
      VertexFormat::Float32x3 => ash::vk::Format::R32G32B32_SFLOAT,
      VertexFormat::Float32x4 => ash::vk::Format::R32G32B32A32_SFLOAT
    }
  }

  /// Maps a HAL `IndexFormat` onto a `VkIndexType`.
  #[ must_use ]
  pub fn index_format_to_vulkan( format : IndexFormat ) -> ash::vk::IndexType
  {
    match format
    {
      IndexFormat::Uint32 => ash::vk::IndexType::UINT32
    }
  }

  /// Maps the HAL's `ShaderStages` flags onto `VkShaderStageFlags`.
  fn shader_stages_to_vulkan( stages : ShaderStages ) -> ash::vk::ShaderStageFlags
  {
    let mut flags = ash::vk::ShaderStageFlags::empty();
    if stages.contains( ShaderStages::VERTEX ) { flags |= ash::vk::ShaderStageFlags::VERTEX; }
    if stages.contains( ShaderStages::FRAGMENT ) { flags |= ash::vk::ShaderStageFlags::FRAGMENT; }
    flags
  }

  /// Maps a HAL `BindingType` onto a `VkDescriptorType`.
  fn binding_type_to_vulkan( binding_type : BindingType ) -> ash::vk::DescriptorType
  {
    match binding_type
    {
      BindingType::UniformBuffer => ash::vk::DescriptorType::UNIFORM_BUFFER,
      BindingType::Texture => ash::vk::DescriptorType::SAMPLED_IMAGE,
      BindingType::Sampler => ash::vk::DescriptorType::SAMPLER
    }
  }

  /// Maps a HAL `FilterMode` onto a `VkFilter`.
  fn filter_mode_to_vulkan( mode : FilterMode ) -> ash::vk::Filter
  {
    match mode
    {
      FilterMode::Nearest => ash::vk::Filter::NEAREST,
      FilterMode::Linear => ash::vk::Filter::LINEAR
    }
  }

  /// Maps a HAL `AddressMode` onto a `VkSamplerAddressMode`.
  fn address_mode_to_vulkan( mode : AddressMode ) -> ash::vk::SamplerAddressMode
  {
    match mode
    {
      AddressMode::ClampToEdge => ash::vk::SamplerAddressMode::CLAMP_TO_EDGE,
      AddressMode::Repeat => ash::vk::SamplerAddressMode::REPEAT
    }
  }

  // ============================================================================
  // Memory allocation ( dedicated, non-suballocated — see module doc comment )
  // ============================================================================

  /// Scans `VkPhysicalDeviceMemoryProperties` for the first memory type
  /// allowed by `type_bits` ( `VkMemoryRequirements::memory_type_bits` )
  /// that also carries every flag in `required`.
  fn memory_type_index_find
  (
    instance : &ash::Instance,
    physical_device : ash::vk::PhysicalDevice,
    type_bits : u32,
    required : ash::vk::MemoryPropertyFlags
  ) -> Result< u32, Error >
  {
    // SAFETY: `physical_device` is a valid handle enumerated from this same `instance`;
    // this query performs no writes through caller-supplied pointers.
    let properties = unsafe { instance.get_physical_device_memory_properties( physical_device ) };
    properties.memory_types_as_slice()
    .iter()
    .enumerate()
    .position
    (
      | ( index, memory_type ) |
      ( type_bits & ( 1 << index ) ) != 0 && memory_type.property_flags.contains( required )
    )
    .map( | index | index as u32 )
    .ok_or_else( || Error::Vulkan( "no memory type satisfies the requested properties".to_string() ) )
  }

  /// Creates a `VkBuffer`, allocates a dedicated memory block satisfying
  /// `required` properties, and binds it — the shared core of every
  /// buffer-shaped allocation ( HAL buffers, and internal staging buffers
  /// used by texture upload/readback ).
  fn buffer_allocate
  (
    device_vulkan : &DeviceVulkan,
    size : u64,
    usage : ash::vk::BufferUsageFlags,
    required : ash::vk::MemoryPropertyFlags
  ) -> Result< ( ash::vk::Buffer, ash::vk::DeviceMemory ), Error >
  {
    let create_info = ash::vk::BufferCreateInfo::default()
    .size( size )
    .usage( usage )
    .sharing_mode( ash::vk::SharingMode::EXCLUSIVE );
    // SAFETY: `create_info` is fully initialized and stack-local; no custom allocator.
    let buffer = unsafe { device_vulkan.device.create_buffer( &create_info, None ) }
    .map_err( | e | Error::Vulkan( format!( "vkCreateBuffer failed :: {e}" ) ) )?;
    // SAFETY: `buffer` was just created above on this same device and is not yet bound.
    let requirements = unsafe { device_vulkan.device.get_buffer_memory_requirements( buffer ) };
    let memory_type_index = memory_type_index_find
    (
      &device_vulkan.instance,
      device_vulkan.physical_device,
      requirements.memory_type_bits,
      required
    )?;
    let allocate_info = ash::vk::MemoryAllocateInfo::default()
    .allocation_size( requirements.size )
    .memory_type_index( memory_type_index );
    // SAFETY: `allocate_info` is stack-local and its `memory_type_index` was just
    // confirmed valid for `requirements.memory_type_bits` above.
    let memory = unsafe { device_vulkan.device.allocate_memory( &allocate_info, None ) }
    .map_err( | e | Error::Vulkan( format!( "vkAllocateMemory failed :: {e}" ) ) )?;
    // SAFETY: `buffer` and `memory` were both just created on this same device, `memory`
    // is sized from `buffer`'s own requirements, and neither has been bound before.
    unsafe { device_vulkan.device.bind_buffer_memory( buffer, memory, 0 ) }
    .map_err( | e | Error::Vulkan( format!( "vkBindBufferMemory failed :: {e}" ) ) )?;
    Ok( ( buffer, memory ) )
  }

  /// Creates a `VkImage` with `OPTIMAL` tiling, allocates dedicated
  /// `DEVICE_LOCAL` memory, and binds it.
  fn image_allocate
  (
    device_vulkan : &DeviceVulkan,
    format : ash::vk::Format,
    extent : ash::vk::Extent3D,
    usage : ash::vk::ImageUsageFlags
  ) -> Result< ( ash::vk::Image, ash::vk::DeviceMemory ), Error >
  {
    let create_info = ash::vk::ImageCreateInfo::default()
    .image_type( ash::vk::ImageType::TYPE_2D )
    .format( format )
    .extent( extent )
    .mip_levels( 1 )
    .array_layers( 1 )
    .samples( ash::vk::SampleCountFlags::TYPE_1 )
    .tiling( ash::vk::ImageTiling::OPTIMAL )
    .usage( usage )
    .sharing_mode( ash::vk::SharingMode::EXCLUSIVE )
    .initial_layout( ash::vk::ImageLayout::UNDEFINED );
    // SAFETY: `create_info` is fully initialized and stack-local; no custom allocator.
    let image = unsafe { device_vulkan.device.create_image( &create_info, None ) }
    .map_err( | e | Error::Vulkan( format!( "vkCreateImage failed :: {e}" ) ) )?;
    // SAFETY: `image` was just created above on this same device and is not yet bound.
    let requirements = unsafe { device_vulkan.device.get_image_memory_requirements( image ) };
    let memory_type_index = memory_type_index_find
    (
      &device_vulkan.instance,
      device_vulkan.physical_device,
      requirements.memory_type_bits,
      ash::vk::MemoryPropertyFlags::DEVICE_LOCAL
    )?;
    let allocate_info = ash::vk::MemoryAllocateInfo::default()
    .allocation_size( requirements.size )
    .memory_type_index( memory_type_index );
    // SAFETY: `allocate_info` is stack-local and its `memory_type_index` was just
    // confirmed valid for `requirements.memory_type_bits` above.
    let memory = unsafe { device_vulkan.device.allocate_memory( &allocate_info, None ) }
    .map_err( | e | Error::Vulkan( format!( "vkAllocateMemory failed :: {e}" ) ) )?;
    // SAFETY: `image` and `memory` were both just created on this same device, `memory`
    // is sized from `image`'s own requirements, and neither has been bound before.
    unsafe { device_vulkan.device.bind_image_memory( image, memory, 0 ) }
    .map_err( | e | Error::Vulkan( format!( "vkBindImageMemory failed :: {e}" ) ) )?;
    Ok( ( image, memory ) )
  }

  // ============================================================================
  // Memory map/copy helpers ( shared by every direct CPU<->GPU-memory transfer:
  // buffer_write's own write, texture_write's staging upload, pixels_read's
  // staging readback )
  // ============================================================================

  /// Maps `memory`, copies `data` into it via `copy_nonoverlapping`, then
  /// unmaps. Caller must ensure `memory` is a live `HOST_VISIBLE |
  /// HOST_COHERENT` allocation of at least `data.len()` bytes, from this
  /// same `device`, not already mapped elsewhere.
  fn memory_write( device : &ash::Device, memory : ash::vk::DeviceMemory, data : &[ u8 ] ) -> Result< (), Error >
  {
    // SAFETY: forwarded from this function's own documented caller contract.
    let ptr = unsafe { device.map_memory( memory, 0, data.len() as u64, ash::vk::MemoryMapFlags::empty() ) }
    .map_err( | e | Error::Vulkan( format!( "vkMapMemory failed :: {e}" ) ) )?;
    // SAFETY: `ptr` is valid for `data.len()` bytes ( just mapped above ); `ptr`
    // ( driver-owned device memory ) and `data` ( a caller-owned CPU slice ) cannot
    // overlap.
    unsafe { std::ptr::copy_nonoverlapping( data.as_ptr(), ptr.cast::< u8 >(), data.len() ); }
    // SAFETY: `memory` was just mapped above by this same call.
    unsafe { device.unmap_memory( memory ); }
    Ok( () )
  }

  /// Maps `memory`, copies `size` bytes out of it into a freshly allocated
  /// `Vec`, then unmaps. Caller must ensure `memory` is a live
  /// `HOST_VISIBLE | HOST_COHERENT` allocation of at least `size` bytes,
  /// from this same `device`, not already mapped elsewhere, and already
  /// fully written ( e.g. a prior GPU copy has been fenced ).
  fn memory_read( device : &ash::Device, memory : ash::vk::DeviceMemory, size : u64 ) -> Result< Vec< u8 >, Error >
  {
    // SAFETY: forwarded from this function's own documented caller contract.
    let ptr = unsafe { device.map_memory( memory, 0, size, ash::vk::MemoryMapFlags::empty() ) }
    .map_err( | e | Error::Vulkan( format!( "vkMapMemory failed :: {e}" ) ) )?;
    let mut data = vec![ 0u8; size as usize ];
    // SAFETY: `ptr` is valid for `size` bytes ( just mapped above ); `data` was just
    // allocated with exactly `size` bytes; the two cannot overlap.
    unsafe { std::ptr::copy_nonoverlapping( ptr.cast::< u8 >(), data.as_mut_ptr(), size as usize ); }
    // SAFETY: `memory` was just mapped above by this same call.
    unsafe { device.unmap_memory( memory ); }
    Ok( data )
  }

  // ============================================================================
  // One-shot command buffer submission ( layout transitions, buffer<->image copies )
  // ============================================================================

  /// Allocates a one-shot primary command buffer from a fresh pool and
  /// begins recording — paired with the pool's own destroy once the
  /// buffer's work is submitted and complete, see
  /// `command_buffer_one_shot_submit`.
  fn command_buffer_allocate_and_begin( device_vulkan : &DeviceVulkan ) -> Result< ( ash::vk::CommandPool, ash::vk::CommandBuffer ), Error >
  {
    let pool_create_info = ash::vk::CommandPoolCreateInfo::default()
    .queue_family_index( device_vulkan.queue_family_index );
    // SAFETY: `pool_create_info` is stack-local and fully initialized.
    let pool = unsafe { device_vulkan.device.create_command_pool( &pool_create_info, None ) }
    .map_err( | e | Error::Vulkan( format!( "vkCreateCommandPool failed :: {e}" ) ) )?;

    let allocate_info = ash::vk::CommandBufferAllocateInfo::default()
    .command_pool( pool )
    .level( ash::vk::CommandBufferLevel::PRIMARY )
    .command_buffer_count( 1 );
    // SAFETY: `pool` was just created above on this same device.
    let command_buffers = unsafe { device_vulkan.device.allocate_command_buffers( &allocate_info ) }
    .map_err( | e | Error::Vulkan( format!( "vkAllocateCommandBuffers failed :: {e}" ) ) )?;
    let command_buffer = command_buffers[ 0 ];

    let begin_info = ash::vk::CommandBufferBeginInfo::default()
    .flags( ash::vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT );
    // SAFETY: `command_buffer` was just allocated above and is in the initial state.
    unsafe { device_vulkan.device.begin_command_buffer( command_buffer, &begin_info ) }
    .map_err( | e | Error::Vulkan( format!( "vkBeginCommandBuffer failed :: {e}" ) ) )?;

    Ok( ( pool, command_buffer ) )
  }

  /// Submits `command_buffer` ( already ended ) and blocks until the GPU
  /// signals completion via a temporary fence.
  fn command_buffer_submit_and_wait( device : &ash::Device, queue : ash::vk::Queue, command_buffer : ash::vk::CommandBuffer ) -> Result< (), Error >
  {
    let fence_create_info = ash::vk::FenceCreateInfo::default();
    // SAFETY: stack-local, fully initialized create info.
    let fence = unsafe { device.create_fence( &fence_create_info, None ) }
    .map_err( | e | Error::Vulkan( format!( "vkCreateFence failed :: {e}" ) ) )?;

    let command_buffers_submit = [ command_buffer ];
    let submit_info = ash::vk::SubmitInfo::default().command_buffers( &command_buffers_submit );
    // SAFETY: `command_buffer` is fully recorded ( ended by caller ); `fence` was just
    // created and is unsignaled; `queue` belongs to the same device that created every
    // handle here.
    unsafe { device.queue_submit( queue, &[ submit_info ], fence ) }
    .map_err( | e | Error::Vulkan( format!( "vkQueueSubmit failed :: {e}" ) ) )?;
    // SAFETY: `fence` was just submitted with above; waiting on it performs no writes
    // through caller-supplied pointers beyond the fence handle itself.
    unsafe { device.wait_for_fences( &[ fence ], true, u64::MAX ) }
    .map_err( | e | Error::Vulkan( format!( "vkWaitForFences failed :: {e}" ) ) )?;
    // SAFETY: the fence signaled ( wait returned Ok above ), so it's safe to destroy.
    unsafe { device.destroy_fence( fence, None ); }
    Ok( () )
  }

  fn command_buffer_one_shot_submit
  (
    device_vulkan : &DeviceVulkan,
    queue : ash::vk::Queue,
    record : impl FnOnce( ash::vk::CommandBuffer )
  ) -> Result< (), Error >
  {
    let ( pool, command_buffer ) = command_buffer_allocate_and_begin( device_vulkan )?;

    record( command_buffer );

    // SAFETY: `command_buffer` was begun immediately above with no intervening error path.
    unsafe { device_vulkan.device.end_command_buffer( command_buffer ) }
    .map_err( | e | Error::Vulkan( format!( "vkEndCommandBuffer failed :: {e}" ) ) )?;

    command_buffer_submit_and_wait( &device_vulkan.device, queue, command_buffer )?;

    // SAFETY: the submit above only returns once the fence signals, so the GPU has
    // finished with `pool`/`command_buffer`; destroying the pool implicitly frees the
    // command buffer allocated from it.
    unsafe { device_vulkan.device.destroy_command_pool( pool, None ); }
    Ok( () )
  }

  // ============================================================================
  // Shader compilation ( WGSL -> SPIR-V via naga )
  // ============================================================================

  /// Parses, validates, and translates a WGSL source string into one SPIR-V
  /// module holding every entry point naga finds — `write_vec`'s
  /// `pipeline_options: None` emits all entry points rather than one, which
  /// is exactly what a single `ShaderModule` shared by both the vertex and
  /// fragment stage needs. Entry point names survive translation verbatim,
  /// so `desc.vertex_entry`/`desc.fragment_entry` name the same functions in
  /// the resulting SPIR-V that they name in `source`.
  fn shader_compile_wgsl_to_spirv( source : &str ) -> Result< Vec< u32 >, Error >
  {
    let module = naga::front::wgsl::parse_str( source )
    .map_err( | e | Error::Vulkan( format!( "WGSL parse failed :: {e}" ) ) )?;
    let info = naga::valid::Validator::new( naga::valid::ValidationFlags::all(), naga::valid::Capabilities::all() )
    .validate( &module )
    .map_err( | e | Error::Vulkan( format!( "WGSL validation failed :: {e}" ) ) )?;
    naga::back::spv::write_vec( &module, &info, &naga::back::spv::Options::default(), None )
    .map_err( | e | Error::Vulkan( format!( "SPIR-V generation failed :: {e}" ) ) )
  }

  // ============================================================================
  // Render pass construction ( shared by pipeline creation and render-pass-begin,
  // so the two are compatible by construction — Vulkan's own requirement )
  // ============================================================================

  /// Builds the color attachment description shared by `render_pass_create`
  /// — `finalLayout` is unconditionally `TRANSFER_SRC_OPTIMAL`, see that
  /// function's own doc comment for why.
  fn color_attachment_description( format : ash::vk::Format ) -> ash::vk::AttachmentDescription
  {
    ash::vk::AttachmentDescription::default()
    .format( format )
    .samples( ash::vk::SampleCountFlags::TYPE_1 )
    .load_op( ash::vk::AttachmentLoadOp::CLEAR )
    .store_op( ash::vk::AttachmentStoreOp::STORE )
    .stencil_load_op( ash::vk::AttachmentLoadOp::DONT_CARE )
    .stencil_store_op( ash::vk::AttachmentStoreOp::DONT_CARE )
    .initial_layout( ash::vk::ImageLayout::UNDEFINED )
    .final_layout( ash::vk::ImageLayout::TRANSFER_SRC_OPTIMAL )
  }

  /// Builds the depth attachment description shared by `render_pass_create`.
  fn depth_attachment_description( format : ash::vk::Format ) -> ash::vk::AttachmentDescription
  {
    ash::vk::AttachmentDescription::default()
    .format( format )
    .samples( ash::vk::SampleCountFlags::TYPE_1 )
    .load_op( ash::vk::AttachmentLoadOp::CLEAR )
    .store_op( ash::vk::AttachmentStoreOp::DONT_CARE )
    .stencil_load_op( ash::vk::AttachmentLoadOp::DONT_CARE )
    .stencil_store_op( ash::vk::AttachmentStoreOp::DONT_CARE )
    .initial_layout( ash::vk::ImageLayout::UNDEFINED )
    .final_layout( ash::vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL )
  }

  /// Builds a `VkRenderPass` from a color format and optional depth format
  /// alone. Used identically at pipeline-creation time ( satisfying
  /// `vkCreateGraphicsPipelines`'s required render-pass parameter ) and at
  /// `render_pass_begin` time ( the actual `vkCmdBeginRenderPass` target ) —
  /// building both from the same two inputs via the same code path makes
  /// them compatible per the Vulkan spec's render pass compatibility rules,
  /// even though they are two distinct objects.
  ///
  /// The color attachment's `finalLayout` is unconditionally
  /// `TRANSFER_SRC_OPTIMAL`: `pixels_read`'s staging-buffer copy is the only
  /// consumer needing a defined final layout in this crate — there is no
  /// swapchain/present path.
  fn render_pass_create
  (
    device : &ash::Device,
    color_format : ash::vk::Format,
    depth_format : Option< ash::vk::Format >
  ) -> Result< ash::vk::RenderPass, Error >
  {
    let color_attachment = color_attachment_description( color_format );
    let color_reference = ash::vk::AttachmentReference::default()
    .attachment( 0 )
    .layout( ash::vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL );
    let color_references = [ color_reference ];

    let depth_attachment = depth_format.map( depth_attachment_description );
    let depth_reference = ash::vk::AttachmentReference::default()
    .attachment( 1 )
    .layout( ash::vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL );

    let mut subpass = ash::vk::SubpassDescription::default()
    .pipeline_bind_point( ash::vk::PipelineBindPoint::GRAPHICS )
    .color_attachments( &color_references );
    if depth_attachment.is_some()
    {
      subpass = subpass.depth_stencil_attachment( &depth_reference );
    }

    let attachments : Vec< ash::vk::AttachmentDescription > = match depth_attachment
    {
      Some( depth_attachment ) => vec![ color_attachment, depth_attachment ],
      None => vec![ color_attachment ]
    };
    let subpasses = [ subpass ];
    let create_info = ash::vk::RenderPassCreateInfo::default()
    .attachments( &attachments )
    .subpasses( &subpasses );
    // SAFETY: `create_info` borrows only stack-local values that outlive this call.
    unsafe { device.create_render_pass( &create_info, None ) }
    .map_err( | e | Error::Vulkan( format!( "vkCreateRenderPass failed :: {e}" ) ) )
  }

  // ============================================================================
  // Public resource operations — called from device.rs / resource.rs / pass.rs
  // ============================================================================

  /// Builds the offscreen render target/readback surface `Device::
  /// new_vulkan` returns: a `DEVICE_LOCAL` color image usable both as a
  /// render pass color attachment and as a `vkCmdCopyImageToBuffer` source.
  ///
  /// # Errors
  ///
  /// Returns [`Error::Vulkan`] if the underlying image allocation or view
  /// creation fails.
  pub fn surface_create( device_vulkan : &DeviceVulkan, width : u32, height : u32 ) -> Result< SurfaceVulkan, Error >
  {
    let format = TextureFormat::Rgba8Unorm;
    let vulkan_format = texture_format_to_vulkan( format, &device_vulkan.instance, device_vulkan.physical_device )?;
    let extent = ash::vk::Extent3D { width, height, depth : 1 };
    let usage = ash::vk::ImageUsageFlags::COLOR_ATTACHMENT | ash::vk::ImageUsageFlags::TRANSFER_SRC;
    let ( image, memory ) = image_allocate( device_vulkan, vulkan_format, extent, usage )?;
    let texture = TextureVulkan
    {
      image,
      memory,
      device : device_vulkan.device.clone(),
      format,
      vulkan_format,
      size : [ width, height, 1 ]
    };
    let view = texture_view_create( &texture )?;
    Ok( SurfaceVulkan { image, memory, view : view.view, format, vulkan_format, size : [ width, height ] } )
  }

  /// Creates an uninitialized buffer of `size` bytes.
  ///
  /// # Errors
  ///
  /// Returns [`Error::Vulkan`] if the underlying memory allocation or
  /// buffer creation fails.
  pub fn buffer_create( device_vulkan : &DeviceVulkan, size : u64, usage : BufferUsage ) -> Result< BufferVulkan, Error >
  {
    let vulkan_usage = buffer_usage_to_vulkan( usage );
    let required = ash::vk::MemoryPropertyFlags::HOST_VISIBLE | ash::vk::MemoryPropertyFlags::HOST_COHERENT;
    let ( buffer, memory ) = buffer_allocate( device_vulkan, size, vulkan_usage, required )?;
    Ok( BufferVulkan { buffer, memory, size } )
  }

  /// Creates a buffer initialized with `data`.
  ///
  /// # Errors
  ///
  /// Returns `Err` if the underlying [`buffer_create`] or [`buffer_write`]
  /// call fails.
  pub fn buffer_init_create( device_vulkan : &DeviceVulkan, data : &[ u8 ], usage : BufferUsage ) -> Result< BufferVulkan, Error >
  {
    let size = data.len() as u64;
    let raw = buffer_create( device_vulkan, size, usage )?;
    buffer_write( device_vulkan, &raw, data )?;
    Ok( raw )
  }

  /// Writes `data` into `buffer` at offset zero via a direct `vkMapMemory`
  /// write — every Vulkan buffer is `HOST_COHERENT`, so no explicit flush is
  /// needed for the write to become visible to the GPU.
  ///
  /// # Errors
  ///
  /// Returns [`Error::InvalidInput`] if `data` is larger than `buffer`'s
  /// allocated size, or [`Error::Vulkan`] if the underlying `vkMapMemory`
  /// call fails.
  pub fn buffer_write( device_vulkan : &DeviceVulkan, buffer : &BufferVulkan, data : &[ u8 ] ) -> Result< (), Error >
  {
    if data.len() as u64 > buffer.size
    {
      return Err( Error::InvalidInput( format!
      (
        "buffer_write: data is {} bytes, buffer was allocated with {} bytes",
        data.len(), buffer.size
      ) ) );
    }
    // SAFETY: `buffer.memory` is a live `HOST_VISIBLE | HOST_COHERENT` allocation from
    // this same device, sized for at least `buffer.size` >= `data.len()` bytes ( checked
    // above ), and this crate never keeps a buffer persistently mapped, so it cannot
    // already be mapped elsewhere.
    memory_write( &device_vulkan.device, buffer.memory, data )
  }

  /// Creates a 2d texture ( one mip, one sample ). `desc.size` is assumed
  /// already validated non-zero by the caller ( `Device::texture_create`
  /// checks this once, before dispatching to any backend ).
  ///
  /// # Errors
  ///
  /// Returns [`Error::Vulkan`] if format resolution or the underlying image
  /// allocation fails.
  pub fn texture_create( device_vulkan : &DeviceVulkan, desc : &TextureDesc ) -> Result< TextureVulkan, Error >
  {
    let vulkan_format = texture_format_to_vulkan( desc.format, &device_vulkan.instance, device_vulkan.physical_device )?;
    let mut usage = ash::vk::ImageUsageFlags::empty();
    if desc.usage.contains( crate::TextureUsage::COPY_DST ) { usage |= ash::vk::ImageUsageFlags::TRANSFER_DST; }
    if desc.usage.contains( crate::TextureUsage::TEXTURE_BINDING ) { usage |= ash::vk::ImageUsageFlags::SAMPLED; }
    if desc.usage.contains( crate::TextureUsage::RENDER_ATTACHMENT )
    {
      usage |= if desc.format == TextureFormat::Depth24Plus
      {
        ash::vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT
      }
      else
      {
        ash::vk::ImageUsageFlags::COLOR_ATTACHMENT
      };
    }
    let extent = ash::vk::Extent3D { width : desc.size[ 0 ], height : desc.size[ 1 ], depth : desc.size[ 2 ] };
    let ( image, memory ) = image_allocate( device_vulkan, vulkan_format, extent, usage )?;
    Ok( TextureVulkan
    {
      image,
      memory,
      device : device_vulkan.device.clone(),
      format : desc.format,
      vulkan_format,
      size : desc.size
    } )
  }

  /// Validates that `data` covers `texture`'s full extent and returns the
  /// required byte count ( `width * height * depth * bytes_per_texel` ).
  fn texture_write_data_len_validate( texture : &TextureVulkan, data : &[ u8 ] ) -> Result< u64, Error >
  {
    let bytes_per_texel = texture.format.bytes_per_texel()?;
    let width = texture.size[ 0 ];
    let height = texture.size[ 1 ];
    let depth = texture.size[ 2 ];
    let required = u64::from( width ) * u64::from( height ) * u64::from( depth ) * u64::from( bytes_per_texel );
    if ( data.len() as u64 ) < required
    {
      return Err( Error::InvalidInput( format!
      (
        "texture_write: data is {} bytes, but the {width}x{height}x{depth} region \
         ( {bytes_per_texel} bytes/texel ) requires {required} bytes",
        data.len()
      ) ) );
    }
    Ok( required )
  }

  /// Builds an image memory barrier transitioning `image` between two
  /// layouts — shared by every barrier `texture_write` records.
  fn image_layout_barrier
  (
    image : ash::vk::Image,
    subresource_range : ash::vk::ImageSubresourceRange,
    old_layout : ash::vk::ImageLayout,
    new_layout : ash::vk::ImageLayout,
    src_access : ash::vk::AccessFlags,
    dst_access : ash::vk::AccessFlags
  ) -> ash::vk::ImageMemoryBarrier< 'static >
  {
    ash::vk::ImageMemoryBarrier::default()
    .old_layout( old_layout )
    .new_layout( new_layout )
    .src_access_mask( src_access )
    .dst_access_mask( dst_access )
    .image( image )
    .subresource_range( subresource_range )
  }

  /// Builds the copy region and subresource range describing a full,
  /// single-mip, single-layer upload of a `width x height x depth` image —
  /// shared by `texture_write`'s copy-recording and barrier setup.
  fn texture_write_copy_region_build( width : u32, height : u32, depth : u32 ) -> ( ash::vk::BufferImageCopy, ash::vk::ImageSubresourceRange )
  {
    let subresource = ash::vk::ImageSubresourceLayers::default()
    .aspect_mask( ash::vk::ImageAspectFlags::COLOR )
    .mip_level( 0 )
    .base_array_layer( 0 )
    .layer_count( 1 );
    let region = ash::vk::BufferImageCopy::default()
    .buffer_offset( 0 )
    .buffer_row_length( 0 )
    .buffer_image_height( 0 )
    .image_subresource( subresource )
    .image_offset( ash::vk::Offset3D::default() )
    .image_extent( ash::vk::Extent3D { width, height, depth } );
    let subresource_range = ash::vk::ImageSubresourceRange::default()
    .aspect_mask( ash::vk::ImageAspectFlags::COLOR )
    .base_mip_level( 0 )
    .level_count( 1 )
    .base_array_layer( 0 )
    .layer_count( 1 );
    ( region, subresource_range )
  }

  /// Records the transfer-dst barrier, buffer-to-image copy, and
  /// shader-read-only barrier that upload `staging_buffer`'s contents into
  /// `image` — the recording closure `texture_write` hands to
  /// `command_buffer_one_shot_submit`.
  fn texture_write_record_copy
  (
    device : &ash::Device,
    command_buffer : ash::vk::CommandBuffer,
    image : ash::vk::Image,
    staging_buffer : ash::vk::Buffer,
    region : ash::vk::BufferImageCopy,
    subresource_range : ash::vk::ImageSubresourceRange
  )
  {
    let to_transfer_dst = image_layout_barrier
    (
      image, subresource_range,
      ash::vk::ImageLayout::UNDEFINED, ash::vk::ImageLayout::TRANSFER_DST_OPTIMAL,
      ash::vk::AccessFlags::empty(), ash::vk::AccessFlags::TRANSFER_WRITE
    );
    let to_shader_read = image_layout_barrier
    (
      image, subresource_range,
      ash::vk::ImageLayout::TRANSFER_DST_OPTIMAL, ash::vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
      ash::vk::AccessFlags::TRANSFER_WRITE, ash::vk::AccessFlags::SHADER_READ
    );
    // SAFETY: `command_buffer` is currently recording ( this runs inside the closure
    // `command_buffer_one_shot_submit` invokes between its own begin/end calls );
    // `image` and `staging_buffer` are both live objects from this same device.
    unsafe
    {
      device.cmd_pipeline_barrier
      (
        command_buffer, ash::vk::PipelineStageFlags::TOP_OF_PIPE, ash::vk::PipelineStageFlags::TRANSFER,
        ash::vk::DependencyFlags::empty(), &[], &[], &[ to_transfer_dst ]
      );
      device.cmd_copy_buffer_to_image
      (
        command_buffer, staging_buffer, image, ash::vk::ImageLayout::TRANSFER_DST_OPTIMAL, &[ region ]
      );
      device.cmd_pipeline_barrier
      (
        command_buffer, ash::vk::PipelineStageFlags::TRANSFER, ash::vk::PipelineStageFlags::FRAGMENT_SHADER,
        ash::vk::DependencyFlags::empty(), &[], &[], &[ to_shader_read ]
      );
    }
  }

  /// Uploads `data` into `texture` via a `TRANSFER_SRC` staging buffer and a
  /// one-shot command buffer that copies it into the ( `DEVICE_LOCAL` )
  /// image, transitioning the image from `UNDEFINED` to
  /// `SHADER_READ_ONLY_OPTIMAL` in the process.
  ///
  /// # Errors
  ///
  /// Returns [`Error::InvalidInput`] if `data` is smaller than `texture`'s
  /// full extent requires, or [`Error::Vulkan`] if any underlying Vulkan
  /// call ( staging allocation, mapping, command submission ) fails.
  pub fn texture_write
  (
    device_vulkan : &DeviceVulkan,
    queue : ash::vk::Queue,
    texture : &TextureVulkan,
    data : &[ u8 ]
  ) -> Result< (), Error >
  {
    let required = texture_write_data_len_validate( texture, data )?;
    let width = texture.size[ 0 ];
    let height = texture.size[ 1 ];
    let depth = texture.size[ 2 ];

    let staging_usage = ash::vk::BufferUsageFlags::TRANSFER_SRC;
    let staging_required = ash::vk::MemoryPropertyFlags::HOST_VISIBLE | ash::vk::MemoryPropertyFlags::HOST_COHERENT;
    let ( staging_buffer, staging_memory ) = buffer_allocate( device_vulkan, required, staging_usage, staging_required )?;
    // SAFETY: `staging_memory` was just allocated above as `HOST_VISIBLE`, sized for
    // exactly `required` bytes, and is not mapped elsewhere; `data` covers at least
    // `required` bytes ( checked above ).
    memory_write( &device_vulkan.device, staging_memory, &data[ .. required as usize ] )?;

    let ( region, subresource_range ) = texture_write_copy_region_build( width, height, depth );

    command_buffer_one_shot_submit
    (
      device_vulkan,
      queue,
      | command_buffer |
      texture_write_record_copy( &device_vulkan.device, command_buffer, texture.image, staging_buffer, region, subresource_range )
    )?;

    // SAFETY: `command_buffer_one_shot_submit` above only returns once its internal
    // fence confirms the GPU has finished all recorded work, so `staging_buffer`/
    // `staging_memory` are provably no longer in use.
    unsafe
    {
      device_vulkan.device.destroy_buffer( staging_buffer, None );
      device_vulkan.device.free_memory( staging_memory, None );
    }
    Ok( () )
  }

  /// Creates a full view of `texture` — aspect is derived from its format
  /// ( `DEPTH` for `Depth24Plus`, `COLOR` otherwise ). Takes no separate
  /// `&DeviceVulkan` parameter because `Texture::view()`, the cross-backend
  /// method this backs, takes none either — `texture.device`/`texture.
  /// vulkan_format` supply everything a live device reference would have.
  ///
  /// # Errors
  ///
  /// Returns [`Error::Vulkan`] if `vkCreateImageView` fails.
  pub fn texture_view_create( texture : &TextureVulkan ) -> Result< TextureViewVulkan, Error >
  {
    let aspect_mask = if texture.format == TextureFormat::Depth24Plus
    {
      ash::vk::ImageAspectFlags::DEPTH
    }
    else
    {
      ash::vk::ImageAspectFlags::COLOR
    };
    let create_info = ash::vk::ImageViewCreateInfo::default()
    .image( texture.image )
    .view_type( ash::vk::ImageViewType::TYPE_2D )
    .format( texture.vulkan_format )
    .subresource_range
    (
      ash::vk::ImageSubresourceRange::default()
      .aspect_mask( aspect_mask )
      .base_mip_level( 0 )
      .level_count( 1 )
      .base_array_layer( 0 )
      .layer_count( 1 )
    );
    // SAFETY: `create_info` is stack-local and `texture.image` is a live image from
    // this same device.
    let view = unsafe { texture.device.create_image_view( &create_info, None ) }
    .map_err( | e | Error::Vulkan( format!( "vkCreateImageView failed :: {e}" ) ) )?;
    Ok( TextureViewVulkan
    {
      view,
      format : texture.format,
      vulkan_format : texture.vulkan_format,
      size : [ texture.size[ 0 ], texture.size[ 1 ] ]
    } )
  }

  /// Creates a sampler.
  ///
  /// # Errors
  ///
  /// Returns [`Error::Vulkan`] if `vkCreateSampler` fails.
  pub fn sampler_create( device_vulkan : &DeviceVulkan, desc : SamplerDesc ) -> Result< ash::vk::Sampler, Error >
  {
    let filter = filter_mode_to_vulkan( desc.filter );
    let address = address_mode_to_vulkan( desc.address );
    let create_info = ash::vk::SamplerCreateInfo::default()
    .mag_filter( filter )
    .min_filter( filter )
    .address_mode_u( address )
    .address_mode_v( address )
    .address_mode_w( address );
    // SAFETY: `create_info` is stack-local and fully initialized.
    unsafe { device_vulkan.device.create_sampler( &create_info, None ) }
    .map_err( | e | Error::Vulkan( format!( "vkCreateSampler failed :: {e}" ) ) )
  }

  /// Compiles `source` ( canonical WGSL ) to SPIR-V via naga and creates the
  /// resulting shader module.
  ///
  /// # Errors
  ///
  /// Returns [`Error::Vulkan`] if WGSL-to-SPIR-V compilation or
  /// `vkCreateShaderModule` fails.
  pub fn shader_module_create( device_vulkan : &DeviceVulkan, source : &str ) -> Result< ash::vk::ShaderModule, Error >
  {
    let spirv = shader_compile_wgsl_to_spirv( source )?;
    let create_info = ash::vk::ShaderModuleCreateInfo::default().code( &spirv );
    // SAFETY: `spirv` is well-formed SPIR-V produced by naga's own validated backend,
    // and `create_info` borrows it for the duration of this call only.
    unsafe { device_vulkan.device.create_shader_module( &create_info, None ) }
    .map_err( | e | Error::Vulkan( format!( "vkCreateShaderModule failed :: {e}" ) ) )
  }

  /// Creates a bind group layout; binding indices follow entry order.
  ///
  /// # Errors
  ///
  /// Returns [`Error::Vulkan`] if `vkCreateDescriptorSetLayout` fails.
  pub fn bind_group_layout_create
  (
    device_vulkan : &DeviceVulkan,
    entries : &[ BindGroupLayoutEntry ]
  ) -> Result< BindGroupLayoutVulkan, Error >
  {
    let raw_entries : Vec< ash::vk::DescriptorSetLayoutBinding< '_ > > = entries.iter().enumerate()
    .map
    (
      | ( index, entry ) |
      ash::vk::DescriptorSetLayoutBinding::default()
      .binding( index as u32 )
      .descriptor_type( binding_type_to_vulkan( entry.ty ) )
      .descriptor_count( 1 )
      .stage_flags( shader_stages_to_vulkan( entry.visibility ) )
    )
    .collect();
    let create_info = ash::vk::DescriptorSetLayoutCreateInfo::default().bindings( &raw_entries );
    // SAFETY: `create_info` borrows `raw_entries`, a stack-local `Vec` outliving this call.
    let layout = unsafe { device_vulkan.device.create_descriptor_set_layout( &create_info, None ) }
    .map_err( | e | Error::Vulkan( format!( "vkCreateDescriptorSetLayout failed :: {e}" ) ) )?;
    Ok( BindGroupLayoutVulkan { layout, entries : entries.to_vec() } )
  }

  /// Allocates a descriptor pool sized exactly for `layout.entries` and one
  /// descriptor set from it.
  ///
  /// # Errors
  ///
  /// Returns [`Error::Vulkan`] if descriptor pool creation or descriptor
  /// set allocation fails.
  fn bind_group_descriptor_pool_and_set_allocate( device_vulkan : &DeviceVulkan, layout : &BindGroupLayoutVulkan ) -> Result< ( ash::vk::DescriptorPool, ash::vk::DescriptorSet ), Error >
  {
    let pool_sizes : Vec< ash::vk::DescriptorPoolSize > = layout.entries.iter()
    .map( | entry | ash::vk::DescriptorPoolSize::default().ty( binding_type_to_vulkan( entry.ty ) ).descriptor_count( 1 ) )
    .collect();
    let pool_create_info = ash::vk::DescriptorPoolCreateInfo::default()
    .pool_sizes( &pool_sizes )
    .max_sets( 1 );
    // SAFETY: `pool_create_info` borrows `pool_sizes`, a stack-local `Vec` outliving
    // this call.
    let pool = unsafe { device_vulkan.device.create_descriptor_pool( &pool_create_info, None ) }
    .map_err( | e | Error::Vulkan( format!( "vkCreateDescriptorPool failed :: {e}" ) ) )?;

    let set_layouts = [ layout.layout ];
    let allocate_info = ash::vk::DescriptorSetAllocateInfo::default()
    .descriptor_pool( pool )
    .set_layouts( &set_layouts );
    // SAFETY: `pool` was just created above with exactly one set's worth of capacity,
    // and `set_layouts` borrows a stack-local array outliving this call.
    let sets = unsafe { device_vulkan.device.allocate_descriptor_sets( &allocate_info ) }
    .map_err( | e | Error::Vulkan( format!( "vkAllocateDescriptorSets failed :: {e}" ) ) )?;
    Ok( ( pool, sets[ 0 ] ) )
  }

  /// Builds parallel `buffer_infos`/`image_infos` vectors from `resources`,
  /// index-aligned with `resources` itself — exactly one of the pair is a
  /// real descriptor per resource, the other a default placeholder, so
  /// `bind_group_create`'s write-building pass can index either by position
  /// regardless of which variant is present at that index.
  fn bind_group_infos_build( resources : &[ BindingResource< '_ > ] ) -> ( Vec< ash::vk::DescriptorBufferInfo >, Vec< ash::vk::DescriptorImageInfo > )
  {
    let mut buffer_infos = Vec::with_capacity( resources.len() );
    let mut image_infos = Vec::with_capacity( resources.len() );
    for resource in resources
    {
      match resource
      {
        BindingResource::Buffer( buffer ) =>
        {
          let raw = buffer.expect_vulkan();
          buffer_infos.push( ash::vk::DescriptorBufferInfo::default().buffer( raw.buffer ).offset( 0 ).range( raw.size ) );
          image_infos.push( ash::vk::DescriptorImageInfo::default() );
        }
        BindingResource::TextureView( view ) =>
        {
          let raw = view.expect_vulkan();
          image_infos.push
          (
            ash::vk::DescriptorImageInfo::default()
            .image_view( raw.view )
            .image_layout( ash::vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL )
          );
          buffer_infos.push( ash::vk::DescriptorBufferInfo::default() );
        }
        BindingResource::Sampler( sampler ) =>
        {
          image_infos.push( ash::vk::DescriptorImageInfo::default().sampler( *sampler.expect_vulkan() ) );
          buffer_infos.push( ash::vk::DescriptorBufferInfo::default() );
        }
      }
    }
    ( buffer_infos, image_infos )
  }

  /// Creates a bind group; `resources` follow the layout's entry order — a
  /// dedicated descriptor pool sized exactly for `layout.entries`, one set
  /// allocated from it, and one `vkUpdateDescriptorSets` writing every entry.
  ///
  /// # Errors
  ///
  /// Returns [`Error::Vulkan`] if descriptor pool creation or descriptor
  /// set allocation fails.
  pub fn bind_group_create
  (
    device_vulkan : &DeviceVulkan,
    layout : &BindGroupLayoutVulkan,
    resources : &[ BindingResource< '_ > ]
  ) -> Result< BindGroupVulkan, Error >
  {
    let ( pool, set ) = bind_group_descriptor_pool_and_set_allocate( device_vulkan, layout )?;
    let ( buffer_infos, image_infos ) = bind_group_infos_build( resources );
    let writes : Vec< ash::vk::WriteDescriptorSet< '_ > > = resources.iter().enumerate()
    .map
    (
      | ( index, resource ) |
      {
        let write = ash::vk::WriteDescriptorSet::default()
        .dst_set( set )
        .dst_binding( index as u32 )
        .dst_array_element( 0 )
        .descriptor_count( 1 )
        .descriptor_type( binding_type_to_vulkan( layout.entries[ index ].ty ) );
        match resource
        {
          BindingResource::Buffer( _ ) => write.buffer_info( core::slice::from_ref( &buffer_infos[ index ] ) ),
          BindingResource::TextureView( _ ) | BindingResource::Sampler( _ ) =>
          write.image_info( core::slice::from_ref( &image_infos[ index ] ) )
        }
      }
    )
    .collect();
    // SAFETY: `writes` borrows `buffer_infos`/`image_infos`, both stack-local `Vec`s
    // outliving this call; `set` was just allocated above from this same device.
    unsafe { device_vulkan.device.update_descriptor_sets( &writes, &[] ); }
    Ok( BindGroupVulkan { pool, set } )
  }

  /// Converts an entry point name to a nul-terminated `CString`, tagging
  /// the error with which stage ( `"vertex"`/`"fragment"` ) it came from.
  ///
  /// # Errors
  ///
  /// Returns [`Error::Vulkan`] if `name` contains an interior nul byte.
  fn entry_point_cstring( name : &str, which : &str ) -> Result< std::ffi::CString, Error >
  {
    std::ffi::CString::new( name ).map_err( | e | Error::Vulkan( format!( "{which} entry point name : {e}" ) ) )
  }

  /// Builds the vertex input binding/attribute descriptions for every
  /// buffer slot in `vertex_buffers`, index-aligned to slot position.
  fn render_pipeline_vertex_layout_build( vertex_buffers : &[ VertexBufferLayout ] ) ->
  ( Vec< ash::vk::VertexInputBindingDescription >, Vec< ash::vk::VertexInputAttributeDescription > )
  {
    let bindings : Vec< ash::vk::VertexInputBindingDescription > = vertex_buffers.iter().enumerate()
    .map
    (
      | ( slot, layout ) |
      ash::vk::VertexInputBindingDescription::default()
      .binding( slot as u32 )
      .stride( layout.stride )
      .input_rate( ash::vk::VertexInputRate::VERTEX )
    )
    .collect();
    let attributes : Vec< ash::vk::VertexInputAttributeDescription > = vertex_buffers.iter().enumerate()
    .flat_map
    (
      | ( slot, layout ) |
      layout.attributes.iter().map
      (
        move | attribute |
        ash::vk::VertexInputAttributeDescription::default()
        .binding( slot as u32 )
        .location( attribute.location )
        .format( vertex_format_to_vulkan( attribute.format ) )
        .offset( attribute.offset )
      )
    )
    .collect();
    ( bindings, attributes )
  }

  /// Builds the pipeline's fixed, non-borrowing state blocks — input
  /// assembly, dynamic viewport/scissor counts, rasterization,
  /// multisampling, and depth — the v0 fixed function set every backend
  /// shares ( see `render_pipeline_create`'s own doc comment ). None of
  /// these borrow slice/array data, so they carry no meaningful lifetime.
  fn render_pipeline_scalar_states( desc : &RenderPipelineDesc< '_ > ) ->
  (
    ash::vk::PipelineInputAssemblyStateCreateInfo< 'static >,
    ash::vk::PipelineViewportStateCreateInfo< 'static >,
    ash::vk::PipelineRasterizationStateCreateInfo< 'static >,
    ash::vk::PipelineMultisampleStateCreateInfo< 'static >,
    ash::vk::PipelineDepthStencilStateCreateInfo< 'static >
  )
  {
    let input_assembly_state = ash::vk::PipelineInputAssemblyStateCreateInfo::default()
    .topology( ash::vk::PrimitiveTopology::TRIANGLE_LIST );
    // Actual values come from `vkCmdSetViewport`/`vkCmdSetScissor` at
    // render-pass-begin time — these counts only satisfy pipeline creation's
    // required-but-dynamic viewport/scissor state.
    let viewport_state = ash::vk::PipelineViewportStateCreateInfo::default()
    .viewport_count( 1 )
    .scissor_count( 1 );
    let rasterization_state = ash::vk::PipelineRasterizationStateCreateInfo::default()
    .polygon_mode( ash::vk::PolygonMode::FILL )
    .cull_mode( if desc.cull_back { ash::vk::CullModeFlags::BACK } else { ash::vk::CullModeFlags::NONE } )
    .front_face( ash::vk::FrontFace::COUNTER_CLOCKWISE )
    .line_width( 1.0 );
    let multisample_state = ash::vk::PipelineMultisampleStateCreateInfo::default()
    .rasterization_samples( ash::vk::SampleCountFlags::TYPE_1 );
    let depth_stencil_state = ash::vk::PipelineDepthStencilStateCreateInfo::default()
    .depth_test_enable( desc.depth.is_some() )
    .depth_write_enable( desc.depth.is_some() )
    .depth_compare_op( ash::vk::CompareOp::LESS );
    ( input_assembly_state, viewport_state, rasterization_state, multisample_state, depth_stencil_state )
  }

  /// Builds the vertex+fragment `PipelineShaderStageCreateInfo` pair —
  /// both stages share one shader module, split by entry point.
  fn render_pipeline_stages_build< 'a >( shader : ash::vk::ShaderModule, vertex_entry : &'a std::ffi::CString, fragment_entry : &'a std::ffi::CString ) -> [ ash::vk::PipelineShaderStageCreateInfo< 'a >; 2 ]
  {
    [
      ash::vk::PipelineShaderStageCreateInfo::default()
      .stage( ash::vk::ShaderStageFlags::VERTEX )
      .module( shader )
      .name( vertex_entry ),
      ash::vk::PipelineShaderStageCreateInfo::default()
      .stage( ash::vk::ShaderStageFlags::FRAGMENT )
      .module( shader )
      .name( fragment_entry )
    ]
  }

  /// Creates a pipeline layout from `bind_group_layouts`' own descriptor
  /// set layouts, in order.
  ///
  /// # Errors
  ///
  /// Returns [`Error::Vulkan`] if pipeline layout creation fails.
  fn render_pipeline_layout_create( device_vulkan : &DeviceVulkan, bind_group_layouts : &[ &BindGroupLayout ] ) -> Result< ash::vk::PipelineLayout, Error >
  {
    let set_layouts : Vec< ash::vk::DescriptorSetLayout > = bind_group_layouts.iter()
    .map( | layout | layout.expect_vulkan().layout )
    .collect();
    let layout_create_info = ash::vk::PipelineLayoutCreateInfo::default().set_layouts( &set_layouts );
    // SAFETY: `layout_create_info` borrows `set_layouts`, a stack-local `Vec` outliving
    // this call.
    unsafe { device_vulkan.device.create_pipeline_layout( &layout_create_info, None ) }
    .map_err( | e | Error::Vulkan( format!( "vkCreatePipelineLayout failed :: {e}" ) ) )
  }

  /// Assembles the final `VkGraphicsPipelineCreateInfo` from every
  /// already-built piece and creates the pipeline, destroying the
  /// compatibility-only `render_pass` immediately after — see
  /// `render_pipeline_create`'s own doc comment for why `render_pass` is
  /// not retained.
  #[ allow( clippy::too_many_arguments, reason = "each parameter is a distinct, already-built Vulkan create-info piece; bundling them into a struct would only move the same count into field assignments" ) ]
  fn render_pipeline_assemble_and_create
  (
    device_vulkan : &DeviceVulkan,
    stages : &[ ash::vk::PipelineShaderStageCreateInfo< '_ > ],
    vertex_input_state : &ash::vk::PipelineVertexInputStateCreateInfo< '_ >,
    input_assembly_state : &ash::vk::PipelineInputAssemblyStateCreateInfo< '_ >,
    viewport_state : &ash::vk::PipelineViewportStateCreateInfo< '_ >,
    rasterization_state : &ash::vk::PipelineRasterizationStateCreateInfo< '_ >,
    multisample_state : &ash::vk::PipelineMultisampleStateCreateInfo< '_ >,
    depth_stencil_state : &ash::vk::PipelineDepthStencilStateCreateInfo< '_ >,
    color_blend_state : &ash::vk::PipelineColorBlendStateCreateInfo< '_ >,
    dynamic_state : &ash::vk::PipelineDynamicStateCreateInfo< '_ >,
    layout : ash::vk::PipelineLayout,
    render_pass : ash::vk::RenderPass
  ) -> Result< ash::vk::Pipeline, Error >
  {
    let create_info = ash::vk::GraphicsPipelineCreateInfo::default()
    .stages( stages )
    .vertex_input_state( vertex_input_state )
    .input_assembly_state( input_assembly_state )
    .viewport_state( viewport_state )
    .rasterization_state( rasterization_state )
    .multisample_state( multisample_state )
    .depth_stencil_state( depth_stencil_state )
    .color_blend_state( color_blend_state )
    .dynamic_state( dynamic_state )
    .layout( layout )
    .render_pass( render_pass )
    .subpass( 0 );
    // SAFETY: every `p_*` field above borrows a parameter outliving this call; `layout`
    // and `render_pass` were both already created by the caller on this same device.
    let pipelines = unsafe
    {
      device_vulkan.device.create_graphics_pipelines( ash::vk::PipelineCache::null(), &[ create_info ], None )
    }
    .map_err( | ( _, e ) | Error::Vulkan( format!( "vkCreateGraphicsPipelines failed :: {e}" ) ) )?;

    // SAFETY: `create_graphics_pipelines` above has already returned, so `render_pass`
    // is no longer needed even by the driver ( pipelines retain no live reference to
    // the render pass used at creation time — only pipeline creation itself needed it,
    // to check render pass compatibility ).
    unsafe { device_vulkan.device.destroy_render_pass( render_pass, None ); }
    Ok( pipelines[ 0 ] )
  }

  /// Creates a render pipeline: triangle list, one color target without
  /// blending, dynamic viewport/scissor, optional always-on depth ( test
  /// `less`, write on ) — the v0 fixed function set every backend shares.
  ///
  /// # Errors
  ///
  /// Returns [`Error::Vulkan`] if an entry point name contains an interior
  /// nul byte, or if pipeline layout / render pass / graphics pipeline
  /// creation fails.
  pub fn render_pipeline_create
  (
    device_vulkan : &DeviceVulkan,
    desc : &RenderPipelineDesc< '_ >
  ) -> Result< RenderPipelineVulkan, Error >
  {
    let shader = *desc.shader.expect_vulkan();
    let vertex_entry = entry_point_cstring( desc.vertex_entry, "vertex" )?;
    let fragment_entry = entry_point_cstring( desc.fragment_entry, "fragment" )?;
    let stages = render_pipeline_stages_build( shader, &vertex_entry, &fragment_entry );

    let ( bindings, attributes ) = render_pipeline_vertex_layout_build( desc.vertex_buffers );
    let vertex_input_state = ash::vk::PipelineVertexInputStateCreateInfo::default()
    .vertex_binding_descriptions( &bindings )
    .vertex_attribute_descriptions( &attributes );

    let ( input_assembly_state, viewport_state, rasterization_state, multisample_state, depth_stencil_state ) =
    render_pipeline_scalar_states( desc );

    let color_blend_attachments =
    [ ash::vk::PipelineColorBlendAttachmentState::default().color_write_mask( ash::vk::ColorComponentFlags::RGBA ) ];
    let color_blend_state = ash::vk::PipelineColorBlendStateCreateInfo::default().attachments( &color_blend_attachments );
    let dynamic_states = [ ash::vk::DynamicState::VIEWPORT, ash::vk::DynamicState::SCISSOR ];
    let dynamic_state = ash::vk::PipelineDynamicStateCreateInfo::default().dynamic_states( &dynamic_states );

    let layout = render_pipeline_layout_create( device_vulkan, desc.bind_group_layouts )?;

    let color_format = texture_format_to_vulkan( desc.color_format, &device_vulkan.instance, device_vulkan.physical_device )?;
    let depth_format = desc.depth
    .map( | depth | texture_format_to_vulkan( depth.format, &device_vulkan.instance, device_vulkan.physical_device ) )
    .transpose()?;
    // A pipeline needs no live reference to the render pass it was created
    // against beyond this call — only compatibility, checked here — so this
    // one is destroyed immediately below rather than leaked.
    let render_pass = render_pass_create( &device_vulkan.device, color_format, depth_format )?;

    let pipeline = render_pipeline_assemble_and_create
    (
      device_vulkan, &stages, &vertex_input_state, &input_assembly_state, &viewport_state, &rasterization_state,
      &multisample_state, &depth_stencil_state, &color_blend_state, &dynamic_state, layout, render_pass
    )?;
    Ok( RenderPipelineVulkan { pipeline, layout } )
  }

  /// Creates a command encoder for one frame's passes: a dedicated command
  /// pool plus one primary command buffer, already begun ( any number of
  /// render passes can be begun/ended into it before `Queue::submit` ends
  /// the recording ).
  ///
  /// # Errors
  ///
  /// Returns [`Error::Vulkan`] if command pool creation, command buffer
  /// allocation, or `vkBeginCommandBuffer` fails.
  pub fn command_encoder_create( device_vulkan : &DeviceVulkan ) -> Result< CommandEncoderVulkan, Error >
  {
    let pool_create_info = ash::vk::CommandPoolCreateInfo::default()
    .queue_family_index( device_vulkan.queue_family_index );
    // SAFETY: `pool_create_info` is stack-local and fully initialized.
    let pool = unsafe { device_vulkan.device.create_command_pool( &pool_create_info, None ) }
    .map_err( | e | Error::Vulkan( format!( "vkCreateCommandPool failed :: {e}" ) ) )?;
    let allocate_info = ash::vk::CommandBufferAllocateInfo::default()
    .command_pool( pool )
    .level( ash::vk::CommandBufferLevel::PRIMARY )
    .command_buffer_count( 1 );
    // SAFETY: `pool` was just created above on this same device.
    let command_buffers = unsafe { device_vulkan.device.allocate_command_buffers( &allocate_info ) }
    .map_err( | e | Error::Vulkan( format!( "vkAllocateCommandBuffers failed :: {e}" ) ) )?;
    let command_buffer = command_buffers[ 0 ];
    let begin_info = ash::vk::CommandBufferBeginInfo::default();
    // SAFETY: `command_buffer` was just allocated above and is in the initial state.
    unsafe { device_vulkan.device.begin_command_buffer( command_buffer, &begin_info ) }
    .map_err( | e | Error::Vulkan( format!( "vkBeginCommandBuffer failed :: {e}" ) ) )?;
    Ok( CommandEncoderVulkan { device : device_vulkan.device.clone(), pool, command_buffer } )
  }

  /// Builds a compatible render pass ( see `render_pass_create` ) and a
  /// framebuffer over `color_view` plus, if present, `depth_view`.
  ///
  /// # Errors
  ///
  /// Returns [`Error::Vulkan`] if render pass or framebuffer creation
  /// fails.
  fn render_pass_framebuffer_create
  (
    encoder : &CommandEncoderVulkan,
    color_view : &TextureViewVulkan,
    depth_view : Option< &TextureViewVulkan >
  ) -> Result< ( ash::vk::RenderPass, ash::vk::Framebuffer ), Error >
  {
    let render_pass = render_pass_create
    (
      &encoder.device,
      color_view.vulkan_format,
      depth_view.map( | view | view.vulkan_format )
    )?;

    let mut attachments = vec![ color_view.view ];
    if let Some( view ) = depth_view
    {
      attachments.push( view.view );
    }
    let framebuffer_create_info = ash::vk::FramebufferCreateInfo::default()
    .render_pass( render_pass )
    .attachments( &attachments )
    .width( color_view.size[ 0 ] )
    .height( color_view.size[ 1 ] )
    .layers( 1 );
    // SAFETY: `framebuffer_create_info` borrows `attachments`, a stack-local `Vec`
    // outliving this call; `render_pass` was just created above on this same device.
    let framebuffer = unsafe { encoder.device.create_framebuffer( &framebuffer_create_info, None ) }
    .map_err( | e | Error::Vulkan( format!( "vkCreateFramebuffer failed :: {e}" ) ) )?;
    Ok( ( render_pass, framebuffer ) )
  }

  /// Issues `vkCmdBeginRenderPass` against `render_pass`/`framebuffer`,
  /// with clear values for the color attachment and, if `has_depth`, a
  /// depth/stencil clear appended to match.
  fn render_pass_begin_cmd
  (
    encoder : &CommandEncoderVulkan,
    color : &ColorAttachmentDesc< '_ >,
    has_depth : bool,
    render_pass : ash::vk::RenderPass,
    framebuffer : ash::vk::Framebuffer,
    size : [ u32; 2 ]
  )
  {
    let mut clear_values = vec!
    [
      ash::vk::ClearValue { color : ash::vk::ClearColorValue { float32 : color.clear } }
    ];
    if has_depth
    {
      clear_values.push
      (
        ash::vk::ClearValue { depth_stencil : ash::vk::ClearDepthStencilValue { depth : 1.0, stencil : 0 } }
      );
    }
    let render_area = ash::vk::Rect2D
    {
      offset : ash::vk::Offset2D::default(),
      extent : ash::vk::Extent2D { width : size[ 0 ], height : size[ 1 ] }
    };
    let begin_info = ash::vk::RenderPassBeginInfo::default()
    .render_pass( render_pass )
    .framebuffer( framebuffer )
    .render_area( render_area )
    .clear_values( &clear_values );
    // SAFETY: `begin_info` borrows `clear_values`, a stack-local `Vec` outliving this
    // call; `encoder.command_buffer` is already recording ( begun by
    // `command_encoder_create` ); `render_pass`/`framebuffer` were both just created
    // above on this same device by the caller.
    unsafe { encoder.device.cmd_begin_render_pass( encoder.command_buffer, &begin_info, ash::vk::SubpassContents::INLINE ); }
  }

  /// Sets the dynamic viewport/scissor to cover `size` in full — called
  /// immediately after `vkCmdBeginRenderPass` since both are declared
  /// `DynamicState` at pipeline-creation time ( see `render_pipeline_create` ).
  fn render_pass_viewport_scissor_set( encoder : &CommandEncoderVulkan, size : [ u32; 2 ] )
  {
    let viewport = ash::vk::Viewport
    {
      x : 0.0,
      y : 0.0,
      width : size[ 0 ] as f32,
      height : size[ 1 ] as f32,
      min_depth : 0.0,
      max_depth : 1.0
    };
    let scissor = ash::vk::Rect2D
    {
      offset : ash::vk::Offset2D::default(),
      extent : ash::vk::Extent2D { width : size[ 0 ], height : size[ 1 ] }
    };
    // SAFETY: the render pass was just begun by the caller on this same command buffer.
    unsafe
    {
      encoder.device.cmd_set_viewport( encoder.command_buffer, 0, &[ viewport ] );
      encoder.device.cmd_set_scissor( encoder.command_buffer, 0, &[ scissor ] );
    }
  }

  /// Begins a render pass with one color attachment and an optional depth
  /// attachment: builds a fresh, compatible render pass + framebuffer
  /// ( see `render_pass_create` ), begins it, then sets the dynamic
  /// viewport/scissor from the color view's own size. Neither the render
  /// pass nor the framebuffer is destroyed here — see the module doc
  /// comment for why that would be premature. Takes no separate
  /// `&DeviceVulkan` parameter because `CommandEncoder::render_pass_begin`,
  /// the cross-backend method this backs, takes none either — `encoder.
  /// device` and each view's pre-resolved `vulkan_format` supply everything
  /// a live device reference would have.
  ///
  /// # Errors
  ///
  /// Returns [`Error::Vulkan`] if render pass or framebuffer creation
  /// fails.
  pub fn render_pass_begin
  (
    encoder : &CommandEncoderVulkan,
    color : &ColorAttachmentDesc< '_ >,
    depth : Option< &DepthAttachmentDesc< '_ > >
  ) -> Result< RenderPassVulkan, Error >
  {
    let color_view = color.view.expect_vulkan();
    let depth_view = depth.map( | depth | depth.view.expect_vulkan() );

    let ( render_pass, framebuffer ) = render_pass_framebuffer_create( encoder, color_view, depth_view )?;
    render_pass_begin_cmd( encoder, color, depth_view.is_some(), render_pass, framebuffer, color_view.size );
    render_pass_viewport_scissor_set( encoder, color_view.size );

    Ok( RenderPassVulkan
    {
      device : encoder.device.clone(),
      command_buffer : encoder.command_buffer,
      render_pass,
      framebuffer,
      current_pipeline_layout : None
    } )
  }

  /// Sets the active render pipeline and records its layout for
  /// `bind_group_set`'s later `vkCmdBindDescriptorSets` call.
  pub fn pipeline_set( pass : &mut RenderPassVulkan, pipeline : &RenderPipelineVulkan )
  {
    // SAFETY: `pass.command_buffer` is currently recording a render pass
    // ( begun by `render_pass_begin` ); `pipeline.pipeline` was created
    // compatible with that render pass by `render_pipeline_create`.
    unsafe { pass.device.cmd_bind_pipeline( pass.command_buffer, ash::vk::PipelineBindPoint::GRAPHICS, pipeline.pipeline ); }
    pass.current_pipeline_layout = Some( pipeline.layout );
  }

  /// Binds `group` at descriptor set `index`. No-op if called before
  /// `pipeline_set` — mirrors the WebGL backend's own eager-state-tracking
  /// contract ( see `RenderPassVulkan::current_pipeline_layout` ).
  pub fn bind_group_set( pass : &mut RenderPassVulkan, index : u32, group : &BindGroupVulkan )
  {
    let Some( layout ) = pass.current_pipeline_layout
    else
    {
      return;
    };
    // SAFETY: `pass.command_buffer` is currently recording; `layout` is the
    // layout of the pipeline bound by the most recent `pipeline_set`, and
    // `group.set` was allocated compatible with that same pipeline's bind
    // group layouts by `bind_group_create`.
    unsafe
    {
      pass.device.cmd_bind_descriptor_sets
      (
        pass.command_buffer,
        ash::vk::PipelineBindPoint::GRAPHICS,
        layout,
        index,
        &[ group.set ],
        &[]
      );
    }
  }

  /// Binds `buffer` at vertex buffer `slot`.
  pub fn vertex_buffer_set( pass : &mut RenderPassVulkan, slot : u32, buffer : &BufferVulkan )
  {
    // SAFETY: `pass.command_buffer` is currently recording; `buffer.buffer`
    // was created with `VERTEX_BUFFER` usage by the HAL caller.
    unsafe { pass.device.cmd_bind_vertex_buffers( pass.command_buffer, slot, &[ buffer.buffer ], &[ 0 ] ); }
  }

  /// Binds `buffer` as the index buffer, translating `format` via
  /// `index_format_to_vulkan`.
  pub fn index_buffer_set( pass : &mut RenderPassVulkan, buffer : &BufferVulkan, format : IndexFormat )
  {
    // SAFETY: `pass.command_buffer` is currently recording; `buffer.buffer`
    // was created with `INDEX_BUFFER` usage by the HAL caller.
    unsafe { pass.device.cmd_bind_index_buffer( pass.command_buffer, buffer.buffer, 0, index_format_to_vulkan( format ) ); }
  }

  /// Draws `vertex_count` vertices, non-instanced.
  pub fn draw( pass : &mut RenderPassVulkan, vertex_count : u32 )
  {
    // SAFETY: `pass.command_buffer` is currently recording a render pass
    // with a pipeline already bound by `pipeline_set`.
    unsafe { pass.device.cmd_draw( pass.command_buffer, vertex_count, 1, 0, 0 ); }
  }

  /// Draws `index_count` indices from the bound index buffer, non-instanced.
  pub fn draw_indexed( pass : &mut RenderPassVulkan, index_count : u32 )
  {
    // SAFETY: `pass.command_buffer` is currently recording a render pass
    // with a pipeline and index buffer already bound.
    unsafe { pass.device.cmd_draw_indexed( pass.command_buffer, index_count, 1, 0, 0, 0 ); }
  }

  /// Ends the render pass, consuming the recorder. The render pass and
  /// framebuffer are intentionally not destroyed here — see the module doc
  /// comment.
  #[ allow( clippy::needless_pass_by_value, reason = "takes `pass` by value \
deliberately -- consuming it is what makes the recorder unusable after \
`end`, mirroring every other backend's `RenderPass::end( self )`" ) ]
  pub fn render_pass_end( pass : RenderPassVulkan )
  {
    // SAFETY: `pass.command_buffer` is currently recording a render pass
    // begun by `render_pass_begin`.
    unsafe { pass.device.cmd_end_render_pass( pass.command_buffer ); }
  }

  /// Records the one `vkCmdCopyImageToBuffer` call that `pixels_read`
  /// submits to copy `image`'s contents into a staging buffer — see
  /// `pixels_read`'s own doc comment for why no layout transition is
  /// needed here.
  fn pixels_read_record_copy
  (
    device : &ash::Device,
    command_buffer : ash::vk::CommandBuffer,
    image : ash::vk::Image,
    staging_buffer : ash::vk::Buffer,
    region : ash::vk::BufferImageCopy
  )
  {
    // SAFETY: `command_buffer` is recording; `image` is left in
    // `TRANSFER_SRC_OPTIMAL` by every render pass's fixed `finalLayout`
    // ( see `render_pass_create` ), so no layout transition is needed here.
    unsafe
    {
      device.cmd_copy_image_to_buffer( command_buffer, image, ash::vk::ImageLayout::TRANSFER_SRC_OPTIMAL, staging_buffer, &[ region ] );
    }
  }

  /// Reads the surface's pixels back as tightly-packed rgba8 bytes, top row
  /// first, via a staging buffer copy. Every render pass's fixed
  /// `finalLayout` already leaves the surface image in
  /// `TRANSFER_SRC_OPTIMAL`, so no extra layout transition is needed here.
  ///
  /// # Errors
  ///
  /// Returns [`Error::Vulkan`] if staging buffer allocation or the
  /// underlying `vkMapMemory` call fails.
  pub fn pixels_read( device_vulkan : &DeviceVulkan, queue : ash::vk::Queue, surface : &SurfaceVulkan ) -> Result< Vec< u8 >, Error >
  {
    let bytes_per_texel = surface.format.bytes_per_texel()?;
    let width = surface.size[ 0 ];
    let height = surface.size[ 1 ];
    let size = u64::from( width ) * u64::from( height ) * u64::from( bytes_per_texel );

    let staging_usage = ash::vk::BufferUsageFlags::TRANSFER_DST;
    let staging_required = ash::vk::MemoryPropertyFlags::HOST_VISIBLE | ash::vk::MemoryPropertyFlags::HOST_COHERENT;
    let ( staging_buffer, staging_memory ) = buffer_allocate( device_vulkan, size, staging_usage, staging_required )?;

    let subresource = ash::vk::ImageSubresourceLayers::default()
    .aspect_mask( ash::vk::ImageAspectFlags::COLOR )
    .mip_level( 0 )
    .base_array_layer( 0 )
    .layer_count( 1 );
    let region = ash::vk::BufferImageCopy::default()
    .buffer_offset( 0 )
    .buffer_row_length( 0 )
    .buffer_image_height( 0 )
    .image_subresource( subresource )
    .image_offset( ash::vk::Offset3D::default() )
    .image_extent( ash::vk::Extent3D { width, height, depth : 1 } );

    command_buffer_one_shot_submit
    (
      device_vulkan,
      queue,
      | command_buffer | pixels_read_record_copy( &device_vulkan.device, command_buffer, surface.image, staging_buffer, region )
    )?;

    // SAFETY: `staging_memory` is `HOST_VISIBLE`, sized for exactly `size` bytes, and
    // the one-shot submit above only returns once its fence confirms the copy has
    // completed.
    let pixels = memory_read( &device_vulkan.device, staging_memory, size )?;
    // SAFETY: the copy and the read above are both already complete at this point.
    unsafe
    {
      device_vulkan.device.destroy_buffer( staging_buffer, None );
      device_vulkan.device.free_memory( staging_memory, None );
    }
    Ok( pixels )
  }

  /// Transitions a just-rendered swapchain image from the layout every render
  /// pass leaves it in to the one `vkQueuePresentKHR` requires.
  ///
  /// `color_attachment_description`'s `final_layout` is
  /// `TRANSFER_SRC_OPTIMAL` — chosen so `pixels_read` can copy straight out of
  /// an offscreen surface — while presentation requires `PRESENT_SRC_KHR`.
  /// Bridging the two here rather than making the render pass's final layout
  /// conditional keeps every existing pipeline and render pass untouched :
  /// Vulkan's render pass compatibility rules ignore attachment layouts, so a
  /// pipeline built against the offscreen pass stays valid for a windowed one.
  ///
  /// No semaphores are involved, and none are needed : `submit` above ends
  /// with `vkQueueWaitIdle`, so all rendering is provably complete before this
  /// transition is recorded, and `Swapchain::frame_acquire` waits on a fence
  /// rather than a semaphore, so the image is provably owned before rendering
  /// begins. That is a deliberate v0 simplification — correctness without
  /// pipelining — matching this backend's synchronous `submit`.
  ///
  /// # Errors
  ///
  /// Returns [`Error::Vulkan`] if allocating, recording or submitting the
  /// one-shot command buffer carrying the barrier fails.
  pub fn present_transition
  (
    device_vulkan : &DeviceVulkan,
    queue : ash::vk::Queue,
    image : ash::vk::Image
  ) -> Result< (), Error >
  {
    let subresource_range = ash::vk::ImageSubresourceRange::default()
    .aspect_mask( ash::vk::ImageAspectFlags::COLOR )
    .base_mip_level( 0 )
    .level_count( 1 )
    .base_array_layer( 0 )
    .layer_count( 1 );
    let barrier = image_layout_barrier
    (
      image,
      subresource_range,
      ash::vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
      ash::vk::ImageLayout::PRESENT_SRC_KHR,
      ash::vk::AccessFlags::TRANSFER_READ,
      ash::vk::AccessFlags::empty()
    );

    command_buffer_one_shot_submit
    (
      device_vulkan,
      queue,
      | command_buffer |
      {
        // SAFETY: `command_buffer` is recording ( `command_buffer_one_shot_submit`
        // began it ), and `barrier` references only `image`, which belongs to a
        // swapchain created on this same device.
        unsafe
        {
          device_vulkan.device.cmd_pipeline_barrier
          (
            command_buffer,
            ash::vk::PipelineStageFlags::TRANSFER,
            ash::vk::PipelineStageFlags::BOTTOM_OF_PIPE,
            ash::vk::DependencyFlags::empty(),
            &[],
            &[],
            &[ barrier ]
          );
        }
      }
    )
  }

  /// Ends `encoder`'s command buffer recording and submits it, blocking
  /// until the GPU finishes — the synchronous completion this call
  /// guarantees is what lets a caller safely read back results ( e.g.
  /// `Surface::pixels_read` ) immediately after this returns, with no
  /// fencing of its own to manage.
  ///
  /// Infallible in signature, matching every other backend's `Queue::
  /// submit` ( native's own submission errors are likewise unrecoverable
  /// through this call, surfacing instead through wgpu's internal
  /// uncaptured-error sink — see `Queue::texture_write`'s BUG-204 note ).
  ///
  /// # Panics
  ///
  /// Panics if `vkEndCommandBuffer`, `vkQueueSubmit`, or `vkQueueWaitIdle`
  /// fails — a genuine driver failure here panics rather than being
  /// silently lost.
  #[ allow( clippy::needless_pass_by_value, reason = "takes `encoder` by \
value deliberately -- consuming it is what makes the encoder unusable \
after submission, mirroring every other backend's `Queue::submit`" ) ]
  pub fn submit( device_vulkan : &DeviceVulkan, queue : ash::vk::Queue, encoder : CommandEncoderVulkan )
  {
    // SAFETY: `encoder.command_buffer` was left recording by `command_encoder_create`,
    // and any `render_pass_begin`/`RenderPass::end` pairs recorded since are balanced,
    // so ending it here is well-formed.
    unsafe { device_vulkan.device.end_command_buffer( encoder.command_buffer ) }
    .unwrap_or_else( | e | panic!( "vkEndCommandBuffer failed :: {e}" ) );
    let command_buffers = [ encoder.command_buffer ];
    let submit_info = ash::vk::SubmitInfo::default().command_buffers( &command_buffers );
    // SAFETY: `command_buffer` is fully recorded ( ended above ); `queue` belongs to
    // the same device that created every handle referenced by the recorded commands.
    unsafe { device_vulkan.device.queue_submit( queue, &[ submit_info ], ash::vk::Fence::null() ) }
    .unwrap_or_else( | e | panic!( "vkQueueSubmit failed :: {e}" ) );
    // SAFETY: `queue` belongs to this same device; waiting for it to go idle before
    // returning is what lets a caller's next call safely assume this submission's
    // writes are visible.
    unsafe { device_vulkan.device.queue_wait_idle( queue ) }
    .unwrap_or_else( | e | panic!( "vkQueueWaitIdle failed :: {e}" ) );
  }
}

crate::mod_interface!
{
  own use surface_create;
  own use buffer_create;
  own use buffer_init_create;
  own use buffer_write;
  own use texture_create;
  own use texture_write;
  own use texture_view_create;
  own use sampler_create;
  own use shader_module_create;
  own use bind_group_layout_create;
  own use bind_group_create;
  own use render_pipeline_create;
  own use command_encoder_create;
  own use render_pass_begin;
  own use pipeline_set;
  own use bind_group_set;
  own use vertex_buffer_set;
  own use index_buffer_set;
  own use draw;
  own use draw_indexed;
  own use render_pass_end;
  own use pixels_read;
  own use present_transition;
  own use submit;
  own use index_format_to_vulkan;

  orphan use
  {
    DeviceVulkan,
    QueueVulkan,
    SurfaceVulkan,
    SurfaceVulkanWindow,
    BufferVulkan,
    TextureVulkan,
    TextureViewVulkan,
    BindGroupLayoutVulkan,
    BindGroupVulkan,
    RenderPipelineVulkan,
    CommandEncoderVulkan,
    RenderPassVulkan
  };
}
