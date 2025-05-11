//! Implementation of JFA outline

use minwebgl as gl;
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
use ndarray_cg::mat::DescriptorOrderColumnMajor;
use std::collections::HashMap;

fn create_texture( 
  gl : &gl::WebGl2RenderingContext,
  slot : u32,
  size : ( i32, i32 ),
  internal_format : i32,
  format : u32,
  pixel_type : u32,
  data : Option< &[ u8 ] >
) -> Option< WebGlTexture >
{
  let Some( texture ) = gl.create_texture() else {
    return None;   
  };
  gl.active_texture( slot );
  gl.bind_texture( GL::TEXTURE_2D, Some( &texture ) );
  gl.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_u8_array( 
    GL::TEXTURE_2D,
    0,
    internal_format,
    size.0,
    size.1,
    0,
    format,
    pixel_type,
    data,
  )
  .unwrap();
  gl.bind_texture( GL::TEXTURE_2D, None );
  Some( texture )
}

fn upload_texture(
  gl : &gl::WebGl2RenderingContext,
  texture : &WebGlTexture,
  location : &WebGlUniformLocation,
  slot : u32,
)
{
  gl.active_texture( slot );
  gl.bind_texture( GL::TEXTURE_2D, Some( &texture ) );
  gl.uniform1i( Some( location ), ( slot - GL::TEXTURE0 ) as i32 );
}

fn create_framebuffer(
  gl : &gl::WebGl2RenderingContext,
  size : ( i32, i32 ),
  color_attachment : u32
) -> Option< ( WebGlFramebuffer, WebGlTexture ) >
{
  let color = gl.create_texture()?;
  gl.bind_texture( GL::TEXTURE_2D, Some( &color ) );
  gl.tex_storage_2d( GL::TEXTURE_2D, 1, gl::RGBA8, size.0, size.1 );
  gl.tex_parameteri( GL::TEXTURE_2D, GL::TEXTURE_MIN_FILTER, GL::LINEAR as i32 );
  gl.tex_parameteri( GL::TEXTURE_2D, GL::TEXTURE_WRAP_S, GL::REPEAT as i32 );
  gl.tex_parameteri( GL::TEXTURE_2D, GL::TEXTURE_WRAP_T, GL::REPEAT as i32 );

  let framebuffer = gl.create_framebuffer()?;
  gl.bind_framebuffer( GL::FRAMEBUFFER, Some( &framebuffer ) );
  gl.framebuffer_texture_2d( GL::FRAMEBUFFER, GL::COLOR_ATTACHMENT0 + color_attachment, GL::TEXTURE_2D, Some( &color ), 0 );
  gl.bind_framebuffer( gl::FRAMEBUFFER, None ); 

  Some( ( framebuffer, color ) ) 
}

fn upload_framebuffer(
  gl : &gl::WebGl2RenderingContext,
  framebuffer : &WebGlFramebuffer,
  size : ( i32, i32 )
)
{
  gl.bind_framebuffer( GL::FRAMEBUFFER, Some( framebuffer ) );
  gl.viewport( 0, 0, size.0, size.1 );
}

fn collect_mesh_data
(
  node : &gltf::Node,
  buffers : &[ gltf::buffer::Data ],
  parent_transform : ndarray_cg::Mat4< f32, DescriptorOrderColumnMajor >,
  positions : &mut Vec< [ f32; 3 ] >, 
  indices : &mut Vec< u32 >,         
  vertex_offset : &mut u32 
)
{
  let transform = node.transform().matrix();
  let mut transform_raw : [ f32; 16 ] = [ 0.0; 16 ];
  for ( i, r ) in transform_raw.chunks_mut( 4 ).enumerate()
  {
    r[ 0 ] = transform[ i ][ 0 ];
    r[ 1 ] = transform[ i ][ 1 ];
    r[ 2 ] = transform[ i ][ 2 ];
    r[ 3 ] = transform[ i ][ 3 ];
  }

  let local_transform : ndarray_cg::Mat4< f32, DescriptorOrderColumnMajor > = ndarray_cg::Mat4::from_column_major( &transform_raw ); 

  let current_transform = parent_transform * local_transform;

  if let Some( mesh ) = node.mesh()
  {
    for primitive in mesh.primitives()
    {
      let reader = primitive.reader( | buffer | Some( &buffers[ buffer.index() ] ) );

      if let Some( positions_iter ) = reader.read_positions()
      {
        let mut current_primitive_positions : Vec< [ f32; 3 ] > = Vec::new();
        
        for p in positions_iter
        {
          let pos_vec = ndarray_cg::F32x4::from_array( [ p[ 0 ], p[ 1 ], p[ 2 ], 1.0 ] );
          let tp = current_transform * pos_vec; 
          current_primitive_positions.push( [ tp[ 0 ], tp[ 1 ], tp[ 2 ] ].into() ); 
        }

        let num_current_vertices = current_primitive_positions.len();

        positions.extend( current_primitive_positions ); 

        if let Some( indices_iter ) = reader.read_indices()
        {
          for index in indices_iter.into_u32()
          {
             indices.push( index + *vertex_offset ); 
          }
        }

        *vertex_offset += num_current_vertices as u32;
      }
    }
  }

  for child in node.children()
  {
    collect_mesh_data( 
      &child, 
      buffers, 
      current_transform, 
      positions, 
      indices,  
      vertex_offset 
    );
  }
}

