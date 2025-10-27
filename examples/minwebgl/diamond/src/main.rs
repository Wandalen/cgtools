//! Draws diamond figure that reflects cube map texture

#![ allow( clippy::needless_range_loop ) ]
#![ allow( clippy::needless_borrow ) ]

use std::rc::Rc;
use web_sys::wasm_bindgen::prelude::Closure;


use minwebgl as gl;
use gl::
{
  GL,
  JsValue,
  JsCast,
};

fn load_cube_texture( name : &str, document : &gl::web_sys::Document, gl : &gl::WebGl2RenderingContext ) -> Option< gl::web_sys::WebGlTexture >
{
  let upload_texture = | src : String, texture : Option< gl::web_sys::WebGlTexture >, cube_face : u32 | 
  {
    let img_element = document.create_element( "img" ).unwrap().dyn_into::< gl::web_sys::HtmlImageElement >().unwrap();
    img_element.style().set_property( "display", "none" ).unwrap();
    let load_texture : Closure< dyn Fn() > = Closure::new
    (
      {
        let gl = gl.clone();
        let img = img_element.clone();
        move ||
        {
          gl.bind_texture( gl::TEXTURE_CUBE_MAP, texture.as_ref() );
          //gl.pixel_storei( gl::UNPACK_FLIP_Y_WEBGL, 1 );
          gl.tex_image_2d_with_u32_and_u32_and_html_image_element
          (
            gl::TEXTURE_CUBE_MAP_POSITIVE_X + cube_face,
            0,
            gl::RGBA as i32,
            gl::RGBA,
            gl::UNSIGNED_BYTE,
            &img
          ).expect( "Failed to upload data to texture" );
          //gl.pixel_storei( gl::UNPACK_FLIP_Y_WEBGL, 0 );

          if cube_face == 5
          {
            gl.generate_mipmap( gl::TEXTURE_CUBE_MAP );
          }

          gl.tex_parameteri( gl::TEXTURE_CUBE_MAP, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32 );
          gl.tex_parameteri( gl::TEXTURE_CUBE_MAP, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32 );
          gl.tex_parameteri( gl::TEXTURE_CUBE_MAP, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32 );
          gl.tex_parameteri( gl::TEXTURE_CUBE_MAP, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32 );
          gl.tex_parameteri( gl::TEXTURE_CUBE_MAP, gl::TEXTURE_WRAP_R, gl::CLAMP_TO_EDGE as i32 );

          img.remove();
        }
      }
    );

    img_element.set_onload( Some( load_texture.as_ref().unchecked_ref() ) );
    img_element.set_src( &src );
    load_texture.forget();
  };
  
  let texture = gl.create_texture();
  upload_texture( format!( "static/{name}/PX.png" ), texture.clone(), 0 );
  upload_texture( format!( "static/{name}/NX.png" ), texture.clone(), 1 );
  upload_texture( format!( "static/{name}/PY.png" ), texture.clone(), 2 );
  upload_texture( format!( "static/{name}/NY.png" ), texture.clone(), 3 );
  upload_texture( format!( "static/{name}/PZ.png" ), texture.clone(), 4 );
  upload_texture( format!( "static/{name}/NZ.png" ), texture.clone(), 5 );

  texture
}


async fn run() -> Result< (), gl::WebglError >
{
  gl::browser::setup( Default::default() );
  let gl = gl::context::retrieve_or_make()?;
  let window = gl::web_sys::window().unwrap();
  let document = window.document().unwrap();

  // Vertex and fragment shaders
  let vertex_shader_src = include_str!( "../shaders/shader.vert" );
  let fragment_shader_src = include_str!( "../shaders/shader.frag" );
  let background_vertex_shader_src = include_str!( "../shaders/background.vert" );
  let background_fragment_shader_src = include_str!( "../shaders/background.frag" );
  let program = gl::ProgramFromSources::new( vertex_shader_src, fragment_shader_src ).compile_and_link( &gl )?;
  let background_program = gl::ProgramFromSources::new( background_vertex_shader_src, background_fragment_shader_src ).compile_and_link( &gl )?;
  gl.use_program( Some( &program ) );

  // Load textures
  let env_map = load_cube_texture( "skybox", &document, &gl );
  let cube_normal_map = load_cube_texture( "normal_cube", &document, &gl );

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

  let background_view_matrix_location = gl.get_uniform_location( &background_program, "viewMatrix" );

  let env_map_location = 0;
  let cube_normal_map_location = 1;

  let width = gl.drawing_buffer_width() as f32;
  let height = gl.drawing_buffer_height() as f32;

  // Camera setup

  let eye = gl::F32x3::new(  0.0, 5.0, 10.0 );
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

  gl.use_program( Some( &background_program ) );
  gl.uniform1i( gl.get_uniform_location( &background_program, "envMap" ).as_ref(), env_map_location );

  gl.active_texture( gl::TEXTURE0 + env_map_location as u32 );
  gl.bind_texture( gl::TEXTURE_CUBE_MAP, env_map.as_ref() );
  gl.active_texture( gl::TEXTURE0 + cube_normal_map_location as u32 );
  gl.bind_texture( gl::TEXTURE_CUBE_MAP, cube_normal_map.as_ref() );

  gl.enable( gl::DEPTH_TEST );
  gl.depth_func( gl::LEQUAL );

  // Define the update and draw logic
  let update_and_draw =
  {
    let indices_amount = indices.len();
    move | t : f64 |
    {
      let time = t as f32 / 1000.0;
      let rotation = gl::math::mat3x3::from_angle_y( time );
      //let eye = rotation * eye;


      let view_matrix = gl::math::mat3x3h::look_at_rh( eye, gl::F32x3::ZERO, up );
      let inverse_model_matrix = model_matrix.inverse().unwrap();

      gl.use_program( Some( &background_program ) );
      gl::uniform::matrix_upload( &gl, background_view_matrix_location.clone(), &view_matrix.to_array(), true ).unwrap();

      gl.draw_arrays( gl::TRIANGLES, 0, 3 );

      gl.use_program( Some( &program ) );
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
