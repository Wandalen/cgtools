mod private
{
  use minwebgl as gl;
  use gl::F32x3;

  /// General type for supported light types
  #[ derive( Debug, Clone ) ]
  pub enum Light
  {
    /// Point light source
    Point( PointLight ),
    /// Direct light source
    Direct( DirectLight ),
    /// Spot light source
    Spot( SpotLight )
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

  /// Spot light source description
  #[ derive( Debug, Clone ) ]
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
    Light,
    PointLight,
    DirectLight,
    SpotLight
  };
}