struct Camera
{
  eye : ndarray_cg::F32x3,
  up : ndarray_cg::F32x3,
  projection : ndarray_cg::Mat4< f32, DescriptorOrderColumnMajor >,
  model : ndarray_cg::Mat4< f32, DescriptorOrderColumnMajor >
}

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
  draw_count : i32
}

impl Renderer
{
  async fn new() -> Self
  {
    gl::browser::setup( Default::default() );
    let gl = gl::context::retrieve_or_make().unwrap();

    // Other
    let viewport = ( gl.drawing_buffer_width(), gl.drawing_buffer_height() );
  
    // Camera setup
    let eye = ndarray_cg::F32x3::from_array( [  0.0, 1.4, 2.5 ] );
    let up = ndarray_cg::F32x3::Y;
  
    let aspect_ratio = viewport.0 as f32 / viewport.1 as f32;
    let u_projection = ndarray_cg::mat3x3h::perspective_rh_gl
    (
      70.0f32.to_radians(),  
      aspect_ratio, 
      0.1, 
      1000.0
    );
    let u_model = glam::Mat4::from_scale_rotation_translation
    (
      glam::Vec3::ONE, 
      glam::Quat::from_rotation_y( 0.0 ), 
      glam::Vec3::ZERO
    );
    let u_model : ndarray_cg::Mat4< f32, DescriptorOrderColumnMajor > = ndarray_cg::Mat4::from_column_major( u_model.to_cols_array() );

    let camera = Camera{
      eye,
      up,
      projection : u_projection,
      model : u_model
    };

    // Renderer

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

    // Vertex and fragment shaders
    let object_vs_src = include_str!( "../resources/shaders/object.vert" );
    let object_fs_src = include_str!( "../resources/shaders/object.frag" );
    let fullscreen_vs_src = include_str!( "../resources/shaders/fullscreen.vert" );
    let jfa_init_fs_src = include_str!( "../resources/shaders/jfa_init.frag" );
    let jfa_step_fs_src = include_str!( "../resources/shaders/jfa_step.frag" );
    let outline_fs_src = include_str!( "../resources/shaders/outline.frag" );
  
    // Programs
    let object_program = gl::ProgramFromSources::new( object_vs_src, object_fs_src ).compile_and_link( gl ).unwrap();
    let jfa_init_program = gl::ProgramFromSources::new( fullscreen_vs_src, jfa_init_fs_src ).compile_and_link( gl ).unwrap();
    let jfa_step_program = gl::ProgramFromSources::new( fullscreen_vs_src, jfa_step_fs_src ).compile_and_link( gl ).unwrap();
    let outline_program = gl::ProgramFromSources::new( fullscreen_vs_src, outline_fs_src ).compile_and_link( gl ).unwrap();
  
    renderer.programs.insert( "object".to_string(), object_program );
    renderer.programs.insert( "jfa_init".to_string(), jfa_init_program );
    renderer.programs.insert( "jfa_step".to_string(), jfa_step_program );
    renderer.programs.insert( "outline".to_string(), outline_program );
  
    // Textures
  
    // Framebuffers
    let ( object_fb, object_fb_color ) = create_framebuffer( gl, viewport, 0 ).unwrap();
    let ( jfa_init_fb, jfa_init_fb_color ) = create_framebuffer( gl, viewport, 0 ).unwrap();
    let ( jfa_step_fb_0, jfa_step_fb_color_0 ) = create_framebuffer( gl, viewport, 0 ).unwrap();
    let ( jfa_step_fb_1, jfa_step_fb_color_1 ) = create_framebuffer( gl, viewport, 0 ).unwrap();

    renderer.textures.insert( "object_fb_color".to_string(), object_fb_color );
    renderer.textures.insert( "jfa_init_fb_color".to_string(), jfa_init_fb_color );
    renderer.textures.insert( "jfa_step_fb_color_0".to_string(), jfa_step_fb_color_0 );
    renderer.textures.insert( "jfa_step_fb_color_1".to_string(), jfa_step_fb_color_1 );

    renderer.framebuffers.insert( "object_fb".to_string(), object_fb );
    renderer.framebuffers.insert( "jfa_init_fb".to_string(), jfa_init_fb );
    renderer.framebuffers.insert( "jfa_step_fb_0".to_string(), jfa_step_fb_0 );
    renderer.framebuffers.insert( "jfa_step_fb_1".to_string(), jfa_step_fb_1 );
  
    // Buffers
    let pos_buffer =  gl::buffer::create( gl ).unwrap();
    let index_buffer = gl::buffer::create( gl ).unwrap();
    let vao = gl::vao::create( gl ).unwrap();

    renderer.buffers.insert( "pos_buffer".to_string(), pos_buffer.clone() );
    renderer.buffers.insert( "index_buffer".to_string(), index_buffer.clone() );
    renderer.vaos.insert( "vao".to_string(), vao.clone() );
  
    // Model
    let obj_buffer = gl::file::load( "model.glb" ).await.expect( "Failed to load the model" );
    let ( document, buffers, _ ) = gltf::import_slice( &obj_buffer[ .. ] ).expect( "Failed to parse the glb file" );
  
    let mut positions : Vec< [ f32; 3 ] > = vec![];
    let mut indices : Vec< u32 > = vec![];
  
    {
      let scene = document.default_scene().unwrap();
      let mut vertex_offset : u32 = 0;
      for node in scene.nodes()
      {
         collect_mesh_data( 
           &document.nodes().nth( node.index() ).expect("Node not found"),
           &buffers, 
           u_model, 
           &mut positions, 
           &mut indices,   
           &mut vertex_offset 
         );
      }
      renderer.draw_count = indices.len() as i32;
    }
  
    gl::buffer::upload( &gl, &pos_buffer, &positions, GL::STATIC_DRAW );
    gl::index::upload( &gl, &index_buffer, &indices, GL::STATIC_DRAW );
  
    gl.bind_vertex_array( Some( &vao ) );
    gl::BufferDescriptor::new::< [ f32; 3 ] >().stride( 3 ).offset( 0 ).attribute_pointer( &gl, 0, &pos_buffer ).unwrap();
    gl.bind_buffer( GL::ELEMENT_ARRAY_BUFFER, Some( &index_buffer ) );
    gl.bind_vertex_array( None );

    gl.bind_vertex_array( None );
    gl.bind_buffer( GL::ARRAY_BUFFER, None );
    gl.bind_buffer( GL::ELEMENT_ARRAY_BUFFER, None );

    renderer
  }

