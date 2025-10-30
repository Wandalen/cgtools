mod private
{
  use minwebgl as gl;
  use gl::F32x3;

  /// General type for supported light types
  pub enum Light
  {
    Point( PointLight ),
    Direct( DirectLight )
  }

  /// Point light source description
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
