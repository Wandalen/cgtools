//! Implementation of JFA outline

use minwebgl as gl;
use gl::
{
  GL,
  web_sys::{
    WebGl2RenderingContext,
    WebGlProgram,
    WebGlUniformLocation,
    WebGlBuffer,
    WebGlTexture, 
    WebGlVertexArrayObject,
    WebGlFramebuffer
  }
};
use ndarray_cg::Mat;
use ndarray_cg::mat::DescriptorOrderColumnMajor;
use std::collections::HashMap;

fn create_texture( 
  gl : &gl::WebGl2RenderingContext,
  slot : u32,
  size : ( i32, i32 ),
  internal_format : i32,
  format : u32,
  pixel_type : u32,
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
  )
  .unwrap();
  gl.bind_texture( GL::TEXTURE_2D, None );
  Some( texture )
}

fn upload_texture(
  gl : &gl::WebGl2RenderingContext,
  texture : &WebGlTexture,
  location : &WebGlUniformLocation,
  slot : u32,
)
{
  gl.active_texture( 33_984u32 + slot );
  gl.bind_texture( GL::TEXTURE_2D, Some( &texture ) );
  gl.uniform1i( Some( location ), slot as i32 );
}

fn create_framebuffer(
  gl : &gl::WebGl2RenderingContext,
  size : ( i32, i32 ),
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
  gl.framebuffer_texture_2d( GL::FRAMEBUFFER, GL::COLOR_ATTACHMENT0 + color_attachment, GL::TEXTURE_2D, Some( &color ), 0 );
  gl.bind_framebuffer( gl::FRAMEBUFFER, None ); 

  Some( ( framebuffer, color ) ) 
}

fn upload_framebuffer(
  gl : &gl::WebGl2RenderingContext,
  framebuffer : &WebGlFramebuffer,
  size : ( i32, i32 )
)
{
  gl.bind_framebuffer( GL::FRAMEBUFFER, Some( framebuffer ) );
  gl.viewport( 0, 0, size.0, size.1 );
}

fn s( s: &str ) -> String
{
  s.to_string()
}

struct Camera
{
  eye : ndarray_cg::F32x3,
  up : ndarray_cg::F32x3,
  projection : Mat< 4, 4, f32, DescriptorOrderColumnMajor >,
  model : Mat< 4, 4, f32, DescriptorOrderColumnMajor >
}

struct Renderer
{
  gl : WebGl2RenderingContext,
  programs : HashMap< String, WebGlProgram >,
  buffers : HashMap< String, WebGlBuffer >,
  textures : HashMap< String, WebGlTexture >,
  vaos : HashMap< String, WebGlVertexArrayObject >,
  framebuffers : HashMap< String, WebGlFramebuffer >,
  viewport : ( i32, i32 ),
  camera : Camera,
  draw_count : i32
}

