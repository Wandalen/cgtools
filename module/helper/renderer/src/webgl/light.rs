mod private
{
  use minwebgl as gl;
  use gl::F32x3;

  /// Defines light type that supported by Renderer
  #[ derive( Debug, PartialOrd, Ord, PartialEq, Eq, Hash, Clone, Copy ) ]
  pub enum LightType
  {
    /// Point light source type
    Point,
    /// Directional light source type
    Direct,
    /// Spot light source type
    Spot
  }

  impl std::fmt::Display for LightType
  {
    fn fmt( &self, f: &mut std::fmt::Formatter< '_ > ) -> std::fmt::Result
    {
      match self
      {
        LightType::Point => write!( f, "Point" ),
        LightType::Direct => write!( f, "Direct" ),
        LightType::Spot => write!( f, "Spot" )
      }
    }
  }

  /// General type for supported light types
  #[ derive( Debug, Clone, Copy ) ]
  pub enum Light
  {
    /// Point light source
    Point( PointLight ),
    /// Direct light source
    Direct( DirectLight ),
    /// Spot light source
    Spot( SpotLight )
  }

  impl From< &Light > for LightType
  {
    fn from( value : &Light ) -> Self
    {
      match value
      {
        Light::Point( _ ) => LightType::Point,
        Light::Direct( _ ) => LightType::Direct,
        Light::Spot( _ ) => LightType::Spot
      }
    }
  }

  /// Point light source description
  #[ derive( Debug, Clone, Copy ) ]
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
  #[ derive( Debug, Clone, Copy ) ]
  pub struct DirectLight
  {
    /// Light direction
    pub direction : F32x3,
    /// Light color
    pub color : F32x3,
    /// Light strength
    pub strength : f32
  }

  /// Spot light source description
  #[ derive( Debug, Clone, Copy ) ]
  pub struct SpotLight
  {
    /// Light position
    pub position : F32x3,
    /// Light direction (unit vector)
    pub direction : F32x3,
    /// Light color
    pub color : F32x3,
    /// Light strength
    pub strength : f32,
    /// Light range
    pub range : f32,
    /// Inner cone angle in radians (full brightness)
    pub inner_cone_angle : f32,
    /// Outer cone angle in radians (defines edge of light cone)
    pub outer_cone_angle : f32,
    /// Whether to use lightmap for this light
    pub use_light_map : bool,
  }
}

crate::mod_interface!
{
  orphan use
  {
    LightType,
    Light,
    PointLight,
    DirectLight,
    SpotLight
  };
}
