//! Polygonal light demo

mod precompute;
mod plane;
mod lil_gui;

use minwebgl as gl;
use gl::{ math::mat3x3h, JsCast as _, GL, IntoArray as _ };
use renderer::webgl::loaders::gltf;
use web_sys::{ js_sys::Float32Array, HtmlCanvasElement, WebGlTexture };

fn main()
{
  gl::browser::setup( Default::default() );
  gl::spawn_local( async { gl::info!( "{:?}", run().await ) } );
}

async fn run() -> Result< (), gl::WebglError >
{
  let window = web_sys::window().unwrap();
  let document = window.document().unwrap();

  let gui = lil_gui::new_gui();

  let fwidth = window.inner_width().unwrap().as_f64().unwrap();
  let fheight = window.inner_height().unwrap().as_f64().unwrap();
  let dpr = window.device_pixel_ratio();
  let width = ( fwidth * dpr ) as i32;
  let height = ( fheight * dpr ) as i32;
  let aspect = width as f32 / height as f32;

  let gl = gl::context::retrieve_or_make().expect( "Failed to retrieve WebGl context" );
  let canvas = gl.canvas().unwrap().dyn_into::< HtmlCanvasElement >().unwrap();
  canvas.set_width( width as u32 );
  canvas.set_height( height as u32 );
  gl.enable( gl::DEPTH_TEST );
  gl.enable( gl::CULL_FACE );
  gl.clear_color( 0.0, 0.0, 0.0, 1.0 );
  gl.viewport( 0, 0, width, height );
  gl.get_extension( "EXT_color_buffer_float" ).unwrap().unwrap();
  gl.get_extension( "OES_texture_float_linear" ).unwrap().unwrap();

  let vertex_src = include_str!( "../shaders/light_body.vert" );
  let fragment_src = include_str!( "../shaders/light_body.frag" );
  let light_body_shader = gl::Program::new( gl.clone(), vertex_src, fragment_src )?;

  let vertex_src = include_str!( "../shaders/main.vert" );
  let fragment_src = include_str!( "../shaders/area_light.frag" );
  let area_light_shader = gl::Program::new( gl.clone(), vertex_src, fragment_src )?;
  area_light_shader.activate();
  area_light_shader.uniform_upload( "u_base_color", &0 );
  area_light_shader.uniform_upload( "u_arm", &1 );
  area_light_shader.uniform_upload( "u_LTC1", &2 );
  area_light_shader.uniform_upload( "u_LTC2", &3 );

  let light_transform = mat3x3h::translation( [ 0.0, 1.0, 5.0 ] )
  * mat3x3h::rot( 0.0_f32.to_radians(), 0.0, 0.0 )
  * mat3x3h::scale( [ 4.0, 1.0, 0.0 ] );
  let mut light = RectangularLight
  {
    vertices :
    [
      [ -1.0,  1.0, 0.0 ].into(),
      [  1.0,  1.0, 0.0 ].into(),
      [ -1.0, -1.0, 0.0 ].into(),
      [  1.0, -1.0, 0.0 ].into(),
      // [ -1.0,  1.0, 4.0 ].into(),
      // [  1.0,  1.0, 4.0 ].into(),
      // [  1.0, -1.0, 4.0 ].into(),
      // [ -1.0, -1.0, 4.0 ].into(),
    ],
    color : [ 1.0, 0.95, 0.9 ],
    intensity : 20.0,
    two_sided : true,
  };
  light.apply_transform( &light_transform );
  // gl::info!
  // (
  //   "{:#?}",
  //   ( light.vertices[ 1 ] - light.vertices[ 0 ] )
  //   .cross( light.vertices[ 2 ] - light.vertices[ 0 ] )
  //   .normalize()
  // );

  let light_body_mesh = light_body_vao( &gl, &light )?;
  let plane_mesh = plane::plane_vao( &gl )?;
  let ( plane_base_color, plane_arm ) = plane::plane_material( &gl, [ 55, 57, 65, 255 ], 1.0, 0.3, 0.3 );
  let plane_model = mat3x3h::translation( [ 0.0, -1.0, 0.0 ] ) * mat3x3h::scale( [ 10.0, 1.0, 10.0 ] );

  let ltc1 = load_table( &gl, &precompute::LTC1 );
  let ltc2 = load_table( &gl, &precompute::LTC2 );

  let skull_mesh = gltf::load( &document, "gltf/skull_salazar_downloadable.glb", &gl ).await?;
  let skull_model = mat3x3h::scale( [ 1.0, 1.0, 1.0 ] );

  let mut camera = renderer::webgl::Camera::new
  (
    [ 0.0, 0.0, 10.0 ].into(),
    [ 0.0, 1.0, 0.0 ].into(),
    [ 0.0, 0.0, 0.0 ].into(),
    aspect,
    45.0_f32.to_radians(),
    0.1,
    100.0
  );
  camera.set_window_size( [ width as f32, height as f32 ].into() );
  camera.bind_controls( &canvas );

  let update = move | _time |
  {
    let view = camera.get_view_matrix();
    let projection = camera.get_projection_matrix();
    let view_projection = projection * view;
    let skull_mvp = view_projection * skull_model;
    let plane_mvp = view_projection * plane_model;

    gl.clear( gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT );

    gl.active_texture( gl::TEXTURE2 );
    gl.bind_texture( gl::TEXTURE_2D, ltc1.as_ref() );
    gl.active_texture( gl::TEXTURE3 );
    gl.bind_texture( gl::TEXTURE_2D, ltc2.as_ref() );

    area_light_shader.activate();
    area_light_shader.uniform_upload( "u_points", light.vertices().as_slice() );
    area_light_shader.uniform_upload( "u_light_intensity", &light.intensity );
    area_light_shader.uniform_upload( "u_light_color", &light.color );
    area_light_shader.uniform_upload( "u_two_sided", &( light.two_sided as u32 ) );
    area_light_shader.uniform_upload( "u_view_position", camera.get_eye().as_slice() );

    gl.enable( gl::CULL_FACE );
    area_light_shader.uniform_matrix_upload( "u_model", skull_model.raw_slice(), true );
    // object_shader.uniform_matrix_upload( "u_rotation", model.raw_slice(), true );
    area_light_shader.uniform_matrix_upload( "u_mvp", skull_mvp.raw_slice(), true );
    for mesh in &skull_mesh.meshes
    {
      for primitive in &mesh.borrow().primitives
      {
        let primitive = primitive.borrow();
        let material = primitive.material.borrow();

        let base_color = material.base_color_texture.as_ref().unwrap();
        let metallic_roughness = material.metallic_roughness_texture.as_ref().unwrap();
        gl.active_texture( gl::TEXTURE0 );
        base_color.bind( &gl );
        gl.active_texture( gl::TEXTURE1 );
        metallic_roughness.bind( &gl );
        primitive.geometry.borrow().bind( &gl );
        primitive.draw( &gl );
      }
    }
    gl.disable( gl::CULL_FACE );

    area_light_shader.uniform_matrix_upload( "u_model", plane_model.raw_slice(), true );
    // object_shader.uniform_matrix_upload( "u_rotation", model.raw_slice(), true );
    area_light_shader.uniform_matrix_upload( "u_mvp", plane_mvp.raw_slice(), true );
    gl.active_texture( gl::TEXTURE0 );
    gl.bind_texture( GL::TEXTURE_2D, plane_base_color.as_ref() );
    gl.active_texture( gl::TEXTURE1 );
    gl.bind_texture( GL::TEXTURE_2D, plane_arm.as_ref() );
    gl.bind_vertex_array( Some( &plane_mesh ) );
    gl.draw_arrays( gl::TRIANGLE_STRIP, 0, 4 );

    light_body_shader.activate();
    light_body_shader.uniform_matrix_upload( "u_view_projection", view_projection.raw_slice(), true );
    light_body_shader.uniform_upload( "u_color", &light.color );
    gl.bind_vertex_array( Some( &light_body_mesh ) );
    gl.draw_arrays( gl::TRIANGLE_STRIP, 0, 4 );

    true
  };

  gl::exec_loop::run( update );

  Ok( () )
}

