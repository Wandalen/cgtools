use minwebgl as gl;
use std::{ cell::RefCell, rc::Rc };
use web_sys::wasm_bindgen::prelude::*;

pub struct CursorControls;

impl CursorControls
{
  pub fn setup
  (
    this : &Rc< RefCell< Self > >,
    // camera : &Rc< RefCell< free_camera::FreeCamera > >,
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
        // let camera = camera.clone();
        let is_pointer_locked = is_pointer_locked.clone();
        let width = canvas.width() as f32;

        move | event : web_sys::MouseEvent |
        {
          if *is_pointer_locked.borrow()
          {
            // camera.borrow_mut().rotate( &[ 0.0, -delta_x, 0.0 ] );
            // camera.borrow_mut().rotate_local( &[ -delta_y, 0.0, 0.0 ] );
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
