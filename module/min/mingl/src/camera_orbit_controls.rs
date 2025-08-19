//! This module provides an implementation of a camera with orbit controls,
//! allowing for easy 3D scene navigation through rotation, panning, and zooming.
//! It is designed to be independent of any specific graphics backend.

/// Internal namespace for implementation details.
mod private
{
  use crate::{ F32x3, F32x2, math };
  use std::{ cell::RefCell, rc::Rc };
  use wasm_bindgen::{ JsCast, prelude::Closure };
  use crate::web::web_sys;

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

  /// Represents the current state of the camera controls, based on user input.
  pub enum CameraState
  {
    /// The camera is not being manipulated.
    None,
    /// The user is rotating the camera.
    Rotate,
    /// The user is panning the camera.
    Pan,
  }

  /// Binds mouse and pointer events to the camera controls for interaction.
  ///
  /// This function sets up event listeners on an `HtmlCanvasElement` to handle
  /// camera rotation, panning, and zooming. Left-click (pointer button 0) is used
  /// for rotation, right-click (pointer button 2) for panning, and the mouse wheel
  /// is used for zooming. It also prevents the default context menu from appearing on right-click.
  ///
  /// # Arguments
  ///
  /// * `canvas` - A reference to the HTML canvas element where the events will be bound.
  /// * `camera` - A reference-counted, mutable reference to the `CameraOrbitControls`
  ///   instance that will be manipulated by the user input.
  pub fn bind_controls_to_input
  (
    canvas : &web_sys::HtmlCanvasElement,
    camera : &Rc< RefCell< CameraOrbitControls > >
  )
  {
    let state =  Rc::new( RefCell::new( CameraState::None ) );
    let prev_screen_pos = Rc::new( RefCell::new( [ 0.0, 0.0 ] ) );

    let on_pointer_down : Closure< dyn Fn( _ ) > = Closure::new
    (
      {
        let state = state.clone();
        let prev_screen_pos = prev_screen_pos.clone();
        move | e : web_sys::PointerEvent |
        {
          *prev_screen_pos.borrow_mut() = [ e.screen_x() as f32, e.screen_y() as f32 ];
          match e.button()
          {
            0 => *state.borrow_mut() = CameraState::Rotate,
            2 => *state.borrow_mut() = CameraState::Pan,
            _ => {}
          }
        }
      }
    );

    let on_mouse_move : Closure< dyn Fn( _ ) > = Closure::new
    (
      {
        let state = state.clone();
        let camera = camera.clone();
        let prev_screen_pos = prev_screen_pos.clone();
        move | e : web_sys::MouseEvent |
        {
          let prev_pos = *prev_screen_pos.borrow_mut();
          let new_pos = [ e.screen_x() as f32, e.screen_y() as f32 ];
          let delta = [ new_pos[ 0 ] - prev_pos[ 0 ], new_pos[ 1 ] - prev_pos[ 1 ] ];
          *prev_screen_pos.borrow_mut() = new_pos;
          match *state.borrow_mut()
          {
            CameraState::Rotate =>
            {
              camera.borrow_mut().rotate( delta );
            },
            CameraState::Pan =>
            {
              camera.borrow_mut().pan( delta );
            }
            _ => {}
          }
        }
      }
    );

    let on_wheel : Closure< dyn Fn( _ ) > = Closure::new
    (
      {
        let state = state.clone();
        let camera = camera.clone();
        move | e : web_sys::WheelEvent |
        {
          match *state.borrow_mut()
          {
            CameraState::None => 
            {
              let delta_y = e.delta_y() as f32;
              camera.borrow_mut().zoom( delta_y );
            },
            _ => {}
          }
        }
      }
    );

    let on_pointer_up : Closure< dyn Fn() > = Closure::new
    (
      {
        let state = state.clone();
        move | |
        {
          *state.borrow_mut() = CameraState::None;
        }
      }
    );

    let on_pointer_out : Closure< dyn Fn() > = Closure::new
    (
      {
        let state = state.clone();
        move | |
        {
          *state.borrow_mut() = CameraState::None;
        }
      }
    );

    let on_context_menu : Closure< dyn Fn( _ ) > = Closure::new
    (
      {
        move | e : web_sys::PointerEvent |
        {
          e.prevent_default();
        }
      }
    );

    canvas.set_oncontextmenu( Some( on_context_menu.as_ref().unchecked_ref() ) );
    on_context_menu.forget();

    canvas.set_onpointerdown( Some( on_pointer_down.as_ref().unchecked_ref() ) );
    on_pointer_down.forget();

    canvas.set_onmousemove( Some( on_mouse_move.as_ref().unchecked_ref() ) );
    on_mouse_move.forget();

    canvas.set_onwheel( Some( on_wheel.as_ref().unchecked_ref() ) );
    on_wheel.forget();

    canvas.set_onpointerup( Some( on_pointer_up.as_ref().unchecked_ref() ) );
    on_pointer_up.forget();

    canvas.set_onpointerout( Some( on_pointer_out.as_ref().unchecked_ref() ) );
    on_pointer_out.forget();
  }
}

// This macro exposes the public interface of the module.
crate::mod_interface!
{
  /// Exposes the `CameraOrbitControls` struct for public use.
  exposed use
  {
    CameraOrbitControls,
    CameraState,
    bind_controls_to_input
  };
}
