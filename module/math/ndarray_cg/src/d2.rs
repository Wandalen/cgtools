mod private
{
}

crate::mod_interface!
{

  // Matrix arithmetics.
  layer arithmetics;
  // /// Access to matrix.
  // layer mat_access;

  /// Matrix and all related.
  layer mat;
  orphan use super::mat;


  /// 2D entities with 2 along both dimensions.
  /// Useful for 2D graphics.
  layer mat2x2;
  orphan use super::mat2x2;

  /// 2D entities with 2+homogenous coordinate along both dimensions.
  /// Useful for 2D graphics.
  layer mat2x2h;
  orphan use super::mat2x2h;

  /// 3D entities with 3+homogenous coordinate along both dimensions.
  /// Useful for 3D graphics.
  layer mat3x3h;
  orphan use super::mat3x3h;

  /// Rotation.
  layer rotation;
  // orphan use super::rotation;

  /// Rotation 2d.
  layer rotation2;
  // orphan use super::rotation2;

}
