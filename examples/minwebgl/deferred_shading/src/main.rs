use minwebgl as gl;
use gl::
{
  math::d2::mat3x3h,
  BufferDescriptor,
  JsCast as _,
  WebGlBuffer,
  WebGlVertexArrayObject,
  WebglError,
  GL
};
use web_sys::{ HtmlCanvasElement, WebGlTexture };

fn main()
{
  gl::browser::setup( Default::default() );
  gl::info!( "{:?}", run() );
  // gl::spawn_local( async {  } );
}

// cube geometry
static CUBE_VERTICES : &[ f32 ] =
&[
  // Front face
  -1.0, -1.0,  1.0,
   1.0, -1.0,  1.0,
   1.0,  1.0,  1.0,
  -1.0,  1.0,  1.0,
  // Back face
  -1.0, -1.0, -1.0,
   1.0, -1.0, -1.0,
   1.0,  1.0, -1.0,
  -1.0,  1.0, -1.0,
];

static CUBE_INDICES : &[ u32 ] =
&[
  // Front face
  0, 1, 2, 0, 2, 3,
  // Back face
  4, 6, 5, 4, 7, 6,
  // Top face
  3, 2, 6, 3, 6, 7,
  // Bottom face
  0, 5, 1, 0, 4, 5,
  // Right face
  1, 5, 6, 1, 6, 2,
  // Left face
  0, 3, 7, 0, 7, 4
];

