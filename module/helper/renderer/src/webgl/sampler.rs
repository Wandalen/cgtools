mod private
{
  use minwebgl::{ self as gl };

  #[ derive( Default, Clone, Copy ) ]
  pub enum WrappingMode
  {
    #[ default ]
    Repeat,
    ClampToEdge,
    MirroredRepeat
  }

  #[ derive( Default, Clone, Copy ) ]
  pub enum MagFilterMode
  {
    #[ default ]
    Linear,
    Nearest
  }

  #[ derive( Default, Clone, Copy ) ]
  pub enum MinFilterMode
  {
    Linear,
    Nearest,
    NearestMipmapNearest,
    #[ default ]
    NearestMipmapLinear,
    LinearMipmapNearest,
    LinearMipMapLinear
  }

  #[ derive( Default, Clone, Copy ) ]
  pub enum CompareFunction
  {
    #[ default ]
    Lequal,
    Gequal,
    Less,
    Greater,
    Equal,
    Notequal,
    Always,
    Never
  }

  #[ derive( Default, Clone, Copy ) ]
  pub struct Sampler
  {
    pub mag_filter : MagFilterMode,
    pub min_filter : MinFilterMode,
    pub wrap_s : WrappingMode,
    pub wrap_t : WrappingMode,
    pub wrap_r : WrappingMode,
    pub compare_func : CompareFunction
  }

  impl Sampler 
  {
    pub fn new() -> Self
    {
      Self::default()
    }

    pub fn apply( &self, gl : &gl::WebGl2RenderingContext, target : u32 )
    {
      gl.tex_parameteri( target, gl::TEXTURE_MAG_FILTER, self.mag_filter.to_gl() as i32 );
      gl.tex_parameteri( target, gl::TEXTURE_MIN_FILTER, self.min_filter.to_gl() as i32 );

      gl.tex_parameteri( target, gl::TEXTURE_WRAP_S, self.wrap_s.to_gl() as i32 );
      gl.tex_parameteri( target, gl::TEXTURE_WRAP_T, self.wrap_t.to_gl() as i32 );
      gl.tex_parameteri( target, gl::TEXTURE_WRAP_R, self.wrap_r.to_gl() as i32 );

      gl.tex_parameteri( target, gl::TEXTURE_COMPARE_FUNC, self.compare_func.to_gl() as i32 );
    }
  }

  impl WrappingMode
  {
    pub fn to_gl( &self ) -> u32
    {
      match self
      {
        Self::Repeat => gl::REPEAT,
        Self::ClampToEdge => gl::CLAMP_TO_EDGE,
        Self::MirroredRepeat => gl::MIRRORED_REPEAT
      }
    }

    pub fn from_gl( value : u32 ) -> Self
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

  impl MagFilterMode
  {
    pub fn to_gl( &self ) -> u32
    {
      match self
      {
        Self::Linear => gl::LINEAR,
        Self::Nearest => gl::NEAREST
      }
    }

    pub fn from_gl( value : u32 ) -> Self
    {
      match value
      {
        gl::LINEAR => Self::Linear,
        gl::NEAREST => Self::Nearest,
        e => panic!( "Invalid MagFilterMode value: {}", e )
      }
    }
  }

  impl MinFilterMode
  {
    pub fn to_gl( &self ) -> u32
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

    pub fn from_gl( value : u32 ) -> Self
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

  impl CompareFunction
  {
    pub fn to_gl( &self ) -> u32
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

    pub fn from_gl( value : u32 ) -> Self
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
    MagFilterMode,
    MinFilterMode,
    WrappingMode,
    CompareFunction,
    Sampler
  };
}