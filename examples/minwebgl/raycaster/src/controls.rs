use std::{ cell::RefCell, rc::Rc };
use browser_input::{ Input, keyboard::KeyboardKey };

pub struct Controls
{
  input : Input,
}

impl Controls
{
  pub fn move_direction( &self ) -> f32
  {
    i32::from( self.input.is_key_down( KeyboardKey::KeyW ) ) as f32
    - i32::from( self.input.is_key_down( KeyboardKey::KeyS ) ) as f32
  }

  pub fn rotation_direction( &self ) -> f32
  {
    i32::from( self.input.is_key_down( KeyboardKey::KeyA ) ) as f32
    - i32::from( self.input.is_key_down( KeyboardKey::KeyD ) ) as f32
  }

  /// Applies queued browser input events to the tracked key state. Must be called
  /// once per frame before `move_direction`/`rotation_direction` are read.
  pub fn state_update( &mut self )
  {
    self.input.state_update();
  }

  pub fn setup() -> Rc< RefCell< Self > >
  {
    let input = Input::new( None, browser_input::CLIENT ).expect( "Failed to initialize browser input" );
    Rc::new( RefCell::new( Self { input } ) )
  }
}
