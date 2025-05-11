use std::{cell::RefCell, rc::Rc};

use mingl::CameraOrbitControls;
use minwebgl as gl;

use crate::program::ProgramInfo;


#[allow(dead_code)]
pub struct Camera
{
  controls : Rc< RefCell< CameraOrbitControls > >,
  aspect_ratio : f32,
  fov : f32,
  near : f32,
  far : f32,
  projection_matrix : gl::F32x4x4,
}

impl Camera
{
  pub fn new
  (
    eye : gl::F32x3,
    up : gl::F32x3,
    look_at : gl::F32x3,
    aspect_ratio : f32,
    fov : f32,
    near : f32,
    far : f32
  ) -> Self
  {
    let projection_matrix = gl::math::mat3x3h::perspective_rh_gl
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
    program_info : &ProgramInfo
  )
  {
    let locations = program_info.get_locations();

    let view_matrix = self.get_view_matrix().to_array();
    let eye = self.get_eye().to_array();
    let projection_matrix = self.get_projection_matrix();

    gl::uniform::upload
    (
      &gl,
      locations.get( "cameraPosition" ).unwrap().clone(),
      &eye[ .. ]
    ).unwrap();

    gl::uniform::matrix_upload
    (
      &gl,
      locations.get( "viewMatrix" ).unwrap().clone(),
      &view_matrix[ .. ],
      true
    ).unwrap();

    gl::uniform::matrix_upload
    (
      &gl,
      locations.get( "projectionMatrix" ).unwrap().clone(),
      projection_matrix.to_array().as_slice(),
      true
    ).unwrap();
  }

  pub fn set_window_size( &mut self, window_size : gl::F32x2 )
  {
    self.controls.borrow_mut().set_size( window_size.to_array() );
  }

  pub fn get_controls( &self ) -> Rc< RefCell< CameraOrbitControls > >
  {
    self.controls.clone()
  }

  pub fn get_eye( &self ) -> gl::F32x3
  {
    self.controls.borrow().eye
  }

  pub fn get_view_matrix( &self ) -> gl::F32x4x4
  {
    self.controls.borrow().view()
  }

  pub fn get_projection_matrix( &self ) -> gl::F32x4x4
  {
    self.projection_matrix
  }
}