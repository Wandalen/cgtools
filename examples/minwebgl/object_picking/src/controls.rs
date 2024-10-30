use std::{ cell::RefCell, rc::Rc };
use mingl::free_camera;
use web_sys::wasm_bindgen::prelude::*;
use minwebgl as gl;

#[ derive( Debug, Default ) ]
pub struct ButtonControls
{
  forward : bool,
  backward : bool,
  right : bool,
  left : bool,
  accelerate : bool,
}

impl ButtonControls
{
  pub fn setup_wasd( this : &Rc< RefCell< Self > > )
  {
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
            "ShiftLeft" => this.borrow_mut().accelerate = true,
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
            "ShiftLeft" => this.borrow_mut().accelerate = false,
            _ => {}
          }
        }
      }
    );

    window.add_event_listener_with_callback("keydown", on_keydown.as_ref().unchecked_ref() ).unwrap();
    window.add_event_listener_with_callback("keyup", on_keyup.as_ref().unchecked_ref() ).unwrap();
    on_keydown.forget();
    on_keyup.forget();
  }

  /// Retruns direction vector depending on pressed keys
  ///
  /// Forward direction is -Z axis
  pub fn as_vec( &self ) -> [ f32; 3 ]
  {
    let right = self.right as i32 as f32;
    let left = self.left as i32 as f32;
    let forward = self.forward as i32 as f32;
    let backward = self.backward as i32 as f32;

    glam::vec3( right - left, 0.0, -forward + backward ).normalize_or_zero().to_array()
  }

  /// Returns 1.0 if accelarate is pressed, 0.0 if not
  pub fn accelerate( &self ) -> f32
  {
    self.accelerate as i32 as f32
  }
}

pub struct CursorControls
{
  pub sensitivity : f32,
}

impl CursorControls
{
  pub fn setup
  (
    this : &Rc< RefCell< Self > >,
    camera : &Rc< RefCell< free_camera::FreeCamera > >,
    click_pos : &Rc< RefCell< [ i32; 2 ] > >
  )
  {
    let canvas = gl::canvas::retrieve().expect( "Canvas should exist" );
    let is_pointer_locked = Rc::new( RefCell::new( false ) );

    let on_button_down : Closure< dyn Fn( _ ) > = Closure::new
    (
      {
        let canvas = canvas.clone();
        let is_pointer_locked = is_pointer_locked.clone();
        let click_pos = click_pos.clone();
        move | event : web_sys::MouseEvent |
        {
          match event.button()
          {
            0 =>
            {
              if !*is_pointer_locked.borrow()
              {
                click_pos.borrow_mut()[ 0 ] = event.client_x();
                click_pos.borrow_mut()[ 1 ] = canvas.height() as i32 - event.client_y();
              }
            }
            2 =>
            {
              let val = *is_pointer_locked.borrow();
              *is_pointer_locked.borrow_mut() = !val;
              if *is_pointer_locked.borrow()
              {
                canvas.request_pointer_lock();
              }
              else
              {
                web_sys::window().unwrap().document().unwrap().exit_pointer_lock();
              }
            }
            _ => {}
          }
        }
      }
    );

    let on_mouse_move : Closure< dyn Fn( web_sys::MouseEvent ) > = Closure::new
    (
      {
        let this = this.clone();
        let camera = camera.clone();
        let is_pointer_locked = is_pointer_locked.clone();
        let width = canvas.width() as f32;

        move | event : web_sys::MouseEvent |
        {
          if *is_pointer_locked.borrow()
          {
            let sensitivity = this.borrow().sensitivity;
            let delta_x = event.movement_x() as f32 / width * sensitivity;
            let delta_y = event.movement_y() as f32 / width * sensitivity;
            camera.borrow_mut().rotate( &[ 0.0, -delta_x, 0.0 ] );
            camera.borrow_mut().rotate_local( &[ -delta_y, 0.0, 0.0 ] );
          }
        }
      }
    );

    let on_context_menu : Closure< dyn Fn( _ ) > = Closure::new
    (
      | event : web_sys::MouseEvent |
      {
        event.prevent_default();
      }
    );

    canvas.set_onmousedown( Some( &on_button_down.as_ref().unchecked_ref() ) );
    canvas.set_onmousemove( Some( &on_mouse_move.as_ref().unchecked_ref() ) );
    canvas.set_oncontextmenu( Some( &on_context_menu.as_ref().unchecked_ref() ) );

    on_button_down.forget();
    on_mouse_move.forget();
    on_context_menu.forget();
  }
}
