use minwebgl::{self as gl, F32x4x4, JsValue};
use gl::
{
  GL,
  web_sys::
  {
    WebGl2RenderingContext,
    WebGlProgram,
    WebGlUniformLocation,
    WebGlBuffer,
    WebGlTexture,
    WebGlVertexArrayObject,
    WebGlFramebuffer
  }
};
use ndarray_cg::
{
  Mat4,
  F32x4x4,
  F32x4,
  F32x3
};
use rand::Rng;
use std::collections::HashMap;
use bevy::prelude::
{
  Mesh,
  Torus,
  Cone,
  Cylinder,
  Sphere,
  Cuboid,
  Capsule3d
};
use bevy::render::mesh::VertexAttributeValues;

/// Creates a WebGL2 texture.
///
/// # Arguments
///
/// * `gl` - The WebGL2 rendering context.
/// * `slot` - The texture unit to activate and bind to ( e.g., `GL::TEXTURE0` ).
/// * `size` - The size of the texture ( width, height ).
/// * `internal_format` - The internal format of the texture ( e.g., `GL::RGBA8` ).
/// * `format` - The format of the pixel data ( e.g., `GL::RGBA` ).
/// * `pixel_type` - The data type of the pixel data ( e.g., `GL::UNSIGNED_BYTE` ).
/// * `data` - Optional initial pixel data.
///
/// # Returns
///
/// An `Option< WebGlTexture >` containing the created texture, or `None` if creation fails.
fn create_texture
(
  gl : &gl::WebGl2RenderingContext,
  slot : u32,
  size : ( i32, i32 ),
  internal_format : i32,
  format : u32,
  pixel_type : u32,
  data : Option< &[ u8 ] >
) 
-> Option< WebGlTexture >
{
  let Some( texture ) = gl.create_texture() 
  else 
  {
    return None;
  };
  gl.active_texture( slot );
  gl.bind_texture( GL::TEXTURE_2D, Some( &texture ) );
  // Used to upload data.
  gl.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_u8_array
  (
    GL::TEXTURE_2D,  // target
    0,               // level
    internal_format, 
    size.0,         
    size.1,          
    0,               // border
    format,         
    pixel_type,     
    data,            // pixels data
  )
  .unwrap();
  gl.bind_texture( GL::TEXTURE_2D, None );
  Some( texture )
}

/// Binds a texture to a texture unit and uploads its location to a uniform.
///
/// # Arguments
///
/// * `gl` - The WebGL2 rendering context.
/// * `texture` - The texture to bind.
/// * `location` - The uniform location in the shader for the sampler.
/// * `slot` - The texture unit to bind to ( e.g., `GL::TEXTURE0` ).
fn upload_texture
(
  gl : &gl::WebGl2RenderingContext,
  texture : &WebGlTexture,
  location : &WebGlUniformLocation,
  slot : u32,
)
{
  gl.active_texture( slot ); 
  gl.bind_texture( GL::TEXTURE_2D, Some( &texture ) ); 
  // Tell the sampler uniform in the shader which texture unit to use ( 0 for GL_TEXTURE0, 1 for GL_TEXTURE1, etc. )
  gl.uniform1i( Some( location ), ( slot - GL::TEXTURE0 ) as i32 );
}

