use minwebgl::{ self as gl, GL };
use gl::
{
  F32x4,
  drawbuffers::drawbuffers,
  WebGl2RenderingContext,
  web_sys::
  {
    WebGlFramebuffer, 
    WebGlProgram, 
    WebGlTexture
  }
};
use renderer::webgl::
{
  Object3D,
  Node,
  Camera,
  Scene
};
use std::collections::HashMap;
use std::cell::RefCell;
use std::rc::Rc;

/// Creates a WebGL2 framebuffer and a color attachment texture.
///
/// # Arguments
///
/// * `gl` - The WebGL2 rendering context.
/// * `width`, `height` - The size of the framebuffer and its attachment.
///
/// # Returns
///
/// An `Option< ( WebGlFramebuffer, WebGlTexture ) >` containing the created framebuffer and
/// its color attachment texture, or `None` if creation fails.
fn create_framebuffer
(
  gl : &gl::WebGl2RenderingContext,
  width : u32,
  height : u32
) 
-> Option< ( WebGlFramebuffer, WebGlTexture ) >
{
  let color = gl.create_texture()?;
  gl.bind_texture( GL::TEXTURE_2D, Some( &color ) );
  gl.tex_storage_2d( GL::TEXTURE_2D, 1, gl::RGBA8, width as i32, height as i32 );
  gl.tex_parameteri( GL::TEXTURE_2D, GL::TEXTURE_MIN_FILTER, GL::LINEAR as i32 );
  gl.tex_parameteri( GL::TEXTURE_2D, GL::TEXTURE_WRAP_S, GL::CLAMP_TO_EDGE as i32 );
  gl.tex_parameteri( GL::TEXTURE_2D, GL::TEXTURE_WRAP_T, GL::CLAMP_TO_EDGE as i32 );

  let depthbuffer = gl.create_renderbuffer().unwrap();
  gl.bind_renderbuffer( GL::RENDERBUFFER, Some( &depthbuffer ) );
  gl.renderbuffer_storage( GL::RENDERBUFFER, GL::DEPTH_COMPONENT24, width as i32, height as i32 );

  let framebuffer = gl.create_framebuffer()?;
  gl.bind_framebuffer( GL::FRAMEBUFFER, Some( &framebuffer ) );
  gl.framebuffer_texture_2d( GL::FRAMEBUFFER, GL::COLOR_ATTACHMENT0, GL::TEXTURE_2D, Some( &color ), 0 );
  gl.framebuffer_renderbuffer( GL::FRAMEBUFFER, GL::DEPTH_ATTACHMENT, GL::RENDERBUFFER, Some( &depthbuffer ) );

  drawbuffers( gl, &[ 0 ] );

  gl.bind_framebuffer( gl::FRAMEBUFFER, None );

  Some( ( framebuffer, color ) )
}

pub struct CanvasRenderer
{
  program : WebGlProgram,
  uniforms : HashMap< String, Option< gl::WebGlUniformLocation > >,
  framebuffer : WebGlFramebuffer,
  output_texture : WebGlTexture
} 

impl CanvasRenderer
{
  pub fn new( gl : &WebGl2RenderingContext, width : u32, height : u32 ) -> Result< Self, gl::WebglError >
  {
    let vertex_shader_src = include_str!( "../shaders/canvas.vert" );
    let fragment_shader_src = include_str!( "../shaders/canvas.frag" );
    let program = gl::ProgramFromSources::new( vertex_shader_src, fragment_shader_src )
    .compile_and_link( &gl )?;

    let mut uniforms = HashMap::new(); 
    let mut add_location = 
    | name : &str | 
    {
      uniforms.insert
      ( 
        name.to_string(), 
        gl.get_uniform_location( &program, name ) 
      )
    };

    add_location( "color" );
    add_location( "worldMatrix" );
    add_location( "viewMatrix" );
    add_location( "projectionMatrix" );
    add_location( "normalMatrix" );

    let Some( ( framebuffer, output_texture ) ) = create_framebuffer( gl, width, height )
    else
    {
      return Err( gl::WebglError::FailedToAllocateResource( "Framebuffer" ) );
    };

    Ok(
      Self
      {
        program,
        uniforms,
        framebuffer,
        output_texture
      }
    )
  }

  pub fn render
  ( 
    &self, 
    gl : &WebGl2RenderingContext, 
    scene : &mut Scene, 
    camera : &Camera 
  ) -> Result< (), gl::WebglError >
  {
    scene.update_world_matrix();

    gl.enable( gl::DEPTH_TEST );
    gl.disable( gl::CULL_FACE );
    gl.disable( gl::BLEND );
    gl.clear_depth( 1.0 );
    gl.front_face( gl::CCW );

    gl.bind_framebuffer( GL::FRAMEBUFFER, Some( &self.framebuffer ) );

    gl::drawbuffers::drawbuffers( gl, &[ 0 ] );
    gl.clear_bufferfv_with_f32_array( gl::COLOR, 0, &[ 0.0, 0.0, 0.0, 1.0 ] );
    gl.clear( gl::DEPTH_BUFFER_BIT );

    gl.use_program( Some( &self.program ) );

    let color = F32x4::from_array( [ 1.0, 0.0, 0.0, 1.0 ] ); 
    gl::uniform::upload
    (
      &gl,
      self.uniforms.get( "color" ).unwrap().clone(),
      &color.0[ .. ]
    ).unwrap();
    
    // Define a closure to handle the drawing of each node in the scene.
    let mut draw_node = 
    | 
      node : Rc< RefCell< Node > >
    | -> Result< (), gl::WebglError >
    {
      // If the node contains a mesh...
      if let Object3D::Mesh( ref mesh ) = node.borrow().object
      {
        // Iterate over each primitive in the mesh.
        for primitive_rc in mesh.borrow().primitives.iter()
        {
          let primitive = primitive_rc.borrow();

          camera.upload( &gl, &self.uniforms );
          node.borrow().upload( &gl, &self.uniforms );

          primitive.geometry.borrow().bind( gl );
          primitive.draw( gl );
        }
      } 

      Ok( () )
    };

    // Traverse the scene and draw all opaque objects.
    scene.traverse( &mut draw_node )?;

    Ok( () )
  }

  pub fn set_texture
  ( 
    &mut self, 
    gl : &WebGl2RenderingContext, 
    output_texture : WebGlTexture 
  )
  {
    gl.bind_framebuffer( GL::FRAMEBUFFER, Some( &self.framebuffer ) );
    gl.framebuffer_texture_2d( GL::FRAMEBUFFER, GL::COLOR_ATTACHMENT0, GL::TEXTURE_2D, Some( &output_texture ), 0 );
    gl.bind_framebuffer( gl::FRAMEBUFFER, None );

    self.output_texture = output_texture;
  }

  pub fn get_texture( &self ) -> WebGlTexture
  {
    self.output_texture.clone()
  }
}