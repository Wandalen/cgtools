/// This module provides an implementation of character controls,
/// allowing for WASD movement and mouse-based rotation. It is designed to
/// be independent of any specific graphics backend.
mod private
{
  use crate::{ F64x3, QuatF64 };
  use std::{ cell::RefCell, rc::Rc };
  use wasm_bindgen::{ JsCast, prelude::Closure };
  use crate::web::web_sys;
  use std::ops::Range;

  /// Controls for character movement and rotation using WASD and mouse.
  ///
  /// This controller allows for first-person or third-person character movement
  /// where WASD keys control movement and mouse controls rotation.
  /// - W/S: Move forward/backward along the character's forward direction
  /// - A/D: Move left/right perpendicular to the forward direction (strafe)
  /// - Mouse: Rotate the character (yaw and pitch)
  pub struct CharacterControls
  {
    /// Current character position in world space
    position : F64x3,
    /// Current character rotation as a quaternion
    rotation : QuatF64,
    /// Current yaw angle (rotation around Y axis) in radians
    yaw : f64,
    /// Current pitch angle (rotation around X axis) in radians
    pitch : f64,
    /// Distance between character and camera
    pub zoom : f64,
    /// Movement speed in units per second
    pub move_speed : f64,
    /// Rotation sensitivity for mouse movement
    pub rotation_sensitivity : f64,
    /// A scaling factor to adjust the sensitivity of camera zooming.
    pub zoom_speed_scale : f64,
    /// Minimum and maximum pitch angle in radians (looking down)
    pitch_range : Range< f64 >,
    /// Minimum and maximum zoom distance
    pub zoom_range : Range< f64 >,
  }

  impl CharacterControls
  {
    /// Returns the current position of the character.
    pub fn position( &self ) -> F64x3
    {
      self.position
    }

    /// Returns the current rotation of the character.
    pub fn rotation( &self ) -> QuatF64
    {
      self.rotation
    }

    /// Returns the current yaw of the character.
    pub fn yaw( &self ) -> f64
    {
      self.yaw
    }

    /// Returns the current pitch of the character.
    pub fn pitch( &self ) -> f64
    {
      self.pitch
    }

    /// Returns the forward direction vector based on current rotation.
    ///
    /// This is the direction the character is facing.
    pub fn forward( &self ) -> F64x3
    {
      // let [ x, y, z, w ] = self.rotation.to_array();
      // F64x3::from_array
      // (
      // [
      //     2.0 * ( x * z + w * y ),
      //     2.0 * ( y * z - w * x ),
      //     1.0 - 2.0 * ( x * x + y * y )
      //   ]
      // )

      let forward = QuatF64::from( [ 0.0, 0.0, 1.0, 0.0 ] );
      let direction = self.rotation * forward * self.rotation.conjugate();
      F64x3::from_slice( &direction.to_array()[ ..3 ] )
    }

    /// Returns the right direction vector based on current rotation.
    ///
    /// This is perpendicular to the forward direction, used for strafing.
    pub fn right( &self ) -> F64x3
    {
      // let [ x, y, z, w ] = self.rotation.to_array();
      // F64x3::from_array
      // (
      // [
      //     2.0 * ( y * y + z * z ) - 1.0,
      //     - 2.0 * ( x * y + w * z ),
      //     - 2.0 * ( x * z - w * y )
      //   ]
      // )

      let right = QuatF64::from( [ -1.0, 0.0, 0.0, 0.0 ] );
      let direction = self.rotation * right * self.rotation.conjugate();
      F64x3::from_slice( &direction.to_array()[ ..3 ] )
    }

    /// Returns the up direction vector based on current rotation.
    pub fn up( &self ) -> F64x3
    {
      // let [ x, y, z, w ] = self.rotation.to_array();
      // F64x3::from_array
      // (
      // [
      //     2.0 * ( x * y - w * z ),
      //     1.0 - 2.0 * ( x * x + z * z ),
      //     2.0 * ( y * z + w * x )
      //   ]
      // )

      let up = QuatF64::from( [ 0.0, 1.0, 0.0, 0.0 ] );
      let direction = self.rotation * up * self.rotation.conjugate();
      F64x3::from_slice( &direction.to_array()[ ..3 ] )
    }

    /// Updates character rotation based on mouse movement delta.
    ///
    /// # Arguments
    /// * `delta_x` - Horizontal mouse movement (affects yaw)
    /// * `delta_y` - Vertical mouse movement (affects pitch)
    pub fn rotate( &mut self, delta_x : f64, delta_y : f64 )
    {
      // Update yaw (left/right rotation around Y axis)
      self.yaw -= delta_x * self.rotation_sensitivity;

      // Update pitch (up/down rotation around X axis)
      self.pitch -= delta_y * self.rotation_sensitivity;

      // Clamp pitch to prevent over-rotation
      self.pitch = self.pitch.clamp( self.pitch_range.start, self.pitch_range.end );

      // Create rotation quaternion from yaw and pitch
      // Order: Yaw around Y axis, then Pitch around X axis
      let quat_yaw = QuatF64::from_angle_y( self.yaw ).normalize();
      let quat_pitch = QuatF64::from_angle_x( self.pitch ).normalize();

      self.rotation = ( quat_yaw * quat_pitch ).normalize();
    }

