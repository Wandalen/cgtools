mod private
{
  use minwebgl as gl;
  use std::rc::Rc;
  use core::cell::RefCell;
  use crate::
  {
    Error,
    TextureFormat,
    VertexFormat,
    BindGroupLayoutEntry,
    VertexBufferLayout,
    DepthState
  };

  /// Saturating u32 → i32 conversion for GL parameters — GL sizes never
  /// approach the boundary, so saturation is a formality, not a data path.
  #[ must_use ]
  pub fn to_i32( value : u32 ) -> i32
  {
    i32::try_from( value ).unwrap_or( i32::MAX )
  }

  /// Saturating usize → u32 conversion for binding indices — group and
  /// binding counts never approach the boundary, so saturation is a
  /// formality, not a data path.
  #[ must_use ]
  pub fn to_u32( value : usize ) -> u32
  {
    u32::try_from( value ).unwrap_or( u32::MAX )
  }

  /// ( group, binding ) → GL binding point or texture unit.
  pub type BindingMap = Vec< ( ( u32, u32 ), u32 ) >;

  impl TextureFormat
  {
    /// The sized WebGL2 internal format, when one exists.
    pub( crate ) fn webgl_internal_format( self ) -> Result< u32, Error >
    {
      match self
      {
        Self::Rgba8Unorm => Ok( gl::GL::RGBA8 ),
        Self::Rgba8UnormSrgb => Ok( gl::GL::SRGB8_ALPHA8 ),
        Self::Bgra8Unorm | Self::Bgra8UnormSrgb =>
        {
          Err( Error::Unsupported( "bgra8 has no WebGL2 internal format".to_string() ) )
        }
        Self::Rgba16Float => Ok( gl::GL::RGBA16F ),
        Self::Depth24Plus => Ok( gl::GL::DEPTH_COMPONENT24 )
      }
    }

    /// The ( format, type ) pair `texSubImage2D` expects for a
    /// tightly-packed upload of this format's texels.
    pub( crate ) fn webgl_format_and_type( self ) -> Result< ( u32, u32 ), Error >
    {
      match self
      {
        Self::Rgba8Unorm | Self::Rgba8UnormSrgb => Ok( ( gl::GL::RGBA, gl::GL::UNSIGNED_BYTE ) ),
        Self::Bgra8Unorm | Self::Bgra8UnormSrgb =>
        {
          Err( Error::Unsupported( "bgra8 has no WebGL2 internal format".to_string() ) )
        }
        Self::Rgba16Float => Ok( ( gl::GL::RGBA, gl::GL::HALF_FLOAT ) ),
        Self::Depth24Plus =>
        {
          Err( Error::Unsupported( "depth24plus is not a valid texSubImage2D upload target".to_string() ) )
        }
      }
    }
  }

  impl VertexFormat
  {
    /// Component count of the attribute — every v0 format is f32-based.
    pub( crate ) fn webgl_component_count( self ) -> i32
    {
      match self
      {
        Self::Float32x2 => 2,
        Self::Float32x3 => 3,
        Self::Float32x4 => 4
      }
    }
  }

  /// WebGL backend data of a buffer handle.
  #[ derive( Debug ) ]
  pub struct BufferWebGl
  {
    /// Raw GL buffer object.
    pub buffer : web_sys::WebGlBuffer,
    /// GL bind target the buffer was created for.
    pub target : u32,
    /// Size in bytes the buffer was allocated with ( `Fix(BUG-200)` : needed
    /// to validate a write against the buffer's actual capacity before
    /// calling `bufferSubData`, which silently no-ops past this size ).
    pub size : u64
  }

  /// WebGL backend data of a texture handle.
  #[ derive( Debug ) ]
  pub struct TextureWebGl
  {
    /// Raw GL texture object.
    pub texture : web_sys::WebGlTexture,
    /// Width, height, depth-or-layers.
    pub size : [ u32; 3 ],
    /// Texel format.
    pub format : TextureFormat
  }

  /// WebGL backend data of a texture view — GL has no view objects, so a
  /// view is the texture itself, or the canvas backbuffer.
  #[ derive( Debug, Clone ) ]
  pub enum TextureViewWebGl
  {
    /// View of a texture object.
    Texture
    {
      /// Raw GL texture object.
      texture : web_sys::WebGlTexture,
      /// Width, height.
      size : [ u32; 2 ],
      /// Texel format.
      format : TextureFormat
    },
    /// The canvas backbuffer ( the default framebuffer ).
    CanvasBackbuffer
  }

  /// WebGL backend data of a shader module : GLSL sources, compiled at
  /// pipeline creation — GL links per program, not per module.
  #[ derive( Debug ) ]
  pub struct ShaderModuleWebGl
  {
    /// GLSL ES vertex stage source.
    pub vertex : String,
    /// GLSL ES fragment stage source.
    pub fragment : String
  }

  /// WebGL backend data of a bind group layout : the entry list itself —
  /// GL has no layout objects.
  #[ derive( Debug ) ]
  pub struct BindGroupLayoutWebGl
  {
    /// Entries, binding index order.
    pub entries : Vec< BindGroupLayoutEntry >
  }

  /// WebGL backend data of a bind group : owned handles of the bound
  /// resources, binding index order.
  #[ derive( Debug ) ]
  pub struct BindGroupWebGl
  {
    /// Bound resources, binding index order.
    pub entries : Vec< BindGroupEntryWebGl >
  }

  /// One bound resource of a WebGL bind group.
  #[ derive( Debug ) ]
  pub enum BindGroupEntryWebGl
  {
    /// A uniform buffer.
    Buffer( web_sys::WebGlBuffer ),
    /// A sampled texture.
    Texture( web_sys::WebGlTexture ),
    /// A sampler object.
    Sampler( web_sys::WebGlSampler )
  }

  /// WebGL backend data of a render pipeline : a linked program plus the
  /// state GL applies at draw time.
  ///
  /// Bindings resolve by name convention in the GLSL override : uniform
  /// block `ub_{group}_{binding}`, sampler uniform `tex_{group}_{binding}`.
  /// A `Sampler` entry pairs with the nearest preceding `Texture` entry of
  /// its group. Names the linker pruned are skipped silently, matching GL
  /// practice for optimized-out uniforms.
  #[ derive( Debug ) ]
  pub struct RenderPipelineWebGl
  {
    /// Linked program.
    pub program : web_sys::WebGlProgram,
    /// Vertex buffer slot layouts, applied at draw time.
    pub vertex_buffers : Vec< VertexBufferLayout >,
    /// Depth attachment state, when depth testing is on.
    pub depth : Option< DepthState >,
    /// Whether back faces are culled.
    pub cull_back : bool,
    /// Uniform buffer binding points of the program.
    pub ubo_points : BindingMap,
    /// Texture units of the program.
    pub texture_units : BindingMap
  }

  /// WebGL backend data of a render pass : the context, the framebuffer
  /// the pass renders into ( `None` for the canvas backbuffer ), and the
  /// pipeline state GL needs at draw time.
  #[ derive( Debug ) ]
  pub struct RenderPassWebGl
  {
    /// The GL context commands execute against.
    pub gl : gl::GL,
    /// Framebuffer of the pass; deleted when the pass ends.
    pub fbo : Option< web_sys::WebGlFramebuffer >,
    pipeline : RefCell< Option< Rc< RenderPipelineWebGl > > >
  }

  impl RenderPassWebGl
  {
    pub( crate ) fn new( gl : gl::GL, fbo : Option< web_sys::WebGlFramebuffer > ) -> Self
    {
      Self
      {
        gl,
        fbo,
        pipeline : RefCell::new( None )
      }
    }

    pub( crate ) fn current_pipeline_set( &self, pipeline : Rc< RenderPipelineWebGl > )
    {
      *self.pipeline.borrow_mut() = Some( pipeline );
    }

    pub( crate ) fn current_pipeline( &self ) -> Option< Rc< RenderPipelineWebGl > >
    {
      self.pipeline.borrow().clone()
    }
  }
}

crate::mod_interface!
{
  own use to_i32;
  own use to_u32;

  orphan use
  {
    BindingMap,
    BufferWebGl,
    TextureWebGl,
    TextureViewWebGl,
    ShaderModuleWebGl,
    BindGroupLayoutWebGl,
    BindGroupWebGl,
    BindGroupEntryWebGl,
    RenderPipelineWebGl,
    RenderPassWebGl
  };
}
