use minwebgl::{ self as gl };
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
    WebGlFramebuffer, 
  }
};
use ndarray_cg::
{
  mat3x3h::
  {
    translation,
    rot,
  },
  Mat4,
  F32x4x4,
  F32x4,
  F32x3,
  F32x2
};
use csgrs::CSG;
use rand::Rng;
use web_sys::WebGlRenderbuffer;
use std::collections::HashMap;

mod camera;

use camera::*;

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
  ( width, height ) : ( i32, i32 ),
  color_attachment : u32
) 
-> Option< ( WebGlFramebuffer, WebGlTexture, WebGlTexture ) >
{
  let color = gl.create_texture()?;
  gl.bind_texture( GL::TEXTURE_2D, Some( &color ) );
  // Use tex_storage_2d for immutable texture storage ( WebGL2 )
  gl.tex_storage_2d( GL::TEXTURE_2D, 1, gl::RGBA8, width, height );
  // Configure texture parameters (filtering, wrapping)
  gl.tex_parameteri( GL::TEXTURE_2D, GL::TEXTURE_MIN_FILTER, GL::LINEAR as i32 );
  gl.tex_parameteri( GL::TEXTURE_2D, GL::TEXTURE_WRAP_S, GL::CLAMP_TO_EDGE as i32 );
  gl.tex_parameteri( GL::TEXTURE_2D, GL::TEXTURE_WRAP_T, GL::CLAMP_TO_EDGE as i32 );

  let depth = gl.create_texture()?;
  gl.bind_texture( GL::TEXTURE_2D, Some( &depth ) );
  gl.tex_storage_2d( GL::TEXTURE_2D, 1, gl::DEPTH_COMPONENT16, width, height );
  // Configure texture parameters (filtering, wrapping)
  gl.tex_parameteri( GL::TEXTURE_2D, GL::TEXTURE_MIN_FILTER, GL::NEAREST as i32 );
  gl.tex_parameteri( GL::TEXTURE_2D, GL::TEXTURE_WRAP_S, GL::CLAMP_TO_EDGE as i32 );
  gl.tex_parameteri( GL::TEXTURE_2D, GL::TEXTURE_WRAP_T, GL::CLAMP_TO_EDGE as i32 );

  let framebuffer = gl.create_framebuffer()?;
  gl.bind_framebuffer( GL::FRAMEBUFFER, Some( &framebuffer ) );
  // Attach the texture to the framebuffer's color attachment point
  gl.framebuffer_texture_2d( GL::FRAMEBUFFER, GL::COLOR_ATTACHMENT0 + color_attachment, GL::TEXTURE_2D, Some( &color ), 0 );
  gl.framebuffer_texture_2d( GL::FRAMEBUFFER, GL::DEPTH_ATTACHMENT, GL::TEXTURE_2D, Some( &depth ), 0 );
  gl.bind_framebuffer( gl::FRAMEBUFFER, None );

  Some( ( framebuffer, color, depth ) )
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

pub fn primitives_data_csgrs
(
    positions: &mut Vec<[f32; 3]>,
    indices: &mut Vec<u32>,
    vertex_offset: &mut u32,
)
{
  let meshes: Vec< CSG< () > > = vec![
    {
      // Cone is constructed using frustum with one radius near zero.
      // Parameters: radius1, radius2, height, segments
      CSG::frustum( 1.0, 0.001, 2.0, 32, None )
    },
    {
      // Torus is constructed by revolving a 2D circle.
      // A circle with minor_radius is translated by major_radius along X, then revolved.
      let minor_radius = 0.5;
      let major_radius = 1.5;
      let segments = 32; // Segments for the circle cross-section
      let revolve_segments = 64; // Segments for the revolution

      let circle_2d = CSG::circle( minor_radius, segments, None );
      // Translate the circle away from the origin to define the major radius.
      // The `rotate_extrude` revolves around the Y-axis.
      circle_2d
      .translate_vector( [ major_radius, 0.0, 0.0 ].into() )
      .rotate_extrude( 360.0, revolve_segments )
    },
    {
      // Direct cylinder primitive.
      // Parameters: radius, height, segments
      CSG::cylinder( 1.0, 2.0, 32, None )
    },
    {
      // Direct sphere primitive.
      // Parameters: radius, segments, stacks
      CSG::sphere( 1.0, 32, 16, None )
    },
    {
      // Direct cube/cuboid primitive.
      // Parameters: width, length, height
      CSG::cube( 1.0, 1.0, 1.0, None )
    },
    {
      // Capsule3d is constructed by unioning a cylinder with two hemispheres (spheres).
      let radius = 0.5;
      let height = 1.0;
      let segments = 32;
      let stacks = 16;

      let cylinder = CSG::cylinder( radius, height, segments, None );
      let top_sphere = CSG::sphere( radius, segments, stacks, None )
      .translate_vector( [ 0.0, 0.0, height ].into() );
      let bottom_sphere = CSG::sphere( radius, segments, stacks, None );

      cylinder.union( &top_sphere )
      .union( &bottom_sphere )
    }
  ];

  // Define ranges for random transformation parameters.
  // t[0-2]: translation (x, y, z)
  // t[3-5]: rotation (Euler XYZ, in degrees)
  // t[6-8]: scale (x, y, z)
  let ranges =
  [
    ( 3..6, 0.0..360.0 ),
    ( 6..9, 0.35..0.6 ),
  ];

  let mut position = F32x4::new( 2.0, 0.0, 1.0, 1.0 );

  // Generate random transformation parameters for each mesh.
  let mut rng = rand::thread_rng();
  let count = meshes.len();
  let rot_matrix = rot(  0.0, ( 360.0 / count as f32 ).to_radians(), 0.0 );
  let primitives = ( 0..count )
  .into_iter()
  .map(
    | i |
    {
      let mut t = [ 0.0; 9 ];

      for ( indices, values) in &ranges
      {
        for i in indices.clone()
        {
          t[i] = rng.gen_range( values.clone() );
        }
      }

      position = rot_matrix * position;

      for j in 0..3
      {
        t[ j ] = position.0[ j ];
      }

      ( meshes[ i ].clone(), t )
    }
  )
  .collect::< Vec<( CSG< () >, [f32; 9] ) > >();

  for ( p, t ) in primitives
  {
    let p = p.scale( t[ 6 ] as f64, t[ 7 ] as f64, t[ 8 ] as f64 )
    .rotate( t[ 3 ] as f64, t[ 4 ] as f64, t[ 5 ] as f64 )
    .translate( t[ 0 ] as f64, t[ 1 ] as f64, t[ 2 ] as f64 );

    let mesh = p.to_trimesh();
    let mesh = mesh.as_trimesh().unwrap();

    let primitive_positions = mesh.vertices()
    .iter()
    .map( | p | [ p.coords.x as f32, p.coords.y as f32, p.coords.z as f32 ] )
    .collect::< Vec< _ > >();
    positions.extend( primitive_positions );

    let primitive_indices = mesh.indices()
    .iter()
    .flatten()
    .map( | i | i + *vertex_offset )
    .collect::< Vec< _ > >();
    indices.extend( primitive_indices );
    
    let vertices_count = mesh.vertices().len();
    *vertex_offset += vertices_count as u32;
  }
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
  model_matrix : F32x4x4,
  draw_count : i32 // Number of indices to draw for the object
}

impl Renderer
{
  /// Creates a new Renderer instance, initializes WebGL, loads resources,
  /// and prepares the scene for rendering.
  async fn new() -> Self
  {
    gl::browser::setup( Default::default() );
    let canvas = gl::canvas::make().unwrap();
    let gl = gl::context::from_canvas( &canvas ).unwrap();

    // --- Initialization ---

    let viewport = ( gl.drawing_buffer_width(), gl.drawing_buffer_height() );

    // Camera setup (initial position, up vector, projection matrix, initial model matrix)
    let eye = F32x3::from_array( [  0.0, 1.4, 2.5 ] ) * 1.5;
    let up = F32x3::Y;

    let aspect_ratio = viewport.0 as f32 / viewport.1 as f32;
    let fov =  70.0f32.to_radians();
    let near = 0.1;
    let far = 1000.0;
    
    let model_matrix : F32x4x4 = translation( F32x3::default() );

    let mut camera = Camera::new(
      eye,
      up,
      F32x3::new( 0.0, 0.4, 0.0 ),
      aspect_ratio,
      fov,
      near,
      far
    );
    camera.set_window_size( F32x2::new(viewport.0 as f32, viewport.1 as f32 ) );
    setup_controls( &canvas, &camera.get_controls() );

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
      model_matrix,
      draw_count : 0
    };

    let gl = &renderer.gl;

    // --- Load and Compile Shaders ---

    let object_vs_src = include_str!( "../resources/shaders/object.vert" );
    let object_fs_src = include_str!( "../resources/shaders/object.frag" );
    let fullscreen_vs_src = include_str!( "../resources/shaders/fullscreen.vert" );
    let outline_fs_src = include_str!( "../resources/shaders/outline.frag" );

    // Compile and link shader programs and store them
    let object_program = gl::ProgramFromSources::new( object_vs_src, object_fs_src ).compile_and_link( gl ).unwrap();
    let outline_program = gl::ProgramFromSources::new( fullscreen_vs_src, outline_fs_src ).compile_and_link( gl ).unwrap();

    renderer.programs.insert( "object".to_string(), object_program );
    renderer.programs.insert( "outline".to_string(), outline_program );

    // --- Create Framebuffers and Textures ---

    // Framebuffer for rendering the initial object silhouette
    let ( object_fb, object_fb_color, object_fb_depth ) = create_framebuffer( gl, viewport, 0 ).unwrap();

    // Store the color attachment textures
    renderer.textures.insert( "object_fb_color".to_string(), object_fb_color );
    renderer.textures.insert( "object_fb_depth".to_string(), object_fb_depth );

    // Store the framebuffers
    renderer.framebuffers.insert( "object_fb".to_string(), object_fb );

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
          model_matrix, // Initial model transform ( applied to the root node's children )
          &mut positions, 
          &mut indices,  
          &mut vertex_offset // Output counter for vertex offset
        );
      }
      
      primitives_data_csgrs
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
    self.object_pass();
    self.outline_pass( t );
  }

  /// Renders the 3D object silhouette to the `object_fb`.
  ///
  /// Sets up the model-view-projection matrices and draws the loaded mesh.
  /// The fragment shader for this pass simply outputs white.
  ///
  /// # Arguments
  ///
  /// * `t` - The current time in milliseconds ( used for rotating the camera/view ).
  fn object_pass( &self )
  {
    let gl = &self.gl;

    let object_program = self.programs.get( "object" ).unwrap();
    let object_fb = self.framebuffers.get( "object_fb" ).unwrap();
    let vao = self.vaos.get( "vao" ).unwrap();

    let u_projection_loc = gl.get_uniform_location( object_program, "u_projection" ).unwrap();
    let u_view_loc = gl.get_uniform_location( object_program, "u_view" ).unwrap();
    let u_model_loc = gl.get_uniform_location( object_program, "u_model" ).unwrap();

    gl.use_program( Some( object_program ) );

    upload_framebuffer( gl, object_fb, self.viewport );

    gl.clear_color( 0.0, 0.0, 0.0, 0.0 ); 
    gl.clear_depth( 1.0 );
    gl.clear( GL::COLOR_BUFFER_BIT | GL::DEPTH_BUFFER_BIT );
    gl.enable( GL::DEPTH_TEST );

    self.camera.apply( &self.gl, &u_view_loc, &u_projection_loc );
    gl::uniform::matrix_upload( gl, Some( u_model_loc.clone() ), &self.model_matrix.to_array()[ .. ], true ).unwrap();

    gl.bind_vertex_array( Some( vao ) );
    gl.draw_elements_with_i32( gl::TRIANGLES, self.draw_count, gl::UNSIGNED_INT, 0 );
    gl.bind_vertex_array( None );
  }

  fn outline_pass( &self, t : f64 )
  {
    let gl = &self.gl;

    let outline_program = self.programs.get( "outline" ).unwrap();
    let object_fb_color = self.textures.get( "object_fb_color" ).unwrap();
    let object_fb_depth = self.textures.get( "object_fb_depth" ).unwrap();

    let u_color_texture_loc = gl.get_uniform_location( outline_program, "u_color_texture" ).unwrap();
    let u_depth_texture_loc = gl.get_uniform_location( outline_program, "u_depth_texture" ).unwrap();
    let u_projection_loc = gl.get_uniform_location( outline_program, "u_projection" ).unwrap();
    let u_resolution_loc = gl.get_uniform_location( outline_program, "u_resolution" ).unwrap();
    let u_outline_thickness_loc = gl.get_uniform_location( outline_program, "u_outline_thickness" ).unwrap();
    let u_outline_color_loc = gl.get_uniform_location( outline_program, "u_outline_color" ).unwrap();
    let u_object_color_loc = gl.get_uniform_location( outline_program, "u_object_color" ).unwrap();
    let u_background_color_loc = gl.get_uniform_location( outline_program, "u_background_color" ).unwrap();

    gl.use_program( Some( outline_program ) );

    let outline_thickness = [ ( 8.0 * ( t / 1000.0 ).sin().abs() ) as f32 ]; // Example animation
    let outline_color = [ 1.0, 1.0, 1.0, 1.0 ]; // White outline
    let object_color = [ 0.5, 0.5, 0.5, 1.0 ]; // Grey object
    let background_color = [ 0.0, 0.0, 0.0, 1.0 ]; 

    gl.bind_framebuffer( GL::FRAMEBUFFER, None );

    gl.clear_color( background_color[ 0 ], background_color[ 1 ], background_color[ 2 ], background_color[ 3 ] );
    gl.clear( GL::COLOR_BUFFER_BIT );

    upload_texture( gl, object_fb_color, &u_color_texture_loc, GL::TEXTURE0 );
    upload_texture( gl, object_fb_depth, &u_depth_texture_loc, GL::TEXTURE1 );
    gl::uniform::matrix_upload( gl, Some( u_projection_loc.clone() ), &self.camera.get_projection_matrix().to_array()[ .. ], true ).unwrap();
    gl::uniform::upload( gl, Some( u_resolution_loc.clone() ), &[ self.viewport.0 as f32, self.viewport.1 as f32 ] ).unwrap();
    gl::uniform::upload( gl, Some( u_outline_thickness_loc.clone() ), &outline_thickness ).unwrap();
    gl::uniform::upload( gl, Some( u_outline_color_loc.clone() ), &outline_color ).unwrap();
    gl::uniform::upload( gl, Some( u_object_color_loc.clone() ), &object_color ).unwrap();
    gl::uniform::upload( gl, Some( u_background_color_loc.clone() ), &background_color ).unwrap();

    gl.draw_arrays( GL::TRIANGLES, 0, 3 );
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