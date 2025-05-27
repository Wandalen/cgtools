use minwebgl::F32x3;
use rand::Rng;
use std::f32::consts::PI;

#[ derive( Debug, Clone, Copy ) ]
pub struct EllipticalOrbit
{
  pub center : F32x3,
  pub semi_major_axis : f32,    // 'a' - half the length of the major axis
  pub semi_minor_axis : f32,    // 'b' - half the length of the minor axis
  pub inclination : f32,        // rotation around x-axis (radians)
  pub longitude : f32,          // rotation around z-axis (radians)
  pub argument_of_periapsis : f32, // rotation of ellipse in orbital plane (radians)
}

impl EllipticalOrbit
{
  pub fn new
  (
    center : F32x3,
    semi_major_axis : f32,
    semi_minor_axis : f32,
    inclination : f32,
    longitude : f32,
    argument_of_periapsis : f32,
  ) -> Self
  {
    Self
    {
      center,
      semi_major_axis,
      semi_minor_axis,
      inclination,
      longitude,
      argument_of_periapsis,
    }
  }

  /// Create a random elliptical orbit
  pub fn random() -> Self
  {
    let mut rng = rand::rng();

    Self::new
    (
      F32x3::new
      (
        rng.random_range( -5.0..=5.0 ),
        rng.random_range( -5.0..=5.0 ),
        rng.random_range( -5.0..=5.0 ),
      ),
      rng.random_range( 10.0..=100.0 ),
      rng.random_range( 1.0..=20.0 ),
      rng.random_range( 0.0..=PI ),
      rng.random_range( 0.0..=2.0 * PI ),
      rng.random_range( 0.0..=2.0 * PI ),
    )
  }

  /// Calculate position on ellipse at given angle (0 to 2π)
  pub fn position_at_angle( &self, angle : f32 ) -> F32x3
  {
    // Start with basic ellipse in XY plane
    let x = self.semi_major_axis * angle.cos();
    let y = self.semi_minor_axis * angle.sin();
    let z = 0.0;

    // Apply rotations: argument of periapsis, inclination, longitude
    let ( sin_w, cos_w ) = self.argument_of_periapsis.sin_cos();
    let ( sin_i, cos_i ) = self.inclination.sin_cos();
    let ( sin_o, cos_o ) = self.longitude.sin_cos();

    // Rotate by argument of periapsis (in orbital plane)
    let x1 = x * cos_w - y * sin_w;
    let y1 = x * sin_w + y * cos_w;
    let z1 = z;

    // Rotate by inclination (around x-axis)
    let x2 = x1;
    let y2 = y1 * cos_i - z1 * sin_i;
    let z2 = y1 * sin_i + z1 * cos_i;

    // Rotate by longitude of ascending node (around z-axis)
    let x3 = x2 * cos_o - y2 * sin_o;
    let y3 = x2 * sin_o + y2 * cos_o;
    let z3 = z2;

    // Translate to center
    F32x3::new
    (
      x3 + self.center.x(),
      y3 + self.center.y(),
      z3 + self.center.z(),
    )
  }
}
