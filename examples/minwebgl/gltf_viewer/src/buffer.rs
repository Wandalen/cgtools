use minwebgl as gl;

pub struct Buffer
{
  target : u32,
  buffer : gl::WebGlBuffer
}

impl Buffer 
{
  pub fn new
  ( 
    gl : &gl::WebGl2RenderingContext, 
    view : &gltf::buffer::View,
    buffers : &[ gl::js_sys::Uint8Array ] 
  ) -> Result< Self, gl::WebglError >
  {
    let target = view.target().expect( "Buffer doesn't have a target set" );
    let target = match target
    {
      gltf::buffer::Target::ArrayBuffer => gl::ARRAY_BUFFER ,
      gltf::buffer::Target::ElementArrayBuffer => gl::ELEMENT_ARRAY_BUFFER    
    };

    let index = view.buffer().index();
    let buffer = gl::buffer::create( &gl )?;
    gl.bind_buffer( target, Some( &buffer ) );
    gl.buffer_data_with_js_u8_array_and_src_offset_and_length
    ( 
      target, 
      &buffers[ index ], 
      gl::STATIC_DRAW,
      view.offset() as u32,
      view.length() as u32
    );

    Ok( 
      Self
      {
        target,
        buffer
      }
    )
  }

  pub fn bind( &self, gl : &gl::WebGl2RenderingContext )
  {
    gl.bind_buffer( self.target, Some( &self.buffer ) );
  }

  pub fn as_option( &self ) -> Option< &gl::WebGlBuffer >
  { 
    Some( &self.buffer )
  }
}