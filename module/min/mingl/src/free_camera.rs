mod private
{
  pub struct FreeCamera
  {
    pub position : [ f32; 3 ],
    pub rotation : [ f32; 3 ],
  }

  impl FreeCamera
  {
    pub fn new() -> Self
    {
      Self
      {
        position: [ 0.0; 3 ],
        rotation: [ 0.0; 3 ],
      }
    }

    pub fn view( &self ) -> [ f32; 16 ]
    {
      let rotation = glam::Quat::from_euler
      (
        glam::EulerRot::XYZ,
        self.rotation[ 0 ],
        self.rotation[ 1 ],
        self.rotation[ 2 ],
      ).inverse();
      let position = -glam::Vec3::from_array( self.position );

      let view : glam::Mat4 = ( glam::Affine3A::from_quat( rotation ) * glam::Affine3A::from_translation( position ) ).into();
      view.to_cols_array()
    }

    pub fn move_local( &mut self, delta : &[ f32; 3 ] )
    {
      let local_axes = self.get_local_axes();
      let local_x = glam::Vec3::from_array( local_axes[ 0 ] );
      let local_y = glam::Vec3::from_array( local_axes[ 1 ] );
      let local_z = glam::Vec3::from_array( local_axes[ 2 ] );
      let pos = glam::Vec3::from_array( self.position );
      let delta = delta[ 0 ] * local_x + delta[ 1 ] * local_y + delta[ 2 ] * local_z;
      self.position = ( pos + delta ).to_array();
    }

    pub fn rotate_local( &mut self, euler : &[ f32; 3 ] )
    {
      let local_axes = self.get_local_axes();

      let local_x = glam::Vec3::from_array( local_axes[ 0 ] );
      let local_y = glam::Vec3::from_array( local_axes[ 1 ] );
      let local_z = glam::Vec3::from_array( local_axes[ 2 ] );
      let x_rotation = glam::Quat::from_axis_angle( local_x, euler[ 0 ] );
      let y_rotation = glam::Quat::from_axis_angle( local_y, euler[ 1 ] );
      let z_rotation = glam::Quat::from_axis_angle( local_z, euler[ 2 ] );

      let rotation = glam::Quat::from_euler
      (
        glam::EulerRot::XYZ,
        self.rotation[ 0 ],
        self.rotation[ 1 ],
        self.rotation[ 2 ],
      );
      let rotation = ( z_rotation * y_rotation * x_rotation * rotation ).normalize().to_euler( glam::EulerRot::XYZ );
      self.rotation = [ rotation.0, rotation.1, rotation.2 ];
    }

    pub fn get_local_axes( &self ) -> [ [ f32; 3 ]; 3 ]
    {
      let rotation = glam::Quat::from_euler
      (
        glam::EulerRot::XYZ,
        self.rotation[ 0 ],
        self.rotation[ 1 ],
        self.rotation[ 2 ],
      );
      let local_x = rotation.mul_vec3( glam::Vec3::X );
      let local_y = rotation.mul_vec3( glam::Vec3::Y );
      let local_z = rotation.mul_vec3( glam::Vec3::Z );
      [ local_x.to_array(), local_y.to_array(), local_z.to_array() ]
    }

    pub fn rotate( &mut self, euler : &[ f32; 3 ] )
    {
      let current_rotation = glam::Quat::from_euler
      (
        glam::EulerRot::XYZ,
        self.rotation[ 0 ],
        self.rotation[ 1 ],
        self.rotation[ 2 ],
      );
      let rotation = glam::Quat::from_euler( glam::EulerRot::XYZ, euler[ 0 ], euler[ 1 ], euler[ 2 ] );
      let rotation = ( rotation * current_rotation ).normalize().to_euler( glam::EulerRot::XYZ );
      self.rotation = [ rotation.0, rotation.1, rotation.2 ];
    }
  }
}

crate::mod_interface!
{
  exposed use
  {
    FreeCamera
  };
}
