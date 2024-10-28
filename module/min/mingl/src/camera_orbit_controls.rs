mod private
{
  use crate::*;
  use cgmath::InnerSpace;

  /// Provides camera controls independent of the API backend
  pub struct CameraOrbitControls
  {
    /// Position of the camera
    pub eye : [ f32; 3 ],
    /// Orientation of camera
    pub up : [ f32; 3 ],
    /// Look at point, which is also the center of the sphere of rotation
    pub center : [ f32; 3 ],
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
      self.eye
    }

    pub fn up( &self ) -> [ f32; 3 ]
    {
      self.up
    }

    pub fn center( &self ) -> [ f32 ; 3 ]
    {
      self.center
    }

    /// Return a righthanded view matrix of the current camera state
    pub fn view( &self ) -> [ f32 ; 16 ]
    {
      *cgmath::Matrix4::look_at_rh( self.eye.into(), self.center.into(), self.up.into() ).as_ref()
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

      //Convert to cgmath Vectors
      let center = cgmath::Vector3::from( self.center );
      let mut up_prev = cgmath::Vector3::from( self.up );
      let mut eye_prev = cgmath::Vector3::from( self.eye );

      let dir = ( center - eye_prev ).normalize();
      let x = dir.cross( up_prev ).normalize();

      // We rotate aroung the y axis based on the movement in x direction.
      // And we rotate aroung the axix perpendicular to the current up and direction vectors 
      // based on the movement in y direction
      let rot_y = cgmath::Matrix3::from_angle_y( cgmath::Rad( -screen_d[ 0 ] ) );
      let rot_x = cgmath::Matrix3::from_axis_angle( x, cgmath::Rad( -screen_d[ 1 ] ) );
      // Combine two rotations
      let rot = rot_y * rot_x;

      // We need the center to be at the origin before we can apply rotation
      eye_prev -= center;
      eye_prev = rot * eye_prev;
      eye_prev += center;

      up_prev = rot * up_prev;

      self.eye = eye_prev.into();  
      self.up = up_prev.normalize().into();

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
      let up = cgmath::Vector3::from( self.up );
      let mut center_prev = cgmath::Vector3::from( self.center );
      let mut eye_prev = cgmath::Vector3::from( self.eye );

      // Here we get the x and y direction vectors based on camera's orientation and direction.
      // Both vectors line in the plane that the dir vector is perpendicular to.
      let dir = center_prev - eye_prev;
      let dir_norm = dir.normalize();
      let x = dir_norm.cross( up ).normalize();
      let y = x.cross( dir_norm ).normalize();

      // Find the vertical distance to the edge of frustum from center
      let y_center =  ( self.fov / 2.0 ).tan() * dir.magnitude();
      // Find the ration between half of screen height and the frustum height
      let k = 2.0 * y_center / self.window_size[ 1 ];

      // Scale the movement in screen spcae to the appropriate movement in worldspace
      let mut offset = y * screen_d[ 1 ] - x * screen_d[ 0 ];
      offset *= k;

      center_prev += offset;
      eye_prev += offset;

      self.center = center_prev.into();
      self.eye = eye_prev.into();
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
      let center = cgmath::Vector3::from( self.center );
      let mut eye_prev = cgmath::Vector3::from( self.eye );

      // If scroll is up (-) then zoom in
      // If scroll is down (+) then zoom out
      let k = if delta_y < 0.0 { 1.0 + delta_y.abs() } else { 1.0 - delta_y.abs() };

      // We need the center to be at the origin before we can apply zoom
      eye_prev -= center;
      eye_prev /= k;
      eye_prev += center;

      self.eye = eye_prev.into();
    }
  }

  impl Default for CameraOrbitControls {
      fn default() -> Self {
          CameraOrbitControls
          {
            eye : [ 1.0, 0.0, 0.0 ],
            up : [ 0.0, 1.0, 0.0 ],
            center : [ 0.0, 0.0, 0.0 ],
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