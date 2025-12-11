mod private
{
  use minwebgl as gl;
  use gl::F32x3;

  /// Defines light type that supported by Renderer
  #[ derive( Debug, PartialOrd, Ord, PartialEq, Eq, Hash ) ]
  pub enum LightType
  {
    /// Point light source type
    Point,
    /// Directional light source type
    Direct
  }

  impl std::fmt::Display for LightType
  {
    fn fmt( &self, f: &mut std::fmt::Formatter< '_ > ) -> std::fmt::Result
    {
      match self
      {
        LightType::Point => write!( f, "Point" ),
        LightType::Direct => write!( f, "Direct" )
      }
    }
  }

  /// General type for supported light types
  #[ derive( Debug, Clone ) ]
  pub enum Light
  {
    /// Point light source
    Point( PointLight ),
    /// Direct light source
    Direct( DirectLight )
  }

  impl From< &Light > for LightType
  {
    fn from( value : &Light ) -> Self
    {
      match value
      {
        Light::Point( _ ) => LightType::Point,
        Light::Direct( _ ) => LightType::Direct
      }
    }
  }

  /// Point light source description
  #[ derive( Debug, Clone ) ]
  pub struct PointLight
  {
    /// Light position
    pub position : F32x3,
    /// Light color
    pub color : F32x3,
    /// Light strength
    pub strength : f32,
    /// Light range
    pub range : f32
  }

  /// Direct light source description
  #[ derive( Debug, Clone ) ]
  pub struct DirectLight
  {
    /// Light direction
    pub direction : F32x3,
    /// Light color
    pub color : F32x3,
    /// Light strength
    pub strength : f32
  }
}

crate::mod_interface!
{
  orphan use
  {
    LightType,
    Light,
    PointLight,
    DirectLight
  };
}
