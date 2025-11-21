#![ allow( dead_code ) ]

use minwebgl as gl;
use gl::{ GL, web_sys::{ WebGlProgram, WebGlTexture } };
use renderer::webgl::Camera;
use crate::cube_normal_map_generator::CubeNormalMapGenerator;
use crate::helpers;

fn get_cube_data() -> &'static [ f32 ]
{
  &[
  //  X     Y     Z     U    V
    -0.5, -0.5, -0.5,  0.0, 0.0,
    0.5, -0.5, -0.5,  1.0, 0.0,
    0.5,  0.5, -0.5,  1.0, 1.0,
    0.5,  0.5, -0.5,  1.0, 1.0,
    -0.5,  0.5, -0.5,  0.0, 1.0,
    -0.5, -0.5, -0.5,  0.0, 0.0,

    -0.5, -0.5,  0.5,  0.0, 0.0,
    0.5, -0.5,  0.5,  1.0, 0.0,
    0.5,  0.5,  0.5,  1.0, 1.0,
    0.5,  0.5,  0.5,  1.0, 1.0,
    -0.5,  0.5,  0.5,  0.0, 1.0,
    -0.5, -0.5,  0.5,  0.0, 0.0,

    -0.5,  0.5,  0.5,  1.0, 0.0,
    -0.5,  0.5, -0.5,  1.0, 1.0,
    -0.5, -0.5, -0.5,  0.0, 1.0,
    -0.5, -0.5, -0.5,  0.0, 1.0,
    -0.5, -0.5,  0.5,  0.0, 0.0,
    -0.5,  0.5,  0.5,  1.0, 0.0,

    0.5,  0.5,  0.5,  1.0, 0.0,
    0.5,  0.5, -0.5,  1.0, 1.0,
    0.5, -0.5, -0.5,  0.0, 1.0,
    0.5, -0.5, -0.5,  0.0, 1.0,
    0.5, -0.5,  0.5,  0.0, 0.0,
    0.5,  0.5,  0.5,  1.0, 0.0,

    -0.5, -0.5, -0.5,  0.0, 1.0,
    0.5, -0.5, -0.5,  1.0, 1.0,
    0.5, -0.5,  0.5,  1.0, 0.0,
    0.5, -0.5,  0.5,  1.0, 0.0,
    -0.5, -0.5,  0.5,  0.0, 0.0,
    -0.5, -0.5, -0.5,  0.0, 1.0,

    -0.5,  0.5, -0.5,  0.0, 1.0,
    0.5,  0.5, -0.5,  1.0, 1.0,
    0.5,  0.5,  0.5,  1.0, 0.0,
    0.5,  0.5,  0.5,  1.0, 0.0,
    -0.5,  0.5,  0.5,  0.0, 0.0,
    -0.5,  0.5, -0.5,  0.0, 1.0
  ]
}

fn prepare( gl : &GL, max_distance : f32, cube_texture : Option< WebGlTexture > ) -> Result< WebGlProgram, Box< dyn std::error::Error > >
{
  let vertex_shader_src = include_str!( "../shaders/shader.vert" );
  let fragment_shader_src = include_str!( "../shaders/shader.frag" );
  let program = gl::ProgramFromSources::new( vertex_shader_src, fragment_shader_src ).compile_and_link( &gl )?;
  gl.use_program( Some( &program ) );

  let projection_matrix_location = gl.get_uniform_location( &program, "projectionMatrix" );
  let model_matrix_location = gl.get_uniform_location( &program, "modelMatrix" );

  let max_distance_location = gl.get_uniform_location( &program, "max_distance" );

  // Prepare attributes
  // We don't really need uvs, but they came with a model, so I decided to leave them be
  let cube_attr = get_cube_data();
  let cube_attr_buffer = gl::buffer::create( &gl )?;
  gl::buffer::upload( &gl, &cube_attr_buffer, cube_attr, gl::STATIC_DRAW );

  let vao = gl::vao::create( &gl )?;
  gl.bind_vertex_array( Some( &vao ) );
  gl::BufferDescriptor::new::< [ f32; 3 ] >().offset( 0 ).stride( 5 ).attribute_pointer( &gl, 0, &cube_attr_buffer )?;
  gl::BufferDescriptor::new::< [ f32; 2 ] >().offset( 3 ).stride( 5 ).attribute_pointer( &gl, 1, &cube_attr_buffer )?;

  let width = gl.drawing_buffer_width() as f32;
  let height = gl.drawing_buffer_height() as f32;

  let perspective_matrix = gl::math::mat3x3h::perspective_rh_gl
  (
     70.0f32.to_radians(),
     width / height,
     0.1,
     1000.0
  );

  let model_matrix = gl::F32x4x4::from_scale_rotation_translation
  (
    gl::F32x3::splat( 1.0 ) * 2.0,
    gl::QuatF32::from_angle_y( 0.0 ),
    gl::F32x3::ZERO
  );

  gl::uniform::matrix_upload( &gl, projection_matrix_location, &perspective_matrix.to_array(), true )?;
  gl::uniform::matrix_upload( &gl, model_matrix_location, &model_matrix.to_array(), true )?;

  gl::uniform::upload( &gl, max_distance_location, &max_distance )?;

  gl.viewport( 0, 0, width as i32, height as i32);

  gl.bind_framebuffer( gl::FRAMEBUFFER, None );
  gl.active_texture( gl::TEXTURE0 );
  gl.bind_texture( gl::TEXTURE_CUBE_MAP, Some( &cube_texture.unwrap() ) );

  gl.enable( gl::DEPTH_TEST );
  gl.enable( gl::BLEND );
  // Blending will look weird, becase you need to sort vertices from
  // front to back to blend them properly
  gl.blend_func( gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA );
  gl.clear_color( 0.0, 0.0, 0.0, 1.0 );

  Ok( program )
}