fn run() -> Result< (), gl::WebglError >
{
  let gl = gl::context::retrieve_or_make().expect( "Failed to retrieve WebGl context" );
  let res = gl.get_extension( "EXT_color_buffer_float" ).unwrap().unwrap();
  gl::info!( "{}", res.to_string() );

  let width = 1280;
  let height = 720;
  let canvas = gl.canvas().unwrap().dyn_into::< HtmlCanvasElement >().unwrap();
  canvas.set_width( width as u32 );
  canvas.set_height( height as u32 );

  gl.viewport( 0, 0, width, height );
  gl.enable( GL::DEPTH_TEST );
  gl.enable( GL::CULL_FACE );
  gl.clear_color( 0.0, 0.0, 0.0, 1.0 );
  gl.clear_stencil( 0 );
  gl.stencil_mask( 0x1 );

  // shaders
  let vert = include_str!( "../shaders/light_volume.vert" );
  let frag = include_str!( "../shaders/deferred.frag" );
  let light_shader = gl::shader::Program::new( gl.clone(), vert, frag )?;

  // let vert = include_str!( "../shaders/light_volume.vert" );
  let frag = include_str!( "../shaders/empty.frag" );
  let stencil_shader = gl::shader::Program::new( gl.clone(), vert, frag )?;

  let vert = include_str!( "../shaders/main.vert" );
  let frag = include_str!( "../shaders/gbuffer.frag" );
  let object_shader = gl::shader::Program::new( gl.clone(), vert, frag )?;

  let vert = include_str!( "../shaders/big_triangle.vert" );
  let frag = include_str!( "../shaders/screen_texture.frag" );
  let screen_shader = gl::shader::Program::new( gl.clone(), vert, frag )?;

  let position_buffer = gl::buffer::create( &gl )?;
  gl::buffer::upload( &gl, &position_buffer, CUBE_VERTICES, GL::STATIC_DRAW );
  let index_buffer = gl::buffer::create( &gl )?;
  gl::index::upload( &gl, &index_buffer, CUBE_INDICES, GL::STATIC_DRAW );
  let position_attribute = AttributePointer::new( &gl, BufferDescriptor::new::< [ f32; 3 ] >(), position_buffer, 0 );
  let cube = Geometry::with_elements( &gl, position_attribute, index_buffer, CUBE_INDICES.len() as i32 )?;
  cube.activate();

  let object_transform = mat3x3h::translation( [ 0.0f32, 0.0, -30.0 ] ) * mat3x3h::scale( [ 100.0, 100.0, 1.0 ] );
  let projection = mat3x3h::perspective_rh_gl( 45.0f32.to_radians(), width as f32 / height as f32, 0.1, 100.0 );
  let light_radius = 3.0;
  let light_position = [ 0.0, 0.0, -27.0 ];

  // gbuffer
  let gbuffer = gl.create_framebuffer();
  let position_buffer = tex_storage_2d( &gl, GL::RGBA16F, width, height );
  let normal_buffer = tex_storage_2d( &gl, GL::RGBA16F, width, height );
  let depthbuffer = gl.create_renderbuffer();
  gl.bind_renderbuffer( GL::RENDERBUFFER, depthbuffer.as_ref() );
  gl.renderbuffer_storage( GL::RENDERBUFFER, GL::DEPTH24_STENCIL8, width, height );
  gl.bind_framebuffer( GL::FRAMEBUFFER, gbuffer.as_ref() );
  gl.framebuffer_texture_2d( GL::FRAMEBUFFER, GL::COLOR_ATTACHMENT0, GL::TEXTURE_2D, position_buffer.as_ref(), 0 );
  gl.framebuffer_texture_2d( GL::FRAMEBUFFER, GL::COLOR_ATTACHMENT1, GL::TEXTURE_2D, normal_buffer.as_ref(), 0 );
  gl.framebuffer_renderbuffer( GL::FRAMEBUFFER, GL::DEPTH_STENCIL_ATTACHMENT, GL::RENDERBUFFER, depthbuffer.as_ref() );

  // draw gbuffer geometry for once, because it is static
  gl::drawbuffers::drawbuffers( &gl, &[ GL::COLOR_ATTACHMENT0, GL::COLOR_ATTACHMENT1 ] );
  object_shader.activate();
  object_shader.uniform_matrix_upload( "u_model", object_transform.raw_slice(), true );
  object_shader.uniform_matrix_upload( "u_mvp", ( projection * object_transform ).raw_slice(), true );
  gl.vertex_attrib3f( 1, 0.0, 0.0, 1.0 );
  gl.draw_elements_with_i32( GL::TRIANGLES, cube.element_count, GL::UNSIGNED_INT, 0 );

  // use same framebuffer object for offscreen rendering
  let offscreen_buffer = gbuffer;
  let color_buffer = tex_storage_2d( &gl, GL::RGBA8, width, height );
  gl.framebuffer_texture_2d( GL::FRAMEBUFFER, GL::COLOR_ATTACHMENT0, GL::TEXTURE_2D, color_buffer.as_ref(), 0 );
  gl.framebuffer_texture_2d( GL::FRAMEBUFFER, GL::COLOR_ATTACHMENT1, GL::TEXTURE_2D, None, 0 );

  gl.depth_mask( false );

  let update = move | _ |
  {
    gl.bind_framebuffer( GL::FRAMEBUFFER, offscreen_buffer.as_ref() );
    gl.enable( GL::DEPTH_TEST );
    gl.enable( GL::STENCIL_TEST );
    gl.clear( GL::STENCIL_BUFFER_BIT );
    gl.stencil_func( GL::ALWAYS, 0, 0xFF );
    // dont output any values now except stencil values
    gl::drawbuffers::drawbuffers( &gl, &[] );

    stencil_shader.activate();
    stencil_shader.uniform_matrix_upload( "u_mvp", projection.raw_slice(), true );
    gl.vertex_attrib3fv_with_f32_array( 1, &light_position );
    gl.vertex_attrib1f( 2, light_radius );

    // draw front faces that pass depth test and increment fragments
    gl.cull_face( GL::BACK );
    gl.stencil_op( GL::KEEP, GL::KEEP, GL::INCR );
    gl.draw_elements_with_i32( GL::TRIANGLES, cube.element_count, GL::UNSIGNED_INT, 0 );

    // draw back faces that pass depth test and decrement fragment
    gl.cull_face( GL::FRONT );
    gl.stencil_op( GL::KEEP, GL::KEEP, GL::DECR );
    gl.draw_elements_with_i32( GL::TRIANGLES, cube.element_count, GL::UNSIGNED_INT, 0 );

    // do the deffered shading againts pixels that passed the stencil test
    gl::drawbuffers::drawbuffers( &gl, &[ GL::COLOR_ATTACHMENT0 ] );
    gl.cull_face( GL::BACK );
    gl.clear( gl::COLOR_BUFFER_BIT );
    gl.stencil_func( GL::EQUAL, 1, 0xFF );
    gl.stencil_op( GL::KEEP, GL::KEEP, GL::KEEP );

    gl.active_texture( GL::TEXTURE0 );
    gl.bind_texture( GL::TEXTURE_2D, position_buffer.as_ref() );
    gl.active_texture( GL::TEXTURE1 );
    gl.bind_texture( GL::TEXTURE_2D, normal_buffer.as_ref() );

    light_shader.activate();
    light_shader.uniform_matrix_upload( "u_mvp", projection.raw_slice(), true );
    light_shader.uniform_upload( "u_screen_size", [ width as f32, height as f32 ].as_slice() );
    light_shader.uniform_upload( "u_positions", &0 );
    light_shader.uniform_upload( "u_normals", &1 );
    gl.vertex_attrib3fv_with_f32_array( 1, &light_position );
    gl.vertex_attrib1f( 2, light_radius );
    gl.draw_elements_with_i32( GL::TRIANGLES, cube.element_count, GL::UNSIGNED_INT, 0 );

    // show on screen
    gl.disable( GL::DEPTH_TEST );
    gl.disable( GL::STENCIL_TEST );
    gl.bind_framebuffer( GL::FRAMEBUFFER, None );
    gl.active_texture( GL::TEXTURE0 );
    gl.bind_texture( GL::TEXTURE_2D, color_buffer.as_ref() );
    screen_shader.activate();
    gl.draw_arrays( GL::TRIANGLES, 0, 3 );

    true
  };
  gl::exec_loop::run( update );

  Ok( () )
}

