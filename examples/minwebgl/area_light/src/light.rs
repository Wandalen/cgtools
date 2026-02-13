use minwebgl as gl;

pub struct RectangularLight
{
  pub vertices : [ gl::F32x3; 4 ],
  pub intensity : f32,
  pub color : [ f32; 3 ],
  pub two_sided : bool,
}

impl RectangularLight
{
  pub fn apply_transform( &mut self, t : &gl::F32x4x4 )
  {
    self.vertices =
    [
      [ -1.0,  1.0, 0.0 ].into(), // 0 TL
      [ -1.0, -1.0, 0.0 ].into(), // 1 BL
      [  1.0,  1.0, 0.0 ].into(), // 2 TR
      [  1.0, -1.0, 0.0 ].into(), // 3 BR
    ];

    self.vertices.iter_mut().for_each
    (
      | v |
      {
        let v4 = gl::F32x4::new( v.x(), v.y(), v.z(), 1.0 );
        let v4 = *t * v4;
        *v = gl::F32x3::new( v4.x(), v4.y(), v4.z() );
      }
    );
  }

  pub fn vertices( &self ) -> [ [ f32; 3 ]; 4 ]
  {
    let mut ret = [ [ 0.0; 3 ]; 4 ];
    ret.iter_mut().zip( self.vertices ).for_each( | ( dest, source ) | *dest = source.to_array() );
    ret
  }
}

#[ derive( Debug, serde::Serialize, serde::Deserialize ) ]
pub struct GuiParams
{
  pub rot_x : f32,
  pub rot_y : f32,
  pub rot_z : f32,
  pub scale_x : f32,
  pub scale_y : f32,
  pub color : [ f32; 3 ],
  pub intensity : f32,
  pub two_sided : bool,
}
