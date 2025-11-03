mod private
{
  use minwebgl as gl;
  use gl::F32x3;

  /// General type for supported light types
  #[ derive( Clone ) ]
  pub enum Light
  {
    /// Point light source
    Point( PointLight ),
    /// Direct light source
    Direct( DirectLight )
  }

  /// Point light source description
  #[ derive( Clone ) ]
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
  #[ derive( Clone ) ]
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
    Light,
    PointLight,
    DirectLight
  };
}
