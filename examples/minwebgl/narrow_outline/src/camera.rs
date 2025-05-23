use mingl::CameraOrbitControls;
use minwebgl::{ self as gl, JsCast };
use gl::
{
  web_sys::
  {
    WebGlUniformLocation,
    wasm_bindgen::prelude::Closure, 
    HtmlCanvasElement
  }
};
use ndarray_cg::
{
  mat3x3h::perspective_rh_gl,
  F32x4x4,
  F32x3,
  F32x2
};
use std::
{ 
  cell::RefCell, 
  rc::Rc 
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
      move | e : gl::web_sys::PointerEvent |
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
      move | e : gl::web_sys::MouseEvent |
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
      move | e : gl::web_sys::WheelEvent |
      {
        match *state.borrow_mut()
        {
          CameraState::None => {
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

pub struct Camera
{
  controls : Rc< RefCell< CameraOrbitControls > >,
  aspect_ratio : f32,
  fov : f32,
  near : f32,
  far : f32,
  projection_matrix : F32x4x4,
}

impl Camera
{
  pub fn new
  (
    eye : F32x3,
    up : F32x3,
    look_at : F32x3,
    aspect_ratio : f32,
    fov : f32,
    near : f32,
    far : f32
  ) -> Self
  {
    let projection_matrix = perspective_rh_gl
    (
      fov,
      aspect_ratio,
      near,
      far
    );

    let controls = CameraOrbitControls
    {
      eye : eye,
      up : up,
      center : look_at,
      fov,
      rotation_speed_scale : 200.0,
      ..Default::default()
    };

    let controls = Rc::new( RefCell::new( controls ) );

    Self
    {
      controls,
      near,
      far,
      aspect_ratio,
      fov,
      projection_matrix
    }
  }

  pub fn apply
  (
    &self,
    gl : &gl::WebGl2RenderingContext,
    view_loc : &WebGlUniformLocation,
    projection_loc : &WebGlUniformLocation
  )
  {
    let view_matrix = self.get_view_matrix().to_array();
    let projection_matrix = self.get_projection_matrix();

    gl::uniform::matrix_upload
    (
      &gl,
      Some( view_loc.clone() ),
      &view_matrix[ .. ],
      true
    ).unwrap();

    gl::uniform::matrix_upload
    (
      &gl,
      Some( projection_loc.clone() ),
      projection_matrix.to_array().as_slice(),
      true
    ).unwrap();
  }

  pub fn set_window_size( &mut self, window_size : F32x2 )
  {
    self.controls.borrow_mut().set_size( window_size.to_array() );
  }

  pub fn get_controls( &self ) -> Rc< RefCell< CameraOrbitControls > >
  {
    self.controls.clone()
  }

  pub fn get_eye( &self ) -> F32x3
  {
    self.controls.borrow().eye
  }

  pub fn get_view_matrix( &self ) -> F32x4x4
  {
    self.controls.borrow().view()
  }

  pub fn get_projection_matrix( &self ) -> F32x4x4
  {
    self.projection_matrix
  }
}