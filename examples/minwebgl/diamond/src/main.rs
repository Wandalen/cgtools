//! Draws diamond figure that reflects cube map texture

#![ allow( clippy::needless_range_loop ) ]
#![ allow( clippy::needless_borrow ) ]

use minwebgl as gl;
use gl::
{
  GL,
  JsValue
};

async fn load_cube_texture( name : &str ) -> Result< [ image::RgbaImage; 6 ], JsValue >
{
  let px = gl::file::load( &format!( "{}/PX.png", name ) ).await.expect( "Failed to load PX face" );
  let nx = gl::file::load( &format!( "{}/NX.png", name ) ).await.expect( "Failed to load NX face" );

  let py = gl::file::load( &format!( "{}/PY.png", name ) ).await.expect( "Failed to load PY face" );
  let ny = gl::file::load( &format!( "{}/NY.png", name ) ).await.expect( "Failed to load NY face" );

  let pz = gl::file::load( &format!( "{}/PZ.png", name ) ).await.expect( "Failed to load PZ face" );
  let nz = gl::file::load( &format!( "{}/NZ.png", name ) ).await.expect( "Failed to load NZ face" );

  let px = image::load_from_memory( &px ).unwrap().to_rgba8();
  let nx = image::load_from_memory( &nx ).unwrap().to_rgba8();
  let py = image::load_from_memory( &py ).unwrap().to_rgba8();
  let ny = image::load_from_memory( &ny ).unwrap().to_rgba8();
  let pz = image::load_from_memory( &pz).unwrap().to_rgba8();
  let nz = image::load_from_memory( &nz ).unwrap().to_rgba8();

  Ok( [ px, nx, py, ny, pz, nz ] )
}

fn upload_cube_texture( gl : &GL, faces : &[ image::RgbaImage ], location: u32 )
{
  let texture = gl.create_texture();
  gl.active_texture( gl::TEXTURE0 + location );
  gl.bind_texture( gl::TEXTURE_CUBE_MAP, texture.as_ref() );

  for i in 0..faces.len()
  {
    let image = &faces[ i ];
    let ( width, height ) = image.dimensions();
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
      Some( &image )
    ).expect( "Failed to upload data to texture" );
  }

  gl.tex_parameteri( gl::TEXTURE_CUBE_MAP, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32 );
  gl.tex_parameteri( gl::TEXTURE_CUBE_MAP, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32 );
  gl.tex_parameteri( gl::TEXTURE_CUBE_MAP, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32 );
  gl.tex_parameteri( gl::TEXTURE_CUBE_MAP, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32 );
  gl.tex_parameteri( gl::TEXTURE_CUBE_MAP, gl::TEXTURE_WRAP_R, gl::CLAMP_TO_EDGE as i32 );
}