impl Renderer
{
  async fn new() -> Self
  {
    gl::browser::setup( Default::default() );
    let gl = gl::context::retrieve_or_make().unwrap();

    // Other
    let viewport = ( gl.drawing_buffer_width(), gl.drawing_buffer_height() );
  
    // Camera setup
    let eye = ndarray_cg::F32x3::from_array( [  0.0, 3.0, 10.0 ] );
    let up = ndarray_cg::F32x3::Y;
  
    let aspect_ratio = viewport.0 as f32 / viewport.1 as f32;
    let u_projection = ndarray_cg::mat3x3h::perspective_rh_gl
    (
      70.0f32.to_radians(),  
      aspect_ratio, 
      0.1, 
      1000.0
    );
    let u_model = glam::Mat4::from_scale_rotation_translation
    (
      glam::Vec3::ONE, 
      glam::Quat::from_rotation_y( 0.0 ), 
      glam::Vec3::ZERO
    );
    let u_model : Mat< 4, 4, f32, DescriptorOrderColumnMajor > = ndarray_cg::Mat4::from_column_major( u_model.to_cols_array() );

    let camera = Camera{
      eye,
      up,
      projection : u_projection,
      model : u_model
    };

    // Renderer

    let mut renderer = Self 
    { 
      gl,
      programs : HashMap::new(),
      buffers : HashMap::new(),
      textures : HashMap::new(),
      vaos : HashMap::new(),
      framebuffers : HashMap::new(),
      viewport,
      camera,
      draw_count : 0
    };

    let gl = &renderer.gl;

    // Vertex and fragment shaders
    let object_vs_src = include_str!( "../resources/shaders/object.vert" );
    let object_fs_src = include_str!( "../resources/shaders/object.frag" );
    let fullscreen_vs_src = include_str!( "../resources/shaders/fullscreen.vert" );
    let jfa_init_fs_src = include_str!( "../resources/shaders/jfa_init.frag" );
    let jfa_step_fs_src = include_str!( "../resources/shaders/jfa_step.frag" );
    let outline_fs_src = include_str!( "../resources/shaders/outline.frag" );
  
    // Programs
    let object_program = gl::ProgramFromSources::new( object_vs_src, object_fs_src ).compile_and_link( gl ).unwrap();
    let jfa_init_program = gl::ProgramFromSources::new( fullscreen_vs_src, jfa_init_fs_src ).compile_and_link( gl ).unwrap();
    let jfa_step_program = gl::ProgramFromSources::new( fullscreen_vs_src, jfa_step_fs_src ).compile_and_link( gl ).unwrap();
    let outline_program = gl::ProgramFromSources::new( fullscreen_vs_src, outline_fs_src ).compile_and_link( gl ).unwrap();
  
    renderer.programs.insert( s( "object" ), object_program );
    renderer.programs.insert( s( "jfa_init" ), jfa_init_program );
    renderer.programs.insert( s( "jfa_step" ), jfa_step_program );
    renderer.programs.insert( s( "outline" ), outline_program );
  
    // Textures
  
    // Framebuffers
    let ( object_fb, object_fb_color ) = create_framebuffer( gl, viewport, 0 ).unwrap();
    let ( jfa_init_fb, jfa_init_fb_color ) = create_framebuffer( gl, viewport, 0 ).unwrap();
    let ( jfa_step_fb_0, jfa_step_fb_color_0 ) = create_framebuffer( gl, viewport, 0 ).unwrap();
    let ( jfa_step_fb_1, jfa_step_fb_color_1 ) = create_framebuffer( gl, viewport, 0 ).unwrap();

    renderer.textures.insert( s( "object_fb_color" ), object_fb_color );
    renderer.textures.insert( s( "jfa_init_fb_color" ), jfa_init_fb_color );
    renderer.textures.insert( s( "jfa_step_fb_color_0" ), jfa_step_fb_color_0 );
    renderer.textures.insert( s( "jfa_step_fb_color_1" ), jfa_step_fb_color_1 );

    renderer.framebuffers.insert( s( "object_fb" ), object_fb );
    renderer.framebuffers.insert( s( "jfa_init_fb" ), jfa_init_fb );
    renderer.framebuffers.insert( s( "jfa_step_fb_0" ), jfa_step_fb_0 );
    renderer.framebuffers.insert( s( "jfa_step_fb_1" ), jfa_step_fb_1 );
  
    // Buffers
    let pos_buffer =  gl::buffer::create( gl ).unwrap();
    let index_buffer = gl::buffer::create( gl ).unwrap();
    let vao = gl::vao::create( gl ).unwrap();

    renderer.buffers.insert( s( "pos_buffer" ), pos_buffer.clone() );
    renderer.buffers.insert( s( "index_buffer" ), index_buffer.clone() );
    renderer.vaos.insert( s( "vao" ), vao.clone() );
  
    // Model
    let obj_buffer = gl::file::load( "model.glb" ).await.expect( "Failed to load the model" );
    let ( document, buffers, _ ) = gltf::import_slice( &obj_buffer[ .. ] ).expect( "Failed to parse the glb file" );
  
    let positions : Vec< [ f32; 3 ] >;
    let indices : Vec< u32 >;
  
    {
      let mesh = document.meshes().next().expect( "No meshes were found" );
      let primitive = mesh.primitives().next().expect( "No primitives were found" );
      let reader = primitive.reader( | buffer | Some( &buffers[ buffer.index() ] ) );
  
      let pos_iter = reader.read_positions().expect( "Failed to read positions" );
      positions = pos_iter.collect();
  
      let index_iter = reader.read_indices().expect( "Failed to read indices" );
      indices = index_iter.into_u32().collect();
      renderer.draw_count = indices.len() as i32;
    }
  
    gl::buffer::upload( &gl, &pos_buffer, &positions, GL::STATIC_DRAW );
    gl::index::upload( &gl, &index_buffer, &indices, GL::STATIC_DRAW );
  
    gl.bind_vertex_array( Some( &vao ) );
    gl::BufferDescriptor::new::< [ f32; 3 ] >().stride( 3 ).offset( 0 ).attribute_pointer( &gl, 0, &pos_buffer ).unwrap();

    renderer
  }