fn light_body_vao( gl : &GL, light : &RectangularLight ) -> Result< web_sys::WebGlVertexArrayObject, gl::WebglError >
{
  let light_body_vao = gl::vao::create( gl )?;
  gl.bind_vertex_array( Some( &light_body_vao ) );
  let vbo = gl::buffer::create( gl )?;
  gl::buffer::upload( gl, &vbo, light.vertices().as_flattened(), gl::DYNAMIC_DRAW );
  gl::BufferDescriptor::new::< [ f32; 3 ] >().attribute_pointer( gl, 0, &vbo )?;
  Ok( light_body_vao )
}

fn load_table( gl : &GL, table : &[ f32 ] ) -> Option< WebGlTexture >
{
  let array = Float32Array::new_from_slice( table );
  let texture = gl.create_texture();
  gl.bind_texture( gl::TEXTURE_2D, texture.as_ref() );
  gl.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_array_buffer_view_and_src_offset
  (
    gl::TEXTURE_2D,
    0,
    gl::RGBA32F as i32,
    64,
    64,
    0,
    gl::RGBA,
    gl::FLOAT,
    &array,
    0
  ).expect( "Failed to load data" );
  gl::texture::d2::filter_linear( gl );
  gl::texture::d2::wrap_clamp( gl );

  texture
}

struct RectangularLight
{
  vertices : [ gl::F32x3; 4 ],
  intensity : f32,
  color : [ f32; 3 ],
  two_sided : bool,
}

impl RectangularLight
{
  fn apply_transform( &mut self, t : &gl::F32x4x4 )
  {
    self.vertices.iter_mut().for_each
    (
      | v |
      {
        let v4 = gl::F32x4::new( v.x(), v.y(), v.z(), 1.0 );
        let v4 = *t * v4;
        *v = gl::F32x3::new( v4.x(), v4.y(), v4.z() );
      }
    );
  }

  fn vertices( &self ) -> [ [ f32; 3 ]; 4 ]
  {
    self.vertices
    .into_iter()
    .map( | v | v.into_array() )
    .collect::< Vec< _ > >()
    .try_into()
    .unwrap()
  }
}

struct GuiParams
{
  rot_x : f32,
  rot_y : f32,
  rot_z : f32,
  color : [ f32; 3 ],
  intensity : f32,
  two_sided : bool,
}