async fn run() -> Result< (), gl::WebglError >
{
  gl::browser::setup( Default::default() );
  let gl = gl::context::retrieve_or_make()?;

  // Vertex and fragment shaders
  let vertex_shader_src = include_str!( "../shaders/shader.vert" );
  let fragment_shader_src = include_str!( "../shaders/shader.frag" );
  let program = gl::ProgramFromSources::new( vertex_shader_src, fragment_shader_src ).compile_and_link( &gl )?;
  gl.use_program( Some( &program ) );

  // Load textures
  let env_map = load_cube_texture( "skybox" ).await.expect( "Failed to load environment map" );
  let cube_normal_map = load_cube_texture( "normal_cube" ).await.expect( "Failed to load cube normal map" );

  // Load model
  let obj_buffer = gl::file::load( "diamond.glb" ).await.expect( "Failed to load the model" );
  let ( document, buffers, _ ) = gltf::import_slice( &obj_buffer[ .. ] ).expect( "Failed to parse the glb file" );

  let positions : Vec< [ f32; 3 ] >;
  let normals : Vec< [ f32; 3 ] >;
  let tex_coords : Vec< [ f32; 2 ] >;
  let indices : Vec< u32 >;

  {
    let mesh = document.meshes().next().expect( "No meshes were found" );
    let primitive = mesh.primitives().next().expect( "No primitives were found" );
    let reader = primitive.reader( | buffer | Some( &buffers[ buffer.index() ] ) );

    let pos_iter = reader.read_positions().expect( "Failed to read positions" );
    positions = pos_iter.collect();

    let normals_iter = reader.read_normals().expect( "Failed to read normals" );
    normals = normals_iter.collect();

    let tex_iter = reader.read_tex_coords( primitive.index() as u32 ).expect( "Failed to read texture coordinates" );
    tex_coords = tex_iter.into_f32().collect();

    let index_iter = reader.read_indices().expect( "Failed to read indices" );
    indices = index_iter.into_u32().collect();
  }

  let pos_buffer =  gl::buffer::create( &gl )?;
  let normal_buffer = gl::buffer::create( &gl )?;
  let uv_buffer = gl::buffer::create( &gl )?;

  gl::buffer::upload( &gl, &pos_buffer, &positions, GL::STATIC_DRAW );
  gl::buffer::upload( &gl, &normal_buffer, &normals, GL::STATIC_DRAW );
  gl::buffer::upload( &gl, &uv_buffer, &tex_coords, GL::STATIC_DRAW );

  let vao = gl::vao::create( &gl )?;
  gl.bind_vertex_array( Some( &vao ) );
  gl::BufferDescriptor::new::< [ f32; 3 ] >().stride( 3 ).offset( 0 ).attribute_pointer( &gl, 0, &pos_buffer )?;
  gl::BufferDescriptor::new::< [ f32; 3 ] >().stride( 3 ).offset( 0 ).attribute_pointer( &gl, 1, &normal_buffer )?;
  gl::BufferDescriptor::new::< [ f32; 2 ] >().stride( 3 ).offset( 0 ).attribute_pointer( &gl, 2, &uv_buffer )?;

  let index_buffer = gl::buffer::create( &gl )?;
  gl::index::upload( &gl, &index_buffer, &indices, GL::STATIC_DRAW );

  // Get variables locations
  let projection_matrix_location = gl.get_uniform_location( &program, "projectionMatrix" );
  let model_matrix_location = gl.get_uniform_location( &program, "modelMatrix" );
  let view_matrix_location = gl.get_uniform_location( &program, "viewMatrix" );
  let inverse_model_matrix_location = gl.get_uniform_location( &program, "inverseModelMatrix" );

  let env_map_intensity_location = gl.get_uniform_location( &program, "envMapIntensity" );
  let rainbow_delta_location = gl.get_uniform_location( &program, "rainbowDelta" );
  let squash_factor_location = gl.get_uniform_location( &program, "squashFactor" );
  let radius_location = gl.get_uniform_location( &program, "radius" );
  let geometry_factor_location = gl.get_uniform_location( &program, "geometryFactor" );
  let absorption_factor_location = gl.get_uniform_location( &program, "absorptionFactor" );

  let color_absorption_location = gl.get_uniform_location( &program, "colorAbsorption" );
  let camera_position_location = gl.get_uniform_location( &program, "cameraPosition" );

  let env_map_location = 0;
  let cube_normal_map_location = 1;

  let width = gl.drawing_buffer_width() as f32;
  let height = gl.drawing_buffer_height() as f32;

  // Camera setup

  let eye = gl::F32x3::new(  0.0, 3.0, 10.0 );
  let up = gl::F32x3::Y;

  let aspect_ratio = width / height;
  let perspective_matrix = gl::math::mat3x3h::perspective_rh_gl
  (
     70.0f32.to_radians(),
     aspect_ratio,
     0.1,
     1000.0
  );

  let model_matrix = gl::F32x4x4::from_scale_rotation_translation
  (
    gl::F32x3::splat( 1.0 ),
    gl::QuatF32::from_angle_y( 0.0 ),
    gl::F32x3::ZERO
  );


  // Update uniform values
  gl::uniform::matrix_upload( &gl, projection_matrix_location, &perspective_matrix.to_array(), true ).unwrap();

  gl::uniform::upload( &gl, env_map_intensity_location.clone(), &0.7 ).unwrap();
  gl::uniform::upload( &gl, rainbow_delta_location.clone(), &0.01 ).unwrap();
  gl::uniform::upload( &gl, squash_factor_location.clone(), &0.8 ).unwrap();
  gl::uniform::upload( &gl, radius_location.clone(), &7.0 ).unwrap();
  gl::uniform::upload( &gl, geometry_factor_location.clone(), &0.5 ).unwrap();
  gl::uniform::upload( &gl, absorption_factor_location.clone(), &0.8 ).unwrap();

  gl::uniform::upload( &gl, color_absorption_location.clone(), &[ 0.9911, 0.9911, 0.9911 ][ .. ] ).unwrap();

  gl.uniform1i( gl.get_uniform_location( &program, "envMap" ).as_ref(), env_map_location );
  gl.uniform1i( gl.get_uniform_location( &program, "cubeNormalMap" ).as_ref(), cube_normal_map_location );

  upload_cube_texture( &gl, &env_map, env_map_location as u32 );
  upload_cube_texture( &gl, &cube_normal_map, cube_normal_map_location as u32 );

  gl.enable( gl::DEPTH_TEST );

  // Define the update and draw logic
  let update_and_draw =
  {
    let indices_amount = indices.len();
    move | t : f64 |
    {
      let time = t as f32 / 1000.0;
      let rotation = gl::math::mat3x3::from_angle_y( time );
      let eye = rotation * eye;


      let view_matrix = gl::math::mat3x3h::look_at_rh( eye, gl::F32x3::ZERO, up );
      let inverse_model_matrix = model_matrix.inverse().unwrap();

      gl::uniform::upload( &gl, camera_position_location.clone(), &eye.to_array()[ .. ] ).unwrap();

      gl::uniform::matrix_upload( &gl, model_matrix_location.clone(), &model_matrix.to_array(), true ).unwrap();
      gl::uniform::matrix_upload( &gl, inverse_model_matrix_location.clone(), &inverse_model_matrix.to_array(), true ).unwrap();
      gl::uniform::matrix_upload( &gl, view_matrix_location.clone(), &view_matrix.to_array(), true ).unwrap();

      // Draw points
      // Vertex and index buffers are already bound
      gl.draw_elements_with_i32( gl::TRIANGLES, indices_amount as i32, gl::UNSIGNED_INT, 0 );
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
