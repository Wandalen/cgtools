use minwebgl as gl;
use gl::{ GL, JsCast as _, JsValue, F32x4x4, BufferDescriptor, WebglError };
use web_sys::
{
  HtmlCanvasElement,
  WebGlTexture,
  WebGlVertexArrayObject
};

fn main()
{
  gl::browser::setup( Default::default() );
  gl::spawn_local( async { gl::info!( "{:?}", run().await ); } );
}

async fn run() -> Result< (), gl::WebglError >
{
  let gl = gl::context::retrieve_or_make().expect( "Failed to retrieve WebGl context" );
  _ = gl.get_extension( "EXT_color_buffer_float" ).unwrap().unwrap();
  let width = 1280;
  let height = 720;
  let canvas = gl.canvas().unwrap().dyn_into::< HtmlCanvasElement >().unwrap();
  canvas.set_width( width as u32 );
  canvas.set_height( height as u32 );
  gl.viewport( 0, 0, width, height );

  let vert = include_str!( "../shaders/screen_rasterize.vert" );
  let frag = include_str!( "../shaders/deferred.frag" );
  let program = gl::shader::Program::new( gl.clone(), vert, frag )?;
  program.activate();

  let cube_vertices =
  [
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
    // Top face
    -1.0,  1.0, -1.0,
     1.0,  1.0, -1.0,
     1.0,  1.0,  1.0,
    -1.0,  1.0,  1.0,
    // Bottom face
    -1.0, -1.0, -1.0,
     1.0, -1.0, -1.0,
     1.0, -1.0,  1.0,
    -1.0, -1.0,  1.0,
    // Right face
     1.0, -1.0, -1.0,
     1.0,  1.0, -1.0,
     1.0,  1.0,  1.0,
     1.0, -1.0,  1.0,
    // Left face
    -1.0, -1.0, -1.0,
    -1.0,  1.0, -1.0,
    -1.0,  1.0,  1.0,
    -1.0, -1.0,  1.0,
  ];
  let cube_indices =
  [
    0,  1,  2,   0,  2,  3,    // Front face
    4,  5,  6,   4,  6,  7,    // Back face
    8,  9,  10,  8,  10, 11,   // Top face
    12, 13, 14,  12, 14, 15,   // Bottom face
    16, 17, 18,  16, 18, 19,   // Right face
    20, 21, 22,  20, 22, 23,   // Left face
  ];
  let geom = gl::geometry::Positions::new( gl.clone(), &cube_vertices, 3 )?;

  // let plane_mesh =
  // [
  //   [ -1.0, -1.0, 0.0 ],
  //   [  1.0,  1.0, 0.0 ],
  //   [ -1.0,  1.0, 0.0 ],
  //   [ -1.0, -1.0, 0.0 ],
  //   [  1.0, -1.0, 0.0 ],
  //   [  1.0,  1.0, 0.0 ],
  // ].into_iter().flatten().collect::< Vec< f32 > >();

  let update = move | _ |
  {
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
  pub fn new( gl : &GL, descriptor : BufferDescriptor, buffer : gl::WebGlBuffer, slot : u32 ) -> Self
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
  vao : gl::WebGlVertexArrayObject,
  element_count : u32,
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
    vertex_count : i32,
    element_buffer : gl::WebGlBuffer,
    element_count : u32
  )
  -> Result< Self, WebglError >
  {
    let mut this = Self::with_vertices( gl, vertex_attribute, vertex_count )?;
    this.vertex_count = 0;
    this.element_count = element_count;
    gl.bind_buffer( GL::ELEMENT_ARRAY_BUFFER, Some( &element_buffer ) );
    Ok( this )
  }

  pub fn activate( &self )
  {
    self.gl.bind_vertex_array( Some( &self.vao ) );
  }
}
