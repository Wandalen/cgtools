//! This module provides an implementation of a camera with orbit controls,
//! allowing for easy 3D scene navigation through rotation, panning, and zooming.
//! It is designed to be independent of any specific graphics backend.

/// Internal namespace for implementation details.
mod private
{
  use crate::*;

  /// Provides an orbit-style camera controller for 3D scenes.
  ///
  /// This camera rotates around a central `center` point, can pan across the view plane,
  /// and zoom in and out. It's suitable for inspecting 3D models or scenes.
  pub struct CameraOrbitControls
  {
    /// The position of the camera in 3D space.
    pub eye : F32x3,
    /// The "up" direction for the camera, typically `(0, 1, 0)`.
    pub up : F32x3,
    /// The point in space the camera is looking at and orbiting around.
    pub center : F32x3,
    /// The size of the rendering window or viewport, used for panning calculations.
    pub window_size : F32x2,
    /// A scaling factor to adjust the sensitivity of camera rotation.
    pub rotation_speed_scale : f32,
    /// A scaling factor to adjust the sensitivity of camera zooming.
    pub zoom_speed_scale : f32,
    /// The vertical field of view of the camera, in radians.
    pub fov : f32
  }

  impl CameraOrbitControls
  {
    /// Returns the current position of the camera (`eye`).
    pub fn eye( &self ) -> F32x3
    {
      self.eye
    }

    /// Returns the current "up" vector of the camera.
    pub fn up( &self ) -> F32x3
    {
      self.up
    }

    /// Returns the point the camera is centered on.
    pub fn center( &self ) -> F32x3
    {
      self.center
    }

    /// Calculates and returns a right-handed view matrix based on the camera's current state.
    pub fn view( &self ) -> math::F32x4x4
    {
      math::mat3x3h::look_at_rh( self.eye, self.center, self.up )
    }

    /// Updates the camera's knowledge of the window or viewport size.
    pub fn set_size( &mut self, size : [ f32; 2 ] )
    {
      self.window_size = F32x2::from( size );
    }

    /// Rotates the camera around the `center` point.
    ///
    /// The rotation is based on the displacement of the cursor on the screen,
    /// creating an intuitive orbiting effect.
    ///
    /// # Arguments
    /// * `screen_d` - The change in screen coordinates `[dx, dy]` from a mouse movement event.
    pub fn rotate
    (
      &mut self,
      mut screen_d : [ f32; 2 ]
    )
    {
      screen_d[ 0 ] /= self.rotation_speed_scale;
      screen_d[ 1 ] /= self.rotation_speed_scale;

      let dir = ( self.center - self.eye ).normalize();
      let x = dir.cross( self.up ).normalize();

      // We rotate aroung the y axis based on the movement in x direction.
      // And we rotate aroung the axix perpendicular to the current up and direction vectors
      // based on the movement in y direction
      let rot_y = math::mat3x3::from_angle_y( -screen_d[ 0 ] );
      let rot_x = math::mat3x3::from_axis_angle( x, -screen_d[ 1 ] );
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

    /// Pans the camera by moving both its position and its center point in a plane.
    ///
    /// The plane is perpendicular to the camera's viewing direction.
    ///
    /// # Arguments
    /// * `screen_d` - The change in screen coordinates `[dx, dy]` from a mouse movement event.
    pub fn pan
    (
      &mut self,
      screen_d : [ f32; 2 ]
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
      let k = 2.0 * y_center / self.window_size.y();

      // Scale the movement in screen spcae to the appropriate movement in worldspace
      let mut offset = y * screen_d[ 1 ] - x * screen_d[ 0 ];
      offset *= k;

      let center_new = self.center + offset;
      let eye_new = self.eye + offset;

      self.center = center_new;
      self.eye = eye_new;
    }

    /// Zooms the camera in or out along its viewing direction.
    ///
    /// # Arguments
    /// * `delta_y` - The scroll amount, typically from a mouse wheel event.
    ///   A negative value zooms in, and a positive value zooms out.
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

  impl Default for CameraOrbitControls
  {
    /// Creates a new `CameraOrbitControls` with a set of sensible default values.
    fn default() -> Self
    {
      CameraOrbitControls
      {
        eye : F32x3::from( [ 1.0, 0.0, 0.0 ] ),
        up : F32x3::from( [ 0.0, 1.0, 0.0 ] ),
        center : F32x3::from( [ 0.0, 0.0, 0.0 ] ),
        window_size : F32x2::from( [ 1000.0, 1000.0 ] ),
        rotation_speed_scale : 500.0,
        zoom_speed_scale : 1000.0,
        fov : 70f32.to_radians()
      }
    }
  }
}

// This macro exposes the public interface of the module.
crate::mod_interface!
{
  /// Exposes the `CameraOrbitControls` struct for public use.
  exposed use
  {
    CameraOrbitControls
  };
}
