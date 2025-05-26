use std::f32;
use minwebgl as gl;
use gl::
{
  AsBytes,
  math::d2::mat3x3h,
  BufferDescriptor,
  JsCast as _,
  WebGlBuffer,
  WebGlVertexArrayObject,
  WebglError,
  GL
};
use rand::Rng;
use web_sys::{ HtmlCanvasElement, WebGlTexture };

fn main()
{
  gl::browser::setup( Default::default() );
  gl::spawn_local( async move { gl::info!( "{:?}", run().await ) } );
  // gl::info!( "{:?}", run() );
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

async fn run() -> Result< (), gl::WebglError >
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

  // shaders
  let vert = include_str!( "../shaders/light_volume.vert" );
  let frag = include_str!( "../shaders/deferred.frag" );
  let light_shader = gl::shader::Program::new( gl.clone(), vert, frag )?;

  let vert = include_str!( "../shaders/main.vert" );
  let frag = include_str!( "../shaders/gbuffer.frag" );
  let object_shader = gl::shader::Program::new( gl.clone(), vert, frag )?;

  let vert = include_str!( "../shaders/big_triangle.vert" );
  let frag = include_str!( "../shaders/screen_texture.frag" );
  let screen_shader = gl::shader::Program::new( gl.clone(), vert, frag )?;

  // cube geometry
  let position_buffer = gl::buffer::create( &gl )?;
  gl::buffer::upload( &gl, &position_buffer, CUBE_VERTICES, GL::STATIC_DRAW );
  let index_buffer = gl::buffer::create( &gl )?;
  gl::index::upload( &gl, &index_buffer, CUBE_INDICES, GL::STATIC_DRAW );
  let position_attribute = AttributePointer::new( &gl, BufferDescriptor::new::< [ f32; 3 ] >(), position_buffer, 0 );
  let cube = Geometry::with_elements( &gl, position_attribute, index_buffer, CUBE_INDICES.len() as i32 )?;
  cube.activate();

  let projection = mat3x3h::perspective_rh_gl( 45.0f32.to_radians(), width as f32 / height as f32, 0.1, 500.0 );
  let wall_transform = mat3x3h::translation( [ 0.0f32, 0.0, -120.0 ] ) * mat3x3h::scale( [ 100.0, 100.0, 1.0 ] );

  // gbuffer
  let gbuffer = gl.create_framebuffer();
  let position_buffer = tex_storage_2d( &gl, GL::RGBA16F, width, height );
  let normal_buffer = tex_storage_2d( &gl, GL::RGBA16F, width, height );
  let depthbuffer = gl.create_renderbuffer();
  gl.bind_renderbuffer( GL::RENDERBUFFER, depthbuffer.as_ref() );
  gl.renderbuffer_storage( GL::RENDERBUFFER, GL::DEPTH_COMPONENT24, width, height );
  gl.bind_framebuffer( GL::FRAMEBUFFER, gbuffer.as_ref() );
  gl.framebuffer_texture_2d( GL::FRAMEBUFFER, GL::COLOR_ATTACHMENT0, GL::TEXTURE_2D, position_buffer.as_ref(), 0 );
  gl.framebuffer_texture_2d( GL::FRAMEBUFFER, GL::COLOR_ATTACHMENT1, GL::TEXTURE_2D, normal_buffer.as_ref(), 0 );
  gl.framebuffer_renderbuffer( GL::FRAMEBUFFER, GL::DEPTH_ATTACHMENT, GL::RENDERBUFFER, depthbuffer.as_ref() );

  // draw big wall into gbuffer once, because it is static
  gl::drawbuffers::drawbuffers( &gl, &[ GL::COLOR_ATTACHMENT0, GL::COLOR_ATTACHMENT1 ] );
  object_shader.activate();
  object_shader.uniform_matrix_upload( "u_model", wall_transform.raw_slice(), true );
  object_shader.uniform_matrix_upload( "u_mvp", ( projection * wall_transform ).raw_slice(), true );
  gl.vertex_attrib3f( 1, 0.0, 0.0, 1.0 );
  gl.draw_elements_with_i32( GL::TRIANGLES, cube.element_count, GL::UNSIGNED_INT, 0 );

  // use same framebuffer object for offscreen rendering
  let offscreen_buffer = gbuffer;
  let color_buffer = tex_storage_2d( &gl, GL::RGBA8, width, height );
  // attach color buffer to framebuffer
  gl.framebuffer_texture_2d( GL::FRAMEBUFFER, GL::COLOR_ATTACHMENT0, GL::TEXTURE_2D, color_buffer.as_ref(), 0 );
  // remove normal buffer from framebuffer
  gl.framebuffer_texture_2d( GL::FRAMEBUFFER, GL::COLOR_ATTACHMENT1, GL::TEXTURE_2D, None, 0 );

  let light_radius = 3.0;
  // grid of light sources
  let rows = 16;
  let columns = 28;
  // lights z position
  let z = -116.5;
  let light_instances = rows * columns;
  // random z offset for each light source
  let offsets = ( 0..light_instances )
  .map( | _ | rand::random::< f32 >() * f32::consts::PI * 2.0 )
  .collect::< Vec< _ > >();
  let light_colors = ( 0..light_instances )
  .map( | _ | random_rgb_color() )
  .collect::< Vec< _ > >();
  let light_radii = ( 0..light_instances )
  .map( | _ | light_radius + rand::random_range( -1.0..=3.0 ) )
  .collect::< Vec< _ > >();
  // positions of lights sources for instanced rendering
  let mut light_positions = generate_light_positions( columns, rows, light_radius, z );

  let light_position_buffer = gl::buffer::create( &gl )?;
  gl::buffer::upload( &gl, &light_position_buffer, light_positions.as_slice(), GL::DYNAMIC_DRAW );
  let translation_attribute = AttributePointer::new
  (
    &gl,
    BufferDescriptor::new::< [ f32; 3 ] >().divisor( 1 ),
    light_position_buffer.clone(),
    1
  );

  let light_radius_buffer = gl::buffer::create( &gl )?;
  gl::buffer::upload( &gl, &light_radius_buffer, light_radii.as_slice(), GL::STATIC_DRAW );
  let radius_attribute = AttributePointer::new
  (
    &gl,
    BufferDescriptor::new::< f32 >().divisor( 1 ),
    light_radius_buffer,
    2
  );

  // colors of lights sources for instanced rendering
  let light_color_buffer = gl::buffer::create( &gl )?;
  gl::buffer::upload( &gl, &light_color_buffer, light_colors.as_slice(), GL::STATIC_DRAW );
  let color_attribute = AttributePointer::new
  (
    &gl,
    BufferDescriptor::new::< [ f32; 3 ] >().divisor( 1 ),
    light_color_buffer,
    3
  );

  cube.add_attribute( translation_attribute )?;
  cube.add_attribute( radius_attribute )?;
  cube.add_attribute( color_attribute )?;

  // doesn't need to write to depth buffer anymore
  gl.depth_mask( false );

  let update = move | t |
  {
    let t = ( t / 1000.0 ) as f32;
    update_light_positions( &mut light_positions, t, &offsets, z, 2.0 );
    gl.bind_buffer( GL::ARRAY_BUFFER, Some( &light_position_buffer ) );
    gl.buffer_sub_data_with_i32_and_u8_array( GL::ARRAY_BUFFER, 0, light_positions.as_bytes() );

    gl.bind_framebuffer( GL::FRAMEBUFFER, offscreen_buffer.as_ref() );
    gl::drawbuffers::drawbuffers( &gl, &[ GL::COLOR_ATTACHMENT0 ] );
    gl.clear( gl::COLOR_BUFFER_BIT );

    gl.enable( GL::DEPTH_TEST );
    // inverse depth testing to discard fragments out of back bound of light volume
    gl.depth_func( GL::GEQUAL );
    gl.cull_face( GL::FRONT );
    // blending is needed when fragment is affected by several lights
    gl.enable( gl::BLEND );
    gl.blend_func( gl::ONE, gl::ONE );

    gl.active_texture( GL::TEXTURE0 );
    gl.bind_texture( GL::TEXTURE_2D, position_buffer.as_ref() );
    gl.active_texture( GL::TEXTURE1 );
    gl.bind_texture( GL::TEXTURE_2D, normal_buffer.as_ref() );

    light_shader.activate();
    light_shader.uniform_matrix_upload( "u_mvp", projection.raw_slice(), true );
    light_shader.uniform_upload( "u_screen_size", [ width as f32, height as f32 ].as_slice() );
    light_shader.uniform_upload( "u_positions", &0 );
    light_shader.uniform_upload( "u_normals", &1 );
    gl.draw_elements_instanced_with_i32( GL::TRIANGLES, cube.element_count, GL::UNSIGNED_INT, 0, light_instances );

    // show on screen
    gl.bind_framebuffer( GL::FRAMEBUFFER, None );
    gl.disable( GL::DEPTH_TEST );
    gl.cull_face( GL::BACK );
    gl.disable( gl::BLEND );
    gl.active_texture( GL::TEXTURE0 );
    gl.bind_texture( GL::TEXTURE_2D, color_buffer.as_ref() );
    screen_shader.activate();
    gl.draw_arrays( GL::TRIANGLES, 0, 3 );

    true
  };
  gl::exec_loop::run( update );

  Ok( () )
}