    /// Updates character position based on movement input.
    ///
    /// # Arguments
    /// * `input` - Movement input state containing key press information
    /// * `delta_time` - Time elapsed since last update in seconds
    pub fn update( &mut self, input : &CharacterInput, delta_time : f64 )
    {
      let mut movement = F64x3::from( [ 0.0, 0.0, 0.0 ] );

      // Calculate movement direction based on input
      if input.move_forward
      {
        let mut forward = self.forward();
        // forward.0[ 1 ] = 0.0;
        movement += forward;
      }
      if input.move_backward
      {
        let mut forward = self.forward();
        // forward.0[ 1 ] = 0.0;
        movement -= forward;
      }
      if input.move_left
      {
        let mut right = self.right();
        // right.0[ 1 ] = 0.0;
        movement -= right;
      }
      if input.move_right
      {
        let mut right = self.right();
        // right.0[ 1 ] = 0.0;
        movement += right;
      }

      // Normalize movement vector to prevent faster diagonal movement
      let movement_magnitude = movement.mag();
      if movement_magnitude > 0.0
      {
        movement /= movement_magnitude;
      }

      // Apply speed and delta time
      movement *= self.move_speed * delta_time;

      // Update position
      self.position += movement;
    }

    /// Sets the character position.
    pub fn set_position( &mut self, position : F64x3 )
    {
      self.position = position;
    }

    /// Sets the character rotation using yaw and pitch angles.
    ///
    /// # Arguments
    /// * `yaw` - Rotation around Y axis in radians
    /// * `pitch` - Rotation around X axis in radians
    pub fn set_rotation( &mut self, yaw : f64, pitch : f64 )
    {
      self.yaw = yaw;
      self.pitch = pitch.clamp( self.pitch_range.start, self.pitch_range.end );

      let quat_yaw = QuatF64::from_angle_y( self.yaw ).normalize();
      let quat_pitch = QuatF64::from_angle_x( self.pitch ).normalize();

      self.rotation = ( quat_yaw * quat_pitch ).normalize();
    }

    /// Zooms the camera in or out along its viewing direction.
    ///
    /// # Arguments
    /// * `delta_y` - The scroll amount, typically from a mouse wheel event.
    ///   A negative value zooms in, and a positive value zooms out.
    pub fn zoom
    (
      &mut self,
      mut delta_y : f64
    )
    {
      delta_y *= self.zoom_speed_scale;
      self.zoom += delta_y;
      self.zoom = self.zoom.clamp( self.zoom_range.start, self.zoom_range.end );
    }
  }

  impl Default for CharacterControls
  {
    /// Creates a new `CharacterControls` with sensible default values.
    fn default() -> Self
    {
      Self
      {
        position : F64x3::from( [ 0.0, 0.0, 0.0 ] ),
        rotation : QuatF64::default(),
        yaw : 0.0,
        pitch : 0.0,
        zoom : 2.0,
        move_speed : 10.0,
        rotation_sensitivity : 0.002,
        zoom_speed_scale : 0.01,
        pitch_range : -0.5..0.5,
        zoom_range : 0.5..10.0
      }
    }
  }

  /// Tracks the current state of character input (which keys are pressed).
  #[ derive( Debug, Clone, Default ) ]
  pub struct CharacterInput
  {
    /// W key pressed - move forward
    pub move_forward : bool,
    /// S key pressed - move backward
    pub move_backward : bool,
    /// A key pressed - move left
    pub move_left : bool,
    /// D key pressed - move right
    pub move_right : bool,
  }

  impl CharacterInput
  {
    /// Creates a new CharacterInput with all keys unpressed.
    pub fn new() -> Self
    {
      Self::default()
    }

    /// Resets all input states to false.
    pub fn reset( &mut self )
    {
      self.move_forward = false;
      self.move_backward = false;
      self.move_left = false;
      self.move_right = false;
    }
  }

