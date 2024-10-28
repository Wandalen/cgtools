//! Just draw a large point in the middle of the screen.

use minwebgl as gl;
use gl::GL;

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

// The order I took from here
// https://github.com/mrdoob/three.js/blob/master/src/cameras/CubeCamera.js
fn make_cube_camera() -> [ glam::Mat4; 6 ]
{
  let px = glam::Mat4::look_at_rh( glam::Vec3::ZERO, glam::Vec3::NEG_X, glam::Vec3::NEG_Y );
  let nx = glam::Mat4::look_at_rh( glam::Vec3::ZERO, glam::Vec3::X, glam::Vec3::NEG_Y );

  let py = glam::Mat4::look_at_rh( glam::Vec3::ZERO, glam::Vec3::Y, glam::Vec3::Z );
  let ny = glam::Mat4::look_at_rh( glam::Vec3::ZERO, glam::Vec3::NEG_Y, glam::Vec3::NEG_Z );

  let pz = glam::Mat4::look_at_rh( glam::Vec3::ZERO, glam::Vec3::Z, glam::Vec3::NEG_Y );
  let nz = glam::Mat4::look_at_rh( glam::Vec3::ZERO, glam::Vec3::NEG_Z, glam::Vec3::NEG_Y );

  [ px, nx, py, ny, pz, nz ]
}


fn gen_cube_texture( gl : &GL, width: i32, height: i32 ) -> Option< gl::web_sys::WebGlTexture >
{
  let texture = gl.create_texture();
  gl.bind_texture( gl::TEXTURE_CUBE_MAP, texture.as_ref() );

  for i in 0..6 
  {
    gl. tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_u8_array
    (
      gl::TEXTURE_CUBE_MAP_POSITIVE_X + i as u32,
      0,
      gl::RGBA as i32,
      width as i32,
      height as i32,
      0,
      gl::RGBA,
      gl::UNSIGNED_BYTE,
      None
    ).expect( "Failed to upload data to texture" );
  }

  gl.tex_parameteri( gl::TEXTURE_CUBE_MAP, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32 );
  gl.tex_parameteri( gl::TEXTURE_CUBE_MAP, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32 );
  gl.tex_parameteri( gl::TEXTURE_CUBE_MAP, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32 );
  gl.tex_parameteri( gl::TEXTURE_CUBE_MAP, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32 );
  gl.tex_parameteri( gl::TEXTURE_CUBE_MAP, gl::TEXTURE_WRAP_R, gl::CLAMP_TO_EDGE as i32 );

  texture
}