/// Creates a WebGL2 framebuffer and a color attachment texture.
///
/// # Arguments
///
/// * `gl` - The WebGL2 rendering context.
/// * `size` - The size of the framebuffer and its attachment ( width, height ).
/// * `color_attachment` - The index of the color attachment point ( e.g., 0 for `GL::COLOR_ATTACHMENT0` ).
///
/// # Returns
///
/// An `Option< ( WebGlFramebuffer, WebGlTexture ) >` containing the created framebuffer and
/// its color attachment texture, or `None` if creation fails.
fn create_framebuffer
(
  gl : &gl::WebGl2RenderingContext,
  size : ( i32, i32 ),
  color_attachment : u32
) 
-> Option< ( WebGlFramebuffer, WebGlTexture ) >
{
  let color = gl.create_texture()?;
  gl.bind_texture( GL::TEXTURE_2D, Some( &color ) );
  // Use tex_storage_2d for immutable texture storage ( WebGL2 )
  gl.tex_storage_2d( GL::TEXTURE_2D, 1, gl::RGBA8, size.0, size.1 );
  // Configure texture parameters (filtering, wrapping)
  gl.tex_parameteri( GL::TEXTURE_2D, GL::TEXTURE_MIN_FILTER, GL::LINEAR as i32 );
  gl.tex_parameteri( GL::TEXTURE_2D, GL::TEXTURE_WRAP_S, GL::REPEAT as i32 );
  gl.tex_parameteri( GL::TEXTURE_2D, GL::TEXTURE_WRAP_T, GL::REPEAT as i32 );

  let framebuffer = gl.create_framebuffer()?;
  gl.bind_framebuffer( GL::FRAMEBUFFER, Some( &framebuffer ) );
  // Attach the texture to the framebuffer's color attachment point
  gl.framebuffer_texture_2d( GL::FRAMEBUFFER, GL::COLOR_ATTACHMENT0 + color_attachment, GL::TEXTURE_2D, Some( &color ), 0 );
  gl.bind_framebuffer( gl::FRAMEBUFFER, None );

  Some( ( framebuffer, color ) )
}

/// Binds a framebuffer for rendering and sets the viewport to its size.
///
/// # Arguments
///
/// * `gl` - The WebGL2 rendering context.
/// * `framebuffer` - The framebuffer to bind.
/// * `size` - The size of the framebuffer ( width, height ).
fn upload_framebuffer(
  gl : &gl::WebGl2RenderingContext,
  framebuffer : &WebGlFramebuffer,
  size : ( i32, i32 )
)
{
  gl.bind_framebuffer( GL::FRAMEBUFFER, Some( framebuffer ) );
  gl.viewport( 0, 0, size.0, size.1 );
}

/// Recursively collects mesh data ( positions and indices ) from a GLTF node and its children.
/// Transforms vertex positions using the current node's transform combined with parent transforms.
///
/// # Arguments
///
/// * `node` - The current GLTF node to process.
/// * `buffers` - A slice of GLTF buffer data.
/// * `parent_transform` - The accumulated transformation matrix from parent nodes.
/// * `positions` - A mutable vector to accumulate vertex positions. Transformed positions are added here.
/// * `indices` - A mutable vector to accumulate indices. Indices are adjusted by the current vertex offset.
/// * `vertex_offset` - A mutable counter to keep track of the total number of vertices processed so far.
///                     Used to correctly offset indices for the current mesh.
fn gltf_data
(
  node : &gltf::Node,
  buffers : &[ gltf::buffer::Data ],
  parent_transform : F32x4x4,
  positions : &mut Vec< [ f32; 3 ] >,
  indices : &mut Vec< u32 >,
  vertex_offset : &mut u32
)
{
  // Get the node's local transformation matrix
  let transform = node.transform().matrix();
  let mut transform_raw : [ f32; 16 ] = [ 0.0; 16 ];
  for ( i, r ) in transform_raw.chunks_mut( 4 ).enumerate()
  {
    r[ 0 ] = transform[ i ][ 0 ];
    r[ 1 ] = transform[ i ][ 1 ];
    r[ 2 ] = transform[ i ][ 2 ];
    r[ 3 ] = transform[ i ][ 3 ];
  }

  let local_transform : F32x4x4 = Mat4::from_column_major( &transform_raw );

  // Combine parent transform with local transform
  let current_transform = parent_transform * local_transform;

  // If the node has a mesh, process its primitives
  if let Some( mesh ) = node.mesh()
  {
    for primitive in mesh.primitives()
    {
      let reader = primitive.reader( | buffer | Some( &buffers[ buffer.index() ] ) );

      // Read and transform positions
      if let Some( positions_iter ) = reader.read_positions()
      {
        let mut current_primitive_positions : Vec< [ f32; 3 ] > = Vec::new();

        for p in positions_iter
        {
          let pos_vec = F32x4::from_array( [ p[ 0 ], p[ 1 ], p[ 2 ], 1.0 ] );
          // Apply combined transform to vertex position
          let tp = current_transform * pos_vec;
          current_primitive_positions.push( [ tp[ 0 ], tp[ 1 ], tp[ 2 ] ].into() );
        }

        let num_current_vertices = current_primitive_positions.len();

        // Add transformed positions to the global list
        positions.extend( current_primitive_positions );

        // Read and adjust indices
        if let Some( indices_iter ) = reader.read_indices()
        {
          for index in indices_iter.into_u32()
          {
             // Add the current vertex offset to each index
             indices.push( index + *vertex_offset );
          }
        }

        // Update the vertex offset for the next mesh/primitive
        *vertex_offset += num_current_vertices as u32;
      }
    }
  }

  // Recursively process child nodes
  for child in node.children()
  {
    gltf_data
    (
      &child,
      buffers,
      current_transform, // Pass the current combined transform down
      positions,
      indices,
      vertex_offset
    );
  }
}

