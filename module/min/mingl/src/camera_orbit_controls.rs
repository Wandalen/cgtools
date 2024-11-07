mod private
{
  use crate::*;
  use ndarray_cg::vector::*;
  use ndarray_cg::d2;

  /// Provides camera controls independent of the API backend
  pub struct CameraOrbitControls
  {
    /// Position of the camera
    pub eye : Vec3< f32 >,
    /// Orientation of camera
    pub up : Vec3< f32 >,
    /// Look at point, which is also the center of the sphere of rotation
    pub center : Vec3< f32 >,
    /// Size of the drawing window
    pub window_size : [ f32; 2 ],
    /// Scales the speed of rotation
    pub rotation_speed_scale : f32,
    /// Scales the speed of zoom
    pub zoom_speed_scale : f32,
    /// Field of view of the camera
    pub fov : f32
  }

  impl CameraOrbitControls 
  {
    pub fn eye( &self ) -> [ f32; 3 ]
    {
      self.eye.to_array()
    }

    pub fn up( &self ) -> [ f32; 3 ]
    {
      self.up.to_array()
    }

    pub fn center( &self ) -> [ f32 ; 3 ]
    {
      self.center.to_array()
    }

    /// Return a righthanded view matrix of the current camera state
    pub fn view( &self ) -> [ f32; 16 ]
    {
      ndarray_cg::mat3x3h::loot_at_rh( self.eye, self.center, self.up ).to_array()
    }

    pub fn set_size( &mut self, size : [ f32; 2 ] )
    {
      self.window_size = size;
    }

    /// Makes rotation around the sphere with center at self.center and radius equal to length of ( self.center - self.eye ).
    /// As input takes the amount of pixels cursor moved on the screen.
    /// You can get this value from the corresponding MouseMove event
    pub fn rotate
    ( 
      &mut self, 
      mut screen_d :  [ f32; 2 ]
    )
    {
      screen_d[ 0 ] /= self.rotation_speed_scale;
      screen_d[ 1 ] /= self.rotation_speed_scale;

      let dir = ( self.center - self.eye ).normalize();
      let x = dir.cross( self.up ).normalize();

      // We rotate aroung the y axis based on the movement in x direction.
      // And we rotate aroung the axix perpendicular to the current up and direction vectors 
      // based on the movement in y direction
      let rot_y = d2::mat3x3::from_angle_y( -screen_d[ 0 ] );
      let rot_x = d2::mat3x3::from_axis_angle( x, -screen_d[ 1 ] );
      // Combine two rotations
      let rot = rot_y * rot_x;

      // We need the center to be at the origin before we can apply rotation
      let mut eye_new = self.eye - self.center;
      eye_new *= rot;
      eye_new += self.center;

      let up_new = rot * self.up;

      self.eye = eye_new;  
      self.up = up_new;

    }

    /// Moves camera around in the plane that the direction vector of the camera is perpendicular to.
    /// As input takes the amount of pixels cursor moved on the screen.
    /// You can get this value from the corresponding MouseMove event
    pub fn pan
    ( 
      &mut self, 
      screen_d :  [ f32; 2 ] 
    )
    {
      // Convert to cgmath Vectors
      // let up = cgmath::Vector3::from( self.up );
      // let mut center_prev = cgmath::Vector3::from( self.center );
      // let mut eye_prev = cgmath::Vector3::from( self.eye );

      // Here we get the x and y direction vectors based on camera's orientation and direction.
      // Both vectors line in the plane that the dir vector is perpendicular to.
      let dir = self.center - self.eye;
      let dir_norm = dir.normalize();
      let x = dir_norm.cross( self.up ).normalize();
      let y = x.cross( dir_norm ).normalize();

      // Find the vertical distance to the edge of frustum from center
      let y_center =  ( self.fov / 2.0 ).tan() * dir.mag();
      // Find the ration between half of screen height and the frustum height
      let k = 2.0 * y_center / self.window_size[ 1 ];

      // Scale the movement in screen spcae to the appropriate movement in worldspace
      let mut offset = y * screen_d[ 1 ] - x * screen_d[ 0 ];
      offset *= k;

      let center_new = self.center + offset;
      let eye_new = self.eye + offset;

      self.center = center_new;
      self.eye = eye_new;
    }

    /// Zooms in/out camera in the view direction
    /// As input takes the scroll amount, that you usually can take from the ScrollEvent.
    pub fn zoom
    ( 
      &mut self, 
      mut delta_y : f32 
    )
    {
      delta_y /= self.zoom_speed_scale;

      //Convert to cgmath Vectors
      // let center = cgmath::Vector3::from( self.center );
      // let mut eye_prev = cgmath::Vector3::from( self.eye );

      // If scroll is up (-) then zoom in
      // If scroll is down (+) then zoom out
      let k = if delta_y < 0.0 { 1.0 + delta_y.abs() } else { 1.0 - delta_y.abs() };

      // We need the center to be at the origin before we can apply zoom
      let mut eye_new = self.eye - self.center;
      eye_new /= k;
      eye_new += self.center;

      self.eye = eye_new;
    }
  }

  impl Default for CameraOrbitControls {
      fn default() -> Self {
          CameraOrbitControls
          {
            eye : Vec3::from( [ 1.0, 0.0, 0.0 ] ),
            up : Vec3::from( [ 0.0, 1.0, 0.0 ] ),
            center : Vec3::from( [ 0.0, 0.0, 0.0 ] ),
            window_size : [ 1000.0, 1000.0 ],
            rotation_speed_scale : 500.0,
            zoom_speed_scale : 1000.0,
            fov : 70f32.to_radians()
          }
      }
  }
}

crate::mod_interface!
{
  exposed use 
  {
    CameraOrbitControls
  };
}