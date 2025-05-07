//! Implementation of JFA outline

use gltf::json::extensions::texture;
use minwebgl as gl;
use gl::
{
  GL,
  JsValue
};
use web_sys::WebGlTexture;

fn create_texture( 
  gl : &gl::WebGl2RenderingContext,
  size : ( u32, u32 ),
  internal_format : i32,
  format : i32,
  pixel_type : i32,
  data : Option< &[ u8 ] >
) -> Option< WebGlTexture >
{
  let Some( texture ) = gl.create_texture() else {
    return None;   
  };
  gl.active_texture( 33_984u32 + slot );
  gl.bind_texture( GL::TEXTURE_2D, Some( &texture ) );
  gl.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_u8_array( 
    GL::TEXTURE_2D,
    0,
    internal_format,
    size.0,
    size.1,
    0,
    format,
    pixel_type,
    data,
  );
  gl.bind_texture( GL::TEXTURE_2D, None );
  Some( texture )
}

fn upload_texture(
  texture : WebGlTexture,
  location : &WebGlUniformLocation,
  slot : u32,
)
{
  gl.bind_texture( GL::TEXTURE_2D, Some( &texture ) );
  gl.uniform1i( Some( location ), slot );
}

fn create_framebuffer(
  gl : &gl::WebGl2RenderingContext,
  size : ( u32, u32 ),
  color_attachment : u32
) -> Option< ( WebGlFramebuffer, WebGlTexture ) >
{
  let color = gl.create_texture()?;
  gl.bind_texture( GL::TEXTURE_2D, Some( &color ) );
  gl.tex_storage_2d( GL::TEXTURE_2D, 1, gl::RGBA8, size.0, size.1 );
  gl.tex_parameteri( GL::TEXTURE_2D, GL::TEXTURE_MIN_FILTER, GL::LINEAR as i32 );
  gl.tex_parameteri( GL::TEXTURE_2D, GL::TEXTURE_WRAP_S, GL::CLAMP_TO_EDGE as i32 );
  gl.tex_parameteri( GL::TEXTURE_2D, GL::TEXTURE_WRAP_T, GL::CLAMP_TO_EDGE as i32 );

  let framebuffer = gl.create_framebuffer()?;
  gl.bind_framebuffer( GL::FRAMEBUFFER, Some( &framebuffer ) );
  gl.framebuffer_texture_2d( GL::FRAMEBUFFER, GL::COLOR_ATTACHMENT0 + color_attachment, GL::TEXTURE_2D, Some( &texture ), 0 );
  gl.bind_framebuffer( gl::FRAMEBUFFER, None );
  gl.bind_framebuffer( gl::FRAMEBUFFER, None );
  Some( ( framebuffer, color ) ) 
}

async fn run() -> Result< (), gl::WebglError >
{
  gl::browser::setup( Default::default() );
  let gl = gl::context::retrieve_or_make()?;

  // Vertex and fragment shaders
  let object_vs_src = include_str!( "../shaders/object.vert" );
  let object_fs_src = include_str!( "../shaders/object.frag" );
  let fullscreen_vs_src = include_str!( "../shaders/fullscreen.vert" );
  let jfa_init_fs_src = include_str!( "../shaders/jfa_init.frag" );
  let jfa_step_fs_src = include_str!( "../shaders/jfa_step.frag" );
  let outline_fs_src = include_str!( "../shaders/outline.frag" );

  // Programs
  let object_program = gl::ProgramFromSources::new( object_vs_src, object_fs_src ).compile_and_link( &gl )?;
  let jfa_init_program = gl::ProgramFromSources::new( fullscreen_vs_src, jfa_init_fs_src ).compile_and_link( &gl )?;
  let jfa_step_program = gl::ProgramFromSources::new( fullscreen_vs_src, jfa_step_fs_src ).compile_and_link( &gl )?;
  let outline_program = gl::ProgramFromSources::new( fullscreen_vs_src, outline_fs_src ).compile_and_link( &gl )?;

  // Locations
  // - object program
  let u_projection = gl.get_uniform_location( &object_program, "u_projection" );
  let u_view = gl.get_uniform_location( &object_program, "u_view" );
  let u_model = gl.get_uniform_location( &object_program, "u_model" );
  let u_model = gl.get_uniform_location( &object_program, "u_model" );

  // - jfa init program 
  let jfa_init_u_resolution = gl.get_uniform_location( &jfa_init_program, "u_resolution" );

  // - jfa step program 
  let jfa_step_u_resolution = gl.get_uniform_location( &jfa_step_program, "u_resolution" );
  let u_step_size = gl.get_uniform_location( &jfa_step_program, "u_step_size" );

  // - outline program 
  let outline_u_resolution = gl.get_uniform_location( &outline_program, "u_resolution" );
  let u_outline_thickness = gl.get_uniform_location( &outline_program, "u_outline_thickness" );
  let u_oultine_color = gl.get_uniform_location( &outline_program, "u_oultine_color" );
  let u_object_color = gl.get_uniform_location( &outline_program, "u_object_color" );
  let u_background_color = gl.get_uniform_location( &outline_program, "u_background_color" );

  // Other
  let viewport = ( 1920, 1080 );

  // Buffers
  let index_buffer = gl::buffer::create( &gl )?;
  let pos_buffer =  gl::buffer::create( &gl )?;
  let vao = gl::vao::create( &gl )?;

  // Textures

  // Framebuffers
  let ( object_fb, object_fb_color ) = create_framebuffer( &gl, viewport, 0 ).unwrap();
  let ( jfa_init_fb, jfa_init_fb_color ) = create_framebuffer( &gl, viewport, 0 ).unwrap();
  let ( jfa_step_fb, jfa_step_fb_color ) = create_framebuffer( &gl, viewport, 0 ).unwrap();

  gl.use_program( Some( &object_program ) );

  gl::buffer::upload( &gl, &pos_buffer, &positions, GL::STATIC_DRAW );
  gl.bind_vertex_array( Some( &vao ) );
  gl::BufferDescriptor::new::< [ f32; 3 ] >().stride( 3 ).offset( 0 ).attribute_pointer( &gl, 0, &pos_buffer )?;
  gl::index::upload( &gl, &index_buffer, &indices, GL::STATIC_DRAW );

  gl.bind_framebuffer( GL::FRAMEBUFFER, Some( &object_fb ) );

  let u_projection = gl::math::mat3x3h::perspective_rh_gl
  (
    fov,  
    aspect_ratio, 
    near, 
    far
  );
  let u_view = ndarray_cg::d2::mat3x3h::loot_at_rh( eye, center, up );
  //let u_model = ;

  gl::uniform::upload_matrix( &gl, u_projection.clone(), &u_projection.to_cols_array()[ .. ] ).unwrap();
  gl::uniform::upload_matrix( &gl, u_view.clone(), &u_view.to_cols_array()[ .. ] ).unwrap();
  gl::uniform::upload_matrix( &gl, u_model.clone(), &u_model.to_cols_array()[ .. ] ).unwrap();

  gl.use_program( Some( &jfa_init_program ) );
  
  
  
  gl.use_program( Some( &jfa_step_program ) );
  
  
  
  gl.use_program( Some( &outline_program ) );


  
  // Define the update and draw logic
  let update_and_draw =
  {
    move | t : f64 |
    {
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