/*
fn primitives_data
(
  positions : &mut Vec< [ f32; 3 ] >,
  indices : &mut Vec< u32 >,
  vertex_offset : &mut u32
)
{
  let meshes : Vec< Mesh > = 
  vec![
    Cone::default().into(),
    Torus::default().into(),
    Cylinder::default().into(),
    Sphere::default().into(),
    Cuboid::default().into(),
    Capsule3d::default().into()
  ];

  let ranges = 
  [
    ( 0..3, -3.0..3.0 ),
    ( 3..6, 0.0..360.0 ),
    ( 6..9, 0.75..1.0 )
  ];

  let transforms : Vec< [ f32; 9 ]  > = ( 0..( meshes.len() ) )
  .into_iter()
  .map( 
    | _ | 
    {
      let mut t = [ 0.0; 9 ]; 

      for ( indices, values ) in &ranges
      {
        for i in indices.clone()
        {
          t[ i ] = rand::thread_rng().gen_range( values.clone() );
        }
      }

      t
    }
  )
  .collect::< Vec< _ > >();

  let primitives = meshes.iter().zip( transforms.iter() );

  for ( p, t ) in primitives
  {
    let mut transform = bevy::prelude::Transform::IDENTITY
    .with_rotation( bevy::prelude::Quat::from_euler( bevy::prelude::EulerRot::XYZ, t[ 3 ], t[ 4 ], t[ 5 ] ) )
    .with_scale( glam::Vec3::new( t[ 6 ], t[ 7 ], t[ 8 ] ) )
    .compute_matrix();

    let mut transform_raw : [ f32; 16 ] = [ 0.0; 16 ];
    for ( i, r ) in transform_raw.chunks_mut( 4 ).enumerate()
    {
      let row = transform.row( i );
      for j in 0..4
      {
        r[ j ] = row[ j ];
      }
    }

    transform_raw[ 12 ] = t[ 0 ];
    transform_raw[ 13 ] = t[ 1 ];
    transform_raw[ 14 ] = t[ 2 ];

    let local_transform : F32x4x4 = Mat4::from_column_major( &transform_raw );

    let mesh = p;
    let Some( VertexAttributeValues::Float32x3( primitive_positions ) ) = p.attribute( Mesh::ATTRIBUTE_POSITION )
    else 
    {
      return;
    };
    let vertices_count = mesh.count_vertices();

    // Add transformed positions to the global list
    let primitive_positions = primitive_positions
    .iter()
    .map( 
      | p | 
      {
        local_transform * 
        F32x4::from_array( 
          [ p[ 0 ], p[ 1 ], p[ 2 ], 1.0 ] 
        ) 
      }
    )
    .map( | p | [ p[ 0 ], p[ 1 ], p[ 2 ] ] )
    .collect::< Vec< _ > >();
    positions.extend( primitive_positions );

    // Read and adjust indices
    if let Some( primitive_indices ) = mesh.indices()
    {
      for index in primitive_indices.iter()
      {
          // Add the current vertex offset to each index
          indices.push( index as u32 + *vertex_offset );
      }
    }

    *vertex_offset += vertices_count as u32;
  }
}
*/

