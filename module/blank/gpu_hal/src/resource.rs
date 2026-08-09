mod private
{
  #[ cfg( feature = "webgpu" ) ]
  use minwebgpu as gl;
  #[ cfg( feature = "webgpu" ) ]
  use gl::web_sys;
  #[ cfg( all( feature = "webgl", not( feature = "webgpu" ) ) ) ]
  use minwebgl::web_sys;
  #[ cfg( feature = "webgl" ) ]
  use std::rc::Rc;
  use crate::
  {
    Error,
    TextureFormat,
    DepthState,
    VertexBufferLayout
  };
  #[ cfg( feature = "webgl" ) ]
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

  /// A GPU buffer of the active backend.
  #[ derive( Debug ) ]
  pub enum Buffer
  {
    /// WebGPU backend buffer.
    #[ cfg( feature = "webgpu" ) ]
    WebGpu( web_sys::GpuBuffer ),
    /// WebGL backend buffer.
    #[ cfg( feature = "webgl" ) ]
    WebGl( BufferWebGl )
  }

  impl Buffer
  {
    /// The raw WebGPU object, when the handle belongs to the WebGPU backend.
    #[ cfg( feature = "webgpu" ) ]
    pub fn as_webgpu( &self ) -> Option< &web_sys::GpuBuffer >
    {
      match self
      {
        Self::WebGpu( raw ) => Some( raw ),
        #[ cfg( feature = "webgl" ) ]
        Self::WebGl( _ ) => None
      }
    }

    /// The WebGL backend data, when the handle belongs to the WebGL backend.
    #[ cfg( feature = "webgl" ) ]
    pub fn as_webgl( &self ) -> Option< &BufferWebGl >
    {
      match self
      {
        Self::WebGl( raw ) => Some( raw ),
        #[ cfg( feature = "webgpu" ) ]
        Self::WebGpu( _ ) => None
      }
    }

    #[ cfg( feature = "webgpu" ) ]
    pub( crate ) fn expect_webgpu( &self ) -> &web_sys::GpuBuffer
    {
      match self
      {
        Self::WebGpu( raw ) => raw,
        #[ cfg( feature = "webgl" ) ]
        Self::WebGl( _ ) => panic!( "backend mismatch : expected a WebGPU buffer" )
      }
    }

    #[ cfg( feature = "webgl" ) ]
    pub( crate ) fn expect_webgl( &self ) -> &BufferWebGl
    {
      match self
      {
        Self::WebGl( raw ) => raw,
        #[ cfg( feature = "webgpu" ) ]
        Self::WebGpu( _ ) => panic!( "backend mismatch : expected a WebGL buffer" )
      }
    }
  }

  /// A GPU texture of the active backend.
  #[ derive( Debug ) ]
  pub enum Texture
  {
    /// WebGPU backend texture.
    #[ cfg( feature = "webgpu" ) ]
    WebGpu( web_sys::GpuTexture ),
    /// WebGL backend texture.
    #[ cfg( feature = "webgl" ) ]
    WebGl( TextureWebGl )
  }

  impl Texture
  {
    /// Creates a full default view of the texture.
    pub fn view( &self ) -> Result< TextureView, Error >
    {
      match self
      {
        #[ cfg( feature = "webgpu" ) ]
        Self::WebGpu( raw ) => Ok( TextureView::WebGpu( gl::texture::view( raw )? ) ),
        #[ cfg( feature = "webgl" ) ]
        Self::WebGl( data ) => Ok( TextureView::WebGl( TextureViewWebGl::Texture
        {
          texture : data.texture.clone(),
          size : [ data.size[ 0 ], data.size[ 1 ] ],
          format : data.format
        } ) )
      }
    }

    /// The raw WebGPU object, when the handle belongs to the WebGPU backend.
    #[ cfg( feature = "webgpu" ) ]
    pub fn as_webgpu( &self ) -> Option< &web_sys::GpuTexture >
    {
      match self
      {
        Self::WebGpu( raw ) => Some( raw ),
        #[ cfg( feature = "webgl" ) ]
        Self::WebGl( _ ) => None
      }
    }

    /// The WebGL backend data, when the handle belongs to the WebGL backend.
    #[ cfg( feature = "webgl" ) ]
    pub fn as_webgl( &self ) -> Option< &TextureWebGl >
    {
      match self
      {
        Self::WebGl( raw ) => Some( raw ),
        #[ cfg( feature = "webgpu" ) ]
        Self::WebGpu( _ ) => None
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
    #[ cfg( feature = "webgpu" ) ]
    WebGpu( web_sys::GpuTextureView ),
    /// WebGL backend texture view.
    #[ cfg( feature = "webgl" ) ]
    WebGl( TextureViewWebGl )
  }

  impl TextureView
  {
    /// The raw WebGPU object, when the handle belongs to the WebGPU backend.
    #[ cfg( feature = "webgpu" ) ]
    pub fn as_webgpu( &self ) -> Option< &web_sys::GpuTextureView >
    {
      match self
      {
        Self::WebGpu( raw ) => Some( raw ),
        #[ cfg( feature = "webgl" ) ]
        Self::WebGl( _ ) => None
      }
    }

    /// The WebGL backend data, when the handle belongs to the WebGL backend.
    #[ cfg( feature = "webgl" ) ]
    pub fn as_webgl( &self ) -> Option< &TextureViewWebGl >
    {
      match self
      {
        Self::WebGl( raw ) => Some( raw ),
        #[ cfg( feature = "webgpu" ) ]
        Self::WebGpu( _ ) => None
      }
    }

    #[ cfg( feature = "webgpu" ) ]
    pub( crate ) fn expect_webgpu( &self ) -> &web_sys::GpuTextureView
    {
      match self
      {
        Self::WebGpu( raw ) => raw,
        #[ cfg( feature = "webgl" ) ]
        Self::WebGl( _ ) => panic!( "backend mismatch : expected a WebGPU texture view" )
      }
    }

    #[ cfg( feature = "webgl" ) ]
    pub( crate ) fn expect_webgl( &self ) -> &TextureViewWebGl
    {
      match self
      {
        Self::WebGl( raw ) => raw,
        #[ cfg( feature = "webgpu" ) ]
        Self::WebGpu( _ ) => panic!( "backend mismatch : expected a WebGL texture view" )
      }
    }
  }

  /// A texture sampler of the active backend.
  #[ derive( Debug ) ]
  pub enum Sampler
  {
    /// WebGPU backend sampler.
    #[ cfg( feature = "webgpu" ) ]
    WebGpu( web_sys::GpuSampler ),
    /// WebGL backend sampler.
    #[ cfg( feature = "webgl" ) ]
    WebGl( web_sys::WebGlSampler )
  }

  impl Sampler
  {
    /// The raw WebGPU object, when the handle belongs to the WebGPU backend.
    #[ cfg( feature = "webgpu" ) ]
    pub fn as_webgpu( &self ) -> Option< &web_sys::GpuSampler >
    {
      match self
      {
        Self::WebGpu( raw ) => Some( raw ),
        #[ cfg( feature = "webgl" ) ]
        Self::WebGl( _ ) => None
      }
    }

    /// The raw WebGL object, when the handle belongs to the WebGL backend.
    #[ cfg( feature = "webgl" ) ]
    pub fn as_webgl( &self ) -> Option< &web_sys::WebGlSampler >
    {
      match self
      {
        Self::WebGl( raw ) => Some( raw ),
        #[ cfg( feature = "webgpu" ) ]
        Self::WebGpu( _ ) => None
      }
    }

    #[ cfg( feature = "webgpu" ) ]
    pub( crate ) fn expect_webgpu( &self ) -> &web_sys::GpuSampler
    {
      match self
      {
        Self::WebGpu( raw ) => raw,
        #[ cfg( feature = "webgl" ) ]
        Self::WebGl( _ ) => panic!( "backend mismatch : expected a WebGPU sampler" )
      }
    }

    #[ cfg( feature = "webgl" ) ]
    pub( crate ) fn expect_webgl( &self ) -> &web_sys::WebGlSampler
    {
      match self
      {
        Self::WebGl( raw ) => raw,
        #[ cfg( feature = "webgpu" ) ]
        Self::WebGpu( _ ) => panic!( "backend mismatch : expected a WebGL sampler" )
      }
    }
  }

  /// A compiled shader module of the active backend.
  #[ derive( Debug ) ]
  pub enum ShaderModule
  {
    /// WebGPU backend shader module.
    #[ cfg( feature = "webgpu" ) ]
    WebGpu( web_sys::GpuShaderModule ),
    /// WebGL backend shader module.
    #[ cfg( feature = "webgl" ) ]
    WebGl( ShaderModuleWebGl )
  }

  impl ShaderModule
  {
    /// The raw WebGPU object, when the handle belongs to the WebGPU backend.
    #[ cfg( feature = "webgpu" ) ]
    pub fn as_webgpu( &self ) -> Option< &web_sys::GpuShaderModule >
    {
      match self
      {
        Self::WebGpu( raw ) => Some( raw ),
        #[ cfg( feature = "webgl" ) ]
        Self::WebGl( _ ) => None
      }
    }

    /// The WebGL backend data, when the handle belongs to the WebGL backend.
    #[ cfg( feature = "webgl" ) ]
    pub fn as_webgl( &self ) -> Option< &ShaderModuleWebGl >
    {
      match self
      {
        Self::WebGl( raw ) => Some( raw ),
        #[ cfg( feature = "webgpu" ) ]
        Self::WebGpu( _ ) => None
      }
    }

    #[ cfg( feature = "webgpu" ) ]
    pub( crate ) fn expect_webgpu( &self ) -> &web_sys::GpuShaderModule
    {
      match self
      {
        Self::WebGpu( raw ) => raw,
        #[ cfg( feature = "webgl" ) ]
        Self::WebGl( _ ) => panic!( "backend mismatch : expected a WebGPU shader module" )
      }
    }

    #[ cfg( feature = "webgl" ) ]
    pub( crate ) fn expect_webgl( &self ) -> &ShaderModuleWebGl
    {
      match self
      {
        Self::WebGl( raw ) => raw,
        #[ cfg( feature = "webgpu" ) ]
        Self::WebGpu( _ ) => panic!( "backend mismatch : expected a WebGL shader module" )
      }
    }
  }

  /// A bind group layout of the active backend.
  #[ derive( Debug ) ]
  pub enum BindGroupLayout
  {
    /// WebGPU backend bind group layout.
    #[ cfg( feature = "webgpu" ) ]
    WebGpu( web_sys::GpuBindGroupLayout ),
    /// WebGL backend bind group layout.
    #[ cfg( feature = "webgl" ) ]
    WebGl( BindGroupLayoutWebGl )
  }

  impl BindGroupLayout
  {
    /// The raw WebGPU object, when the handle belongs to the WebGPU backend.
    #[ cfg( feature = "webgpu" ) ]
    pub fn as_webgpu( &self ) -> Option< &web_sys::GpuBindGroupLayout >
    {
      match self
      {
        Self::WebGpu( raw ) => Some( raw ),
        #[ cfg( feature = "webgl" ) ]
        Self::WebGl( _ ) => None
      }
    }

    /// The WebGL backend data, when the handle belongs to the WebGL backend.
    #[ cfg( feature = "webgl" ) ]
    pub fn as_webgl( &self ) -> Option< &BindGroupLayoutWebGl >
    {
      match self
      {
        Self::WebGl( raw ) => Some( raw ),
        #[ cfg( feature = "webgpu" ) ]
        Self::WebGpu( _ ) => None
      }
    }

    #[ cfg( feature = "webgpu" ) ]
    pub( crate ) fn expect_webgpu( &self ) -> &web_sys::GpuBindGroupLayout
    {
      match self
      {
        Self::WebGpu( raw ) => raw,
        #[ cfg( feature = "webgl" ) ]
        Self::WebGl( _ ) => panic!( "backend mismatch : expected a WebGPU bind group layout" )
      }
    }

    #[ cfg( feature = "webgl" ) ]
    pub( crate ) fn expect_webgl( &self ) -> &BindGroupLayoutWebGl
    {
      match self
      {
        Self::WebGl( raw ) => raw,
        #[ cfg( feature = "webgpu" ) ]
        Self::WebGpu( _ ) => panic!( "backend mismatch : expected a WebGL bind group layout" )
      }
    }
  }

  /// A bind group of the active backend.
  #[ derive( Debug ) ]
  pub enum BindGroup
  {
    /// WebGPU backend bind group.
    #[ cfg( feature = "webgpu" ) ]
    WebGpu( web_sys::GpuBindGroup ),
    /// WebGL backend bind group.
    #[ cfg( feature = "webgl" ) ]
    WebGl( BindGroupWebGl )
  }

  impl BindGroup
  {
    /// The raw WebGPU object, when the handle belongs to the WebGPU backend.
    #[ cfg( feature = "webgpu" ) ]
    pub fn as_webgpu( &self ) -> Option< &web_sys::GpuBindGroup >
    {
      match self
      {
        Self::WebGpu( raw ) => Some( raw ),
        #[ cfg( feature = "webgl" ) ]
        Self::WebGl( _ ) => None
      }
    }

    /// The WebGL backend data, when the handle belongs to the WebGL backend.
    #[ cfg( feature = "webgl" ) ]
    pub fn as_webgl( &self ) -> Option< &BindGroupWebGl >
    {
      match self
      {
        Self::WebGl( raw ) => Some( raw ),
        #[ cfg( feature = "webgpu" ) ]
        Self::WebGpu( _ ) => None
      }
    }

    #[ cfg( feature = "webgpu" ) ]
    pub( crate ) fn expect_webgpu( &self ) -> &web_sys::GpuBindGroup
    {
      match self
      {
        Self::WebGpu( raw ) => raw,
        #[ cfg( feature = "webgl" ) ]
        Self::WebGl( _ ) => panic!( "backend mismatch : expected a WebGPU bind group" )
      }
    }

    #[ cfg( feature = "webgl" ) ]
    pub( crate ) fn expect_webgl( &self ) -> &BindGroupWebGl
    {
      match self
      {
        Self::WebGl( raw ) => raw,
        #[ cfg( feature = "webgpu" ) ]
        Self::WebGpu( _ ) => panic!( "backend mismatch : expected a WebGL bind group" )
      }
    }
  }

  /// A render pipeline of the active backend.
  #[ derive( Debug ) ]
  pub enum RenderPipeline
  {
    /// WebGPU backend render pipeline.
    #[ cfg( feature = "webgpu" ) ]
    WebGpu( web_sys::GpuRenderPipeline ),
    /// WebGL backend render pipeline; shared because the pass holds it as
    /// the current draw state.
    #[ cfg( feature = "webgl" ) ]
    WebGl( Rc< RenderPipelineWebGl > )
  }

  impl RenderPipeline
  {
    /// The raw WebGPU object, when the handle belongs to the WebGPU backend.
    #[ cfg( feature = "webgpu" ) ]
    pub fn as_webgpu( &self ) -> Option< &web_sys::GpuRenderPipeline >
    {
      match self
      {
        Self::WebGpu( raw ) => Some( raw ),
        #[ cfg( feature = "webgl" ) ]
        Self::WebGl( _ ) => None
      }
    }

    /// The WebGL backend data, when the handle belongs to the WebGL backend.
    #[ cfg( feature = "webgl" ) ]
    pub fn as_webgl( &self ) -> Option< &RenderPipelineWebGl >
    {
      match self
      {
        Self::WebGl( raw ) => Some( raw.as_ref() ),
        #[ cfg( feature = "webgpu" ) ]
        Self::WebGpu( _ ) => None
      }
    }

    #[ cfg( feature = "webgpu" ) ]
    pub( crate ) fn expect_webgpu( &self ) -> &web_sys::GpuRenderPipeline
    {
      match self
      {
        Self::WebGpu( raw ) => raw,
        #[ cfg( feature = "webgl" ) ]
        Self::WebGl( _ ) => panic!( "backend mismatch : expected a WebGPU render pipeline" )
      }
    }

    #[ cfg( feature = "webgl" ) ]
    pub( crate ) fn expect_webgl( &self ) -> &Rc< RenderPipelineWebGl >
    {
      match self
      {
        Self::WebGl( raw ) => raw,
        #[ cfg( feature = "webgpu" ) ]
        Self::WebGpu( _ ) => panic!( "backend mismatch : expected a WebGL render pipeline" )
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