fn random_rgb_color() -> [ f32; 3 ]
{
  let mut rng = rand::rng();

  let mut r = if rng.random_bool( 0.5 ) { 1.0 } else { 0.0 };
  let mut g = if rng.random_bool( 0.5 ) { 1.0 } else { 0.0 };
  let mut b = if rng.random_bool( 0.5 ) { 1.0 } else { 0.0 };

  if r == 0.0 && g == 0.0 && b == 0.0
  {
    match rng.random_range(0..3)
    {
      0 => r = 1.0,
      1 => g = 1.0,
      2 => b = 1.0,
      _ => {},
    }
  }

  [ r, g, b ]
}

fn update_light_positions( positions : &mut[ f32 ], time : f32, offsets : &[ f32 ], z : f32, amplitude : f32 )
{
  positions.chunks_exact_mut( 3 ).zip( offsets ).for_each
  (
    | ( position, offset ) | position[ 2 ] = z + ( time * f32::consts::PI * 0.3 + offset ).sin() * amplitude
  );
}

fn generate_light_positions( cols : i32, rows : i32, padding : f32, z : f32 ) -> Vec< f32 >
{
  // generate grid of light
  let spacing = padding * 2.0;
  let width = spacing * ( cols - 1 ) as f32;
  let height = spacing * ( rows - 1 ) as f32;
  let x = -width / 2.0;
  let mut y = -height / 2.0;
  let mut ret = vec![];

  for _ in 0..rows
  {
    let mut x = x;
    for _ in 0..cols
    {
      ret.push( x );
      ret.push( y );
      ret.push( z );
      x += spacing;
    }
    y += spacing;
  }

  ret
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
  gl::texture::d2::filter_nearest( gl );
  Some( tex )
}
