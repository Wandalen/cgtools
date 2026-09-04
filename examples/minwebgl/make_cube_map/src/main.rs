//! Loads a diamond glTF model, renders it into a freshly generated 6-face cube map texture
//! (one framebuffer pass per face), then displays a textured cube that orbits while sampling
//! the generated cube map -- demonstrates dynamic cube map generation in WebGL2.

use minwebgl as gl;
use gl::GL;

/// One cube corner's position/uv record — `stride( 5 )` below covers all 5 `f32` fields,
/// matching this struct's own ( `repr( C )`, no padding ) byte layout.
#[ repr( C ) ]
#[ derive( Debug, Default, Clone, Copy, gl::mem::Pod, gl::mem::Zeroable ) ]
struct Vertex
{
  position : [ f32 ; 3 ],
  uv : [ f32 ; 2 ],
}

impl mingl::Attribute for Vertex
{
  fn describe() -> Vec< mingl::VertexAttribute >
  {
    vec!
    [
      mingl::VertexAttribute::new( 0, mingl::VectorDataType::new( mingl::DataType::F32, 3, 1 ), 0 ),
      mingl::VertexAttribute::new( 1, mingl::VectorDataType::new( mingl::DataType::F32, 2, 1 ), 3 ),
    ]
  }
}

fn cube_data_get() -> &'static [ Vertex ]
{
  &[
  //             position                uv
    Vertex { position : [ -0.5, -0.5, -0.5 ], uv : [ 0.0, 0.0 ] },
    Vertex { position : [ 0.5, -0.5, -0.5 ], uv : [ 1.0, 0.0 ] },
    Vertex { position : [ 0.5,  0.5, -0.5 ], uv : [ 1.0, 1.0 ] },
    Vertex { position : [ 0.5,  0.5, -0.5 ], uv : [ 1.0, 1.0 ] },
    Vertex { position : [ -0.5,  0.5, -0.5 ], uv : [ 0.0, 1.0 ] },
    Vertex { position : [ -0.5, -0.5, -0.5 ], uv : [ 0.0, 0.0 ] },

    Vertex { position : [ -0.5, -0.5,  0.5 ], uv : [ 0.0, 0.0 ] },
    Vertex { position : [ 0.5, -0.5,  0.5 ], uv : [ 1.0, 0.0 ] },
    Vertex { position : [ 0.5,  0.5,  0.5 ], uv : [ 1.0, 1.0 ] },
    Vertex { position : [ 0.5,  0.5,  0.5 ], uv : [ 1.0, 1.0 ] },
    Vertex { position : [ -0.5,  0.5,  0.5 ], uv : [ 0.0, 1.0 ] },
    Vertex { position : [ -0.5, -0.5,  0.5 ], uv : [ 0.0, 0.0 ] },

    Vertex { position : [ -0.5,  0.5,  0.5 ], uv : [ 1.0, 0.0 ] },
    Vertex { position : [ -0.5,  0.5, -0.5 ], uv : [ 1.0, 1.0 ] },
    Vertex { position : [ -0.5, -0.5, -0.5 ], uv : [ 0.0, 1.0 ] },
    Vertex { position : [ -0.5, -0.5, -0.5 ], uv : [ 0.0, 1.0 ] },
    Vertex { position : [ -0.5, -0.5,  0.5 ], uv : [ 0.0, 0.0 ] },
    Vertex { position : [ -0.5,  0.5,  0.5 ], uv : [ 1.0, 0.0 ] },

    Vertex { position : [ 0.5,  0.5,  0.5 ], uv : [ 1.0, 0.0 ] },
    Vertex { position : [ 0.5,  0.5, -0.5 ], uv : [ 1.0, 1.0 ] },
    Vertex { position : [ 0.5, -0.5, -0.5 ], uv : [ 0.0, 1.0 ] },
    Vertex { position : [ 0.5, -0.5, -0.5 ], uv : [ 0.0, 1.0 ] },
    Vertex { position : [ 0.5, -0.5,  0.5 ], uv : [ 0.0, 0.0 ] },
    Vertex { position : [ 0.5,  0.5,  0.5 ], uv : [ 1.0, 0.0 ] },

    Vertex { position : [ -0.5, -0.5, -0.5 ], uv : [ 0.0, 1.0 ] },
    Vertex { position : [ 0.5, -0.5, -0.5 ], uv : [ 1.0, 1.0 ] },
    Vertex { position : [ 0.5, -0.5,  0.5 ], uv : [ 1.0, 0.0 ] },
    Vertex { position : [ 0.5, -0.5,  0.5 ], uv : [ 1.0, 0.0 ] },
    Vertex { position : [ -0.5, -0.5,  0.5 ], uv : [ 0.0, 0.0 ] },
    Vertex { position : [ -0.5, -0.5, -0.5 ], uv : [ 0.0, 1.0 ] },

    Vertex { position : [ -0.5,  0.5, -0.5 ], uv : [ 0.0, 1.0 ] },
    Vertex { position : [ 0.5,  0.5, -0.5 ], uv : [ 1.0, 1.0 ] },
    Vertex { position : [ 0.5,  0.5,  0.5 ], uv : [ 1.0, 0.0 ] },
    Vertex { position : [ 0.5,  0.5,  0.5 ], uv : [ 1.0, 0.0 ] },
    Vertex { position : [ -0.5,  0.5,  0.5 ], uv : [ 0.0, 0.0 ] },
    Vertex { position : [ -0.5,  0.5, -0.5 ], uv : [ 0.0, 1.0 ] },
  ]
}

