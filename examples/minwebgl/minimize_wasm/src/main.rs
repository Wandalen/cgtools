//! # Minimize wasm example
//!
//! This program demonstrates how to render a triangle in the middle of the screen using WebGL in Rust. It utilizes shaders with Uniform Block Objects (UBOs) to manage uniforms efficiently.

use gl::GL;
use minwebgl as gl;
use cgmath::{EuclideanSpace, Rotation3};

static POSITION_DATA: [f32; 6] = [
  -0.86602545, // Left
  -0.5,
  0.0, // Top
  1.0,
  0.86602545, // Right
  -0.5,
];

static COLOR_DATA: [f32; 9] = [
  1.0, 0.0, 0.0, // Red
  0.0, 1.0, 0.0, // Green
  0.0, 0.0, 1.0, // Blue
];

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc<'_> = wee_alloc::WeeAlloc::INIT;

fn run() -> Result<(), gl::WebglError> {
  gl::browser::setup(Default::default());
  let gl = gl::context::retrieve_or_make()?;

  // Vertex and fragment shader source code
  let vertex_shader_src = include_str!("../shaders/shader.vert");
  let fragment_shader_src = include_str!("../shaders/shader.frag");
  let program = gl::ProgramFromSources::new(vertex_shader_src, fragment_shader_src).compile_and_link(&gl)?;
  gl.use_program(Some(&program));

  let position_slot = 0;
  let position_buffer = gl::buffer::create(&gl)?;
  gl::buffer::upload(&gl, &position_buffer, &POSITION_DATA, gl::GL::STATIC_DRAW);

  let color_slot = 1;
  let color_buffer = gl::buffer::create(&gl)?;
  gl::buffer::upload(&gl, &color_buffer, &COLOR_DATA, gl::GL::STATIC_DRAW);

  // Create vao.
  // And set attributes.
  // A divisor of 0 indicates that each vertex has its own unique attribute value.
  // A divisor of 1 means that the entire primitive shares the same attribute value.
  // A divisor of 2 or more specifies that the attribute value is shared across multiple primitives.

  let vao = gl::vao::create(&gl)?;
  gl.bind_vertex_array(Some(&vao));
  gl::BufferDescriptor::new::<[f32; 2]>()
    .stride(2)
    .offset(0)
    .divisor(0)
    .attribute_pointer(&gl, position_slot, &position_buffer)?;
  gl::BufferDescriptor::new::<[f32; 3]>()
    .stride(3)
    .offset(0)
    .divisor(0)
    .attribute_pointer(&gl, color_slot, &color_buffer)?;

  let projective_view_location = gl.get_uniform_location(&program, "project_view_matrix");

  let width = gl.drawing_buffer_width() as f32;
  let height = gl.drawing_buffer_height() as f32;

  // Camera setup
  let eye = cgmath::Vector3::new(0.0, 0.0, 1.5);
  let up = cgmath::Vector3::<f32>::unit_y();

  let scale = 1.0;
  let aspect = width / height;
  let perspective = cgmath::PerspectiveFov {
    fovy: cgmath::Deg(70.0).into(),
    aspect,
    near: 0.1,
    far: 1000.0,
  };

  let model_trans = cgmath::Decomposed {
    scale,
    rot: cgmath::Basis3::from_angle_y::<cgmath::Rad<f32>>(cgmath::Deg(180.0).into()),
    disp: cgmath::Vector3::new(0.0, 0.0, 0.0),
  };

  let model_matrix = cgmath::Matrix4::from(model_trans);
  let projection_matrix = cgmath::Matrix4::from(perspective);

  // Define the update and draw logic
  let update_and_draw = {
    move |t: f64| {
      gl.clear_color(0.0, 0.0, 0.0, 1.0);
      gl.clear(gl::COLOR_BUFFER_BIT);

      let time = t as f32 / 1000.0;
      let rotation = cgmath::Matrix3::from_angle_z(cgmath::Rad(time));
      let up = rotation * up;

      let view_matrix = cgmath::Matrix4::look_at_rh(cgmath::Point3::from_vec(eye), cgmath::Point3::origin(), up);

      let projective_view_matrix = projection_matrix * view_matrix * model_matrix;
      let projective_view_matrix = &<cgmath::Matrix4<f32> as AsRef<[f32; 16]>>::as_ref(&projective_view_matrix)[..];

      gl::uniform::matrix_upload(&gl, projective_view_location.clone(), projective_view_matrix.as_ref(), true).unwrap();

      gl.draw_arrays(GL::TRIANGLES, 0, 3);
      true
    }
  };

  // Run the render loop
  gl::exec_loop::run(update_and_draw);
  Ok(())
}

fn main() {
  run().unwrap()
}