  /// Binds keyboard and mouse events to character controls for interaction.
  ///
  /// This function sets up event listeners on an `HtmlCanvasElement` to handle
  /// character movement via WASD keys and rotation via mouse movement.
  /// The canvas element needs to be focused to receive keyboard events.
  ///
  /// # Arguments
  ///
  /// * `canvas` - A reference to the HTML canvas element where events will be bound
  /// * `controls` - A reference-counted, mutable reference to the `CharacterControls`
  /// * `input` - A reference-counted, mutable reference to the `CharacterInput`
  ///
  /// # Example
  ///
  /// ```ignore
  /// let controls = Rc::new( RefCell::new( CharacterControls::default() ) );
  /// let input = Rc::new( RefCell::new( CharacterInput::new() ) );
  /// bind_controls_to_input( &canvas, &controls, &input );
  ///
  /// // In your update loop:
  /// controls.borrow_mut().update( &input.borrow(), delta_time );
  /// ```
  pub fn bind_controls_to_input
  (
    canvas : &web_sys::HtmlCanvasElement,
    controls : &Rc< RefCell< CharacterControls > >,
    input : &Rc< RefCell< CharacterInput > >
  )
  {
    let is_pointer_locked = Rc::new( RefCell::new( false ) );

    let on_key_down : Closure< dyn Fn( _ ) > = Closure::new
    (
      {
        let input = input.clone();
        move | e : web_sys::KeyboardEvent |
        {
          let key = e.key();
          let mut input = input.borrow_mut();

          match key.as_str()
          {
            "w" | "W" => input.move_forward = true,
            "s" | "S" => input.move_backward = true,
            "a" | "A" => input.move_left = true,
            "d" | "D" => input.move_right = true,
            _ => {}
          }
        }
      }
    );

    let on_key_up : Closure< dyn Fn( _ ) > = Closure::new
    (
      {
        let input = input.clone();
        move | e : web_sys::KeyboardEvent |
        {
          let key = e.key();
          let mut input = input.borrow_mut();

          match key.as_str()
          {
            "w" | "W" => input.move_forward = false,
            "s" | "S" => input.move_backward = false,
            "a" | "A" => input.move_left = false,
            "d" | "D" => input.move_right = false,
            _ => {}
          }
        }
      }
    );

    let on_mouse_move : Closure< dyn Fn( _ ) > = Closure::new
    (
      {
        let controls = controls.clone();
        let is_pointer_locked = is_pointer_locked.clone();
        move | e : web_sys::MouseEvent |
        {
          if *is_pointer_locked.borrow()
          {
            let delta_x = e.movement_x() as f64;
            let delta_y = e.movement_y() as f64;
            controls.borrow_mut().rotate( delta_x, delta_y );
          }
        }
      }
    );

    let on_wheel : Closure< dyn Fn( _ ) > = Closure::new
    (
      {
        let controls = controls.clone();
        let is_pointer_locked = is_pointer_locked.clone();
        move | e : web_sys::WheelEvent |
        {
          if *is_pointer_locked.borrow()
          {
            let delta_y = e.delta_y();
            controls.borrow_mut().zoom( delta_y );
          }
        }
      }
    );

    let on_context_menu : Closure< dyn Fn( _ ) > = Closure::new
    (
      {
        move | e : web_sys::MouseEvent |
        {
          e.prevent_default();
        }
      }
    );

    let on_click : Closure< dyn Fn() > = Closure::new
    (
      {
        let canvas = canvas.clone();
        let is_pointer_locked = is_pointer_locked.clone();
        move | |
        {
          if !*is_pointer_locked.borrow()
          {
            let _ = canvas.request_pointer_lock();
          }
        }
      }
    );

    let on_pointer_lock_change : Closure< dyn Fn() > = Closure::new
    (
      {
        let is_pointer_locked = is_pointer_locked.clone();
        move | |
        {
          if let Some( document ) = web_sys::window().and_then( | w | w.document() )
          {
            let locked = document.pointer_lock_element().is_some();
            *is_pointer_locked.borrow_mut() = locked;
          }
        }
      }
    );

    let on_pointer_lock_error : Closure< dyn Fn() > = Closure::new
    (
      {
        move | |
        {
          crate::web::error!( "Pointer lock error" );
        }
      }
    );

    let _ = canvas.set_attribute( "tabindex", "0" );
    let _ = canvas.focus();

    let _ = canvas.add_event_listener_with_callback( "click", on_click.as_ref().unchecked_ref() );
    on_click.forget();

    let _ = canvas.add_event_listener_with_callback( "keydown", on_key_down.as_ref().unchecked_ref() );
    on_key_down.forget();

    let _ = canvas.add_event_listener_with_callback( "keyup", on_key_up.as_ref().unchecked_ref() );
    on_key_up.forget();

    let _ = canvas.add_event_listener_with_callback( "mousemove", on_mouse_move.as_ref().unchecked_ref() );
    on_mouse_move.forget();

    let _ = canvas.add_event_listener_with_callback( "wheel", on_wheel.as_ref().unchecked_ref() );
    on_wheel.forget();

    canvas.set_oncontextmenu( Some( on_context_menu.as_ref().unchecked_ref() ) );
    on_context_menu.forget();

    if let Some( document ) = web_sys::window().and_then( | w | w.document() )
    {
      let _ = document.add_event_listener_with_callback
      (
        "pointerlockchange",
        on_pointer_lock_change.as_ref().unchecked_ref()
      );
      on_pointer_lock_change.forget();

      let _ = document.add_event_listener_with_callback
      (
        "pointerlockerror",
        on_pointer_lock_error.as_ref().unchecked_ref()
      );
      on_pointer_lock_error.forget();
    }
  }
}

crate::mod_interface!
{
  own use
  {
    bind_controls_to_input
  };

  exposed use
  {
    CharacterControls,
    CharacterInput
  };
}