// The order I took from here
// https://github.com/mrdoob/three.js/blob/master/src/cameras/CubeCamera.js
fn cube_camera_make() -> [ gl::F32x4x4; 6 ]
{
  let px = gl::math::mat3x3h::look_at_rh( gl::F32x3::ZERO, gl::F32x3::NEG_X, gl::F32x3::NEG_Y );
  let nx = gl::math::mat3x3h::look_at_rh( gl::F32x3::ZERO, gl::F32x3::X, gl::F32x3::NEG_Y );

  let py = gl::math::mat3x3h::look_at_rh( gl::F32x3::ZERO, gl::F32x3::Y, gl::F32x3::Z );
  let ny = gl::math::mat3x3h::look_at_rh( gl::F32x3::ZERO, gl::F32x3::NEG_Y, gl::F32x3::NEG_Z );

  let pz = gl::math::mat3x3h::look_at_rh( gl::F32x3::ZERO, gl::F32x3::Z, gl::F32x3::NEG_Y );
  let nz = gl::math::mat3x3h::look_at_rh( gl::F32x3::ZERO, gl::F32x3::NEG_Z, gl::F32x3::NEG_Y );

  [ px, nx, py, ny, pz, nz ]
}

fn cube_texture_gen( gl : &GL, width: i32, height: i32 ) -> Option< gl::web_sys::WebGlTexture >
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
      width,
      height,
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

