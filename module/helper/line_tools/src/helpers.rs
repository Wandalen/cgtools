mod private
{
  pub const BODY_GEOMETRY : [ [ f32; 2 ]; 6 ] =
  [
    [ 0.0, -0.5 ],
    [ 1.0, -0.5 ],
    [ 1.0,  0.5 ],
    [ 0.0, -0.5 ],
    [ 1.0,  0.5 ],
    [ 0.0,  0.5 ]
  ];

  pub fn round_geometry( segments : usize ) -> Vec< [ f32; 2 ] >
  {
    let mut positions = Vec::with_capacity( segments );
    for wedge in 0..=segments
    {
      let theta = 2.0 * std::f32::consts::PI * wedge as f32 / segments as f32;
      let ( s, c ) = theta.sin_cos();
      positions.push( [ 0.5 * c, 0.5 * s ] )
    }

    positions
  }
}

crate::mod_interface!
{
  own use
  {
    round_geometry,
    BODY_GEOMETRY
  };
}