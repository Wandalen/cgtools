//! This module provides an implementation of a character controls,
//! allowing for easy character moving, rotation. It is designed to
//! be independent of any specific graphics backend.

mod private
{
  use crate::{ F64x3, QuatF64 };

  /// Controls for animated models
  pub struct CharacterControls
  {
    /// Current character position
    pub transition : F64x3,
    /// Current character rotation
    pub rotation : QuatF64,
    /// Current character scale
    pub scale : F64x3,
    /// Speed of change for transition
    pub transition_speed : f64,
    /// Speed of change for rotation for every angle
    pub rotation_speed : F64x3,
    /// Speed of change for scale
    pub scale_speed : f64
  }

  impl Default for CharacterControls
  {
    /// Creates a new `CharacterControls` with a set of sensible default values.
    fn default() -> Self
    {
      Self
      {
        transition : F64x3::from( [ 0.0, 0.0, 0.0 ] ),
        rotation : QuatF64::from_euler_xyz( [ 0.0, 0.0, 0.0 ] ),
        scale : F64x3::from( [ 1.0, 1.0, 1.0 ] ),
        transition_speed : 1.0,
        rotation_speed : F64x3::from_array( [ 1.0, 1.0, 1.0 ] ),
        scale_speed : 1.0
      }
    }
  }

  /// Represents the current state of the character controls, based on user input.
  enum CharacterState
  {
    /// The character is not being manipulated.
    None,
    /// The user is only moving the character
    Move,
    /// The user is rotating and moving the character.
    RotateMove,
  }

  /// Binds mouse and pointer events to the character controls for interaction.
  ///
  /// This function sets up event listeners on an `HtmlCanvasElement` to handle
  /// character movement, rotation. Left-click (pointer button 0) is used
  /// for rotation. It also prevents the default context menu from appearing on right-click.
  ///
  /// # Arguments
  ///
  /// * `canvas` - A reference to the HTML canvas element where the events will be bound.
  /// * `controls` - A reference-counted, mutable reference to the `CharacterControls`
  ///   instance that will be manipulated by the user input.
  pub fn bind_controls_to_input
  (
    canvas : &web_sys::HtmlCanvasElement,
    controls : &Rc< RefCell< CharacterControls > >
  )
  {
    let state =  Rc::new( RefCell::new( CameraState::None ) );
    let prev_screen_pos = Rc::new( RefCell::new( [ 0.0, 0.0 ] ) );

    let on_key_press : Closure< dyn Fn( _ ) > = Closure::new
    (
      {
        move | e : web_sys::PointerEvent |
        {

        }
      }
    );

    let on_key_down : Closure< dyn Fn( _ ) > = Closure::new
    (
      {
        move | e : web_sys::PointerEvent |
        {

        }
      }
    );

    let on_key_up : Closure< dyn Fn( _ ) > = Closure::new
    (
      {
        move | e : web_sys::PointerEvent |
        {

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

    canvas.set_onkeypress( Some( on_key_press.as_ref().unchecked_ref() ) );
    on_key_press.forget();

    canvas.set_onkeydown( Some( on_key_down.as_ref().unchecked_ref() ) );
    on_key_down.forget();

    canvas.set_onkeyup( Some( on_key_up.as_ref().unchecked_ref() ) );
    on_key_up.forget();

    canvas.set_onmousemove( Some( on_mouse_move.as_ref().unchecked_ref() ) );
    on_mouse_move.forget();

    canvas.set_onpointerdown( Some( on_pointer_down.as_ref().unchecked_ref() ) );
    on_pointer_down.forget();

    canvas.set_onpointerup( Some( on_pointer_up.as_ref().unchecked_ref() ) );
    on_pointer_up.forget();

    canvas.set_onpointerout( Some( on_pointer_out.as_ref().unchecked_ref() ) );
    on_pointer_out.forget();

    canvas.set_oncontextmenu( Some( on_context_menu.as_ref().unchecked_ref() ) );
    on_context_menu.forget();
  }
}

crate::mod_interface!
{
  own use
  {
    bind_controls_to_input
  };

  orphan use
  {
    CharacterControls
  };
}
