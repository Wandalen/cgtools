mod private
{
  #[ cfg( all( feature = "webgpu", target_arch = "wasm32" ) ) ]
  use minwebgpu as gl;
  #[ cfg( all( feature = "webgpu", target_arch = "wasm32" ) ) ]
  use gl::web_sys;
  #[ cfg( all( feature = "webgl", target_arch = "wasm32", not( feature = "webgpu" ) ) ) ]
  use minwebgl::web_sys;
  #[ cfg( all( feature = "webgl", target_arch = "wasm32" ) ) ]
  use std::rc::Rc;
  use crate::
  {
    Error,
    TextureFormat,
    DepthState,
    VertexBufferLayout
  };
  #[ cfg( all( feature = "webgl", target_arch = "wasm32" ) ) ]
  use crate::
  {
    BufferWebGl,
    TextureWebGl,
    TextureViewWebGl,
    ShaderModuleWebGl,
    BindGroupLayoutWebGl,
    BindGroupWebGl,
    RenderPipelineWebGl
  };
  #[ cfg( all( feature = "vulkan", not( target_arch = "wasm32" ) ) ) ]
  use crate::
  {
    BufferVulkan,
    TextureVulkan,
    TextureViewVulkan,
    BindGroupLayoutVulkan,
    BindGroupVulkan,
    RenderPipelineVulkan,
    vulkan::texture_view_create
  };

  /// A GPU buffer of the active backend.
  #[ derive( Debug ) ]
  pub enum Buffer
  {
    /// WebGPU backend buffer.
    #[ cfg( all( feature = "webgpu", target_arch = "wasm32" ) ) ]
    WebGpu( web_sys::GpuBuffer ),
    /// WebGL backend buffer.
    #[ cfg( all( feature = "webgl", target_arch = "wasm32" ) ) ]
    WebGl( BufferWebGl ),
    /// Native backend buffer.
    #[ cfg( all( feature = "native", not( target_arch = "wasm32" ) ) ) ]
    Native( wgpu::Buffer ),
    /// Native Vulkan backend buffer.
    #[ cfg( all( feature = "vulkan", not( target_arch = "wasm32" ) ) ) ]
    Vulkan( BufferVulkan )
  }

  impl Buffer
  {
    /// The raw WebGPU object, when the handle belongs to the WebGPU backend.
    #[ cfg( all( feature = "webgpu", target_arch = "wasm32" ) ) ]
    #[must_use]
    pub fn as_webgpu( &self ) -> Option< &web_sys::GpuBuffer >
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
    pub fn as_webgl( &self ) -> Option< &BufferWebGl >
    {
      match self
      {
        Self::WebGl( raw ) => Some( raw ),
        #[ cfg( all( feature = "webgpu", target_arch = "wasm32" ) ) ]
        Self::WebGpu( _ ) => None
      }
    }

    #[ cfg( all( feature = "webgpu", target_arch = "wasm32" ) ) ]
    pub( crate ) fn expect_webgpu( &self ) -> &web_sys::GpuBuffer
    {
      match self
      {
        Self::WebGpu( raw ) => raw,
        #[ cfg( all( feature = "webgl", target_arch = "wasm32" ) ) ]
        Self::WebGl( _ ) => panic!( "backend mismatch : expected a WebGPU buffer" )
      }
    }

    #[ cfg( all( feature = "webgl", target_arch = "wasm32" ) ) ]
    pub( crate ) fn expect_webgl( &self ) -> &BufferWebGl
    {
      match self
      {
        Self::WebGl( raw ) => raw,
        #[ cfg( all( feature = "webgpu", target_arch = "wasm32" ) ) ]
        Self::WebGpu( _ ) => panic!( "backend mismatch : expected a WebGL buffer" )
      }
    }

    /// The raw wgpu object, when the handle belongs to the native backend.
    #[ cfg( all( feature = "native", not( target_arch = "wasm32" ) ) ) ]
    #[must_use]
    pub fn as_native( &self ) -> Option< &wgpu::Buffer >
    {
      match self
      {
        Self::Native( raw ) => Some( raw ),
        #[ cfg( all( feature = "vulkan", not( target_arch = "wasm32" ) ) ) ]
        Self::Vulkan( _ ) => None
      }
    }

    #[ cfg( all( feature = "native", not( target_arch = "wasm32" ) ) ) ]
    pub( crate ) fn expect_native( &self ) -> &wgpu::Buffer
    {
      match self
      {
        Self::Native( raw ) => raw,
        #[ cfg( all( feature = "vulkan", not( target_arch = "wasm32" ) ) ) ]
        Self::Vulkan( _ ) => panic!( "backend mismatch : expected a native buffer" )
      }
    }

    /// The raw Vulkan object, when the handle belongs to the Vulkan backend.
    #[ cfg( all( feature = "vulkan", not( target_arch = "wasm32" ) ) ) ]
    #[must_use]
    pub fn as_vulkan( &self ) -> Option< &BufferVulkan >
    {
      match self
      {
        Self::Vulkan( raw ) => Some( raw ),
        #[ cfg( all( feature = "native", not( target_arch = "wasm32" ) ) ) ]
        Self::Native( _ ) => None
      }
    }

    #[ cfg( all( feature = "vulkan", not( target_arch = "wasm32" ) ) ) ]
    pub( crate ) fn expect_vulkan( &self ) -> &BufferVulkan
    {
      match self
      {
        Self::Vulkan( raw ) => raw,
        #[ cfg( all( feature = "native", not( target_arch = "wasm32" ) ) ) ]
        Self::Native( _ ) => panic!( "backend mismatch : expected a Vulkan buffer" )
      }
    }
  }

  /// A GPU texture of the active backend.
  #[ derive( Debug ) ]
  pub enum Texture
  {
    /// WebGPU backend texture.
    #[ cfg( all( feature = "webgpu", target_arch = "wasm32" ) ) ]
    WebGpu( web_sys::GpuTexture ),
    /// WebGL backend texture.
    #[ cfg( all( feature = "webgl", target_arch = "wasm32" ) ) ]
    WebGl( TextureWebGl ),
    /// Native backend texture.
    #[ cfg( all( feature = "native", not( target_arch = "wasm32" ) ) ) ]
    Native( wgpu::Texture ),
    /// Native Vulkan backend texture. Boxed : `TextureVulkan` embeds an
    /// `ash::Device` clone plus image/memory/format state, dwarfing every
    /// other variant ( `large_enum_variant` ) -- unboxed, every WebGPU/
    /// WebGL/native `Texture` would pay that size in padding regardless of
    /// which backend is active.
    #[ cfg( all( feature = "vulkan", not( target_arch = "wasm32" ) ) ) ]
    Vulkan( Box< TextureVulkan > )
  }

  impl Texture
  {
    /// Creates a full default view of the texture.
    ///
    /// # Errors
    ///
    /// Returns [`Error::WebGpu`] if the underlying WebGPU view-creation
    /// call fails. The WebGL and native backends never fail this call.
    pub fn view( &self ) -> Result< TextureView, Error >
    {
      match self
      {
        #[ cfg( all( feature = "webgpu", target_arch = "wasm32" ) ) ]
        Self::WebGpu( raw ) => Ok( TextureView::WebGpu( gl::texture::view( raw )? ) ),
        #[ cfg( all( feature = "webgl", target_arch = "wasm32" ) ) ]
        Self::WebGl( data ) => Ok( TextureView::WebGl( TextureViewWebGl::Texture
        {
          texture : data.texture.clone(),
          size : [ data.size[ 0 ], data.size[ 1 ] ],
          format : data.format
        } ) ),
        #[ cfg( all( feature = "native", not( target_arch = "wasm32" ) ) ) ]
        Self::Native( raw ) =>
        {
          Ok( TextureView::Native( raw.create_view( &wgpu::TextureViewDescriptor::default() ) ) )
        }
        #[ cfg( all( feature = "vulkan", not( target_arch = "wasm32" ) ) ) ]
        Self::Vulkan( raw ) => Ok( TextureView::Vulkan( texture_view_create( raw )? ) )
      }
    }

    /// The raw WebGPU object, when the handle belongs to the WebGPU backend.
    #[ cfg( all( feature = "webgpu", target_arch = "wasm32" ) ) ]
    #[must_use]
    pub fn as_webgpu( &self ) -> Option< &web_sys::GpuTexture >
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
    pub fn as_webgl( &self ) -> Option< &TextureWebGl >
    {
      match self
      {
        Self::WebGl( raw ) => Some( raw ),
        #[ cfg( all( feature = "webgpu", target_arch = "wasm32" ) ) ]
        Self::WebGpu( _ ) => None
      }
    }

    #[ cfg( all( feature = "webgpu", target_arch = "wasm32" ) ) ]
    pub( crate ) fn expect_webgpu( &self ) -> &web_sys::GpuTexture
    {
      match self
      {
        Self::WebGpu( raw ) => raw,
        #[ cfg( all( feature = "webgl", target_arch = "wasm32" ) ) ]
        Self::WebGl( _ ) => panic!( "backend mismatch : expected a WebGPU texture" )
      }
    }

    #[ cfg( all( feature = "webgl", target_arch = "wasm32" ) ) ]
    pub( crate ) fn expect_webgl( &self ) -> &TextureWebGl
    {
      match self
      {
        Self::WebGl( raw ) => raw,
        #[ cfg( all( feature = "webgpu", target_arch = "wasm32" ) ) ]
        Self::WebGpu( _ ) => panic!( "backend mismatch : expected a WebGL texture" )
      }
    }

    /// The raw wgpu object, when the handle belongs to the native backend.
    #[ cfg( all( feature = "native", not( target_arch = "wasm32" ) ) ) ]
    #[must_use]
    pub fn as_native( &self ) -> Option< &wgpu::Texture >
    {
      match self
      {
        Self::Native( raw ) => Some( raw ),
        #[ cfg( all( feature = "vulkan", not( target_arch = "wasm32" ) ) ) ]
        Self::Vulkan( _ ) => None
      }
    }

    #[ cfg( all( feature = "native", not( target_arch = "wasm32" ) ) ) ]
    pub( crate ) fn expect_native( &self ) -> &wgpu::Texture
    {
      match self
      {
        Self::Native( raw ) => raw,
        #[ cfg( all( feature = "vulkan", not( target_arch = "wasm32" ) ) ) ]
        Self::Vulkan( _ ) => panic!( "backend mismatch : expected a native texture" )
      }
    }

    /// The raw Vulkan object, when the handle belongs to the Vulkan backend.
    #[ cfg( all( feature = "vulkan", not( target_arch = "wasm32" ) ) ) ]
    #[must_use]
    pub fn as_vulkan( &self ) -> Option< &TextureVulkan >
    {
      match self
      {
        Self::Vulkan( raw ) => Some( raw ),
        #[ cfg( all( feature = "native", not( target_arch = "wasm32" ) ) ) ]
        Self::Native( _ ) => None
      }
    }

    #[ cfg( all( feature = "vulkan", not( target_arch = "wasm32" ) ) ) ]
    pub( crate ) fn expect_vulkan( &self ) -> &TextureVulkan
    {
      match self
      {
        Self::Vulkan( raw ) => raw,
        #[ cfg( all( feature = "native", not( target_arch = "wasm32" ) ) ) ]
        Self::Native( _ ) => panic!( "backend mismatch : expected a Vulkan texture" )
      }
    }
  }

  /// A view onto a GPU texture of the active backend.
  ///
  /// Cloning duplicates the lightweight view handle, not the texture.
  #[ derive( Debug, Clone ) ]
  pub enum TextureView
  {
    /// WebGPU backend texture view.
    #[ cfg( all( feature = "webgpu", target_arch = "wasm32" ) ) ]
    WebGpu( web_sys::GpuTextureView ),
    /// WebGL backend texture view.
    #[ cfg( all( feature = "webgl", target_arch = "wasm32" ) ) ]
    WebGl( TextureViewWebGl ),
    /// Native backend texture view.
    #[ cfg( all( feature = "native", not( target_arch = "wasm32" ) ) ) ]
    Native( wgpu::TextureView ),
    /// Native Vulkan backend texture view.
    #[ cfg( all( feature = "vulkan", not( target_arch = "wasm32" ) ) ) ]
    Vulkan( TextureViewVulkan )
  }

  impl TextureView
  {
    /// The raw WebGPU object, when the handle belongs to the WebGPU backend.
    #[ cfg( all( feature = "webgpu", target_arch = "wasm32" ) ) ]
    #[must_use]
    pub fn as_webgpu( &self ) -> Option< &web_sys::GpuTextureView >
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
    pub fn as_webgl( &self ) -> Option< &TextureViewWebGl >
    {
      match self
      {
        Self::WebGl( raw ) => Some( raw ),
        #[ cfg( all( feature = "webgpu", target_arch = "wasm32" ) ) ]
        Self::WebGpu( _ ) => None
      }
    }

    #[ cfg( all( feature = "webgpu", target_arch = "wasm32" ) ) ]
    pub( crate ) fn expect_webgpu( &self ) -> &web_sys::GpuTextureView
    {
      match self
      {
        Self::WebGpu( raw ) => raw,
        #[ cfg( all( feature = "webgl", target_arch = "wasm32" ) ) ]
        Self::WebGl( _ ) => panic!( "backend mismatch : expected a WebGPU texture view" )
      }
    }

    #[ cfg( all( feature = "webgl", target_arch = "wasm32" ) ) ]
    pub( crate ) fn expect_webgl( &self ) -> &TextureViewWebGl
    {
      match self
      {
        Self::WebGl( raw ) => raw,
        #[ cfg( all( feature = "webgpu", target_arch = "wasm32" ) ) ]
        Self::WebGpu( _ ) => panic!( "backend mismatch : expected a WebGL texture view" )
      }
    }

    /// The raw wgpu object, when the handle belongs to the native backend.
    #[ cfg( all( feature = "native", not( target_arch = "wasm32" ) ) ) ]
    #[must_use]
    pub fn as_native( &self ) -> Option< &wgpu::TextureView >
    {
      match self
      {
        Self::Native( raw ) => Some( raw ),
        #[ cfg( all( feature = "vulkan", not( target_arch = "wasm32" ) ) ) ]
        Self::Vulkan( _ ) => None
      }
    }

    #[ cfg( all( feature = "native", not( target_arch = "wasm32" ) ) ) ]
    pub( crate ) fn expect_native( &self ) -> &wgpu::TextureView
    {
      match self
      {
        Self::Native( raw ) => raw,
        #[ cfg( all( feature = "vulkan", not( target_arch = "wasm32" ) ) ) ]
        Self::Vulkan( _ ) => panic!( "backend mismatch : expected a native texture view" )
      }
    }

    /// The raw Vulkan object, when the handle belongs to the Vulkan backend.
    #[ cfg( all( feature = "vulkan", not( target_arch = "wasm32" ) ) ) ]
    #[must_use]
    pub fn as_vulkan( &self ) -> Option< &TextureViewVulkan >
    {
      match self
      {
        Self::Vulkan( raw ) => Some( raw ),
        #[ cfg( all( feature = "native", not( target_arch = "wasm32" ) ) ) ]
        Self::Native( _ ) => None
      }
    }

    #[ cfg( all( feature = "vulkan", not( target_arch = "wasm32" ) ) ) ]
    pub( crate ) fn expect_vulkan( &self ) -> &TextureViewVulkan
    {
      match self
      {
        Self::Vulkan( raw ) => raw,
        #[ cfg( all( feature = "native", not( target_arch = "wasm32" ) ) ) ]
        Self::Native( _ ) => panic!( "backend mismatch : expected a Vulkan texture view" )
      }
    }
  }

  /// A texture sampler of the active backend.
  #[ derive( Debug ) ]
  pub enum Sampler
  {
    /// WebGPU backend sampler.
    #[ cfg( all( feature = "webgpu", target_arch = "wasm32" ) ) ]
    WebGpu( web_sys::GpuSampler ),
    /// WebGL backend sampler.
    #[ cfg( all( feature = "webgl", target_arch = "wasm32" ) ) ]
    WebGl( web_sys::WebGlSampler ),
    /// Native backend sampler.
    #[ cfg( all( feature = "native", not( target_arch = "wasm32" ) ) ) ]
    Native( wgpu::Sampler ),
    /// Native Vulkan backend sampler — the raw handle directly, unlike most
    /// other Vulkan resources, since a `VkSampler` carries no companion
    /// memory allocation or extra bookkeeping worth wrapping.
    #[ cfg( all( feature = "vulkan", not( target_arch = "wasm32" ) ) ) ]
    Vulkan( ash::vk::Sampler )
  }

  impl Sampler
  {
    /// The raw WebGPU object, when the handle belongs to the WebGPU backend.
    #[ cfg( all( feature = "webgpu", target_arch = "wasm32" ) ) ]
    #[must_use]
    pub fn as_webgpu( &self ) -> Option< &web_sys::GpuSampler >
    {
      match self
      {
        Self::WebGpu( raw ) => Some( raw ),
        #[ cfg( all( feature = "webgl", target_arch = "wasm32" ) ) ]
        Self::WebGl( _ ) => None
      }
    }

    /// The raw WebGL object, when the handle belongs to the WebGL backend.
    #[ cfg( all( feature = "webgl", target_arch = "wasm32" ) ) ]
    #[ must_use ]
    pub fn as_webgl( &self ) -> Option< &web_sys::WebGlSampler >
    {
      match self
      {
        Self::WebGl( raw ) => Some( raw ),
        #[ cfg( all( feature = "webgpu", target_arch = "wasm32" ) ) ]
        Self::WebGpu( _ ) => None
      }
    }

    #[ cfg( all( feature = "webgpu", target_arch = "wasm32" ) ) ]
    pub( crate ) fn expect_webgpu( &self ) -> &web_sys::GpuSampler
    {
      match self
      {
        Self::WebGpu( raw ) => raw,
        #[ cfg( all( feature = "webgl", target_arch = "wasm32" ) ) ]
        Self::WebGl( _ ) => panic!( "backend mismatch : expected a WebGPU sampler" )
      }
    }

    #[ cfg( all( feature = "webgl", target_arch = "wasm32" ) ) ]
    pub( crate ) fn expect_webgl( &self ) -> &web_sys::WebGlSampler
    {
      match self
      {
        Self::WebGl( raw ) => raw,
        #[ cfg( all( feature = "webgpu", target_arch = "wasm32" ) ) ]
        Self::WebGpu( _ ) => panic!( "backend mismatch : expected a WebGL sampler" )
      }
    }

    /// The raw wgpu object, when the handle belongs to the native backend.
    #[ cfg( all( feature = "native", not( target_arch = "wasm32" ) ) ) ]
    #[must_use]
    pub fn as_native( &self ) -> Option< &wgpu::Sampler >
    {
      match self
      {
        Self::Native( raw ) => Some( raw ),
        #[ cfg( all( feature = "vulkan", not( target_arch = "wasm32" ) ) ) ]
        Self::Vulkan( _ ) => None
      }
    }

    #[ cfg( all( feature = "native", not( target_arch = "wasm32" ) ) ) ]
    pub( crate ) fn expect_native( &self ) -> &wgpu::Sampler
    {
      match self
      {
        Self::Native( raw ) => raw,
        #[ cfg( all( feature = "vulkan", not( target_arch = "wasm32" ) ) ) ]
        Self::Vulkan( _ ) => panic!( "backend mismatch : expected a native sampler" )
      }
    }

    /// The raw Vulkan object, when the handle belongs to the Vulkan backend.
    #[ cfg( all( feature = "vulkan", not( target_arch = "wasm32" ) ) ) ]
    #[must_use]
    pub fn as_vulkan( &self ) -> Option< &ash::vk::Sampler >
    {
      match self
      {
        Self::Vulkan( raw ) => Some( raw ),
        #[ cfg( all( feature = "native", not( target_arch = "wasm32" ) ) ) ]
        Self::Native( _ ) => None
      }
    }

    #[ cfg( all( feature = "vulkan", not( target_arch = "wasm32" ) ) ) ]
    pub( crate ) fn expect_vulkan( &self ) -> &ash::vk::Sampler
    {
      match self
      {
        Self::Vulkan( raw ) => raw,
        #[ cfg( all( feature = "native", not( target_arch = "wasm32" ) ) ) ]
        Self::Native( _ ) => panic!( "backend mismatch : expected a Vulkan sampler" )
      }
    }
  }

  /// A compiled shader module of the active backend.
  #[ derive( Debug ) ]
  pub enum ShaderModule
  {
    /// WebGPU backend shader module.
    #[ cfg( all( feature = "webgpu", target_arch = "wasm32" ) ) ]
    WebGpu( web_sys::GpuShaderModule ),
    /// WebGL backend shader module.
    #[ cfg( all( feature = "webgl", target_arch = "wasm32" ) ) ]
    WebGl( ShaderModuleWebGl ),
    /// Native backend shader module.
    #[ cfg( all( feature = "native", not( target_arch = "wasm32" ) ) ) ]
    Native( wgpu::ShaderModule ),
    /// Native Vulkan backend shader module — the raw handle directly; naga's
    /// WGSL -> SPIR-V translation happens once at creation time, leaving
    /// nothing else worth wrapping.
    #[ cfg( all( feature = "vulkan", not( target_arch = "wasm32" ) ) ) ]
    Vulkan( ash::vk::ShaderModule )
  }

  impl ShaderModule
  {
    /// The raw WebGPU object, when the handle belongs to the WebGPU backend.
    #[ cfg( all( feature = "webgpu", target_arch = "wasm32" ) ) ]
    #[must_use]
    pub fn as_webgpu( &self ) -> Option< &web_sys::GpuShaderModule >
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
    pub fn as_webgl( &self ) -> Option< &ShaderModuleWebGl >
    {
      match self
      {
        Self::WebGl( raw ) => Some( raw ),
        #[ cfg( all( feature = "webgpu", target_arch = "wasm32" ) ) ]
        Self::WebGpu( _ ) => None
      }
    }

    #[ cfg( all( feature = "webgpu", target_arch = "wasm32" ) ) ]
    pub( crate ) fn expect_webgpu( &self ) -> &web_sys::GpuShaderModule
    {
      match self
      {
        Self::WebGpu( raw ) => raw,
        #[ cfg( all( feature = "webgl", target_arch = "wasm32" ) ) ]
        Self::WebGl( _ ) => panic!( "backend mismatch : expected a WebGPU shader module" )
      }
    }

    #[ cfg( all( feature = "webgl", target_arch = "wasm32" ) ) ]
    pub( crate ) fn expect_webgl( &self ) -> &ShaderModuleWebGl
    {
      match self
      {
        Self::WebGl( raw ) => raw,
        #[ cfg( all( feature = "webgpu", target_arch = "wasm32" ) ) ]
        Self::WebGpu( _ ) => panic!( "backend mismatch : expected a WebGL shader module" )
      }
    }

    /// The raw wgpu object, when the handle belongs to the native backend.
    #[ cfg( all( feature = "native", not( target_arch = "wasm32" ) ) ) ]
    #[must_use]
    pub fn as_native( &self ) -> Option< &wgpu::ShaderModule >
    {
      match self
      {
        Self::Native( raw ) => Some( raw ),
        #[ cfg( all( feature = "vulkan", not( target_arch = "wasm32" ) ) ) ]
        Self::Vulkan( _ ) => None
      }
    }

    #[ cfg( all( feature = "native", not( target_arch = "wasm32" ) ) ) ]
    pub( crate ) fn expect_native( &self ) -> &wgpu::ShaderModule
    {
      match self
      {
        Self::Native( raw ) => raw,
        #[ cfg( all( feature = "vulkan", not( target_arch = "wasm32" ) ) ) ]
        Self::Vulkan( _ ) => panic!( "backend mismatch : expected a native shader module" )
      }
    }

    /// The raw Vulkan object, when the handle belongs to the Vulkan backend.
    #[ cfg( all( feature = "vulkan", not( target_arch = "wasm32" ) ) ) ]
    #[must_use]
    pub fn as_vulkan( &self ) -> Option< &ash::vk::ShaderModule >
    {
      match self
      {
        Self::Vulkan( raw ) => Some( raw ),
        #[ cfg( all( feature = "native", not( target_arch = "wasm32" ) ) ) ]
        Self::Native( _ ) => None
      }
    }

    #[ cfg( all( feature = "vulkan", not( target_arch = "wasm32" ) ) ) ]
    pub( crate ) fn expect_vulkan( &self ) -> &ash::vk::ShaderModule
    {
      match self
      {
        Self::Vulkan( raw ) => raw,
        #[ cfg( all( feature = "native", not( target_arch = "wasm32" ) ) ) ]
        Self::Native( _ ) => panic!( "backend mismatch : expected a Vulkan shader module" )
      }
    }
  }

  /// A bind group layout of the active backend.
  #[ derive( Debug ) ]
  pub enum BindGroupLayout
  {
    /// WebGPU backend bind group layout.
    #[ cfg( all( feature = "webgpu", target_arch = "wasm32" ) ) ]
    WebGpu( web_sys::GpuBindGroupLayout ),
    /// WebGL backend bind group layout.
    #[ cfg( all( feature = "webgl", target_arch = "wasm32" ) ) ]
    WebGl( BindGroupLayoutWebGl ),
    /// Native backend bind group layout.
    #[ cfg( all( feature = "native", not( target_arch = "wasm32" ) ) ) ]
    Native( wgpu::BindGroupLayout ),
    /// Native Vulkan backend bind group layout.
    #[ cfg( all( feature = "vulkan", not( target_arch = "wasm32" ) ) ) ]
    Vulkan( BindGroupLayoutVulkan )
  }

  impl BindGroupLayout
  {
    /// The raw WebGPU object, when the handle belongs to the WebGPU backend.
    #[ cfg( all( feature = "webgpu", target_arch = "wasm32" ) ) ]
    #[must_use]
    pub fn as_webgpu( &self ) -> Option< &web_sys::GpuBindGroupLayout >
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
    pub fn as_webgl( &self ) -> Option< &BindGroupLayoutWebGl >
    {
      match self
      {
        Self::WebGl( raw ) => Some( raw ),
        #[ cfg( all( feature = "webgpu", target_arch = "wasm32" ) ) ]
        Self::WebGpu( _ ) => None
      }
    }

    #[ cfg( all( feature = "webgpu", target_arch = "wasm32" ) ) ]
    pub( crate ) fn expect_webgpu( &self ) -> &web_sys::GpuBindGroupLayout
    {
      match self
      {
        Self::WebGpu( raw ) => raw,
        #[ cfg( all( feature = "webgl", target_arch = "wasm32" ) ) ]
        Self::WebGl( _ ) => panic!( "backend mismatch : expected a WebGPU bind group layout" )
      }
    }

    #[ cfg( all( feature = "webgl", target_arch = "wasm32" ) ) ]
    pub( crate ) fn expect_webgl( &self ) -> &BindGroupLayoutWebGl
    {
      match self
      {
        Self::WebGl( raw ) => raw,
        #[ cfg( all( feature = "webgpu", target_arch = "wasm32" ) ) ]
        Self::WebGpu( _ ) => panic!( "backend mismatch : expected a WebGL bind group layout" )
      }
    }

    /// The raw wgpu object, when the handle belongs to the native backend.
    #[ cfg( all( feature = "native", not( target_arch = "wasm32" ) ) ) ]
    #[must_use]
    pub fn as_native( &self ) -> Option< &wgpu::BindGroupLayout >
    {
      match self
      {
        Self::Native( raw ) => Some( raw ),
        #[ cfg( all( feature = "vulkan", not( target_arch = "wasm32" ) ) ) ]
        Self::Vulkan( _ ) => None
      }
    }

    #[ cfg( all( feature = "native", not( target_arch = "wasm32" ) ) ) ]
    pub( crate ) fn expect_native( &self ) -> &wgpu::BindGroupLayout
    {
      match self
      {
        Self::Native( raw ) => raw,
        #[ cfg( all( feature = "vulkan", not( target_arch = "wasm32" ) ) ) ]
        Self::Vulkan( _ ) => panic!( "backend mismatch : expected a native bind group layout" )
      }
    }

    /// The raw Vulkan object, when the handle belongs to the Vulkan backend.
    #[ cfg( all( feature = "vulkan", not( target_arch = "wasm32" ) ) ) ]
    #[must_use]
    pub fn as_vulkan( &self ) -> Option< &BindGroupLayoutVulkan >
    {
      match self
      {
        Self::Vulkan( raw ) => Some( raw ),
        #[ cfg( all( feature = "native", not( target_arch = "wasm32" ) ) ) ]
        Self::Native( _ ) => None
      }
    }

    #[ cfg( all( feature = "vulkan", not( target_arch = "wasm32" ) ) ) ]
    pub( crate ) fn expect_vulkan( &self ) -> &BindGroupLayoutVulkan
    {
      match self
      {
        Self::Vulkan( raw ) => raw,
        #[ cfg( all( feature = "native", not( target_arch = "wasm32" ) ) ) ]
        Self::Native( _ ) => panic!( "backend mismatch : expected a Vulkan bind group layout" )
      }
    }
  }

  /// A bind group of the active backend.
  #[ derive( Debug ) ]
  pub enum BindGroup
  {
    /// WebGPU backend bind group.
    #[ cfg( all( feature = "webgpu", target_arch = "wasm32" ) ) ]
    WebGpu( web_sys::GpuBindGroup ),
    /// WebGL backend bind group.
    #[ cfg( all( feature = "webgl", target_arch = "wasm32" ) ) ]
    WebGl( BindGroupWebGl ),
    /// Native backend bind group.
    #[ cfg( all( feature = "native", not( target_arch = "wasm32" ) ) ) ]
    Native( wgpu::BindGroup ),
    /// Native Vulkan backend bind group.
    #[ cfg( all( feature = "vulkan", not( target_arch = "wasm32" ) ) ) ]
    Vulkan( BindGroupVulkan )
  }

  impl BindGroup
  {
    /// The raw WebGPU object, when the handle belongs to the WebGPU backend.
    #[ cfg( all( feature = "webgpu", target_arch = "wasm32" ) ) ]
    #[must_use]
    pub fn as_webgpu( &self ) -> Option< &web_sys::GpuBindGroup >
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
    pub fn as_webgl( &self ) -> Option< &BindGroupWebGl >
    {
      match self
      {
        Self::WebGl( raw ) => Some( raw ),
        #[ cfg( all( feature = "webgpu", target_arch = "wasm32" ) ) ]
        Self::WebGpu( _ ) => None
      }
    }

    #[ cfg( all( feature = "webgpu", target_arch = "wasm32" ) ) ]
    pub( crate ) fn expect_webgpu( &self ) -> &web_sys::GpuBindGroup
    {
      match self
      {
        Self::WebGpu( raw ) => raw,
        #[ cfg( all( feature = "webgl", target_arch = "wasm32" ) ) ]
        Self::WebGl( _ ) => panic!( "backend mismatch : expected a WebGPU bind group" )
      }
    }

    #[ cfg( all( feature = "webgl", target_arch = "wasm32" ) ) ]
    pub( crate ) fn expect_webgl( &self ) -> &BindGroupWebGl
    {
      match self
      {
        Self::WebGl( raw ) => raw,
        #[ cfg( all( feature = "webgpu", target_arch = "wasm32" ) ) ]
        Self::WebGpu( _ ) => panic!( "backend mismatch : expected a WebGL bind group" )
      }
    }

    /// The raw wgpu object, when the handle belongs to the native backend.
    #[ cfg( all( feature = "native", not( target_arch = "wasm32" ) ) ) ]
    #[must_use]
    pub fn as_native( &self ) -> Option< &wgpu::BindGroup >
    {
      match self
      {
        Self::Native( raw ) => Some( raw ),
        #[ cfg( all( feature = "vulkan", not( target_arch = "wasm32" ) ) ) ]
        Self::Vulkan( _ ) => None
      }
    }

    #[ cfg( all( feature = "native", not( target_arch = "wasm32" ) ) ) ]
    pub( crate ) fn expect_native( &self ) -> &wgpu::BindGroup
    {
      match self
      {
        Self::Native( raw ) => raw,
        #[ cfg( all( feature = "vulkan", not( target_arch = "wasm32" ) ) ) ]
        Self::Vulkan( _ ) => panic!( "backend mismatch : expected a native bind group" )
      }
    }

    /// The raw Vulkan object, when the handle belongs to the Vulkan backend.
    #[ cfg( all( feature = "vulkan", not( target_arch = "wasm32" ) ) ) ]
    #[must_use]
    pub fn as_vulkan( &self ) -> Option< &BindGroupVulkan >
    {
      match self
      {
        Self::Vulkan( raw ) => Some( raw ),
        #[ cfg( all( feature = "native", not( target_arch = "wasm32" ) ) ) ]
        Self::Native( _ ) => None
      }
    }

    #[ cfg( all( feature = "vulkan", not( target_arch = "wasm32" ) ) ) ]
    pub( crate ) fn expect_vulkan( &self ) -> &BindGroupVulkan
    {
      match self
      {
        Self::Vulkan( raw ) => raw,
        #[ cfg( all( feature = "native", not( target_arch = "wasm32" ) ) ) ]
        Self::Native( _ ) => panic!( "backend mismatch : expected a Vulkan bind group" )
      }
    }
  }

  /// A render pipeline of the active backend.
  #[ derive( Debug ) ]
  pub enum RenderPipeline
  {
    /// WebGPU backend render pipeline.
    #[ cfg( all( feature = "webgpu", target_arch = "wasm32" ) ) ]
    WebGpu( web_sys::GpuRenderPipeline ),
    /// WebGL backend render pipeline; shared because the pass holds it as
    /// the current draw state.
    #[ cfg( all( feature = "webgl", target_arch = "wasm32" ) ) ]
    WebGl( Rc< RenderPipelineWebGl > ),
    /// Native backend render pipeline.
    #[ cfg( all( feature = "native", not( target_arch = "wasm32" ) ) ) ]
    Native( wgpu::RenderPipeline ),
    /// Native Vulkan backend render pipeline.
    #[ cfg( all( feature = "vulkan", not( target_arch = "wasm32" ) ) ) ]
    Vulkan( RenderPipelineVulkan )
  }

  impl RenderPipeline
  {
    /// The raw WebGPU object, when the handle belongs to the WebGPU backend.
    #[ cfg( all( feature = "webgpu", target_arch = "wasm32" ) ) ]
    #[must_use]
    pub fn as_webgpu( &self ) -> Option< &web_sys::GpuRenderPipeline >
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
    pub fn as_webgl( &self ) -> Option< &RenderPipelineWebGl >
    {
      match self
      {
        Self::WebGl( raw ) => Some( raw.as_ref() ),
        #[ cfg( all( feature = "webgpu", target_arch = "wasm32" ) ) ]
        Self::WebGpu( _ ) => None
      }
    }

    #[ cfg( all( feature = "webgpu", target_arch = "wasm32" ) ) ]
    pub( crate ) fn expect_webgpu( &self ) -> &web_sys::GpuRenderPipeline
    {
      match self
      {
        Self::WebGpu( raw ) => raw,
        #[ cfg( all( feature = "webgl", target_arch = "wasm32" ) ) ]
        Self::WebGl( _ ) => panic!( "backend mismatch : expected a WebGPU render pipeline" )
      }
    }

    #[ cfg( all( feature = "webgl", target_arch = "wasm32" ) ) ]
    pub( crate ) fn expect_webgl( &self ) -> &Rc< RenderPipelineWebGl >
    {
      match self
      {
        Self::WebGl( raw ) => raw,
        #[ cfg( all( feature = "webgpu", target_arch = "wasm32" ) ) ]
        Self::WebGpu( _ ) => panic!( "backend mismatch : expected a WebGL render pipeline" )
      }
    }

    /// The raw wgpu object, when the handle belongs to the native backend.
    #[ cfg( all( feature = "native", not( target_arch = "wasm32" ) ) ) ]
    #[must_use]
    pub fn as_native( &self ) -> Option< &wgpu::RenderPipeline >
    {
      match self
      {
        Self::Native( raw ) => Some( raw ),
        #[ cfg( all( feature = "vulkan", not( target_arch = "wasm32" ) ) ) ]
        Self::Vulkan( _ ) => None
      }
    }

    #[ cfg( all( feature = "native", not( target_arch = "wasm32" ) ) ) ]
    pub( crate ) fn expect_native( &self ) -> &wgpu::RenderPipeline
    {
      match self
      {
        Self::Native( raw ) => raw,
        #[ cfg( all( feature = "vulkan", not( target_arch = "wasm32" ) ) ) ]
        Self::Vulkan( _ ) => panic!( "backend mismatch : expected a native render pipeline" )
      }
    }

    /// The raw Vulkan object, when the handle belongs to the Vulkan backend.
    #[ cfg( all( feature = "vulkan", not( target_arch = "wasm32" ) ) ) ]
    #[must_use]
    pub fn as_vulkan( &self ) -> Option< &RenderPipelineVulkan >
    {
      match self
      {
        Self::Vulkan( raw ) => Some( raw ),
        #[ cfg( all( feature = "native", not( target_arch = "wasm32" ) ) ) ]
        Self::Native( _ ) => None
      }
    }

    #[ cfg( all( feature = "vulkan", not( target_arch = "wasm32" ) ) ) ]
    pub( crate ) fn expect_vulkan( &self ) -> &RenderPipelineVulkan
    {
      match self
      {
        Self::Vulkan( raw ) => raw,
        #[ cfg( all( feature = "native", not( target_arch = "wasm32" ) ) ) ]
        Self::Native( _ ) => panic!( "backend mismatch : expected a Vulkan render pipeline" )
      }
    }
  }

  /// A resource bound by one bind group entry; binding indices are
  /// sequential, mirroring `BindGroupLayoutEntry` order.
  #[ derive( Debug, Clone, Copy ) ]
  pub enum BindingResource< 'a >
  {
    /// A whole uniform buffer.
    Buffer( &'a Buffer ),
    /// A sampled texture view.
    TextureView( &'a TextureView ),
    /// A texture sampler.
    Sampler( &'a Sampler )
  }

  /// Render pipeline description of the v0 surface: triangle list, one
  /// color target without blending, optional always-on depth ( test `less`,
  /// write on ).
  #[ derive( Debug ) ]
  pub struct RenderPipelineDesc< 'a >
  {
    /// Shader module holding both entry points.
    pub shader : &'a ShaderModule,
    /// Vertex entry point name.
    pub vertex_entry : &'a str,
    /// Fragment entry point name.
    pub fragment_entry : &'a str,
    /// Vertex buffer slot layouts.
    pub vertex_buffers : &'a [ VertexBufferLayout ],
    /// Bind group layouts, group index order.
    pub bind_group_layouts : &'a [ &'a BindGroupLayout ],
    /// Format of the color target.
    pub color_format : TextureFormat,
    /// Depth attachment state, when depth testing is wanted.
    pub depth : Option< DepthState >,
    /// Whether to cull back faces.
    pub cull_back : bool
  }
}

crate::mod_interface!
{
  orphan use
  {
    Buffer,
    Texture,
    TextureView,
    Sampler,
    ShaderModule,
    BindGroupLayout,
    BindGroup,
    RenderPipeline,
    BindingResource,
    RenderPipelineDesc
  };
}