/// Represents the camera's view and projection settings.
struct Camera
{
  eye : F32x3,
  up : F32x3,
  projection : F32x4x4,
  model : F32x4x4
}

/// Manages WebGL resources and rendering passes.
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
  draw_count : i32 // Number of indices to draw for the object
}

impl Renderer
{
  /// Creates a new Renderer instance, initializes WebGL, loads resources,
  /// and prepares the scene for rendering.
  async fn new() -> Self
  {
    gl::browser::setup( Default::default() );
    let gl = gl::context::retrieve_or_make().unwrap();

    // --- Initialization ---

    let viewport = ( gl.drawing_buffer_width(), gl.drawing_buffer_height() );

    // Camera setup (initial position, up vector, projection matrix, initial model matrix)
    let eye = F32x3::from_array( [  0.0, 1.4, 2.5 ] ) * 1.5;
    let up = F32x3::Y;

    let aspect_ratio = viewport.0 as f32 / viewport.1 as f32;
    let u_projection = ndarray_cg::mat3x3h::perspective_rh_gl
    (
      70.0f32.to_radians(), 
      aspect_ratio, 
      0.1, 
      1000.0
    );
    let u_model = Mat4::IDENTITY;
    let u_model : F32x4x4 = Mat4::from_column_major( u_model.to_cols_array() );

    let camera = Camera{
      eye,
      up,
      projection : u_projection,
      model : u_model // Note: The actual object model transformation is handled in gltf_data and animated view in object_pass
    };

    // Create and store renderer instance
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

    // --- Load and Compile Shaders ---

    let object_vs_src = include_str!( "../resources/shaders/object.vert" );
    let object_fs_src = include_str!( "../resources/shaders/object.frag" );
    let fullscreen_vs_src = include_str!( "../resources/shaders/fullscreen.vert" );
    let jfa_init_fs_src = include_str!( "../resources/shaders/jfa_init.frag" );
    let jfa_step_fs_src = include_str!( "../resources/shaders/jfa_step.frag" );
    let outline_fs_src = include_str!( "../resources/shaders/outline.frag" );

    // Compile and link shader programs and store them
    let object_program = gl::ProgramFromSources::new( object_vs_src, object_fs_src ).compile_and_link( gl ).unwrap();
    let jfa_init_program = gl::ProgramFromSources::new( fullscreen_vs_src, jfa_init_fs_src ).compile_and_link( gl ).unwrap();
    let jfa_step_program = gl::ProgramFromSources::new( fullscreen_vs_src, jfa_step_fs_src ).compile_and_link( gl ).unwrap();
    let outline_program = gl::ProgramFromSources::new( fullscreen_vs_src, outline_fs_src ).compile_and_link( gl ).unwrap();

    renderer.programs.insert( "object".to_string(), object_program );
    renderer.programs.insert( "jfa_init".to_string(), jfa_init_program );
    renderer.programs.insert( "jfa_step".to_string(), jfa_step_program );
    renderer.programs.insert( "outline".to_string(), outline_program );

    // --- Create Framebuffers and Textures ---

    // Framebuffer for rendering the initial object silhouette
    let ( object_fb, object_fb_color ) = create_framebuffer( gl, viewport, 0 ).unwrap();
    // Framebuffer for the JFA initialization pass
    let ( jfa_init_fb, jfa_init_fb_color ) = create_framebuffer( gl, viewport, 0 ).unwrap();
    // Framebuffers for the JFA step passes ( ping-pong )
    let ( jfa_step_fb_0, jfa_step_fb_color_0 ) = create_framebuffer( gl, viewport, 0 ).unwrap();
    let ( jfa_step_fb_1, jfa_step_fb_color_1 ) = create_framebuffer( gl, viewport, 0 ).unwrap();

    // Store the color attachment textures
    renderer.textures.insert( "object_fb_color".to_string(), object_fb_color );
    renderer.textures.insert( "jfa_init_fb_color".to_string(), jfa_init_fb_color );
    renderer.textures.insert( "jfa_step_fb_color_0".to_string(), jfa_step_fb_color_0 );
    renderer.textures.insert( "jfa_step_fb_color_1".to_string(), jfa_step_fb_color_1 );

    // Store the framebuffers
    renderer.framebuffers.insert( "object_fb".to_string(), object_fb );
    renderer.framebuffers.insert( "jfa_init_fb".to_string(), jfa_init_fb );
    renderer.framebuffers.insert( "jfa_step_fb_0".to_string(), jfa_step_fb_0 );
    renderer.framebuffers.insert( "jfa_step_fb_1".to_string(), jfa_step_fb_1 );

    // --- Create and Upload Mesh Data ---

    // Create GPU buffers and a Vertex Array Object ( VAO )
    let pos_buffer =  gl::buffer::create( gl ).unwrap();
    let index_buffer = gl::buffer::create( gl ).unwrap();
    let vao = gl::vao::create( gl ).unwrap();

    renderer.buffers.insert( "pos_buffer".to_string(), pos_buffer.clone() );
    renderer.buffers.insert( "index_buffer".to_string(), index_buffer.clone() );
    renderer.vaos.insert( "vao".to_string(), vao.clone() );

    // Load the GLTF model file
    let obj_buffer = gl::file::load( "model.glb" ).await.expect( "Failed to load the model" );
    let ( document, buffers, _ ) = gltf::import_slice( &obj_buffer[ .. ] ).expect( "Failed to parse the glb file" );

    let mut positions : Vec< [ f32; 3 ] > = vec![];
    let mut indices : Vec< u32 > = vec![];

    // Process the default scene in the GLTF document
    {
      let scene = document.default_scene().unwrap();
      let mut vertex_offset : u32 = 0; // Counter for correct index offsetting
      for node in scene.nodes()
      {
        // Recursively collect mesh data from the scene graph
        gltf_data
        (
          &document.nodes().nth( node.index() ).expect( "Node not found" ), 
          &buffers, // GLTF buffer data
          u_model, // Initial model transform ( applied to the root node's children )
          &mut positions, 
          &mut indices,  
          &mut vertex_offset // Output counter for vertex offset
        );
      }
      
      primitives_data
      (
        &mut positions, 
        &mut indices,  
        &mut vertex_offset
      );

      renderer.draw_count = indices.len() as i32; // Store the total number of indices to draw
    }

    gl::buffer::upload( &gl, &pos_buffer, &positions, GL::STATIC_DRAW );
    gl::index::upload( &gl, &index_buffer, &indices, GL::STATIC_DRAW );

    gl.bind_vertex_array( Some( &vao ) );
    gl::BufferDescriptor::new::< [ f32; 3 ] >()
        .stride( 3 ) 
        .offset( 0 )
        .attribute_pointer( &gl, 0, &pos_buffer ) 
        .unwrap();
    gl.bind_buffer( GL::ELEMENT_ARRAY_BUFFER, Some( &index_buffer ) );
    gl.bind_vertex_array( None );

    gl.bind_buffer( GL::ARRAY_BUFFER, None );
    gl.bind_buffer( GL::ELEMENT_ARRAY_BUFFER, None );

    renderer
  }