  fn render( &self, t : f64 )
  {
    self.object_pass( t );
    self.jfa_init_pass();

    let num_passes = ( self.viewport.0.max( self.viewport.1 ) as f32 ).log2().ceil() as i32;
    for i in 0..num_passes
    {
      let last = false; // Use here i == ( num_passes - 1 ) if you want see JFA step result 
      self.jfa_step_pass( i, last );        
    }
    
    self.outline_pass( t, num_passes );
  }

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

    let rotation = ndarray_cg::mat3x3::from_axis_angle( ndarray_cg::F32x3::Y, t as f32 / 1000.0 );
    let eye = rotation * self.camera.eye;
    let center = ndarray_cg::F32x3::from_array( [ 0.0, 0.3, 0.0 ] );

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

  fn jfa_init_pass( &self )
  {
    let gl = &self.gl;

    let jfa_init_program = self.programs.get( "jfa_init" ).unwrap();
    let jfa_init_fb = self.framebuffers.get( "jfa_init_fb" ).unwrap();
    let object_fb_color = self.textures.get( "object_fb_color" ).unwrap();

    let u_object_texture = gl.get_uniform_location( jfa_init_program, "u_object_texture" ).unwrap();

    gl.use_program( Some( jfa_init_program ) );
  
    upload_framebuffer( gl, jfa_init_fb, self.viewport );
    upload_texture( gl, object_fb_color, &u_object_texture, GL::TEXTURE0 );

    gl.draw_arrays( GL::TRIANGLES, 0, 6 );
    gl.bind_vertex_array( None );
  }

