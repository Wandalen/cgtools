use std::{ cell::RefCell, rc::Rc };
use web_sys::wasm_bindgen::prelude::*;

#[ derive( Default ) ]
pub struct Controls
{
  forward : bool,
  backward : bool,
  left : bool,
  right : bool,
}

impl Controls
{
  pub fn move_direction( &self ) -> f32
  {
    self.forward as i32 as f32 - self.backward as i32 as f32
  }

  pub fn rotation_direction( &self ) -> f32
  {
    self.left as i32 as f32 - self.right as i32 as f32
  }

  pub fn setup() -> Rc< RefCell< Self > >
  {
    let this = Self::default();
    let this = Rc::new( RefCell::new( this ) );
    let window = web_sys::window().expect( "Window should exist" );

    let on_keydown : Closure< dyn Fn( _ ) > = Closure::new
    (
      {
        let this = this.clone();
        move | event : web_sys::KeyboardEvent |
        {
          match event.code().as_str()
          {
            "KeyW" => this.borrow_mut().forward = true,
            "KeyA" => this.borrow_mut().left = true,
            "KeyS" => this.borrow_mut().backward = true,
            "KeyD" => this.borrow_mut().right = true,
            _ => {}
          }
        }
      }
    );

    let on_keyup : Closure< dyn Fn( _ ) > = Closure::new
    (
      {
        let this = this.clone();
        move | event : web_sys::KeyboardEvent |
        {
          match event.code().as_str()
          {
            "KeyW" => this.borrow_mut().forward = false,
            "KeyA" => this.borrow_mut().left = false,
            "KeyS" => this.borrow_mut().backward = false,
            "KeyD" => this.borrow_mut().right = false,
            _ => {}
          }
        }
      }
    );

    window.set_onkeydown( Some( on_keydown.as_ref().unchecked_ref() ) );
    window.set_onkeyup( Some( on_keyup.as_ref().unchecked_ref() ) );
    on_keydown.forget();
    on_keyup.forget();

    this
  }
}