  /// Executes all rendering passes for a single frame.
  ///
  /// # Arguments
  ///
  /// * `t` - The current time in milliseconds ( used for animation ).
  fn render( &self, t : f64 )
  {
    // 1. Object Rendering Pass: Render the object silhouette to a texture
    self.object_pass( t );
    // 2. JFA Initialization Pass: Initialize JFA texture from the silhouette
    self.jfa_init_pass();

    // 3. JFA Step Passes: Perform Jump Flooding Algorithm steps
    // The number of passes required is log2( max( width, height ) ).
    let num_passes = ( self.viewport.0.max( self.viewport.1 ) as f32 ).log2().ceil() as i32;
    for i in 0..num_passes
    {
      let last = false; // Use here i == ( num_passes - 1 ) if you want see JFA step result
      self.jfa_step_pass( i, last );
    }

    // 4. Outline Pass: Generate and render the final scene with the outline
    self.outline_pass( t, num_passes );
  }

  /// Renders the 3D object silhouette to the `object_fb`.
  ///
  /// Sets up the model-view-projection matrices and draws the loaded mesh.
  /// The fragment shader for this pass simply outputs white.
  ///
  /// # Arguments
  ///
  /// * `t` - The current time in milliseconds ( used for rotating the camera/view ).
  fn object_pass( &self, t : f64 )
  {
    let gl = &self.gl;

    let object_program = self.programs.get( "object" ).unwrap();
    let object_fb = self.framebuffers.get( "object_fb" ).unwrap();
    let vao = self.vaos.get( "vao" ).unwrap();

    let u_projection_loc = gl.get_uniform_location( object_program, "u_projection" ).unwrap();
    let u_view_loc = gl.get_uniform_location( object_program, "u_view" ).unwrap();
    let u_model_loc = gl.get_uniform_location( object_program, "u_model" ).unwrap();

    gl.use_program( Some( object_program ) );

    let rotation = ndarray_cg::mat3x3::from_axis_angle( F32x3::Y, t as f32 / 1000.0 ); 
    let eye = rotation * self.camera.eye;
    let center = F32x3::from_array( [ 0.0, 0.3, 0.0 ] );

    let u_view = ndarray_cg::d2::mat3x3h::look_at_rh( eye, center, self.camera.up );

    upload_framebuffer( gl, object_fb, self.viewport );

    gl.clear_color( 0.0, 0.0, 0.0, 0.0 ); 
    gl.clear_depth( 1.0 );
    gl.clear( GL::COLOR_BUFFER_BIT | GL::DEPTH_BUFFER_BIT );
    gl.enable( GL::DEPTH_TEST );

    gl::uniform::matrix_upload( gl, Some( u_projection_loc.clone() ), &self.camera.projection.to_array()[ .. ], true ).unwrap();
    gl::uniform::matrix_upload( gl, Some( u_view_loc.clone() ), &u_view.to_array()[ .. ], true ).unwrap();
    gl::uniform::matrix_upload( gl, Some( u_model_loc.clone() ), &self.camera.model.to_array()[ .. ], true ).unwrap();

    gl.bind_vertex_array( Some( vao ) );
    gl.draw_elements_with_i32( gl::TRIANGLES, self.draw_count, gl::UNSIGNED_INT, 0 );
    gl.bind_vertex_array( None );
  }