/// Renders the model into a freshly created cube map texture : compiles the
/// generation shaders, uploads the model geometry, and draws every cube face
/// into the texture through a temporary framebuffer. Returns the cube texture
/// together with the model's maximum vertex distance used for normalization.
fn cube_map_generate
(
  gl : &GL,
  document : &gltf::Document,
  buffers : &[ gltf::buffer::Data ]
) -> Result< ( Option< gl::web_sys::WebGlTexture >, f32 ), gl::WebglError >
{
  // Vertex and fragment shaders
  let vertex_shader_src = include_str!( "../shaders/gen_cube_map.vert" );
  let fragment_shader_src = include_str!( "../shaders/gen_cube_map.frag" );
  let program = gl::ProgramFromSources::new( vertex_shader_src, fragment_shader_src ).compile_and_link( gl )?;
  gl.use_program( Some( &program ) );

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
    .map( | p | gl::F32x3::new( p[ 0 ], p[ 1 ], p[ 2 ] ).mag() )
    .reduce( f32::max )
    .unwrap();

    positions = pos_iter.collect();

    let normals_iter = reader.read_normals().expect( "Failed to read normals" );
    normals = normals_iter.collect();

    let index_iter = reader.read_indices().expect( "Failed to read indices" );
    indices = index_iter.into_u32().collect();
  }

  let pos_buffer = gl::buffer::create( gl )?;
  let normal_buffer = gl::buffer::create( gl )?;

  gl::buffer::upload( gl, &pos_buffer, &positions, GL::STATIC_DRAW );
  gl::buffer::upload( gl, &normal_buffer, &normals, GL::STATIC_DRAW );

  let vao = gl::vao::create( gl )?;
  gl.bind_vertex_array( Some( &vao ) );
  let position_attr = mingl::VertexAttribute::new( 0, mingl::VectorDataType::new( mingl::DataType::F32, 3, 1 ), 0 );
  let normal_attr = mingl::VertexAttribute::new( 1, mingl::VectorDataType::new( mingl::DataType::F32, 3, 1 ), 0 );
  gl::BufferDescriptor::from_vector( position_attr.vector ).stride( 3 ).offset( position_attr.offset ).attribute_pointer( gl, position_attr.location, &pos_buffer )?;
  gl::BufferDescriptor::from_vector( normal_attr.vector ).stride( 3 ).offset( normal_attr.offset ).attribute_pointer( gl, normal_attr.location, &normal_buffer )?;

  let index_buffer = gl::buffer::create( gl )?;
  gl::index::upload( gl, &index_buffer, &indices, GL::STATIC_DRAW );

  // Get variables locations
  let projection_matrix_location = gl.get_uniform_location( &program, "projectionMatrix" );
  let model_matrix_location = gl.get_uniform_location( &program, "modelMatrix" );
  let view_matrix_location = gl.get_uniform_location( &program, "viewMatrix" );
  let normal_matrix_location = gl.get_uniform_location( &program, "normalMatrix" );

  let max_distance_location = gl.get_uniform_location( &program, "max_distance" );

  // Camera setup
  let cube_camera = cube_camera_make();

  let perspective_matrix = gl::math::mat3x3h::perspective_rh_gl
  (
     90.0f32.to_radians(),
     1.0,
     0.1,
     10.0
  );

  let model_matrix = gl::F32x4x4::from_scale_rotation_translation
  (
    gl::F32x3::splat( 1.0 ),
    gl::QuatF32::from_angle_y( 0.0 ),
    gl::F32x3::ZERO
  );

  // You need this in case you want to deform your model is some ways
  let normal_matrix = model_matrix.truncate().inverse().unwrap().transpose();

  // Update uniform values
  gl::uniform::matrix_upload( gl, projection_matrix_location, &perspective_matrix.to_array(), true )?;
  gl::uniform::matrix_upload( gl, model_matrix_location, &model_matrix.to_array(), true )?;
  gl::uniform::matrix_upload( gl, normal_matrix_location, &normal_matrix.to_array(), true )?;

  gl::uniform::upload( gl, max_distance_location, &max_distance )?;

  let ( tex_width, tex_height ) = ( 512, 512 );
  let cube_texture = cube_texture_gen( gl, tex_width, tex_height );

  // Render to our cube texture using custom frame buffer
  // All the needed buffers were setup above
  let frame_buffer = gl.create_framebuffer();
  gl.bind_framebuffer( gl::FRAMEBUFFER, frame_buffer.as_ref() );
  gl.viewport( 0, 0, tex_width, tex_height );
  gl.clear_color( 0.0, 0.0, 0.0, 1.0 );
  for ( i, camera ) in cube_camera.iter().enumerate()
  {
    let view_matrix = &camera.to_array();
    gl::uniform::matrix_upload( gl, view_matrix_location.clone(), view_matrix, true )?;
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

  Ok( ( cube_texture, max_distance ) )
}

async fn app_run() -> Result< (), gl::WebglError >
{
  gl::browser::setup( gl::browser::Config::default() );
  let gl = gl::context::retrieve_or_make()?;

  // Load model
  let obj_buffer = gl::file::load( "static/diamond.glb" ).await
  .map_err( | e | gl::dom::Error::BindgenError( "Failed to load static/diamond.glb", format!( "{e:?}" ) ) )?;
  let ( document, buffers, _ ) = gltf::import_slice( &obj_buffer[ .. ] ).expect( "Failed to parse the glb file" );

  let ( cube_texture, max_distance ) = cube_map_generate( &gl, &document, &buffers )?;

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
  let cube_attr = cube_data_get();
  let vertex_count = cube_attr.len();
  let cube_attr_buffer = gl::buffer::create( &gl )?;
  gl::buffer::upload( &gl, &cube_attr_buffer, cube_attr, gl::STATIC_DRAW );

  let vao = gl::vao::create( &gl )?;
  gl.bind_vertex_array( Some( &vao ) );
  let cube_attr_layout = mingl::VertexBufferLayout::from_attribute::< Vertex >( 5 );
  gl::vertex_buffer_layout_bind( &gl, &cube_attr_buffer, &cube_attr_layout )?;

  let width = gl.drawing_buffer_width() as f32;
  let height = gl.drawing_buffer_height() as f32;

  let eye = gl::F32x3::new( 0.0, 0.0, 3.0 );
  let up = gl::F32x3::Y;

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
      let rotatio_y = gl::math::mat3x3::from_angle_y( time );
      let rotatio_z = gl::math::mat3x3::from_angle_z( time + 5.0 );
      let rotatio_z2 = gl::math::mat3x3::from_angle_z( time - 5.0 );
      let eye = rotatio_z2 * rotatio_z * rotatio_y * eye;

      let view_matrix = gl::math::mat3x3h::look_at_rh( eye, gl::F32x3::ZERO, up );

      gl::uniform::matrix_upload( &gl, view_matrix_location.clone(), &view_matrix.to_array(), true ).unwrap();

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
  gl::spawn_local( async move { app_run().await.unwrap() } );
}
