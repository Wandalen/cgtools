use minwebgl as gl;
use gl::
{
  BufferDescriptor,
  WebGlBuffer,
  WebGlVertexArrayObject,
  WebglError,
  GL,
};

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
}

impl Geometry
{
  pub fn with_vertices
  (
    gl : &GL,
    vertex_attribute : AttributePointer,
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
    let mut this = Self::with_vertices( gl, vertex_attribute )?;
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

  pub fn element_count( &self ) -> i32
  {
    self.element_count
  }
}