  fn render( &self, t : f64 )
  {
    let gl = &self.gl;

    self.object_pass( t );
    gl.draw_elements_with_i32( gl::TRIANGLES, self.draw_count, gl::UNSIGNED_INT, 0 );

    self.jfa_init_pass();
    gl.draw_elements_with_i32( gl::TRIANGLES, self.draw_count, gl::UNSIGNED_INT, 0 );

    let num_passes = ( self.viewport.0.max( self.viewport.1 ) as f32 ).log2().ceil() as i32;
    for i in 0..num_passes
    {
      self.jfa_step_pass( i );
      gl.draw_elements_with_i32( gl::TRIANGLES, self.draw_count, gl::UNSIGNED_INT, 0 );
    }
    
    self.outline_pass( num_passes );
    gl.draw_elements_with_i32( gl::TRIANGLES, self.draw_count, gl::UNSIGNED_INT, 0 );
  }

  fn object_pass( &self, t : f64 )
  {
    let gl = &self.gl;

    let object_program = self.programs.get( "object" ).unwrap();
    let object_fb = self.framebuffers.get( "object_fb" ).unwrap();

    let u_projection_loc = gl.get_uniform_location( object_program, "u_projection" ).unwrap();
    let u_view_loc = gl.get_uniform_location( object_program, "u_view" ).unwrap();
    let u_model_loc = gl.get_uniform_location( object_program, "u_model" ).unwrap();

    gl.use_program( Some( object_program ) );

    let rotation = ndarray_cg::mat3x3::from_angle_y( t as f32 / 1000.0 );
    let eye = rotation * self.camera.eye;

    let u_view = ndarray_cg::d2::mat3x3h::look_at_rh( eye, ndarray_cg::F32x3::from_array( [ 0.0; 3 ] ), self.camera.up );
  
    upload_framebuffer( gl, object_fb, self.viewport );
    gl::uniform::matrix_upload( gl, Some( u_projection_loc.clone() ), &self.camera.projection.to_array()[ .. ], true ).unwrap();
    gl::uniform::matrix_upload( gl, Some( u_view_loc.clone() ), &u_view.to_array()[ .. ], true ).unwrap();
    gl::uniform::matrix_upload( gl, Some( u_model_loc.clone() ), &self.camera.model.to_array()[ .. ], true ).unwrap();
  }

  fn jfa_init_pass( &self )
  {
    let gl = &self.gl;

    let jfa_init_program = self.programs.get( "jfa_init" ).unwrap();
    let jfa_init_fb = self.framebuffers.get( "object_fb" ).unwrap();
    let object_fb_color = self.textures.get( "object_fb_color" ).unwrap();

    let u_resolution = gl.get_uniform_location( jfa_init_program, "u_resolution" ).unwrap();
    let u_object_texture = gl.get_uniform_location( jfa_init_program, "u_object_texture" ).unwrap();

    gl.use_program( Some( jfa_init_program ) );
  
    upload_framebuffer( gl, jfa_init_fb, self.viewport );
    gl::uniform::upload( gl, Some( u_resolution.clone() ), &[ self.viewport.0 as f32, self.viewport.1 as f32 ] ).unwrap();
    upload_texture( gl, object_fb_color, &u_object_texture, 0 );
  }