  /// Performs the JFA initialization pass.
  ///
  /// Reads the object silhouette texture and writes texture coordinates for
  /// object pixels and a sentinel value for background pixels to the
  /// `jfa_init_fb`.
  fn jfa_init_pass( &self )
  {
    let gl = &self.gl;

    let jfa_init_program = self.programs.get( "jfa_init" ).unwrap();
    let jfa_init_fb = self.framebuffers.get( "jfa_init_fb" ).unwrap();
    let object_fb_color = self.textures.get( "object_fb_color" ).unwrap();

    // Get uniform location for the object texture ( silhouette )
    let u_object_texture = gl.get_uniform_location( jfa_init_program, "u_object_texture" ).unwrap();

    gl.use_program( Some( jfa_init_program ) );

    upload_framebuffer( gl, jfa_init_fb, self.viewport );

    upload_texture( gl, object_fb_color, &u_object_texture, GL::TEXTURE0 );

    gl.draw_arrays( GL::TRIANGLES, 0, 6 );
    gl.bind_vertex_array( None );
  }

  /// Performs one step of the Jump Flooding Algorithm.
  ///
  /// Reads from the JFA texture of the previous step and writes to one of the
  /// ping-pong JFA framebuffers ( `jfa_step_fb_0` or `jfa_step_fb_1` ).
  ///
  /// # Arguments
  ///
  /// * `i` - The current JFA step index ( 0, 1, 2, ... ).
  /// * `last` - A boolean flag. If true, the result of this step is rendered
  ///            directly to the default framebuffer ( screen ) for debugging.
  fn jfa_step_pass( &self, i : i32, last : bool )
  {
    let gl = &self.gl;

    let jfa_step_program = self.programs.get( "jfa_step" ).unwrap();
    let jfa_step_fb_0 = self.framebuffers.get( "jfa_step_fb_0" ).unwrap();
    let jfa_step_fb_1 = self.framebuffers.get( "jfa_step_fb_1" ).unwrap();
    let jfa_init_fb_color = self.textures.get( "jfa_init_fb_color" ).unwrap(); // Initial JFA texture
    let jfa_step_fb_color_0 = self.textures.get( "jfa_step_fb_color_0" ).unwrap(); // Color texture for FB 0
    let jfa_step_fb_color_1 = self.textures.get( "jfa_step_fb_color_1" ).unwrap(); // Color texture for FB 1

    let u_resolution = gl.get_uniform_location( &jfa_step_program, "u_resolution" ).unwrap();
    let u_step_size = gl.get_uniform_location( &jfa_step_program, "u_step_size" ).unwrap();
    let u_jfa_init_texture = gl.get_uniform_location( &jfa_step_program, "u_jfa_texture" ).unwrap(); // Sampler for input JFA texture

    gl.use_program( Some( jfa_step_program ) );

    // Ping-pong rendering: Determine input texture and output framebuffer based on step index `i`
    if i == 0 // First step uses the initialization result
    {
      upload_framebuffer( gl, jfa_step_fb_0, self.viewport ); // Render to FB 0
      upload_texture( gl, jfa_init_fb_color, &u_jfa_init_texture, GL::TEXTURE0 ); // Input is JFA init texture
    }
    else if i % 2 == 0 // Even steps ( 2, 4, ... ) read from FB 1, render to FB 0
    {
      upload_framebuffer( gl, jfa_step_fb_0, self.viewport ); // Render to FB 0
      upload_texture( gl, &jfa_step_fb_color_1, &u_jfa_init_texture, GL::TEXTURE0 ); // Input is texture from FB 1
    }
    else // Odd steps ( 1, 3, ... ) read from FB 0, render to FB 1
    {
      upload_framebuffer( gl, jfa_step_fb_1, self.viewport ); // Render to FB 1
      upload_texture( gl, jfa_step_fb_color_0, &u_jfa_init_texture, GL::TEXTURE0 ); // Input is texture from FB 0
    }

    // If 'last' is true, bind the default framebuffer ( screen ) instead of the current step's FB.
    // This is for visualizing the JFA result directly.
    if last
    {
      gl.bind_framebuffer( GL::FRAMEBUFFER, None );
      gl.clear_color( 0.0, 0.0, 0.0, 0.0 );
      gl.clear( GL::COLOR_BUFFER_BIT );
    }

    // Upload resolution uniform ( needed for distance calculations in the shader )
    gl::uniform::upload( gl, Some( u_resolution.clone() ), &[ self.viewport.0 as f32, self.viewport.1 as f32 ] ).unwrap();

    // Calculate the jump distance for the current step
    // The step size decreases by half in each pass.
    let s = | c : i32 |
    {
      ( ( c as f32 ) / 2.0f32.powi( i + 1 ) ).max( 1.0 ) // Ensure minimum step size is 1 pixel
    };

    let max = self.viewport.0.max( self.viewport.1 );
    let step_size = ( s( max ), s( max ) );

    gl::uniform::upload( gl, Some( u_step_size.clone() ), &[ step_size.0, step_size.1 ] ).unwrap();

    gl.draw_arrays( GL::TRIANGLES, 0, 6 );
    gl.bind_vertex_array( None );
  }

