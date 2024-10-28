use std::sync::{ Arc, Mutex };

use mingl::CameraOrbitControls;
use minwebgl::{ self as gl, JsCast };
use gl::
{
  GL,
};
use web_sys::
{
  wasm_bindgen::prelude::Closure, 
  HtmlCanvasElement
};



enum CameraState
{
  Rotate,
  Pan,
  None
}


pub fn setup_controls
(
  canvas : &HtmlCanvasElement,
  camera : &Arc< Mutex< CameraOrbitControls > >
)
{
  let state =  Arc::new( Mutex::new( CameraState::None ) );
  let prev_screen_pos = Arc::new( Mutex::new( [ 0.0, 0.0 ] ) );

  let on_pointer_down : Closure< dyn Fn( _ ) > = Closure::new
  (
    {
      let state = state.clone();
      let prev_screen_pos = prev_screen_pos.clone();
      move | e : gl::web_sys::PointerEvent |
      {
        *prev_screen_pos.lock().unwrap() = [ e.screen_x() as f32, e.screen_y() as f32 ];
        match e.button()
        {
          0 => *state.lock().unwrap() = CameraState::Rotate,
          2 => *state.lock().unwrap() = CameraState::Pan,
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
      move | e : gl::web_sys::MouseEvent |
      {
        let prev_pos = *prev_screen_pos.lock().unwrap();
        let new_pos = [ e.screen_x() as f32, e.screen_y() as f32 ];
        let delta = [ new_pos[ 0 ] - prev_pos[ 0 ], new_pos[ 1 ] - prev_pos[ 1 ] ];
        *prev_screen_pos.lock().unwrap() = new_pos;
        match *state.lock().unwrap()
        {
          CameraState::Rotate => 
          {
            camera.lock().unwrap().rotate( delta );
          },
          CameraState::Pan => 
          {
            camera.lock().unwrap().pan( delta );
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
      move | e : gl::web_sys::WheelEvent |
      {
        match *state.lock().unwrap()
        {
          CameraState::None => {
            let delta_y = e.delta_y() as f32;
            camera.lock().unwrap().zoom( delta_y );
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
        *state.lock().unwrap() = CameraState::None;
      }
    }
  );

  let on_pointer_out : Closure< dyn Fn() > = Closure::new
  (
    {
      let state = state.clone();
      move | |
      {
        *state.lock().unwrap() = CameraState::None;
      }
    }
  );

  let on_context_menu : Closure< dyn Fn( _ ) > = Closure::new
  (
    {
      move | e : gl::web_sys::PointerEvent |
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