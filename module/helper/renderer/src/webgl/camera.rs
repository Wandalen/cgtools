mod private
{
  use minwebgl as gl;
  use std::{ cell::RefCell, rc::Rc };
  use rustc_hash::FxHashMap;
  use mingl::
  {
    CameraOrbitControls,
    controls::camera_orbit_controls::controls_bind_to_input,
    geometry::BoundingBox,
  };

  /// A struct representing a 3D camera with orbit controls.
  pub struct Camera
  {
    controls : Rc< RefCell< CameraOrbitControls > >,
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
    /// * `fov` - The vertical field of view in radians.
    /// * `near` - The distance to the near clipping plane.
    /// * `far` - The distance to the far clipping plane.
    ///
    /// # Errors
    ///
    /// Returns `WebglError` if `aspect_ratio`, `fov`, `near`, or `far` is non-finite or outside
    /// the domain `perspective_rh_gl` requires to produce a valid, invertible projection matrix
    /// ( `aspect_ratio > 0`, `0 < fov < PI`, `near > 0`, `far > near` ).
    pub fn new
    (
      eye : gl::F32x3,
      up : gl::F32x3,
      look_at : gl::F32x3,
      aspect_ratio : f32,
      fov : f32,
      near : f32,
      far : f32
    ) -> Result< Self, gl::WebglError >
    {
      // Fix(BUG-174): `perspective_rh_gl` divides by `aspect_ratio`, `tan( fov / 2.0 )`, and
      // `near - far` with no guard -- a zero/negative/non-finite value on any of these 4
      // parameters baked an Inf/NaN-poisoned matrix into `self.projection_matrix` with no error
      // signal, or ( for the narrow `near == 0.0` xor `far == 0.0` case ) produced a matrix whose
      // determinant is exactly zero, deferring the actual panic to an unrelated
      // `.inverse().unwrap()` call several frames later in `Renderer::skybox_draw`.
      // Root cause: no validation existed anywhere between the caller and `perspective_rh_gl`.
      // Pitfall: `aspect_ratio` is routinely computed as `canvas.width() / canvas.height()` ( see
      // this crate's own readme.md example ) -- a transiently zero canvas height ( hidden tab,
      // not yet laid out ) used to silently corrupt every subsequent frame's projection instead
      // of surfacing as a recoverable, attributable error at the point of construction.
      if !aspect_ratio.is_finite() || aspect_ratio <= 0.0
      {
        return Err( gl::WebglError::Other( "Camera::new: aspect_ratio must be finite and > 0.0" ) );
      }
      if !fov.is_finite() || fov <= 0.0 || fov >= std::f32::consts::PI
      {
        return Err( gl::WebglError::Other( "Camera::new: fov must be finite and within ( 0.0, PI ) radians" ) );
      }
      if !near.is_finite() || near <= 0.0
      {
        return Err( gl::WebglError::Other( "Camera::new: near must be finite and > 0.0" ) );
      }
      if !far.is_finite() || far <= near
      {
        return Err( gl::WebglError::Other( "Camera::new: far must be finite and > near" ) );
      }

      let projection_matrix = gl::math::mat3x3h::perspective_rh_gl
      (
        fov,
        aspect_ratio,
        near,
        far
      );

      let mut controls = CameraOrbitControls::default();
      controls.eye = eye;
      controls.up = up;
      controls.center = look_at;
      controls.fov = fov;
      controls.rotation.speed = 200.0;

      let controls = Rc::new( RefCell::new( controls ) );

      Ok
      (
        Self
        {
          controls,
          near,
          far,
          projection_matrix
        }
      )
    }

    /// Builds a camera framing `bounding_box`, viewed from `direction` ( need not be
    /// normalized ) with `up` as the camera's up vector.
    ///
    /// `distance` is derived from the box's bounding sphere ( its half-diagonal, a
    /// conservative over-approximation that always contains the box ) so that the sphere
    /// fits the frustum on both axes, accounting for `fov`/`aspect_ratio` -- unlike a flat
    /// `distance = bounding_box.max.mag()` scale ( a pattern duplicated, with several
    /// mutually-inconsistent tweaks, across a dozen examples in this workspace ), this
    /// scales correctly regardless of where the box sits relative to the world origin, and
    /// regardless of what fov/aspect_ratio the camera uses. `near`/`far` are likewise
    /// derived tightly from the box's own extent rather than a fixed heuristic.
    ///
    /// `near_min` guards the near plane against a degenerate ( near-zero-radius ) box
    /// collapsing it to zero or negative, which `Camera::new` would otherwise reject.
    ///
    /// # Errors
    ///
    /// Returns `WebglError` under the same conditions as `Camera::new` -- `aspect_ratio`,
    /// `fov`, or `near_min` outside their required domains, propagated through unchanged.
    pub fn from_bounding_box
    (
      bounding_box : &BoundingBox,
      direction : gl::F32x3,
      up : gl::F32x3,
      aspect_ratio : f32,
      fov : f32,
      near_min : f32,
    ) -> Result< Self, gl::WebglError >
    {
      let center = bounding_box.center();
      let radius = ( ( bounding_box.max - bounding_box.min ) * 0.5 ).mag();

      let vertical_half = fov * 0.5;
      let horizontal_half = ( aspect_ratio * vertical_half.tan() ).atan();
      let limiting_half = vertical_half.min( horizontal_half );
      let distance = if radius > 0.0 { radius / limiting_half.sin() } else { near_min };

      let eye = center + direction.normalize() * distance;
      let near = ( distance - radius ).max( near_min );
      // `.max( near + near_min )` guards a degenerate ( zero/near-zero-radius ) box, where
      // `distance` collapses to `near_min` and `distance + radius` would otherwise land on or
      // below `near` itself -- `Camera::new` requires `far > near` strictly.
      let far = ( distance + radius ).max( near + near_min );

      Self::new( eye, up, center, aspect_ratio, fov, near, far )
    }

    /// Binds mouse and pointer events to the camera controls for interaction.
    ///
    /// # Arguments
    /// * `canvas` - A reference to the HTML canvas element where the events will be bound.
    pub fn controls_bind
    (
      &self,
      canvas : &web_sys::HtmlCanvasElement
    )
    {
      controls_bind_to_input( canvas, &self.controls );
    }

    /// Uploads the camera's matrices and position to a WebGL2 shader program.
    ///
    /// # Arguments
    /// * `gl` - The WebGL2 rendering context.
    /// * `locations` - A `FxHashMap` containing the uniform locations for the shader program.
    ///
    /// # Panics
    ///
    /// Panics if `locations` misses any of the camera uniforms
    /// ( view/projection matrices, position ) or an upload fails.
    pub fn upload
    (
      &self,
      gl : &gl::WebGl2RenderingContext,
      locations : &FxHashMap< String, Option< gl::WebGlUniformLocation > >
    )
    {
      let view_matrix = self.view_matrix_get().to_array();
      let eye = self.eye_get().to_array();
      let projection_matrix = self.projection_matrix_get();

      if let Some( loc ) = locations.get( "cameraPosition" )
      {
        gl::uniform::upload
        (
          gl,
          loc.clone(),
          &eye[ .. ]
        ).unwrap();
      }

      gl::uniform::matrix_upload
      (
        gl,
        locations.get( "viewMatrix" )
        .expect( "Camera::upload: \"viewMatrix\" missing from the bound shader's impl_locations! list -- see this fn's # Panics doc" )
        .clone(),
        &view_matrix[ .. ],
        true
      ).unwrap();

      gl::uniform::matrix_upload
      (
        gl,
        locations.get( "projectionMatrix" )
        .expect( "Camera::upload: \"projectionMatrix\" missing from the bound shader's impl_locations! list -- see this fn's # Panics doc" )
        .clone(),
        projection_matrix.to_array().as_slice(),
        true
      ).unwrap();
    }

    /// Updates the state of the camera controls
    pub fn update( &mut self, delta_time : f64 )
    {
      self.controls.borrow_mut().update( delta_time );
    }

    /// Sets the window size for the camera controls.
    pub fn window_size_set( &mut self, window_size : gl::F32x2 )
    {
      self.controls.borrow_mut().size_set( window_size.to_array() );
    }

    /// Sets the projection matrix value
    ///
    /// # Errors
    ///
    /// Returns `WebglError` if `projection_matrix` has a non-finite component or is singular
    /// ( not invertible ) -- see BUG-246. Consumers such as `Renderer::skybox_draw` require an
    /// invertible projection matrix.
    // Fix(BUG-246): this setter bypassed `Camera::new`'s ( BUG-174 ) validation entirely --
    // any caller recomputing a projection matrix directly ( e.g. on window resize ) and passing
    // it here fed a possibly Inf/NaN-poisoned or singular matrix straight into `self.projection_matrix`
    // with no check, deferring the same class of panic BUG-174 fixed to an unrelated downstream
    // `.inverse().unwrap()` call.
    // Root cause: `Camera::new` validates its scalar inputs before building the matrix, but
    // `projection_matrix_set` accepts an already-built matrix, so those scalar checks can never run.
    // Pitfall: validating a constructor's inputs does not protect a sibling setter that accepts
    // the constructor's *output* type directly -- the invariant must be enforced at every entry
    // point that can set the field, not just the one a bug happened to be found through.
    pub fn projection_matrix_set( &mut self, projection_matrix : gl::F32x4x4 ) -> Result< (), gl::WebglError >
    {
      if !projection_matrix.to_array().iter().all( | c | c.is_finite() )
      {
        return Err( gl::WebglError::Other( "Camera::projection_matrix_set: projection_matrix must have all-finite components" ) );
      }
      if projection_matrix.inverse().is_none()
      {
        return Err( gl::WebglError::Other( "Camera::projection_matrix_set: projection_matrix must be invertible" ) );
      }

      self.projection_matrix = projection_matrix;
      Ok( () )
    }

    /// Returns a clone of the `Rc` to the camera controls.
    #[ must_use ]
    pub fn controls_get( &self ) -> Rc< RefCell< CameraOrbitControls > >
    {
      self.controls.clone()
    }

    /// Returns the current position of the camera's eye.
    #[ must_use ]
    pub fn eye_get( &self ) -> gl::F32x3
    {
      self.controls.borrow().eye
    }

    /// Returns the camera's view matrix.
    #[ must_use ]
    pub fn view_matrix_get( &self ) -> gl::F32x4x4
    {
      self.controls.borrow().view()
    }

    /// Returns the camera's projection matrix.
    #[ must_use ]
    pub fn projection_matrix_get( &self ) -> gl::F32x4x4
    {
      self.projection_matrix
    }

    /// Returns a `gl::F32x2` containing the near and far clipping plane distances.
    #[ must_use ]
    pub fn near_far_get( &self ) -> gl::F32x2
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