  fn jfa_step_pass( &self, i : i32 )
  {
    let gl = &self.gl;

    let jfa_step_program = self.programs.get( "jfa_step" ).unwrap();
    let jfa_step_fb_0 = self.framebuffers.get( "jfa_step_fb_0" ).unwrap();
    let jfa_step_fb_1 = self.framebuffers.get( "jfa_step_fb_1" ).unwrap();
    let jfa_init_fb_color = self.textures.get( "jfa_init_fb_color" ).unwrap();
    let jfa_step_fb_color_0 = self.textures.get( "jfa_step_fb_color_0" ).unwrap();
    let jfa_step_fb_color_1 = self.textures.get( "jfa_step_fb_color_1" ).unwrap();

    let u_resolution = gl.get_uniform_location( &jfa_step_program, "u_resolution" ).unwrap();
    let u_step_size = gl.get_uniform_location( &jfa_step_program, "u_step_size" ).unwrap();
    let u_jfa_init_texture = gl.get_uniform_location( &jfa_step_program, "u_jfa_texture" ).unwrap();

    gl.use_program( Some( jfa_step_program ) );
  
    if i == 0
    {
      upload_framebuffer( gl, jfa_step_fb_0, self.viewport );
      upload_texture( gl, jfa_init_fb_color, &u_jfa_init_texture, 0 );
    }
    else if i % 2 == 0
    {
      upload_framebuffer( gl, jfa_step_fb_0, self.viewport );
      upload_texture( gl, &jfa_step_fb_color_1, &u_jfa_init_texture, 0 );
    }
    else
    {
      upload_framebuffer( gl, jfa_step_fb_1, self.viewport );
      upload_texture( gl, jfa_step_fb_color_0, &u_jfa_init_texture, 0 );
    } 

    gl::uniform::upload( gl, Some( u_resolution.clone() ), &[ self.viewport.0 as f32, self.viewport.1 as f32 ] ).unwrap();
    let step_size = ( ( self.viewport.0.max( self.viewport.1 ) as f32 ) / 2.0f32.powi( i + 1 ) ).max( 1.0 );
    gl::uniform::upload( gl, Some( u_step_size.clone() ), &step_size ).unwrap();
  }

  fn outline_pass( &self, num_passes : i32 )
  {
    let gl = &self.gl;

    let outline_program = self.programs.get( "outline" ).unwrap();
    let object_fb_color = self.textures.get( "object_fb_color" ).unwrap();
    let jfa_step_fb_color_0 = self.textures.get( "jfa_step_fb_color_0" ).unwrap();
    let jfa_step_fb_color_1 = self.textures.get( "jfa_step_fb_color_1" ).unwrap();

    let outline_u_object_texture = gl.get_uniform_location( outline_program, "u_object_texture" ).unwrap();
    let u_jfa_step_texture = gl.get_uniform_location( outline_program, "u_jfa_texture" ).unwrap();
    let u_resolution = gl.get_uniform_location( outline_program, "u_resolution" ).unwrap();
    let u_outline_thickness = gl.get_uniform_location( outline_program, "u_outline_thickness" ).unwrap();
    let u_outline_color = gl.get_uniform_location( outline_program, "u_outline_color" ).unwrap();
    let u_object_color = gl.get_uniform_location( outline_program, "u_object_color" ).unwrap();
    let u_background_color = gl.get_uniform_location( outline_program, "u_background_color" ).unwrap();

    gl.use_program( Some( outline_program ) );

    let outline_thickness = [ 5.0 ]; 
    let outline_color = [ 1.0, 1.0, 1.0, 1.0 ]; 
    let object_color = [ 0.5, 0.5, 0.5, 1.0 ]; 
    let background_color = [ 0.0, 0.0, 0.0, 1.0 ];
  
    gl.bind_framebuffer( GL::FRAMEBUFFER, None );
    gl::uniform::upload( gl, Some( u_resolution.clone() ), &[ self.viewport.0 as f32, self.viewport.1 as f32 ] ).unwrap();
    gl::uniform::upload( gl, Some( u_outline_thickness.clone() ), &outline_thickness ).unwrap();
    gl::uniform::upload( gl, Some( u_outline_color.clone() ), &outline_color ).unwrap();
    gl::uniform::upload( gl, Some( u_object_color.clone() ), &object_color ).unwrap();
    gl::uniform::upload( gl, Some( u_background_color.clone() ), &background_color ).unwrap();
    upload_texture( gl, object_fb_color, &outline_u_object_texture, 0 );
    if num_passes % 2 == 0
    {
      upload_texture( gl, jfa_step_fb_color_0, &u_jfa_step_texture, 1 );
    }
    else
    {
      upload_texture( gl, jfa_step_fb_color_1, &u_jfa_step_texture, 1 );
    }
  }
}

async fn run() -> Result< (), gl::WebglError >
{
  let renderer = Renderer::new().await;

  // Define the update and draw logic
  let update_and_draw =
  {
    move | t : f64 |
    {
      renderer.render( t );
      
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