async fn run() -> Result< (), gl::WebglError >
{
  gl::browser::setup( Default::default() );
  let gl = gl::context::retrieve_or_make()?;

  // Vertex and fragment shaders
  let vertex_shader_src = include_str!( "../shaders/gen_cube_map.vert" );
  let fragment_shader_src = include_str!( "../shaders/gen_cube_map.frag" );
  let program = gl::ProgramFromSources::new( vertex_shader_src, fragment_shader_src ).compile_and_link( &gl )?;
  gl.use_program( Some( &program ) );

  // Load model
  let obj_buffer = gl::file::load( "diamond.glb" ).await.expect( "Failed to load the model" );
  let ( document, buffers, _ ) = gltf::import_slice( &obj_buffer[ .. ] ).expect( "Failed to parse the glb file" );

  let positions : Vec< [ f32; 3 ] >;
  let normals : Vec< [ f32; 3 ] >;
  let indices : Vec< u32 >;
  let max_distance : f32;

  {
    let mesh = document.meshes().next().expect( "No meshes were found" );
    let primitive = mesh.primitives().next().expect( "No primitives were found" );
    let reader = primitive.reader( | buffer | Some( &buffers[ buffer.index() ] ) );

    let pos_iter = reader.read_positions().expect( "Failed to read positions" );
    
    max_distance = pos_iter
    .clone()
    .map( | p | glam::Vec3::new( p[ 0 ], p[ 1 ], p[ 2 ] ).length() )
    .reduce( f32::max )
    .unwrap();

    positions = pos_iter.collect();

    let normals_iter = reader.read_normals().expect( "Failed to read normals" );
    normals = normals_iter.collect();

    let index_iter = reader.read_indices().expect( "Failed to read indices" );
    indices = index_iter.into_u32().collect();
  }

  let pos_buffer =  gl::buffer::create( &gl )?;
  let normal_buffer = gl::buffer::create( &gl )?;

  gl::buffer::upload( &gl, &pos_buffer, &positions, GL::STATIC_DRAW );
  gl::buffer::upload( &gl, &normal_buffer, &normals, GL::STATIC_DRAW );

  let vao = gl::vao::create( &gl )?;
  gl.bind_vertex_array( Some( &vao ) );
  gl::BufferDescriptor::new::< [ f32; 3 ] >().stride( 3 ).offset( 0 ).attribute_pointer( &gl, 0, &pos_buffer )?;
  gl::BufferDescriptor::new::< [ f32; 3 ] >().stride( 3 ).offset( 0 ).attribute_pointer( &gl, 1, &normal_buffer )?;

  let index_buffer = gl::buffer::create( &gl )?;
  gl::index::upload( &gl, &index_buffer, &indices, GL::STATIC_DRAW );

  // Get variables locations
  let projection_matrix_location = gl.get_uniform_location( &program, "projectionMatrix" );
  let model_matrix_location = gl.get_uniform_location( &program, "modelMatrix" );
  let view_matrix_location = gl.get_uniform_location( &program, "viewMatrix" );
  let normal_matrix_location = gl.get_uniform_location( &program, "normalMatrix" );

  let max_distance_location = gl.get_uniform_location( &program, "max_distance" );


  // Camera setup
  let cube_camera = make_cube_camera();

  let perspective_matrix = glam::Mat4::perspective_rh_gl
  (
     90.0f32.to_radians(),  
     1.0, 
     0.1, 
     10.0
  );

  let model_matrix = glam::Mat4::from_scale_rotation_translation
  (
    glam::Vec3::ONE, 
    glam::Quat::from_rotation_y( 0.0 ), 
    glam::Vec3::ZERO
  );

  // You need this in case you want to deform your model is some ways
  let normal_matrix = glam::Mat3::from_mat4( model_matrix.inverse().transpose() );

  // Update uniform values
  gl::uniform::matrix_upload( &gl, projection_matrix_location, &perspective_matrix.to_cols_array()[ .. ], true )?;
  gl::uniform::matrix_upload( &gl, model_matrix_location, &model_matrix.to_cols_array()[ .. ], true )?;
  gl::uniform::matrix_upload( &gl, normal_matrix_location, &normal_matrix.to_cols_array()[ .. ], true )?;

  gl::uniform::upload( &gl, max_distance_location, &max_distance )?;

  let ( tex_width, tex_height ) = ( 512, 512 );
  let cube_texture = gen_cube_texture( &gl, tex_width, tex_height );

  // Render to our cube texture using custom frame buffer
  // All the needed buffers were setup above
  let frame_buffer = gl.create_framebuffer();
  gl.bind_framebuffer( gl::FRAMEBUFFER , frame_buffer.as_ref() );
  gl.viewport( 0, 0, tex_width, tex_height);
  gl.clear_color( 0.0, 0.0, 0.0, 1.0);
  for i in 0..6
  {
    let view_matrix = &cube_camera[ i ].to_cols_array()[ .. ];
    gl::uniform::matrix_upload( &gl, view_matrix_location.clone(), view_matrix, true )?;
    gl.framebuffer_texture_2d
    ( 
      gl::FRAMEBUFFER, 
      gl::COLOR_ATTACHMENT0, 
      gl::TEXTURE_CUBE_MAP_POSITIVE_X + i as u32, 
      cube_texture.as_ref(), 
      0
    );
    gl.clear( gl::COLOR_BUFFER_BIT );
    gl.draw_elements_with_i32( gl::TRIANGLES, indices.len() as i32, gl::UNSIGNED_INT, 0 );
  }

  gl.delete_buffer( Some( &pos_buffer ) );
  gl.delete_buffer( Some( &normal_buffer ) );
  gl.delete_framebuffer( frame_buffer.as_ref() );

  //
  // Create new program to display the result
  //
  let vertex_shader_src = include_str!( "../shaders/shader.vert" );
  let fragment_shader_src = include_str!( "../shaders/shader.frag" );
  let program = gl::ProgramFromSources::new( vertex_shader_src, fragment_shader_src ).compile_and_link( &gl )?;
  gl.use_program( Some( &program ) );

  let projection_matrix_location = gl.get_uniform_location( &program, "projectionMatrix" );
  let model_matrix_location = gl.get_uniform_location( &program, "modelMatrix" );
  let view_matrix_location = gl.get_uniform_location( &program, "viewMatrix" );

  let max_distance_location = gl.get_uniform_location( &program, "max_distance" );

  // Prepare attributes
  // We don't really need uvs, but they came with a model, so I decided to leave them be
  let cube_attr = get_cube_data();
  let vertex_count = cube_attr.len() / 5;
  let cube_attr_buffer = gl::buffer::create( &gl )?;
  gl::buffer::upload( &gl, &cube_attr_buffer, cube_attr, gl::STATIC_DRAW );

  let vao = gl::vao::create( &gl )?;
  gl.bind_vertex_array( Some( &vao ) );
  gl::BufferDescriptor::new::< [ f32; 3 ] >().offset( 0 ).stride( 5 ).attribute_pointer( &gl, 0, &cube_attr_buffer )?;
  gl::BufferDescriptor::new::< [ f32; 2 ] >().offset( 3 ).stride( 5 ).attribute_pointer( &gl, 1, &cube_attr_buffer )?;

  let width = gl.drawing_buffer_width() as f32;
  let height = gl.drawing_buffer_height() as f32;

  let eye = glam::Vec3::new( 0.0, 0.0, 3.0 );
  let up = glam::Vec3::Y;

  let perspective_matrix = glam::Mat4::perspective_rh_gl
  (
     70.0f32.to_radians(),  
     width / height, 
     0.1, 
     1000.0
  );

  let model_matrix = glam::Mat4::from_scale_rotation_translation
  (
    glam::Vec3::ONE * 2.0, 
    glam::Quat::from_rotation_y( 0.0 ), 
    glam::Vec3::ZERO
  );

  gl::uniform::matrix_upload( &gl, projection_matrix_location, &perspective_matrix.to_cols_array()[ .. ], true )?;
  gl::uniform::matrix_upload( &gl, model_matrix_location, &model_matrix.to_cols_array()[ .. ], true )?;

  gl::uniform::upload( &gl, max_distance_location, &max_distance )?;

  gl.viewport( 0, 0, width as i32, height as i32);

  gl.bind_framebuffer( gl::FRAMEBUFFER, None );
  gl.active_texture( gl::TEXTURE0 );
  gl.bind_texture( gl::TEXTURE_CUBE_MAP, cube_texture.as_ref() );

  gl.enable( gl::DEPTH_TEST );
  gl.enable( gl::BLEND );
  // Blending will look weird, becase you need to sort vertices from
  // front to back to blend them properly
  gl.blend_func( gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA );
  gl.clear_color( 0.0, 0.0, 0.0, 1.0 );

  // Define the update and draw logic
  let update_and_draw =
  {
    move | t : f64 |
    {
      let time = t as f32 / 1000.0;
      let rotatio_y = glam::Mat3::from_rotation_y( time );
      let rotatio_z = glam::Mat3::from_rotation_z( time + 5.0 );
      let rotatio_z2 = glam::Mat3::from_rotation_z( time - 5.0 );
      let eye = rotatio_z2 * rotatio_z * rotatio_y * eye;

      let view_matrix = glam::Mat4::look_at_rh( eye, glam::Vec3::ZERO, up );

      gl::uniform::matrix_upload( &gl, view_matrix_location.clone(), &view_matrix.to_cols_array()[ .. ], true ).unwrap();

      gl.clear( gl::COLOR_BUFFER_BIT );
      // Draw cube
      gl.draw_arrays( gl::TRIANGLES, 0, vertex_count as i32);
      true
    }
  };

  // Run the render loop
  gl::exec_loop::run( update_and_draw );

  Ok( () )
}

fn main()
{
  gl::spawn_local( async move { run().await.unwrap() } );
}
