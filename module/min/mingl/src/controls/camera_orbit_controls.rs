//! This module provides an implementation of a camera with orbit controls,
//! allowing for easy 3D scene navigation through rotation, panning, and zooming.
//! It is designed to be independent of any specific graphics backend.

/// Internal namespace for implementation details.
mod private
{
  use crate::{ F32x3, F32x2, math };

  #[ cfg( feature = "web" ) ]
  pub mod web_imports
  {
    pub use std::{ cell::RefCell, rc::Rc, collections::HashMap };
    pub use wasm_bindgen::{ JsCast, prelude::Closure };
    pub use crate::web::web_sys;
  }

  #[ cfg( feature = "web" ) ]
  use web_imports::{ Closure, HashMap, JsCast, RefCell, Rc, web_sys };

  /// State of the camera that controls its rotation
  pub struct CameraRotationState
  {
    /// Enables or disables rotation
    pub enabled : bool,
    /// Sets whether `movement_decay` is applied or not
    pub movement_smoothing_enabled : bool,
    /// Scaling factor for rotation sensitivity in pixels per radian. Higher values make rotation slower.
    /// Default: 500.0 means 500 pixels of drag equals 1 radian of rotation.
    ///
    /// The speed field must be non-zero. Setting speed to 0.0 will result in
    /// undefined camera behavior (NaN/Inf propagation).
    pub speed : f32,
    /// Determines how fast rotation is going to decrease after dragging is stopped.
    /// In range from 0.0 to 1.0
    movement_decay : f32,
    /// The base longitude angle in degrees in range [0, 360], from which bound are calculated. Has no effect when `longitude_range` is `None`.
    /// 0 degrees points in +X diraction and everything else is specified in counter-clockwise rotation around the Y axis:
    /// 90 = -Z
    /// 180 = -X
    /// 270 = +Z
    base_longitude : f32,
    /// Specifies the radius in degrees around the base_longitude. Should be in range [0, 180]
    longitude_range : Option< f32 >,
    /// The base latitude angle in degrees in range [-90, 90], from which the bounds are calculated. Has no effect when `latitude_range` is `None`.
    base_latitude : f32,
    /// Specifies the radius in degrees around the base_latitude. Should be in range [0, 180]. The rotation will be clamped at poles
    latitude_range : Option< f32 >,
    /// Accumulated speed based on mouse movement
    current_angular_speed : F32x2,
    /// Current angle of rotation for the camera
    current_rotation_angle : F32x2
  }

  impl CameraRotationState
  {
    /// Sets the movement_decay and clamps the value in range [0.0, 1.0]
    pub fn movement_decay_set( &mut self, v : f32 )
    {
      self.movement_decay = v.clamp( 0.0, 1.0 );
    }

    /// Gets the movement_decay
    pub fn movement_decay_get( &self ) -> f32
    {
      self.movement_decay
    }

    /// Sets the base longitude. Clamps the value in range [0, 360] degrees
    pub fn base_longitude_set( &mut self, angle : f32 )
    {
      self.base_longitude = angle.clamp( 0.0, 360.0 );
    }

    /// Return the base longitude
    pub fn base_longitude_get( &self ) -> f32
    {
      self.base_longitude
    }

    /// Sets the longitude range. Clamps the value in range [0, 180] degrees
    pub fn longitude_range_set( &mut self, angle : f32 )
    {
      self.longitude_range = Some( angle.clamp( 0.0, 180.0 ) );
    }

    /// Return the longitude range
    pub fn longitude_range_get( &self ) -> Option< f32 >
    {
      self.longitude_range
    }

    /// Sets the base latitude. Clamps the value in range [-90, 90] degrees
    pub fn base_latitude_set( &mut self, angle : f32 )
    {
      self.base_latitude = angle.clamp( -90.0, 90.0 );
    }

    /// Return the base latitude
    pub fn base_latitude_get( &self,) -> f32
    {
      self.base_latitude
    }

    /// Sets the latitude range. Clamps the value in range [0, 180] degrees
    pub fn latitude_range_set( &mut self, angle : f32 )
    {
      self.latitude_range = Some( angle.clamp( 0.0, 180.0 ) );
    }