  fn jfa_step_pass( &self, i : i32, last : bool )
  {
    let gl = &self.gl;

    let jfa_step_program = self.programs.get( "jfa_step" ).unwrap();
    let jfa_step_fb_0 = self.framebuffers.get( "jfa_step_fb_0" ).unwrap();
    let jfa_step_fb_1 = self.framebuffers.get( "jfa_step_fb_1" ).unwrap();
    let jfa_init_fb_color = self.textures.get( "jfa_init_fb_color" ).unwrap();
    let jfa_step_fb_color_0 = self.textures.get( "jfa_step_fb_color_0" ).unwrap();
    let jfa_step_fb_color_1 = self.textures.get( "jfa_step_fb_color_1" ).unwrap();

    let u_resolution = gl.get_uniform_location( &jfa_step_program, "u_resolution" ).unwrap();
    let u_step_size = gl.get_uniform_location( &jfa_step_program, "u_step_size" ).unwrap();
    let u_jfa_init_texture = gl.get_uniform_location( &jfa_step_program, "u_jfa_texture" ).unwrap();

    gl.use_program( Some( jfa_step_program ) );
  
    if i == 0
    {
      upload_framebuffer( gl, jfa_step_fb_0, self.viewport );
      upload_texture( gl, jfa_init_fb_color, &u_jfa_init_texture, GL::TEXTURE0 );
    }
    else if i % 2 == 0
    {
      upload_framebuffer( gl, jfa_step_fb_0, self.viewport );
      upload_texture( gl, &jfa_step_fb_color_1, &u_jfa_init_texture, GL::TEXTURE0 );
    }
    else
    {
      upload_framebuffer( gl, jfa_step_fb_1, self.viewport );
      upload_texture( gl, jfa_step_fb_color_0, &u_jfa_init_texture, GL::TEXTURE0 );
    } 

    if last
    {
      gl.bind_framebuffer( GL::FRAMEBUFFER, None );
      gl.clear_color( 0.0, 0.0, 0.0, 0.0 );
      gl.clear( GL::COLOR_BUFFER_BIT );
    }

    gl::uniform::upload( gl, Some( u_resolution.clone() ), &[ self.viewport.0 as f32, self.viewport.1 as f32 ] ).unwrap();

    let s = | c : i32 |
    {
      ( ( c as f32 ) / 2.0f32.powi( i + 1 ) ).max( 1.0 )
    };

    let max = self.viewport.0.max( self.viewport.1 );
    let step_size = ( s( max ), s( max ) );
    
    gl::uniform::upload( gl, Some( u_step_size.clone() ), &[ step_size.0, step_size.1 ] ).unwrap();

    gl.draw_arrays( GL::TRIANGLES, 0, 6 );
    gl.bind_vertex_array( None );
  }

  fn outline_pass( &self, t : f64, num_passes : i32 )
  {
    let gl = &self.gl;

    let outline_program = self.programs.get( "outline" ).unwrap();
    let object_fb_color = self.textures.get( "object_fb_color" ).unwrap();
    let jfa_step_fb_color_0 = self.textures.get( "jfa_step_fb_color_0" ).unwrap();
    let jfa_step_fb_color_1 = self.textures.get( "jfa_step_fb_color_1" ).unwrap();

    let outline_u_object_texture = gl.get_uniform_location( outline_program, "u_object_texture" ).unwrap();
    let u_jfa_step_texture = gl.get_uniform_location( outline_program, "u_jfa_texture" ).unwrap();
    let u_resolution = gl.get_uniform_location( outline_program, "u_resolution" ).unwrap();
    let u_outline_thickness = gl.get_uniform_location( outline_program, "u_outline_thickness" ).unwrap();
    let u_outline_color = gl.get_uniform_location( outline_program, "u_outline_color" ).unwrap();
    let u_object_color = gl.get_uniform_location( outline_program, "u_object_color" ).unwrap();
    let u_background_color = gl.get_uniform_location( outline_program, "u_background_color" ).unwrap();

    gl.use_program( Some( outline_program ) );

    let outline_thickness = [ ( 70.0 * ( t / 3000.0 ).sin().abs() ) as f32 + 8.0 ]; 
    let outline_color = [ 1.0, 1.0, 1.0, 1.0 ]; 
    let object_color = [ 0.5, 0.5, 0.5, 1.0 ]; 
    let background_color = [ 0.0, 0.0, 0.0, 1.0 ];
  
    gl.bind_framebuffer( GL::FRAMEBUFFER, None );

    gl.clear_color( background_color[ 0 ], background_color[ 1 ], background_color[ 2 ], background_color[ 3 ] );
    gl.clear( GL::COLOR_BUFFER_BIT );

    gl::uniform::upload( gl, Some( u_resolution.clone() ), &[ self.viewport.0 as f32, self.viewport.1 as f32 ] ).unwrap();
    gl::uniform::upload( gl, Some( u_outline_thickness.clone() ), &outline_thickness ).unwrap();
    gl::uniform::upload( gl, Some( u_outline_color.clone() ), &outline_color ).unwrap();
    gl::uniform::upload( gl, Some( u_object_color.clone() ), &object_color ).unwrap();
    gl::uniform::upload( gl, Some( u_background_color.clone() ), &background_color ).unwrap();
    upload_texture( gl, object_fb_color, &outline_u_object_texture, GL::TEXTURE0 );
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

async fn run() -> Result< (), gl::WebglError >
{
  let renderer = Renderer::new().await;

  // Define the update and draw logic
  let update_and_draw =
  {
    move | t : f64 |
    {
      renderer.render( t );
      
      true
    }
  };

  // Run the render loop
  gl::exec_loop::run( update_and_draw );

  Ok( () )
}

fn main()
{
  gl::spawn_local( async move { run().await.unwrap() } );
}
