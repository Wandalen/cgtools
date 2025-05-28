mod elliptical_orbit;

use std::f32;
use elliptical_orbit::EllipticalOrbit;
use minwebgl as gl;
use gl::
{
  F32x3,
  math::d2::mat3x3h,
  BufferDescriptor,
  WebGlBuffer,
  WebGlVertexArrayObject,
  WebglError,
  GL,
  AsBytes as _,
  JsCast as _,
};
use web_sys::{ HtmlCanvasElement, WebGlFramebuffer, WebGlTexture };

fn main()
{
  gl::browser::setup( Default::default() );
  gl::spawn_local( async move { gl::info!( "{:?}", run().await ) } );
}

async fn run() -> Result< (), gl::WebglError >
{
  let gl = gl::context::retrieve_or_make().expect( "Failed to retrieve WebGl context" );
  let res = gl.get_extension( "EXT_color_buffer_float" ).unwrap().unwrap();
  gl::info!( "{}", res.to_string() );
  let canvas = gl.canvas().unwrap().dyn_into::< HtmlCanvasElement >().unwrap();
  let width = canvas.width() as i32;
  let height = canvas.height() as i32;
  let aspect = width as f32 / height as f32;

  gl.viewport( 0, 0, width, height );
  gl.enable( GL::DEPTH_TEST );
  gl.enable( GL::CULL_FACE );
  gl.blend_func( gl::ONE, gl::ONE );
  gl.clear_color( 0.0, 0.0, 0.0, 1.0 );

  let meshes = load_glb( &gl ).await.unwrap();

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

  let projection = mat3x3h::perspective_rh_gl( 65.0f32.to_radians(), aspect, 0.1, 1000.0 );
  let rotation = mat3x3h::rot( 10.0f32.to_radians(), 0.0, 0.0 )
  * mat3x3h::rot( 0.0, 90.0f32.to_radians(), 0.0 );
  let scale = 0.1;
  let scene_transform = mat3x3h::translation( [ 0.0f32, -40.0, -90.0 ] )
  * rotation * mat3x3h::scale( [ scale, scale, scale ] );

  // gbuffer
  let ( gbuffer, position_gbuffer, normal_gbuffer, color_gbuffer ) = gbuffer( &gl, width, height );

  // render data to gbuffer for once
  gl::drawbuffers::drawbuffers( &gl, &[ 0, 1, 2 ] );
  object_shader.activate();
  object_shader.uniform_matrix_upload( "u_model", scene_transform.raw_slice(), true );
  object_shader.uniform_matrix_upload( "u_rotation", rotation.raw_slice(), true );
  object_shader.uniform_matrix_upload( "u_mvp", ( projection * scene_transform ).raw_slice(), true );
  for mesh in &meshes
  {
    mesh.0.activate();
    gl.bind_texture( GL::TEXTURE_2D, mesh.1.as_ref() );
    gl.draw_elements_with_i32( GL::TRIANGLES, mesh.0.element_count, GL::UNSIGNED_INT, 0 );
  }
  gl.bind_vertex_array( None );

  // reuse same framebuffer object for offscreen rendering
  // use offscreen framebuffer to be able to use values from depthbuffer
  let offscreen_buffer = gbuffer;
  let color_buffer = tex_storage_2d( &gl, GL::RGBA8, width, height );
  // remove gbuffer attachments, and attach new one for colors
  gl.framebuffer_texture_2d( GL::FRAMEBUFFER, GL::COLOR_ATTACHMENT0, GL::TEXTURE_2D, color_buffer.as_ref(), 0 );
  gl.framebuffer_texture_2d( GL::FRAMEBUFFER, GL::COLOR_ATTACHMENT1, GL::TEXTURE_2D, None, 0 );
  gl.framebuffer_texture_2d( GL::FRAMEBUFFER, GL::COLOR_ATTACHMENT2, GL::TEXTURE_2D, None, 0 );

  let light_volume = light_volume( &gl )?;
  let light_instances = 300 + 1;
  let light_radius = 12.0;
  generate_light_positions( 0, 0, light_radius, 0.0 );

  let light_orbits = ( 0..( light_instances  - 1 ) ).map
  (
    | _ |
    EllipticalOrbit
    {
      center : F32x3::new
      (
        rand::random_range( -30.0..=30.0 ),
        rand::random_range( -30.0..=30.0 ),
        rand::random_range( -110.0..=-70.0 )
      ),
      ..EllipticalOrbit::random()
    }
  ).collect::< Vec< _ > >();
  let offsets = ( 0..( light_instances - 1 ) )
  .map( | _ | rand::random_range( 0.0..=( f32::consts::PI * 2.0 ) ) )
  .collect::< Vec< _ > >();
  let mut light_colors = ( 0..( light_instances - 1 ) )
  .map( | _ | random_rgb_color() )
  .collect::< Vec< _ > >();
  light_colors.push( [ 0.5, 0.5, 0.5 ] );
  let mut light_radii = ( 0..( light_instances - 1 ) )
  .map( | _ | light_radius + rand::random_range( -1.0..=5.0 ) )
  .collect::< Vec< _ > >();
  light_radii.push( 100.0 );

  // generate buffers with light data for instanced rendering
  let light_position_buffer = gl::buffer::create( &gl )?;
  gl.bind_buffer( GL::ARRAY_BUFFER, Some( &light_position_buffer ) );
  gl.buffer_data_with_i32( GL::ARRAY_BUFFER, light_instances * 3 * size_of::< f32 >() as i32, GL::DYNAMIC_DRAW );
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
  let light_color_buffer = gl::buffer::create( &gl )?;
  gl::buffer::upload( &gl, &light_color_buffer, light_colors.as_slice(), GL::STATIC_DRAW );
  let color_attribute = AttributePointer::new
  (
    &gl,
    BufferDescriptor::new::< [ f32; 3 ] >().divisor( 1 ),
    light_color_buffer,
    3
  );

  light_volume.add_attribute( translation_attribute )?;
  light_volume.add_attribute( radius_attribute )?;
  light_volume.add_attribute( color_attribute )?;

  gl.bind_vertex_array( None );

  // doesn't need to write to depth buffer anymore
  gl.depth_mask( false );

  let update = move | t |
  {
    let t = ( t / 1000.0 ) as f32;

    // update light positions
    let mut pos = light_orbits
    .iter()
    .zip( offsets.iter() )
    .map( | ( orbit, offset ) | orbit.position_at_angle( 0.2 * t + *offset ).0 )
    .collect::< Vec< _ > >();
    pos.push( [ 0.0, 0.0, -90.0 ] );
    gl.bind_buffer( GL::ARRAY_BUFFER, Some( &light_position_buffer ) );
    gl.buffer_sub_data_with_i32_and_u8_array( GL::ARRAY_BUFFER, 0, pos.as_bytes() );

    gl.bind_framebuffer( GL::FRAMEBUFFER, offscreen_buffer.as_ref() );
    gl::drawbuffers::drawbuffers( &gl, &[ 0 ] );
    gl.clear( gl::COLOR_BUFFER_BIT );

    gl.enable( GL::DEPTH_TEST );
    // inverse depth testing to discard fragments out of back bound of light volume
    gl.depth_func( GL::GEQUAL );
    gl.cull_face( GL::FRONT );
    // blending is needed when fragment is affected by several lights
    gl.enable( gl::BLEND );

    gl.active_texture( GL::TEXTURE0 );
    gl.bind_texture( GL::TEXTURE_2D, position_gbuffer.as_ref() );
    gl.active_texture( GL::TEXTURE1 );
    gl.bind_texture( GL::TEXTURE_2D, normal_gbuffer.as_ref() );
    gl.active_texture( GL::TEXTURE2 );
    gl.bind_texture( GL::TEXTURE_2D, color_gbuffer.as_ref() );

    light_shader.activate();
    light_shader.uniform_matrix_upload( "u_mvp", projection.raw_slice(), true );
    light_shader.uniform_upload( "u_screen_size", [ width as f32, height as f32 ].as_slice() );
    light_shader.uniform_upload( "u_positions", &0 );
    light_shader.uniform_upload( "u_normals", &1 );
    light_shader.uniform_upload( "u_colors", &2 );
    light_volume.activate();
    gl.draw_elements_instanced_with_i32
    (
      GL::TRIANGLES,
      light_volume.element_count,
      GL::UNSIGNED_INT,
      0,
      light_instances
    );
    gl.bind_vertex_array( None );

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

async fn load_glb( gl : &GL ) -> Result< Vec< ( Geometry, Option< WebGlTexture > ) >, gl::WebglError >
{
  let glb = gl::file::load( "sponza.glb" ).await.unwrap();
  let ( document, buffers, images ) = gltf::import_slice( &glb ).unwrap();
  let mut primitives = vec![];

  for mesh in document.meshes()
  {
    for primitive in mesh.primitives()
    {
      let reader = primitive.reader( | buffer | Some( &buffers[ buffer.index() ] ) );

      let Some( positions ) = reader.read_positions() else { continue; };
      let Some( normals ) = reader.read_normals() else { continue; };
      let Some( tex_coords ) = reader.read_tex_coords( 0 ) else { continue; };
      let Some( indices ) = reader.read_indices() else { continue; };
      let Some( material ) = primitive.material().pbr_metallic_roughness().base_color_texture()
      else { continue; };

      let positions : Vec< [ f32; 3 ] > = positions.collect();
      let position_buffer = gl::buffer::create( gl )?;
      gl::buffer::upload( gl, &position_buffer, &positions, GL::STATIC_DRAW );
      let position_attribute = AttributePointer::new( &gl, BufferDescriptor::new::< [ f32; 3 ] >(), position_buffer, 0 );

      let normals : Vec< [ f32; 3 ] > = normals.collect();
      let normal_buffer = gl::buffer::create( gl )?;
      gl::buffer::upload( gl, &normal_buffer, &normals, GL::STATIC_DRAW );
      let normal_attribute = AttributePointer::new( &gl, BufferDescriptor::new::< [ f32; 3 ] >(), normal_buffer, 1 );

      let tex_coords : Vec< [ f32; 2 ] > = tex_coords.into_f32().collect();
      let tex_coord_buffer = gl::buffer::create( gl )?;
      gl::buffer::upload( gl, &tex_coord_buffer, &tex_coords, GL::STATIC_DRAW );
      let tex_coord_attribute = AttributePointer::new( &gl, BufferDescriptor::new::< [ f32; 2 ] >(), tex_coord_buffer, 2 );

      let indices : Vec< u32 > = indices.into_u32().collect();
      let index_buffer = gl::buffer::create( gl )?;
      gl::index::upload( gl, &index_buffer, &indices, GL::STATIC_DRAW );

      let geometry = Geometry::with_elements( gl, position_attribute, index_buffer, indices.len() as i32 )?;
      geometry.add_attribute( normal_attribute )?;
      geometry.add_attribute( tex_coord_attribute )?;
      gl.bind_vertex_array( None );

      let texture = material.texture();
      let source = texture.source();
      let image = &images[ source.index() ];

      let mut rgba_pixels;

      // Convert RGB images to RGBA because WebGL2 does not generate mipmaps for SRGB8 textures, only for SRGB8_ALPHA8
      let pixels = match image.format
      {
        gltf::image::Format::R8G8B8 =>
        {
          rgba_pixels = Vec::with_capacity( ( image.width * image.height * 4 ) as usize );
          image.pixels.chunks_exact( 3 ).for_each
          (
            | rgb |
            {
              rgba_pixels.push( rgb[ 0 ] );
              rgba_pixels.push( rgb[ 1 ] );
              rgba_pixels.push( rgb[ 2 ] );
              rgba_pixels.push( 255 );
            }
          );
          rgba_pixels.as_slice()
        },
        gltf::image::Format::R8G8B8A8 => image.pixels.as_slice(),
        _ => continue,
      };

      let base_color_tex = gl.create_texture();
      gl.bind_texture( GL::TEXTURE_2D , base_color_tex.as_ref() );
      gl.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_u8_array_and_src_offset
      (
        GL::TEXTURE_2D,
        0,
        GL::SRGB8_ALPHA8 as i32,
        image.width as i32,
        image.height as i32,
        0,
        GL::RGBA,
        GL::UNSIGNED_BYTE,
        pixels,
        0
      ).unwrap();
      gl.generate_mipmap( GL::TEXTURE_2D );
      gl.tex_parameteri( GL::TEXTURE_2D, GL::TEXTURE_MIN_FILTER, GL::LINEAR_MIPMAP_LINEAR as i32 );
      gl.tex_parameteri( GL::TEXTURE_2D, GL::TEXTURE_MAG_FILTER, GL::LINEAR as i32 );

      primitives.push( ( geometry, base_color_tex ) );
    }
  }

  Ok( primitives )
}

fn gbuffer( gl : &GL, width : i32, height : i32 )
-> ( Option< WebGlFramebuffer >, Option< WebGlTexture >, Option< WebGlTexture >, Option< WebGlTexture > )
{
  // Just create gbuffer with positions, normals, and colors
  let gbuffer = gl.create_framebuffer();
  let position_gbuffer = tex_storage_2d( gl, GL::RGBA16F, width, height );
  let normal_gbuffer = tex_storage_2d( gl, GL::RGBA16F, width, height );
  let color_gbuffer = tex_storage_2d( gl, GL::RGBA8, width, height );
  let depthbuffer = gl.create_renderbuffer();
  gl.bind_renderbuffer( GL::RENDERBUFFER, depthbuffer.as_ref() );
  gl.renderbuffer_storage( GL::RENDERBUFFER, GL::DEPTH_COMPONENT24, width, height );

  gl.bind_framebuffer( GL::FRAMEBUFFER, gbuffer.as_ref() );
  gl.framebuffer_texture_2d( GL::FRAMEBUFFER, GL::COLOR_ATTACHMENT0, GL::TEXTURE_2D, position_gbuffer.as_ref(), 0 );
  gl.framebuffer_texture_2d( GL::FRAMEBUFFER, GL::COLOR_ATTACHMENT1, GL::TEXTURE_2D, normal_gbuffer.as_ref(), 0 );
  gl.framebuffer_texture_2d( GL::FRAMEBUFFER, GL::COLOR_ATTACHMENT2, GL::TEXTURE_2D, color_gbuffer.as_ref(), 0 );
  gl.framebuffer_renderbuffer( GL::FRAMEBUFFER, GL::DEPTH_ATTACHMENT, GL::RENDERBUFFER, depthbuffer.as_ref() );
  ( gbuffer, position_gbuffer, normal_gbuffer, color_gbuffer )
}

fn light_volume( gl : &GL ) -> Result< Geometry, WebglError >
{
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

  // Create cube mesh to use as light volume
  let position_buffer = gl::buffer::create( gl )?;
  gl::buffer::upload( gl, &position_buffer, CUBE_VERTICES, GL::STATIC_DRAW );
  let index_buffer = gl::buffer::create( gl )?;
  gl::index::upload( gl, &index_buffer, CUBE_INDICES, GL::STATIC_DRAW );
  let position_attribute = AttributePointer::new( gl, BufferDescriptor::new::< [ f32; 3 ] >(), position_buffer, 0 );
  let light_volume = Geometry::with_elements( gl, position_attribute, index_buffer, CUBE_INDICES.len() as i32 )?;
  Ok( light_volume )
}

fn random_rgb_color() -> [ f32; 3 ]
{
  let mut r = rand::random_bool( 0.5 ) as u8 as f32;
  let mut g = rand::random_bool( 0.5 ) as u8 as f32;
  let mut b = rand::random_bool( 0.5 ) as u8 as f32;

  if r == 0.0 && g == 0.0 && b == 0.0
  {
    match rand::random_range( 0..3 )
    {
      0 => r = 1.0,
      1 => g = 1.0,
      2 => b = 1.0,
      _ => {},
    }
  }

  [ r, g, b ]
}

fn tex_storage_2d( gl : &GL, format : u32, width : i32, height : i32 ) -> Option< WebGlTexture >
{
  let tex = gl.create_texture()?;
  gl.bind_texture( GL::TEXTURE_2D, Some( &tex ) );
  gl.tex_storage_2d( GL::TEXTURE_2D, 1, format, width, height );
  gl::texture::d2::filter_nearest( gl );
  Some( tex )
}

fn generate_light_positions( _ : i32, _ : i32, _ : f32, _ : f32 ) {}

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