    /// Return the latitude range
    pub fn latitude_range_get( &self ) -> Option< f32 >
    {
      self.latitude_range
    }
  }

  /// State of the camera that controls its zoom
  pub struct CameraZoomState
  {
    /// Enables or disables zoom
    pub enabled : bool,
    /// A scaling factor to adjust the sensitivity of camera zooming.
    /// The speed field must be non-zero. Setting speed to 0.0 will result in
    /// undefined camera behavior (NaN/Inf propagation).
    pub speed : f32,
    /// The minimum distance from the camera view center
    min_distance : Option< f32 >,
    /// The maximum distance from the camera view center
    max_distance : Option< f32>
  }

  impl CameraZoomState
  {
    /// Sets the minimum zoom distance from the camera center
    /// If d < 0.0 - clamp to 0.0
    /// If d > max_distance - clamp to max_distance
    pub fn min_distance_set( &mut self, mut d : f32 )
    {
      d = d.max( 0.0 );
      if let Some( max_distance ) = self.max_distance
      {
        d = d.min( max_distance );
      }
      self.min_distance = Some( d );
    }

    /// Sets the maximum zoom distance from the camera center
    /// If d < 0.0 - clamp to 0.0
    /// If d < min_distance - clamp to min_distance
    pub fn max_distance_set( &mut self, mut d : f32 )
    {
      d = d.max( 0.0 );
      if let Some( min_distance ) = self.min_distance
      {
        d = d.max( min_distance );
      }
      self.max_distance = Some( d );
    }

    /// Get minimum zoom distance
    pub fn min_distance_get( &self ) -> Option< f32 >
    {
      self.min_distance
    }

    /// Get maximum zoom distance
    pub fn max_distance_get( &self ) -> Option< f32 >
    {
      self.max_distance
    }
  }

  /// State of the camera that controls panning
  pub struct CameraPanState
  {
    /// Enables or disables panning
    pub enabled : bool
  }

