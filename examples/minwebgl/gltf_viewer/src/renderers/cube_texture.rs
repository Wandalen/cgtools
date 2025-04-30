use minwebgl as gl;

pub struct CubeTextureRenderer
{
  frame_buffer : Option< gl::web_sys::WebGlFramebuffer >,
  cube_map : Option< gl::web_sys::WebGlTexture >,
  program : gl::WebGlProgram,
  vao : gl::WebGlVertexArrayObject
}

impl CubeTextureRenderer
{
  pub fn new( gl : &gl::WebGl2RenderingContext, width : u32, height : u32, format : u32 ) -> Result< Self, gl::WebglError >
  {
    let vert = include_str!( "../../shaders/equi_to_cube.vert" );
    let frag = include_str!( "../../shaders/equi_to_cube.frag" );

    let program = gl::ProgramFromSources::new( &vert, &frag ).compile_and_link( &gl )?;

    let frame_buffer = gl.create_framebuffer();
    let cube_map = gl.create_texture();
    gl.bind_texture( gl::TEXTURE_CUBE_MAP, cube_map.as_ref() );
    for i in 0..6
    {
      gl.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_u8_array
      (
        gl::TEXTURE_CUBE_MAP_NEGATIVE_X + i,
        0,
        format as i32,
        width as i32,
        height as i32,
        0,
        format,
        gl::UNSIGNED_BYTE,
        None
      ).expect( "Failed to allocate memory for a cube texture" );
    }

    let vao = gl::vao::create( gl )?;
    gl.bind_vertex_array( Some( &vao ) );

    let vertex_buffer = gl::buffer::create( gl )?;
    gl.bind_buffer( gl::VERTEX_ARRAY_BINDING, Some( &vertex_buffer ) );
    gl::buffer::upload( gl, &vertex_buffer, cube_vertices.as_slice(), gl::STATIC_DRAW );

    gl::BufferDescriptor::new::< [ f32 ; 3 ] >().stride( 3 ).offset( 0 ).attribute_pointer( &gl, 0, &vertex_buffer )?;


    Ok
    (
      Self
      {
        frame_buffer,
        cube_map,
        program,
        vao
      }
    )
  }

  pub fn render( &self, gl : &gl::WebGl2RenderingContext, mip_level : u32 )
  {
    let projection_matrix = gl::math::mat3x3h::perspective_rh( 90.0, 1.0, 0.1, 10.0 );
    
    gl.bind_framebuffer( gl::FRAMEBUFFER, self.frame_buffer.as_ref() ) ;
    gl.use_program( Some( &self.program ) );
    gl::uniform::upload
    (
      &gl, 
      gl.get_uniform_location( &self.program, "projectionMatrix" ) ,
      projection_matrix.raw_slice()
    ).unwrap();

    let view_matrix_location =  gl.get_uniform_location( &self.program, "viewMatrix" );

    let transformations = 
    [
      gl::math::mat3x3h::loot_to_rh( gl::F32x3::ZERO, gl::F32x3::X, -gl::F32x3::Y ),
      gl::math::mat3x3h::loot_to_rh( gl::F32x3::ZERO, -gl::F32x3::X, -gl::F32x3::Y ),
      gl::math::mat3x3h::loot_to_rh( gl::F32x3::ZERO, gl::F32x3::Y, gl::F32x3::Z ),
      gl::math::mat3x3h::loot_to_rh( gl::F32x3::ZERO, -gl::F32x3::Y, -gl::F32x3::Z ),
      gl::math::mat3x3h::loot_to_rh( gl::F32x3::ZERO, gl::F32x3::Z, -gl::F32x3::Y ),
      gl::math::mat3x3h::loot_to_rh( gl::F32x3::ZERO, -gl::F32x3::Z, -gl::F32x3::Y ),
    ];

    for i in 0..6
    {

    }

    gl.bind_framebuffer( gl::FRAMEBUFFER, None ) ;
  }
}

const cube_vertices : [ f32; 108 ] =
[
  // front
  -1.0, -1.0,  1.0,
   1.0, -1.0,  1.0,
  -1.0,  1.0,  1.0,
 
  -1.0,  1.0,  1.0,
   1.0, -1.0,  1.0,
   1.0,  1.0,  1.0,
  // right
   1.0, -1.0,  1.0,
   1.0, -1.0, -1.0,
   1.0,  1.0,  1.0,
 
   1.0,  1.0,  1.0,
   1.0, -1.0, -1.0,
   1.0,  1.0, -1.0,
  // back
   1.0, -1.0, -1.0,
  -1.0, -1.0, -1.0,
   1.0,  1.0, -1.0,
 
   1.0,  1.0, -1.0,
  -1.0, -1.0, -1.0,
  -1.0,  1.0, -1.0,
  // left
  -1.0, -1.0, -1.0,
  -1.0, -1.0,  1.0,
  -1.0,  1.0, -1.0,
 
  -1.0,  1.0, -1.0,
  -1.0, -1.0,  1.0,
  -1.0,  1.0,  1.0,
  // top
   1.0,  1.0, -1.0,
  -1.0,  1.0, -1.0,
   1.0,  1.0,  1.0,
 
   1.0,  1.0,  1.0,
  -1.0,  1.0, -1.0,
  -1.0,  1.0,  1.0,
  // bottom
   1.0, -1.0,  1.0,
  -1.0, -1.0,  1.0,
   1.0, -1.0, -1.0,
 
   1.0, -1.0, -1.0,
  -1.0, -1.0,  1.0,
  -1.0, -1.0, -1.0,
];