pub struct AttributePointer
{
  gl : GL,
  descriptor : BufferDescriptor,
  buffer : gl::WebGlBuffer,
  slot : u32,
}

impl AttributePointer
{
  pub fn new( gl : &GL, descriptor : BufferDescriptor, buffer : WebGlBuffer, slot : u32 ) -> Self
  {
    Self { gl : gl.clone(), descriptor, buffer, slot }
  }

  pub fn enable( &self ) -> Result< u32, WebglError >
  {
    self.descriptor.attribute_pointer( &self.gl, self.slot, &self.buffer )
  }
}

pub struct Geometry
{
  gl : GL,
  vao : WebGlVertexArrayObject,
  element_count : i32,
  vertex_count : i32,
}

impl Geometry
{
  pub fn with_vertices
  (
    gl : &GL,
    vertex_attribute : AttributePointer,
    vertex_count : i32
  )
  -> Result< Self, WebglError >
  {
    let vao = gl::vao::create( gl )?;
    gl.bind_vertex_array( Some( &vao ) );
    vertex_attribute.enable()?;

    Ok
    (
      Self
      {
        gl : gl.clone(),
        vao,
        element_count : 0,
        vertex_count,
      }
    )
  }

  pub fn with_elements
  (
    gl : &GL,
    vertex_attribute : AttributePointer,
    element_buffer : gl::WebGlBuffer,
    element_count : i32
  )
  -> Result< Self, WebglError >
  {
    let mut this = Self::with_vertices( gl, vertex_attribute, 0 )?;
    this.element_count = element_count;
    gl.bind_buffer( GL::ELEMENT_ARRAY_BUFFER, Some( &element_buffer ) );
    Ok( this )
  }

  pub fn activate( &self )
  {
    self.gl.bind_vertex_array( Some( &self.vao ) );
  }

  pub fn add_attribute( &self, attribute : AttributePointer ) -> Result< (), WebglError >
  {
    self.activate();
    attribute.enable()?;
    Ok( () )
  }

  pub fn vertex_count( &self ) -> i32
  {
    self.vertex_count
  }

  pub fn element_count( &self ) -> i32
  {
    self.element_count
  }
}

fn tex_storage_2d( gl : &GL, format : u32, width : i32, height : i32 ) -> Option< WebGlTexture >
{
  let tex = gl.create_texture()?;
  gl.bind_texture( GL::TEXTURE_2D, Some( &tex ) );
  gl.tex_storage_2d( GL::TEXTURE_2D, 1, format, width, height );
  gl.tex_parameteri( GL::TEXTURE_2D, GL::TEXTURE_MIN_FILTER, GL::NEAREST as i32 );
  gl.tex_parameteri( GL::TEXTURE_2D, GL::TEXTURE_MAG_FILTER, GL::NEAREST as i32 );
  Some( tex )
}
