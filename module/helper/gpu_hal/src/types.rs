mod private
{
  #[ cfg( all( feature = "webgpu", target_arch = "wasm32" ) ) ]
  use minwebgpu as gl;
  use crate::Error;

  /// Buffer usage bit flags ( WebGPU bit values ).
  #[ derive( Debug, Clone, Copy, PartialEq, Eq ) ]
  pub struct BufferUsage( u32 );

  impl BufferUsage
  {
    /// Destination of queue writes.
    pub const COPY_DST : Self = Self( 8 );
    /// Index buffer.
    pub const INDEX : Self = Self( 16 );
    /// Vertex buffer.
    pub const VERTEX : Self = Self( 32 );
    /// Uniform buffer.
    pub const UNIFORM : Self = Self( 64 );

    /// The raw WebGPU bit value.
    #[must_use]
    pub fn bits( self ) -> u32
    {
      self.0
    }

    /// Whether every bit of `other` is set in `self`.
    #[must_use]
    pub fn contains( self, other : Self ) -> bool
    {
      self.0 & other.0 == other.0
    }
  }

  impl core::ops::BitOr for BufferUsage
  {
    type Output = Self;
    fn bitor( self, rhs : Self ) -> Self
    {
      Self( self.0 | rhs.0 )
    }
  }

  /// Texture usage bit flags ( WebGPU bit values ).
  #[ derive( Debug, Clone, Copy, PartialEq, Eq ) ]
  pub struct TextureUsage( u32 );

  impl TextureUsage
  {
    /// Destination of texture uploads.
    pub const COPY_DST : Self = Self( 2 );
    /// Sampled or loaded from shaders.
    pub const TEXTURE_BINDING : Self = Self( 4 );
    /// Color or depth attachment of a render pass.
    pub const RENDER_ATTACHMENT : Self = Self( 16 );

    /// The raw WebGPU bit value.
    #[must_use]
    pub fn bits( self ) -> u32
    {
      self.0
    }

    /// Whether every bit of `other` is set in `self`.
    #[must_use]
    pub fn contains( self, other : Self ) -> bool
    {
      self.0 & other.0 == other.0
    }
  }

  impl core::ops::BitOr for TextureUsage
  {
    type Output = Self;
    fn bitor( self, rhs : Self ) -> Self
    {
      Self( self.0 | rhs.0 )
    }
  }

  /// Texture formats of the v0 surface.
  #[ derive( Debug, Clone, Copy, PartialEq, Eq ) ]
  pub enum TextureFormat
  {
    /// 8-bit rgba, linear.
    Rgba8Unorm,
    /// 8-bit rgba, sRGB-encoded.
    Rgba8UnormSrgb,
    /// 8-bit bgra, linear ( common canvas format ).
    Bgra8Unorm,
    /// 16-bit float rgba.
    Rgba16Float,
    /// 24-bit depth.
    Depth24Plus
  }

  impl TextureFormat
  {
    /// Bytes occupied by one texel, for `bytes_per_row` computation on a
    /// tightly-packed ( unpadded ) CPU-side upload buffer.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Unsupported`] for `Depth24Plus`, whose CPU-side
    /// byte layout is platform-defined and not a portable upload target.
    pub fn bytes_per_texel( self ) -> Result< u32, Error >
    {
      match self
      {
        Self::Rgba8Unorm | Self::Rgba8UnormSrgb | Self::Bgra8Unorm => Ok( 4 ),
        Self::Rgba16Float => Ok( 8 ),
        Self::Depth24Plus =>
        {
          Err( Error::Unsupported( "depth24plus has no portable CPU-side texel layout".to_string() ) )
        }
      }
    }
  }

  #[ cfg( all( feature = "webgpu", target_arch = "wasm32" ) ) ]
  impl From< TextureFormat > for gl::GpuTextureFormat
  {
    /// The equivalent raw WebGPU format.
    fn from( value : TextureFormat ) -> Self
    {
      match value
      {
        TextureFormat::Rgba8Unorm => gl::GpuTextureFormat::Rgba8unorm,
        TextureFormat::Rgba8UnormSrgb => gl::GpuTextureFormat::Rgba8unormSrgb,
        TextureFormat::Bgra8Unorm => gl::GpuTextureFormat::Bgra8unorm,
        TextureFormat::Rgba16Float => gl::GpuTextureFormat::Rgba16float,
        TextureFormat::Depth24Plus => gl::GpuTextureFormat::Depth24plus
      }
    }
  }

  #[ cfg( all( feature = "webgpu", target_arch = "wasm32" ) ) ]
  impl TryFrom< gl::GpuTextureFormat > for TextureFormat
  {
    /// The error type returned if the conversion fails.
    type Error = Error;

    /// The HAL equivalent of a raw WebGPU format, when the v0 surface has one.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Unsupported`] when `format` has no equivalent in the
    /// v0 surface.
    fn try_from( format : gl::GpuTextureFormat ) -> Result< Self, Self::Error >
    {
      match format
      {
        gl::GpuTextureFormat::Rgba8unorm => Ok( Self::Rgba8Unorm ),
        gl::GpuTextureFormat::Rgba8unormSrgb => Ok( Self::Rgba8UnormSrgb ),
        gl::GpuTextureFormat::Bgra8unorm => Ok( Self::Bgra8Unorm ),
        gl::GpuTextureFormat::Rgba16float => Ok( Self::Rgba16Float ),
        gl::GpuTextureFormat::Depth24plus => Ok( Self::Depth24Plus ),
        other => Err( Error::Unsupported( format!( "texture format {other:?} is outside the v0 surface" ) ) )
      }
    }
  }

  /// Vertex attribute formats of the v0 surface.
  #[ derive( Debug, Clone, Copy, PartialEq, Eq ) ]
  pub enum VertexFormat
  {
    /// Two 32-bit floats.
    Float32x2,
    /// Three 32-bit floats.
    Float32x3,
    /// Four 32-bit floats.
    Float32x4
  }

  #[ cfg( all( feature = "webgpu", target_arch = "wasm32" ) ) ]
  impl From< VertexFormat > for gl::GpuVertexFormat
  {
    /// The equivalent raw WebGPU format.
    fn from( value : VertexFormat ) -> Self
    {
      match value
      {
        VertexFormat::Float32x2 => gl::GpuVertexFormat::Float32x2,
        VertexFormat::Float32x3 => gl::GpuVertexFormat::Float32x3,
        VertexFormat::Float32x4 => gl::GpuVertexFormat::Float32x4
      }
    }
  }

  /// Index formats of the v0 surface.
  #[ derive( Debug, Clone, Copy, PartialEq, Eq ) ]
  pub enum IndexFormat
  {
    /// 32-bit indices.
    Uint32
  }

  #[ cfg( all( feature = "webgpu", target_arch = "wasm32" ) ) ]
  impl From< IndexFormat > for gl::GpuIndexFormat
  {
    /// The equivalent raw WebGPU format.
    fn from( value : IndexFormat ) -> Self
    {
      match value
      {
        IndexFormat::Uint32 => gl::GpuIndexFormat::Uint32
      }
    }
  }

  /// Shader stage bit flags ( WebGPU bit values ).
  #[ derive( Debug, Clone, Copy, PartialEq, Eq ) ]
  pub struct ShaderStages( u32 );

  impl ShaderStages
  {
    /// Vertex stage.
    pub const VERTEX : Self = Self( 1 );
    /// Fragment stage.
    pub const FRAGMENT : Self = Self( 2 );

    /// The raw WebGPU bit value.
    #[must_use]
    pub fn bits( self ) -> u32
    {
      self.0
    }

    /// Whether every bit of `other` is set in `self`.
    #[must_use]
    pub fn contains( self, other : Self ) -> bool
    {
      self.0 & other.0 == other.0
    }
  }

  impl core::ops::BitOr for ShaderStages
  {
    type Output = Self;
    fn bitor( self, rhs : Self ) -> Self
    {
      Self( self.0 | rhs.0 )
    }
  }

  /// What a bind group layout entry binds.
  #[ derive( Debug, Clone, Copy, PartialEq, Eq ) ]
  pub enum BindingType
  {
    /// A uniform buffer.
    UniformBuffer,
    /// A sampled texture.
    Texture,
    /// A texture sampler.
    Sampler
  }

  /// One entry of a bind group layout; binding indices are sequential.
  #[ derive( Debug, Clone, Copy ) ]
  pub struct BindGroupLayoutEntry
  {
    /// Stages that can see the binding.
    pub visibility : ShaderStages,
    /// What the entry binds.
    pub ty : BindingType
  }

  /// One vertex attribute within a vertex buffer.
  #[ derive( Debug, Clone, Copy ) ]
  pub struct VertexAttribute
  {
    /// Shader location.
    pub location : u32,
    /// Attribute format.
    pub format : VertexFormat,
    /// Byte offset within the stride.
    pub offset : u32
  }

  /// Layout of one vertex buffer slot.
  #[ derive( Debug, Clone ) ]
  pub struct VertexBufferLayout
  {
    /// Byte stride between vertices.
    pub stride : u32,
    /// Attributes read from this buffer.
    pub attributes : Vec< VertexAttribute >
  }

  /// Texture filtering modes.
  #[ derive( Debug, Clone, Copy, PartialEq, Eq ) ]
  pub enum FilterMode
  {
    /// Nearest-texel.
    Nearest,
    /// Linear interpolation.
    Linear
  }

  /// Texture addressing modes.
  #[ derive( Debug, Clone, Copy, PartialEq, Eq ) ]
  pub enum AddressMode
  {
    /// Clamp coordinates to the edge.
    ClampToEdge,
    /// Repeat the texture.
    Repeat
  }

  /// Sampler description; one filter and address mode for every axis.
  ///
  /// Defaults mirror WebGPU's: nearest filtering, clamp-to-edge addressing.
  #[ derive( Debug, Clone, Copy ) ]
  pub struct SamplerDesc
  {
    /// Filtering for minification and magnification.
    pub filter : FilterMode,
    /// Addressing for every axis.
    pub address : AddressMode
  }

  impl Default for SamplerDesc
  {
    fn default() -> Self
    {
      Self
      {
        filter : FilterMode::Nearest,
        address : AddressMode::ClampToEdge
      }
    }
  }

  /// Texture description of the v0 surface: 2d, one mip, one sample.
  #[ derive( Debug, Clone, Copy ) ]
  pub struct TextureDesc
  {
    /// Width, height, depth-or-layers.
    pub size : [ u32; 3 ],
    /// Texel format.
    pub format : TextureFormat,
    /// Usage bits.
    pub usage : TextureUsage
  }

  /// Shader sources: canonical WGSL plus a per-backend override slot
  /// ( ADR-001 §5 ). The WebGPU backend consumes `wgsl`; the WebGL backend
  /// requires the GLSL pair until build-time transpilation exists.
  #[ derive( Debug, Clone, Copy ) ]
  pub struct ShaderSource< 'a >
  {
    /// Canonical WGSL module ( vertex + fragment entry points ).
    pub wgsl : &'a str,
    /// GLSL ES vertex stage override.
    pub glsl_vertex : Option< &'a str >,
    /// GLSL ES fragment stage override.
    pub glsl_fragment : Option< &'a str >
  }

  /// Depth attachment state of a render pipeline: depth test `less`, depth
  /// write on — the v0 fixed function set.
  #[ derive( Debug, Clone, Copy ) ]
  pub struct DepthState
  {
    /// Format of the depth attachment.
    pub format : TextureFormat
  }

  /// Clip-space depth range a backend's projection matrices must target.
  #[ derive( Debug, Clone, Copy, PartialEq, Eq ) ]
  pub enum DepthRange
  {
    /// 0..1 — WebGPU.
    ZeroToOne,
    /// -1..1 — WebGL.
    NegOneToOne
  }
}

crate::mod_interface!
{
  orphan use
  {
    BufferUsage,
    TextureUsage,
    TextureFormat,
    VertexFormat,
    IndexFormat,
    ShaderStages,
    BindingType,
    BindGroupLayoutEntry,
    VertexAttribute,
    VertexBufferLayout,
    FilterMode,
    AddressMode,
    SamplerDesc,
    TextureDesc,
    ShaderSource,
    DepthState,
    DepthRange
  };
}
