//! # Uniforms And Animation Example with UBOs
//!
//! This program demonstrates how to render a triangle in the middle of the screen using WebGL in Rust. It utilizes shaders with Uniform Block Objects (UBOs) to manage uniforms efficiently.

use minwebgl::{ self as gl, wasm_bindgen::prelude::Closure, JsCast };

mod text;
mod json;

fn run() -> Result< (), gl::WebglError >
{
  gl::browser::setup( Default::default() );

  let canvas = gl::canvas::retrieve_or_make()?;
  let gl = gl::context::from_canvas( &canvas )?;

  // Vertex and fragment shader source code
  let vertex_shader_src = include_str!( "../shaders/shader.vert" );
  let fragment_shader_src = include_str!( "../shaders/shader.frag" );
  let program = gl::ProgramFromSources::new( vertex_shader_src, fragment_shader_src ).compile_and_link( &gl )?;
  gl.use_program( Some( &program ) );

  let width = canvas.width() as f32;
  let height = canvas.height() as f32;

  let text = "Cgtools";
  let font_str = include_str!( "../assets/font/Alike-Regular.json" );
  // Parse font from the provided file
  let font = json::MSDFFontJSON::parse_font( font_str );
  // Create render data from the text based on the font
  let fortmatted_text = font.format( text );
  let buffer = gl::buffer::create( &gl )?;

  let view_matrix_location = gl.get_uniform_location( &program, "viewMatrix" );
  let projection_matrix_location = gl.get_uniform_location( &program, "projectionMatrix" );
  let tex_size_location = gl.get_uniform_location( &program, "texSize" );
  let time_location = gl.get_uniform_location( &program, "time" );
  let bounding_box_location = gl.get_uniform_location( &program, "boundingBox" );

  gl::buffer::upload( &gl, &buffer, &fortmatted_text.chars, gl::STATIC_DRAW );

  let vao = gl::vao::create( &gl )?;
  gl.bind_vertex_array( Some( &vao ) );

  let char_data_stride  = std::mem::size_of::< text::CharData >() as i32 / 4;
  // offset
  gl::BufferDescriptor::new::< [ f32 ; 4 ] >()
  .stride( char_data_stride )
  .offset( 0 )
  .divisor( 1 )
  .attribute_pointer( &gl, 0, &buffer )?;

  // uv_info
  gl::BufferDescriptor::new::< [ f32 ; 4 ] >()
  .stride( char_data_stride )
  .offset( 4 )
  .divisor( 1 )
  .attribute_pointer( &gl, 1, &buffer )?;

  // size
  gl::BufferDescriptor::new::< [ f32 ; 2 ] >()
  .stride( char_data_stride )
  .offset( 8 )
  .divisor( 1 )
  .attribute_pointer( &gl, 2, &buffer )?;


  let eye = gl::F32x3::Z * 5.0;
  let up = gl::F32x3::Y;
  let dir = gl::F32x3::NEG_Z;

  let fov = 70.0f32.to_radians();
  let aspect = width / height;
  let near = 0.1;
  let far = 1000.0;

  let projection_matrix = gl::math::mat3x3h::perspective_rh_gl( fov, aspect, near, far );

  gl::uniform::matrix_upload
  (
    &gl,
    projection_matrix_location,
    &projection_matrix.to_array()[ .. ],
    true
  )?;

  gl::uniform::upload( &gl, tex_size_location, &font.scale[ .. ] )?;
  gl::uniform::upload( &gl, bounding_box_location, &fortmatted_text.bounding_box.to_array()[ .. ] )?;

  // Load an image and upload it to the texture when it's loaded
  let img = gl::dom::create_image_element( "static/font/Alike-Regular.png" ).unwrap();
  img.style().set_property( "display", "none" ).unwrap();

  let texture = gl.create_texture();
  let load_texture : Closure< dyn Fn() > = Closure::new
  (
    {
      let texture = texture.clone();
      let gl = gl.clone();
      let img = img.clone();
      move ||
      {
        gl::texture::d2::upload_no_flip( &gl, texture.as_ref(), &img );
        gl::texture::d2::default_parameters( &gl );
        img.remove();
      }
    }
  );

  img.set_onload( Some( load_texture.as_ref().unchecked_ref() ) );
  load_texture.forget();

  gl.enable( gl::DEPTH_TEST );
  gl.enable( gl::BLEND );

  gl.blend_func( gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA );
  gl.clear_color( 0.0, 0.0, 0.0, 1.0 );
  gl.clear_depth( 1.0 );

  // Define the update and draw logic
  let update_and_draw =
  {
    let instances = fortmatted_text.chars.len() as i32;
    move | t : f64 |
    {
      let view_matrix = gl::math::mat3x3h::look_to_rh( eye, dir, up );

      gl::uniform::matrix_upload
      (
        &gl,
        view_matrix_location.clone(),
        &view_matrix.to_array()[ .. ],
        true
      ).unwrap();

      gl.uniform1f( time_location.as_ref(), ( t / 1000.0 ) as f32 );

      gl.clear( gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT );
      gl.draw_arrays_instanced( gl::TRIANGLE_STRIP, 0, 4, instances );
      true
    }
  };

  // Run the render loop
  gl::exec_loop::run( update_and_draw );
  Ok( () )
}

fn main()
{
  run().unwrap()
}
