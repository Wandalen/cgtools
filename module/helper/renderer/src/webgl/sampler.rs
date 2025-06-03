mod private
{
  use mingl::Former;
  use minwebgl::{ self as gl };

  /// Defines how texture coordinates outside the range [0, 1] should be handled.
  #[ derive( Default, Clone, Copy ) ]
  pub enum WrappingMode
  {
    /// Repeats the texture image.
    #[ default ]
    Repeat,
    /// Clamps the texture coordinates to the edge of the texture image.
    ClampToEdge,
    /// Repeats the texture image, mirroring it at each integer boundary.
    MirroredRepeat
  }

  /// Defines how the color of a texel is determined when it is magnified (stretched).
  #[ derive( Default, Clone, Copy ) ]
  pub enum MagFilterMode
  {
    /// Returns the weighted average of the four nearest texels.
    #[ default ]
    Linear,
    /// Returns the value of the nearest texel.
    Nearest
  }

  /// Defines how the color of a texel is determined when it is minified (shrunk).
  #[ derive( Default, Clone, Copy ) ]
  pub enum MinFilterMode
  {
    /// Returns the weighted average of the four nearest texels.
    Linear,
    /// Returns the value of the nearest texel.
    Nearest,
    /// Selects the mipmap that most closely matches the size of the pixel being textured
    /// and uses the nearest texel in that mipmap.
    NearestMipmapNearest,
    /// Selects the mipmap that most closely matches the size of the pixel being textured
    /// and uses the weighted average of the two nearest texels in that mipmap.
    #[default]
    NearestMipmapLinear,
    /// Selects the two mipmaps that most closely match the size of the pixel being textured
    /// and uses the nearest texel from each.
    LinearMipmapNearest,
    /// Selects the two mipmaps that most closely match the size of the pixel being textured
    /// and uses the weighted average of the four nearest texels from each.
    LinearMipMapLinear,
  }

  /// Defines the comparison function used for depth textures.
  #[ derive( Default, Clone, Copy ) ]
  pub enum CompareFunction
  {
    /// Passes if the source sample is less than or equal to the stored sample.
    #[default]
    Lequal,
    /// Passes if the source sample is greater than or equal to the stored sample.
    Gequal,
    /// Passes if the source sample is less than the stored sample.
    Less,
    /// Passes if the source sample is greater than the stored sample.
    Greater,
    /// Passes if the source sample is equal to the stored sample.
    Equal,
    /// Passes if the source sample is not equal to the stored sample.
    Notequal,
    /// Always passes.
    Always,
    /// Never passes.
    Never,
  }

  /// Represents the sampler state for a texture.
  #[ derive( Default, Clone, Copy, Former ) ]
  pub struct Sampler
  {
   /// The magnification filter mode.
    pub mag_filter: Option< MagFilterMode >,
    /// The minification filter mode.
    pub min_filter: Option< MinFilterMode >,
    /// The texture wrapping mode for the S coordinate.
    pub wrap_s: Option< WrappingMode >,
    /// The texture wrapping mode for the T coordinate.
    pub wrap_t: Option< WrappingMode >,
    /// The texture wrapping mode for the R coordinate (for 3D textures).
    pub wrap_r: Option< WrappingMode >,
    /// The comparison function used for depth textures.
    pub compare_func: Option< CompareFunction >,
  }

  impl Sampler 
  {
    /// Creates a new `Sampler` with default values.
    pub fn new() -> Self
    {
      Self::default()
    }

    /// Uploads the sampler settings to the WebGL context.
    pub fn upload( &self, gl : &gl::WebGl2RenderingContext, target : u32 )
    {
      if let Some( value ) =  self.mag_filter
      {
        gl.tex_parameteri( target, gl::TEXTURE_MAG_FILTER, value.to_gl() as i32 );
      }
      if let Some( value ) =  self.min_filter
      {
        gl.tex_parameteri( target, gl::TEXTURE_MIN_FILTER, value.to_gl() as i32 );
      }

      if let Some( value ) =  self.wrap_s
      {
        gl.tex_parameteri( target, gl::TEXTURE_WRAP_S, value.to_gl() as i32 );
      }
      if let Some( value ) =  self.wrap_t
      {
        gl.tex_parameteri( target, gl::TEXTURE_WRAP_T, value.to_gl() as i32 );
      }
      if let Some( value ) =  self.wrap_r
      {
        gl.tex_parameteri( target, gl::TEXTURE_WRAP_R, value.to_gl() as i32 );
      }

      if let Some( value ) =  self.compare_func
      {
        gl.tex_parameteri( target, gl::TEXTURE_COMPARE_FUNC, value.to_gl() as i32 );
      }
    }
  }

  pub trait ToFromGlEnum
  {
    /// Converts the `Self` to its corresponding WebGL enum value.
    fn to_gl( &self ) -> u32;

    /// Converts a WebGL enum value to `Self`.
    fn from_gl( value : u32 ) -> Self; 
  }

  impl ToFromGlEnum for WrappingMode
  {
    fn to_gl( &self ) -> u32
    {
      match self
      {
        Self::Repeat => gl::REPEAT,
        Self::ClampToEdge => gl::CLAMP_TO_EDGE,
        Self::MirroredRepeat => gl::MIRRORED_REPEAT
      }
    }

    fn from_gl( value : u32 ) -> Self
    {
      match value
      {
        gl::REPEAT => Self::Repeat,
        gl::CLAMP_TO_EDGE => Self::ClampToEdge,
        gl::MIRRORED_REPEAT => Self::MirroredRepeat,
        e => panic!( "Invalid WrappingMode value: {}", e )
      }
    }
  }

  impl ToFromGlEnum for  MagFilterMode
  {
    fn to_gl( &self ) -> u32
    {
      match self
      {
        Self::Linear => gl::LINEAR,
        Self::Nearest => gl::NEAREST
      }
    }

    fn from_gl( value : u32 ) -> Self
    {
      match value
      {
        gl::LINEAR => Self::Linear,
        gl::NEAREST => Self::Nearest,
        e => panic!( "Invalid MagFilterMode value: {}", e )
      }
    }
  }

  impl ToFromGlEnum for  MinFilterMode
  {
    fn to_gl( &self ) -> u32
    {
      match self
      {
        Self::Linear => gl::LINEAR,
        Self::Nearest => gl::NEAREST,
        Self::NearestMipmapNearest => gl::NEAREST_MIPMAP_NEAREST,
        Self::NearestMipmapLinear => gl::NEAREST_MIPMAP_LINEAR,
        Self::LinearMipmapNearest => gl::LINEAR_MIPMAP_NEAREST,
        Self::LinearMipMapLinear => gl::LINEAR_MIPMAP_LINEAR
      }
    }

    fn from_gl( value : u32 ) -> Self
    {
      match value
      {
        gl::LINEAR => Self::Linear,
        gl::NEAREST => Self::Nearest,
        gl::NEAREST_MIPMAP_NEAREST => Self::NearestMipmapNearest,
        gl::NEAREST_MIPMAP_LINEAR => Self::NearestMipmapLinear,
        gl::LINEAR_MIPMAP_NEAREST => Self::LinearMipmapNearest,
        gl::LINEAR_MIPMAP_LINEAR => Self::LinearMipMapLinear,
        e => panic!( "Invalid MinFilterMode value: {}", e )
      }
    }
  }

  impl ToFromGlEnum for  CompareFunction
  {
    fn to_gl( &self ) -> u32
    {
      match self
      {
        Self::Lequal => gl::LEQUAL,
        Self::Gequal => gl::GEQUAL,
        Self::Less => gl::LESS,
        Self::Greater => gl::GREATER,
        Self::Equal => gl::EQUAL,
        Self::Notequal => gl::NOTEQUAL,
        Self::Always => gl::ALWAYS,
        Self::Never => gl::NEVER
      }
    }

    fn from_gl( value : u32 ) -> Self
    {
      match value
      {
        gl::LEQUAL => Self::Lequal,
        gl::GEQUAL => Self::Gequal,
        gl::LESS => Self::Less,
        gl::GREATER => Self::Greater,
        gl::EQUAL => Self::Equal,
        gl::NOTEQUAL => Self::Notequal,
        gl::ALWAYS => Self::Always,
        gl::NEVER => Self::Never,
        e => panic!( "Invalid CompareFunction value: {}", e )
      }
    }
  }
}

crate::mod_interface!
{
  orphan use
  {
    ToFromGlEnum,
    MagFilterMode,
    MinFilterMode,
    WrappingMode,
    CompareFunction,
    Sampler
  };
}