  /// Provides an orbit-style camera controller for 3D scenes.
  ///
  /// This camera rotates around a central `center` point, can pan across the view plane,
  /// and zoom in and out. It's suitable for inspecting 3D models or scenes.
  ///
  /// # Example: Constrain camera to hemisphere view
  /// ```rust,ignore
  /// camera.rotation.base_longitude_set( 0.0 );
  /// camera.rotation.longitude_range_set( 90.0 )
  /// camera.zoom.min_distance_set(2.0);
  /// camera.zoom.max_distance_set(10.0);
  /// ```
  // Struct-literal construction with `..Default::default()` is the established public
  // contract, exercised extensively by `tests/tests/camera_orbit_controls.rs` (23+ cases)
  // and by downstream examples — `#[non_exhaustive]` would break that contract.
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
    /// The vertical field of view of the camera, in radians.
    pub fov : f32,
    /// Properties to control camera's rotation
    pub rotation : CameraRotationState,
    /// Properties to control camera's zoom
    pub zoom : CameraZoomState,
    /// Properties that track camera's enabled functionality
    pub pan : CameraPanState
  }

  impl CameraOrbitControls
  {
    /// Returns the current position of the camera (`eye`).
    #[ inline ]
    #[ must_use ]
    pub fn eye( &self ) -> F32x3
    {
      self.eye
    }

    /// Returns the current "up" vector of the camera.
    #[ inline ]
    #[ must_use ]
    pub fn up( &self ) -> F32x3
    {
      self.up
    }

    /// Returns the point the camera is centered on.
    #[ inline ]
    #[ must_use ]
    pub fn center( &self ) -> F32x3
    {
      self.center
    }

    /// Calculates and returns a right-handed view matrix based on the camera's current state.
    #[ inline ]
    #[ must_use ]
    pub fn view( &self ) -> math::F32x4x4
    {
      math::mat3x3h::look_at_rh( self.eye, self.center, self.up )
    }

    /// Updates the camera's knowledge of the window or viewport size.
    #[ inline ]
    pub fn size_set( &mut self, size : [ f32; 2 ] )
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
    ///
    /// # Preconditions
    /// - `eye` must not equal `center` (direction vector must be non-zero)
    /// - `up` must not be parallel to the view direction
    ///
    /// Violating these conditions results in undefined behavior (NaN or panic).
    #[ inline ]
    pub fn rotate
    (
      &mut self,
      screen_d : [ f32; 2 ]
    )
    {
      if !self.rotation.enabled
      {
        return;
      }

      let mut screen_d = F32x2::from( screen_d );
      screen_d /= self.rotation.speed;

      if self.rotation.movement_smoothing_enabled
      {
        self.rotation.current_angular_speed += screen_d;
      }
      else
      {
        self.rotation.current_rotation_angle = screen_d;
        self.rotation_apply();
      }
    }

    fn rotation_apply( &mut self )
    {
      let dir = ( self.eye - self.center ).normalize();
      let x = dir.cross( self.up ).normalize();

      // We rotate around the y axis based on the movement in x direction.
      // And we rotate around the axis perpendicular to the current up and direction vectors
      // based on the movement in y direction
      let mut longitude_angle = self.rotation.current_rotation_angle.x();
      let mut latitude_angle = self.rotation.current_rotation_angle.y();

      if let Some( longitude_range ) = self.rotation.longitude_range
      {
        let angle_range = longitude_range.to_radians();
        // Pivoting angle around which constraints are enforced
        let mut base_angle = self.rotation.base_longitude.to_radians();
        if base_angle > std::f32::consts::PI
        {
          base_angle -= 2.0 * std::f32::consts::PI;
        }
        let min_angle = base_angle - angle_range;
        let max_angle = base_angle + angle_range;

        let current_angle = ( -dir.z() ).atan2( dir.x() );
        let mut new_angle = current_angle + longitude_angle;

        if new_angle < min_angle || new_angle > max_angle
        {
          let delta_min_correction = min_angle - new_angle;
          let delta_max_correction = new_angle - max_angle;

          if delta_max_correction > delta_min_correction
          {
            new_angle -= delta_max_correction;
          }
          else
          {
            new_angle += delta_min_correction;
          }
        }

        longitude_angle = new_angle - current_angle;
      }

      if let Some( latitude_range ) = self.rotation.latitude_range
      {
        let angle_range = latitude_range.to_radians();
        let base_angle = self.rotation.base_latitude.to_radians();
        let min_angle = ( base_angle - angle_range ).max( -std::f32::consts::FRAC_PI_2 );
        let max_angle = ( base_angle + angle_range ).min( std::f32::consts::FRAC_PI_2 );

        let current_angle = dir.y().asin();
        let mut new_angle = current_angle + latitude_angle;

        if new_angle < min_angle || new_angle > max_angle
        {
          let delta_min_correction = min_angle - new_angle;
          let delta_max_correction = new_angle - max_angle;

          if delta_max_correction > delta_min_correction
          {
            new_angle -= delta_max_correction;
          }
          else
          {
            new_angle += delta_min_correction;
          }
        }

        latitude_angle = new_angle - current_angle;
      }


      let rot_x = math::mat3x3::from_axis_angle( x, latitude_angle );
      let rot_y = math::mat3x3::from_angle_y( longitude_angle );
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
    ///   Positive dx corresponds to rightward screen movement; positive dy to downward
    ///   screen movement (`new_pos - prev_pos` convention).
    #[ inline ]
    pub fn pan
    (
      &mut self,
      screen_d : [ f32; 2 ]
    )
    {
      if !self.pan.enabled
      {
        return;
      }

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
    #[ inline ]
    pub fn zoom
    (
      &mut self,
      mut delta_y : f32
    )
    {
      if !self.zoom.enabled
      {
        return;
      }

      delta_y /= self.zoom.speed;

      // If scroll is up (-) then zoom in
      // If scroll is down (+) then zoom out
      // Fix(BUG-126): clamp the zoom-out branch's divisor to a positive floor
      // Root cause: `1.0 - delta_y.abs()` reaches 0.0 (division by zero) or goes negative
      // (sign flip) whenever a single event's `|delta_y|` reaches/exceeds `zoom.speed`; a fast
      // pinch gesture's raw screen-pixel distance delta, or a high-precision wheel event, both
      // reach that magnitude in practice
      // Pitfall: a divisor derived as `1.0 - x.abs()` is only safe while `x` is known to stay
      // inside the unit interval — an external, unbounded input can never be assumed to satisfy
      // that on its own
      let k = if delta_y < 0.0 { 1.0 + delta_y.abs() } else { ( 1.0 - delta_y.abs() ).max( f32::EPSILON ) };

      // We need the center to be at the origin before we can apply zoom
      let mut eye_new = self.eye - self.center;
      eye_new /= k;

      let length = eye_new.mag();

      if let Some( min_distance ) = self.zoom.min_distance
      {
        if length < min_distance
        {
          eye_new = eye_new.normalize() * min_distance;
        }
      }

      if let Some( max_distance ) = self.zoom.max_distance
      {
        if length > max_distance
        {
          eye_new = eye_new.normalize() * max_distance;
        }
      }

      eye_new += self.center;

      self.eye = eye_new;
    }

    /// Updates the state of the controls
    #[ inline ]
    pub fn update
    (
      &mut self,
      delta_time : f64
    )
    {
      // `delta_time` is a per-frame delta in seconds — always minuscule relative to
      // f32's precision limits, so narrowing it once here cannot lose meaningful precision.
      let delta_time = delta_time as f32;
      // Fix(BUG-125): convert to milliseconds before applying the formulas below, which are
      // written and documented in terms of milliseconds
      // Root cause: `delta_time` arrives in seconds (per this function's own doc contract and
      // every real caller), but the /10.0 and /1000.0 constants below assumed milliseconds
      // Pitfall: a doc comment naming a time unit is not proof the formula beneath it agrees —
      // verify the two independently
      let delta_time_ms = delta_time * 1000.0;

      // Decays self.movement_decay% every 10 milliseconds
      let mut decay_percentage = self.rotation.movement_decay * delta_time_ms / 10.0;
      decay_percentage = decay_percentage.min( 1.0 );

      // Fix(BUG-427): guarded with `self.rotation.enabled` -- previously this branch applied
      // smoothed rotation ( and decayed `current_angular_speed` ) purely on
      // `movement_smoothing_enabled`, with no check that rotation was enabled at all, unlike
      // `rotate()` above, which returns immediately when `!self.rotation.enabled`. A caller
      // that disabled rotation ( e.g. `controls.rotation.enabled = false` ) after already
      // accumulating angular speed via smoothing would still see the camera keep rotating on
      // every subsequent `update()` call, since nothing here re-checked `enabled`.
      // Root cause: `rotate()` and `update()` are two independent entry points into the same
      // smoothing state ( `current_angular_speed` ), and only `rotate()` was given the
      // `enabled` guard when smoothing was added -- `update()`'s own smoothing branch was
      // never audited against the same invariant.
      // Pitfall: when a piece of state ( here, "is rotation enabled" ) must gate more than one
      // entry point, grep every reader/writer of that state for the guard, not just the one
      // that happened to be under review when the guard was added.
      if self.rotation.enabled && self.rotation.movement_smoothing_enabled
      {
        self.rotation.current_rotation_angle = self.rotation.current_angular_speed * delta_time_ms / 1000.0;
        self.rotation_apply();
        self.rotation.current_angular_speed *= 1.0 - decay_percentage;
      }
    }
  }

  impl Default for CameraOrbitControls
  {
    /// Creates a new `CameraOrbitControls` with a set of sensible default values.
    #[ inline ]
    fn default() -> Self
    {
      CameraOrbitControls
      {
        eye : F32x3::from( [ 1.0, 0.0, 0.0 ] ),
        up : F32x3::from( [ 0.0, 1.0, 0.0 ] ),
        center : F32x3::from( [ 0.0, 0.0, 0.0 ] ),
        window_size : F32x2::from( [ 1000.0, 1000.0 ] ),
        fov : 70f32.to_radians(),
        zoom : CameraZoomState
        {
          enabled : true,
          speed : 1000.0,
          max_distance : None,
          min_distance : None
        },
        rotation : CameraRotationState
        {
          enabled : true,
          movement_smoothing_enabled : false,
          speed : 500.0,
          current_angular_speed : F32x2::default(),
          current_rotation_angle : F32x2::default(),
          movement_decay : 0.05,
          base_latitude : 0.0,
          base_longitude : 0.0,
          latitude_range : None,
          longitude_range : None
        },
        pan : CameraPanState
        {
          enabled : true
        }
      }
    }
  }

  /// Represents the current state of the camera controls, based on user input.
  #[ cfg( feature = "web" ) ]
  #[ derive( Clone ) ]
  enum CameraState
  {
    /// The camera is not being manipulated.
    None,
    /// The user is rotating the camera.
    Rotate,
    /// The user is panning the camera.
    Pan,
    /// The user is performing a two-finger pinch gesture.
    Pinch,
  }

  /// Creates the `pointerdown` closure that begins tracking a new active pointer
  /// and selects the resulting camera interaction state.
  #[ cfg( feature = "web" ) ]
  #[ inline ]
  fn pointer_down_closure_make
  (
    canvas : &web_sys::HtmlCanvasElement,
    state : &Rc< RefCell< CameraState > >,
    prev_screen_pos : &Rc< RefCell< [ f32; 2 ] > >,
    active_pointers : &Rc< RefCell< HashMap< i32, [ f32; 2 ] > > >
  ) -> Closure< dyn Fn( web_sys::PointerEvent ) >
  {
    let state = state.clone();
    let prev_screen_pos = prev_screen_pos.clone();
    let active_pointers = active_pointers.clone();
    let canvas = canvas.clone();
    Closure::new
    (
      move | e : web_sys::PointerEvent |
      {
        // screen_x/y return f64 under web_sys_unstable_apis (web-sys ≥ 0.3.94); f64→f32 cast is intentional
        let pos = [ e.screen_x() as f32, e.screen_y() as f32 ];
        active_pointers.borrow_mut().insert( e.pointer_id(), pos );
        let count = active_pointers.borrow().len();
        match count
        {
          1 =>
          {
            *prev_screen_pos.borrow_mut() = pos;
            match e.button()
            {
              0 => *state.borrow_mut() = CameraState::Rotate,
              2 => *state.borrow_mut() = CameraState::Pan,
              _ => {}
            }
          }
          _ =>
          {
            // 3+ fingers: enters Pinch, but the "other" anchor is chosen by
            // non-deterministic HashMap iteration order, so zoom may be jittery.
            *state.borrow_mut() = CameraState::Pinch;
          }
        }
        // Keep receiving pointermove even when the finger moves outside the canvas.
        let _ = canvas.set_pointer_capture( e.pointer_id() );
      }
    )
  }

  /// Creates the `pointermove` closure that rotates, pans, or pinch-zooms the
  /// camera depending on the current interaction state.
  #[ cfg( feature = "web" ) ]
  #[ inline ]
  fn pointer_move_closure_make
  (
    camera : &Rc< RefCell< CameraOrbitControls > >,
    state : &Rc< RefCell< CameraState > >,
    prev_screen_pos : &Rc< RefCell< [ f32; 2 ] > >,
    active_pointers : &Rc< RefCell< HashMap< i32, [ f32; 2 ] > > >
  ) -> Closure< dyn Fn( web_sys::PointerEvent ) >
  {
    let state = state.clone();
    let camera = camera.clone();
    let prev_screen_pos = prev_screen_pos.clone();
    let active_pointers = active_pointers.clone();
    Closure::new
    (
      move | e : web_sys::PointerEvent |
      {
        let pointer_id = e.pointer_id();
        // screen_x/y return f64 under web_sys_unstable_apis (web-sys ≥ 0.3.94); f64→f32 cast is intentional
        let new_pos = [ e.screen_x() as f32, e.screen_y() as f32 ];

        let current_state = state.borrow().clone();

        // Snapshot the moved pointer's previous position before updating;
        // the Pinch arm needs it to compute the old inter-finger distance.
        let old_pos = active_pointers.borrow().get( &pointer_id ).copied();

        // Compute movement delta from the single-pointer reference position.
        let prev_pos = *prev_screen_pos.borrow();
        let mut delta = [ new_pos[ 0 ] - prev_pos[ 0 ], new_pos[ 1 ] - prev_pos[ 1 ] ];

        // Update tracking state for all active states.
        *prev_screen_pos.borrow_mut() = new_pos;
        active_pointers.borrow_mut().insert( pointer_id, new_pos );

        match current_state
        {
          CameraState::Pinch =>
          {
            if let Some( old ) = old_pos
            {
              let other_pos = active_pointers
                .borrow()
                .iter()
                .find( |( &id, _ )| id != pointer_id )
                .map( |( _, &pos )| pos );
              if let Some( other ) = other_pos
              {
                let old_dist =
                {
                  let dx = old[ 0 ] - other[ 0 ];
                  let dy = old[ 1 ] - other[ 1 ];
                  ( dx * dx + dy * dy ).sqrt()
                };
                let new_dist =
                {
                  let dx = new_pos[ 0 ] - other[ 0 ];
                  let dy = new_pos[ 1 ] - other[ 1 ];
                  ( dx * dx + dy * dy ).sqrt()
                };
                camera.borrow_mut().zoom( old_dist - new_dist );
              }
            }
          }
          CameraState::Rotate =>
          {
            // Fix(BUG-004): Standardized mouse delta to new-prev convention for both axes; negate X only in rotate arm
            // Root cause: Inconsistent delta sign (prev-new for X, new-prev for Y) inverted pan X-axis direction
            // Pitfall: pan() internally negates X via `- x * dx`; rotate() needs explicit negation for opposite convention
            delta[ 0 ] = -delta[ 0 ];
            camera.borrow_mut().rotate( delta );
          },
          CameraState::Pan => camera.borrow_mut().pan( delta ),
          CameraState::None => {}
        }
      }
    )
  }

  /// Creates the `wheel` closure that zooms the camera when no pointer gesture is active.
  #[ cfg( feature = "web" ) ]
  #[ inline ]
  fn wheel_closure_make
  (
    camera : &Rc< RefCell< CameraOrbitControls > >,
    state : &Rc< RefCell< CameraState > >
  ) -> Closure< dyn Fn( web_sys::WheelEvent ) >
  {
    let state = state.clone();
    let camera = camera.clone();
    Closure::new
    (
      move | e : web_sys::WheelEvent |
      {
        if let CameraState::None = *state.borrow()
        {
          // delta_y is a scroll increment (typically tens to low thousands); f64→f32 cast is intentional
          let delta_y = e.delta_y() as f32;
          camera.borrow_mut().zoom( delta_y );
        }
      }
    )
  }

  /// Creates the shared closure for `pointerup`, `pointerout`, and `pointercancel`,
  /// which all remove the pointer and transition state identically.
  #[ cfg( feature = "web" ) ]
  #[ inline ]
  fn pointer_release_closure_make
  (
    state : &Rc< RefCell< CameraState > >,
    prev_screen_pos : &Rc< RefCell< [ f32; 2 ] > >,
    active_pointers : &Rc< RefCell< HashMap< i32, [ f32; 2 ] > > >
  ) -> Closure< dyn Fn( web_sys::PointerEvent ) >
  {
    let state = state.clone();
    let active_pointers = active_pointers.clone();
    let prev_screen_pos = prev_screen_pos.clone();
    Closure::new
    (
      move | e : web_sys::PointerEvent |
      {
        active_pointers.borrow_mut().remove( &e.pointer_id() );
        let count = active_pointers.borrow().len();
        match count
        {
          0 => *state.borrow_mut() = CameraState::None,
          1 =>
          {
            // One finger remains: resume rotation from its current position.
            let remaining = active_pointers.borrow().values().next().copied();
            if let Some( pos ) = remaining
            {
              *prev_screen_pos.borrow_mut() = pos;
            }
            *state.borrow_mut() = CameraState::Rotate;
          }
          _ => {}
        }
      }
    )
  }

  /// Creates the `contextmenu` closure that suppresses the browser's context menu.
  #[ cfg( feature = "web" ) ]
  #[ inline ]
  fn context_menu_closure_make() -> Closure< dyn Fn( web_sys::PointerEvent ) >
  {
    Closure::new
    (
      move | e : web_sys::PointerEvent |
      {
        e.prevent_default();
      }
    )
  }

  /// Binds pointer events to the camera controls for interaction.
  ///
  /// Sets up event listeners on an `HtmlCanvasElement` to handle camera rotation,
  /// panning, and zooming via both mouse and touch input:
  ///
  /// - **Mouse**: left-click drag → rotate; right-click drag → pan; scroll wheel → zoom.
  /// - **Touch**: one-finger drag → rotate; two-finger pinch → zoom.
  ///   Pan is not available via touch (`PointerEvent.button` is always `0` for touch contacts).
  ///
  /// Also sets `touch-action: none` on the canvas (modifies inline style) so the browser
  /// does not intercept touch gestures before they reach the application, and prevents
  /// the default context menu on right-click.
  ///
  /// # Arguments
  ///
  /// * `canvas` - A reference to the HTML canvas element where the events will be bound.
  /// * `camera` - A reference-counted, mutable reference to the `CameraOrbitControls`
  ///   instance that will be manipulated by the user input.
  #[ cfg( feature = "web" ) ]
  #[ inline ]
  pub fn controls_bind_to_input
  (
    canvas : &web_sys::HtmlCanvasElement,
    camera : &Rc< RefCell< CameraOrbitControls > >
  )
  {
    let state = Rc::new( RefCell::new( CameraState::None ) );
    let prev_screen_pos = Rc::new( RefCell::new( [ 0.0, 0.0 ] ) );
    // pointer_id → last known screen position
    let active_pointers : Rc< RefCell< HashMap< i32, [ f32; 2 ] > > > =
      Rc::new( RefCell::new( HashMap::new() ) );

    // Prevent the browser from handling touch gestures (pinch-to-zoom, scroll) on the canvas.
    let _ = canvas.style().set_property( "touch-action", "none" );

    let on_pointer_down = pointer_down_closure_make( canvas, &state, &prev_screen_pos, &active_pointers );
    let on_pointer_move = pointer_move_closure_make( camera, &state, &prev_screen_pos, &active_pointers );
    let on_wheel = wheel_closure_make( camera, &state );
    let on_pointer_release = pointer_release_closure_make( &state, &prev_screen_pos, &active_pointers );
    let on_context_menu = context_menu_closure_make();

    canvas.set_oncontextmenu( Some( on_context_menu.as_ref().unchecked_ref() ) );
    on_context_menu.forget();

    let _ = canvas.add_event_listener_with_callback( "pointerdown", on_pointer_down.as_ref().unchecked_ref() );
    on_pointer_down.forget();

    let _ = canvas.add_event_listener_with_callback( "pointermove", on_pointer_move.as_ref().unchecked_ref() );
    on_pointer_move.forget();

    let _ = canvas.add_event_listener_with_callback( "wheel", on_wheel.as_ref().unchecked_ref() );
    on_wheel.forget();

    let release_cb = on_pointer_release.as_ref().unchecked_ref();
    let _ = canvas.add_event_listener_with_callback( "pointerup", release_cb );
    let _ = canvas.add_event_listener_with_callback( "pointerout", release_cb );
    let _ = canvas.add_event_listener_with_callback( "pointercancel", release_cb );
    on_pointer_release.forget();
  }
}

// This macro exposes the public interface of the module.
crate::mod_interface!
{
  #[ cfg( feature = "web" ) ]
  own use
  {
    controls_bind_to_input
  };

  /// Exposes the `CameraOrbitControls` struct for public use.
  exposed use
  {
    CameraOrbitControls
  };
}
