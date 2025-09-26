//! Polygonal light demo

mod precompute;

use minwebgl as gl;
use gl::{ math::mat3x3h, AsBytes as _, JsCast as _, GL };
use renderer::webgl::loaders::gltf;
use web_sys::{ HtmlCanvasElement, WebGlTexture };

fn main()
{
  gl::browser::setup( Default::default() );
  gl::spawn_local( async { gl::info!( "{:?}", run().await ) } );
}

async fn run() -> Result< (), gl::WebglError >
{
  let window = web_sys::window().unwrap();
  let document = window.document().unwrap();

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
  gl.enable( GL::DEPTH_TEST );
  gl.clear_color( 0.0, 0.0, 0.0, 1.0 );
  gl.viewport( 0, 0, width, height );
  gl.get_extension( "EXT_color_buffer_float" ).unwrap().unwrap();

  let ltc1 = load_table( &gl, precompute::LTC1 );
  gl.active_texture( GL::TEXTURE2 );
  gl.bind_texture( GL::TEXTURE_2D, ltc1.as_ref() );

  let ltc2 = load_table( &gl, precompute::LTC2 );
  gl.active_texture( GL::TEXTURE3 );
  gl.bind_texture( GL::TEXTURE_2D, ltc2.as_ref() );

  let vertex_src = include_str!( "../shaders/main.vert" );
  let fragment_src = include_str!( "../shaders/area_light.frag" );
  let program = gl::Program::new( gl.clone(), vertex_src, fragment_src )?;
  program.activate();

  program.uniform_upload( "u_base_color", &0 );
  program.uniform_upload( "u_metallic_roughness", &1 );
  program.uniform_upload( "u_LTC1", &2 );
  program.uniform_upload( "u_LTC2", &3 );

  let skull = gltf::load( &document, "skull_salazar_downloadable.glb", &gl ).await?;

  let model = mat3x3h::scale( [1.0, 1.0, 1.0 ] );
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

  let light_points =
  [
    [ -8.0, 2.4, -1.0 ],
	  [ -8.0, 2.4,  1.0 ],
    [ -8.0, 0.4,  1.0 ],
	  [ -8.0, 0.4, -1.0 ],
  ];
  let light_position = [ 0.0, 0.0, 0.0 ];
  let light_intensity = 7.0;
  let light_color = [ 1.0, 1.0, 1.0 ];
  let two_sided = false;

  program.uniform_upload( "u_points", light_points.as_slice() );
  program.uniform_upload( "u_light_intensity", &light_intensity );
  program.uniform_upload( "u_light_color", &light_color );
  program.uniform_upload( "u_two_sided", &( two_sided as i32 ) );
  program.uniform_upload( "u_light_position", &light_position );

  let update = move | _time |
  {
    gl.clear( gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT );
    for mesh in &skull.meshes
    {
      for primitive in &mesh.borrow().primitives
      {
        let primitive = primitive.borrow();
        let material = primitive.material.borrow();

        let base_color = material.base_color_texture.as_ref().unwrap();
        let metallic_roughness = material.metallic_roughness_texture.as_ref().unwrap();
        gl.active_texture( GL::TEXTURE0 );
        base_color.bind( &gl );
        gl.active_texture( GL::TEXTURE1 );
        metallic_roughness.bind( &gl );

        let view = camera.get_view_matrix();
        let projection = camera.get_projection_matrix();

        let mvp = projection * view * model;
        program.uniform_matrix_upload( "u_model", model.raw_slice(), true );
        program.uniform_matrix_upload( "u_rotation", model.raw_slice(), true );
        program.uniform_matrix_upload( "u_mvp", mvp.raw_slice(), true );
        program.uniform_upload( "u_view_position", camera.get_eye().as_slice() );

        primitive.geometry.borrow().bind( &gl );
        primitive.draw( &gl );
      }
    }

    true
  };

  gl::exec_loop::run( update );

  Ok( () )
}

fn load_table( gl : &GL, table : &[ f32 ] ) -> Option< WebGlTexture >
{
  let texture = gl.create_texture();
  gl.bind_texture( GL::TEXTURE_2D, texture.as_ref() );
  gl.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_u8_array
  (
    GL::TEXTURE_2D,
    0,
    GL::RGBA as i32,
    64,
    64,
    0,
    GL::RGBA,
    GL::FLOAT,
    Some( table.as_bytes() )
  ).expect( "Failed to load data" );
  gl::texture::d2::filter_linear( gl );
  gl::texture::d2::wrap_clamp( gl );

  texture
}