  /// Performs the final outline pass.
  ///
  /// Reads the original object silhouette texture and the final JFA result texture
  /// to draw the final scene with object color, outline color, or background color.
  /// Renders to the default framebuffer ( screen ).
  ///
  /// # Arguments
  ///
  /// * `t` - The current time in milliseconds ( used for animating outline thickness ).
  /// * `num_passes` - The total number of JFA step passes performed. Used to determine
  ///                which of the ping-pong textures ( `jfa_step_fb_color_0` or `jfa_step_fb_color_1` )
  ///                holds the final JFA result.
  fn outline_pass( &self, t : f64, num_passes : i32 )
  {
    let gl = &self.gl;

    let outline_program = self.programs.get( "outline" ).unwrap();
    let object_fb_color = self.textures.get( "object_fb_color" ).unwrap(); // Original silhouette
    let jfa_step_fb_color_0 = self.textures.get( "jfa_step_fb_color_0" ).unwrap(); // JFA ping-pong texture 0
    let jfa_step_fb_color_1 = self.textures.get( "jfa_step_fb_color_1" ).unwrap(); // JFA ping-pong texture 1

    let outline_u_object_texture = gl.get_uniform_location( outline_program, "u_object_texture" ).unwrap();
    let u_jfa_step_texture = gl.get_uniform_location( outline_program, "u_jfa_texture" ).unwrap();
    let u_resolution = gl.get_uniform_location( outline_program, "u_resolution" ).unwrap();
    let u_outline_thickness = gl.get_uniform_location( outline_program, "u_outline_thickness" ).unwrap();
    let u_outline_color = gl.get_uniform_location( outline_program, "u_outline_color" ).unwrap();
    let u_object_color = gl.get_uniform_location( outline_program, "u_object_color" ).unwrap();
    let u_background_color = gl.get_uniform_location( outline_program, "u_background_color" ).unwrap();

    gl.use_program( Some( outline_program ) );

    // Define outline parameters ( thickness animated with time )
    let outline_thickness = [ ( 70.0 * ( t / 3000.0 ).sin().abs() ) as f32 + 8.0 ]; // Example animation
    let outline_color = [ 1.0, 1.0, 1.0, 1.0 ]; // White outline
    let object_color = [ 0.5, 0.5, 0.5, 1.0 ]; // Grey object
    let background_color = [ 0.0, 0.0, 0.0, 1.0 ]; // Black background

    // Bind the default framebuffer ( render to canvas )
    gl.bind_framebuffer( GL::FRAMEBUFFER, None );

    gl.clear_color( background_color[ 0 ], background_color[ 1 ], background_color[ 2 ], background_color[ 3 ] );
    gl.clear( GL::COLOR_BUFFER_BIT );

    gl::uniform::upload( gl, Some( u_resolution.clone() ), &[ self.viewport.0 as f32, self.viewport.1 as f32 ] ).unwrap();
    gl::uniform::upload( gl, Some( u_outline_thickness.clone() ), &outline_thickness ).unwrap();
    gl::uniform::upload( gl, Some( u_outline_color.clone() ), &outline_color ).unwrap();
    gl::uniform::upload( gl, Some( u_object_color.clone() ), &object_color ).unwrap();
    gl::uniform::upload( gl, Some( u_background_color.clone() ), &background_color ).unwrap();

    upload_texture( gl, object_fb_color, &outline_u_object_texture, GL::TEXTURE0 );
    // The final JFA result is in jfa_step_fb_color_0 if num_passes is even, otherwise in jfa_step_fb_color_1
    if num_passes % 2 == 0
    {
      upload_texture( gl, jfa_step_fb_color_0, &u_jfa_step_texture, GL::TEXTURE1 );
    }
    else
    {
      upload_texture( gl, jfa_step_fb_color_1, &u_jfa_step_texture, GL::TEXTURE1 );
    }

    gl.draw_arrays( GL::TRIANGLES, 0, 6 );
    gl.bind_vertex_array( None );
  }
}

/// Sets up the application and runs the main rendering loop.
///
/// Initializes the renderer and defines the update/draw function that is called
/// by the `gl::exec_loop::run`.
///
/// # Returns
///
/// A `Result` indicating success or a WebGL error.
async fn run() -> Result< (), gl::WebglError >
{
  let renderer = Renderer::new().await;

  let update_and_draw =
  {
    move | t : f64 |
    {
      renderer.render( t );
      true
    }
  };

  gl::exec_loop::run( update_and_draw );

  Ok( () )
}

/// The main entry point of the application.
///
/// Spawns the asynchronous `run` function using `gl::spawn_local` which is
/// suitable for WebAssembly targets in a browser environment.
fn main()
{
  gl::spawn_local( async move { run().await.unwrap() } );
}