fn setup_camera( canvas : &web_sys::HtmlCanvasElement ) -> Camera
{
  let width = canvas.width() as f32;
  let height = canvas.height() as f32;

  let eye = gl::math::F32x3::from( [ 0.0, 0.0, 1.0 ] );
  let up = gl::math::F32x3::from( [ 0.0, 1.0, 0.0 ] );

  let center = gl::math::F32x3::from( [ 0.0, 0.0, 0.0 ] );

  let aspect_ratio = width / height;
  let fov = 70.0f32.to_radians();
  let near = 0.1;
  let far = 1000.0;

  let mut camera = Camera::new( eye, up, center, aspect_ratio, fov, near, far );
  camera.set_window_size( [ width, height ].into() );
  camera.bind_controls( &canvas );

  camera
}

pub async fn debug_run() -> Result< (), gl::WebglError >
{
  gl::browser::setup( Default::default() );
  let options = gl::context::ContexOptions::default().antialias( false );

  let canvas = gl::canvas::make()?;
  let gl = gl::context::from_canvas_with( &canvas, options )?;

  let _ = gl.get_extension( "EXT_color_buffer_float" ).expect( "Failed to enable EXT_color_buffer_float extension" );
  let _ = gl.get_extension( "EXT_shader_image_load_store" ).expect( "Failed to enable EXT_shader_image_load_store  extension" );

  let window = gl::web_sys::window().unwrap();
  let document = window.document().unwrap();

  let model_id = 0;
  let gltf = renderer::webgl::loaders::gltf::load( &document, format!( "./gltf/{model_id}.glb" ).as_str(), &gl ).await?;
  let gem = helpers::get_node( &gltf.scenes[ 0 ], "Object_2".to_string() ).unwrap();

  let camera = setup_camera( &canvas );

  gem.borrow_mut().set_center_to_origin();

  let generator = CubeNormalMapGenerator::new( &gl ).unwrap();

  let texture = generator.generate( &gl, &gem );

  let bb = gem.borrow().bounding_box();
  let max_distance = bb.min.mag().max( bb.max.mag() );

  let cube_attr = get_cube_data();
  let vertex_count = cube_attr.len() / 5;
  let program = prepare( &gl, max_distance, texture.unwrap().texture.borrow().source.clone() ).unwrap();

  let view_matrix_location = gl.get_uniform_location( &program, "viewMatrix" );

  // Define the update and draw logic
  let update_and_draw =
  {
    move | _t : f64 |
    {
      let view_matrix = camera.get_view_matrix();

      gl::uniform::matrix_upload( &gl, view_matrix_location.clone(), &view_matrix.to_array(), true ).unwrap();

      gl.clear( gl::COLOR_BUFFER_BIT );
      gl.draw_arrays( gl::TRIANGLES, 0, vertex_count as i32 );

      true
    }
  };

  // Run the render loop
  gl::exec_loop::run( update_and_draw );

  Ok( () )
}
