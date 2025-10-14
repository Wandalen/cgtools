mod private
{
  use minwebgl as gl;
  use std::{ cell::RefCell, collections::HashMap, rc::Rc };
  use mingl::
  {
    CameraOrbitControls,
    controls::camera_orbit_controls::bind_controls_to_input
  };

  /// A struct representing a 3D camera with orbit controls.
  #[ allow( dead_code ) ]
  pub struct Camera
  {
    controls : Rc< RefCell< CameraOrbitControls > >,
    aspect_ratio : f32,
    fov : f32,
    near : f32,
    far : f32,
    projection_matrix : gl::F32x4x4,
  }

  impl Camera
  {
    /// Creates a new `Camera` instance.
    ///
    /// # Arguments
    /// * `eye` - The position of the camera in 3D space.
    /// * `up` - The up direction of the camera.
    /// * `look_at` - The point in 3D space the camera is looking at.
    /// * `aspect_ratio` - The ratio of the viewport's width to its height.
    /// * `fov` - The field of view in degrees.
    /// * `near` - The distance to the near clipping plane.
    /// * `far` - The distance to the far clipping plane.
    pub fn new
    (
      eye : gl::F32x3,
      up : gl::F32x3,
      look_at : gl::F32x3,
      aspect_ratio : f32,
      fov : f32,
      near : f32,
      far : f32
    ) -> Self
    {
      let projection_matrix = gl::math::mat3x3h::perspective_rh_gl
      (
        fov,
        aspect_ratio,
        near,
        far
      );

      let controls = CameraOrbitControls
      {
        eye : eye,
        up : up,
        center : look_at,
        fov,
        rotation_speed_scale : 200.0,
        ..Default::default()
      };

      let controls = Rc::new( RefCell::new( controls ) );

      Self
      {
        controls,
        near,
        far,
        aspect_ratio,
        fov,
        projection_matrix
      }
    }

    /// Binds mouse and pointer events to the camera controls for interaction.
    ///
    /// # Arguments
    /// * `canvas` - A reference to the HTML canvas element where the events will be bound.
    pub fn bind_controls
    (
      &self,
      canvas : &web_sys::HtmlCanvasElement
    )
    {
      bind_controls_to_input( canvas, &self.controls );
    }

    /// Uploads the camera's matrices and position to a WebGL2 shader program.
    ///
    /// # Arguments
    /// * `gl` - The WebGL2 rendering context.
    /// * `locations` - A `HashMap` containing the uniform locations for the shader program.
    pub fn upload
    (
      &self,
      gl : &gl::WebGl2RenderingContext,
      locations : &HashMap< String, Option< gl::WebGlUniformLocation > >
    )
    {
      let view_matrix = self.get_view_matrix().to_array();
      let eye = self.get_eye().to_array();
      let projection_matrix = self.get_projection_matrix();

      if let Some( loc ) = locations.get( "cameraPosition" )
      {
        gl::uniform::upload
        (
          &gl,
          loc.clone(),
          &eye[ .. ]
        ).unwrap();
      }

      gl::uniform::matrix_upload
      (
        &gl,
        locations.get( "viewMatrix" ).unwrap().clone(),
        &view_matrix[ .. ],
        true
      ).unwrap();

      gl::uniform::matrix_upload
      (
        &gl,
        locations.get( "projectionMatrix" ).unwrap().clone(),
        projection_matrix.to_array().as_slice(),
        true
      ).unwrap();
    }

    /// Sets the window size for the camera controls.
    pub fn set_window_size( &mut self, window_size : gl::F32x2 )
    {
      self.controls.borrow_mut().set_size( window_size.to_array() );
    }

    /// Sets the projection matrix value
    pub fn set_projection_matrix( &mut self, projection_matrix : gl::F32x4x4 )
    {
      self.projection_matrix = projection_matrix;
    }

    /// Returns a clone of the `Rc` to the camera controls.
    pub fn get_controls( &self ) -> Rc< RefCell< CameraOrbitControls > >
    {
      self.controls.clone()
    }

    /// Returns the current position of the camera's eye.
    pub fn get_eye( &self ) -> gl::F32x3
    {
      self.controls.borrow().eye
    }

    /// Returns the camera's view matrix.
    pub fn get_view_matrix( &self ) -> gl::F32x4x4
    {
      self.controls.borrow().view()
    }

    /// Returns the camera's projection matrix.
    pub fn get_projection_matrix( &self ) -> gl::F32x4x4
    {
      self.projection_matrix
    }

    /// Returns a `gl::F32x2` containing the near and far clipping plane distances.
    pub fn get_near_far( &self ) -> gl::F32x2
    {
      gl::F32x2::new( self.near, self.far )
    }
  }
}

crate::mod_interface!
{
  orphan use
  {
    Camera
  };
}
