use minwebgl as gl;
use gl::{ GL, JsCast as _, JsValue };
use bytemuck::NoUninit;
use ndarray_cg::mat::DescriptorOrderColumnMajor;
use core::f32;
use web_sys::
{
  js_sys::Array,
  HtmlCanvasElement,
  WebGlTexture,
  WebGlVertexArrayObject
};

type Mat4 = ndarray_cg::d2::Mat4< f32, DescriptorOrderColumnMajor >;

fn main()
{
  gl::browser::setup( Default::default() );
  gl::spawn_local( async { gl::info!( "{:?}", run().await ); } );
}

async fn run() -> Result< (), gl::WebglError >
{
  let gl = gl::context::retrieve_or_make().expect( "Failed to retrieve WebGl context" );
  gl.enable( GL::DEPTH_TEST );
  gl.clear_color( 0., 0., 0., 1. );
  gl.enable( GL::CULL_FACE );
  _ = gl.get_extension( "EXT_color_buffer_float" ).unwrap().unwrap();

  let width = 1280;
  let height = 720;
  let canvas = gl.canvas().unwrap().dyn_into::< HtmlCanvasElement >().unwrap();
  canvas.set_width( width as u32 );
  canvas.set_height( height as u32 );
  gl.viewport( 0, 0, width, height );

  let file = gl::file::load( "lowpoly_tree.obj" ).await.unwrap();
  let ( models, _ ) = gl::model::load_model_from_slice( &file, "", &tobj::GPU_LOAD_OPTIONS )
  .await
  .unwrap();

  let tree_mesh = load_meshes( &models, &gl )?;

  let file = gl::file::load( "plane.obj" ).await.unwrap();
  let ( models, _ ) = gl::model::load_model_from_slice( &file, "", &tobj::GPU_LOAD_OPTIONS )
  .await
  .unwrap();
  let plane_mesh = load_meshes( &models, &gl )?;
  let plane_transform = Mat4::from_row_major
  (
    [
      10., 0., 0.,   0.,
      0.,  1., 0.,  -0.705,
      0.,  0., 10., -11.,
      0.,  0., 0.,   1.,
    ]
  );

  // gl::info!( "{:?}", plane_transform );
  let transforms = create_transforms( 100 );

  let aspect_ratio = width as f32 / height as f32;
  let projection = ndarray_cg::mat3x3h::perspective_rh_gl( 45.0_f32.to_radians(), aspect_ratio, 0.1, 1000. );

  let vert = include_str!( "shaders/main.vert" );
  let frag = include_str!( "shaders/main.frag" );
  let object_shader = gl::ProgramFromSources::new( vert, frag ).compile_and_link( &gl ).unwrap();
  let mvp_location = gl.get_uniform_location( &object_shader, "mvp" );
  let model_location = gl.get_uniform_location( &object_shader, "model" );

  let vert = include_str!( "shaders/rasterize.vert" );
  let frag = include_str!( "shaders/deferred.frag" );
  let deferred_shader = gl::ProgramFromSources::new( vert, frag ).compile_and_link( &gl ).unwrap();
  let positions_location = gl.get_uniform_location( &deferred_shader, "positions" );
  let normals_location = gl.get_uniform_location( &deferred_shader, "normals" );
  let lights_index = gl.get_uniform_block_index( &deferred_shader, "Lights" );

  // set texture units
  gl.use_program( Some( &deferred_shader ) );
  gl::uniform::upload( &gl, positions_location, &0 ).unwrap();
  gl::uniform::upload( &gl, normals_location, &1 ).unwrap();

  // set lighting ubo
  const BINDING_POINT : u32 = 0;
  gl.uniform_block_binding( &deferred_shader, lights_index, BINDING_POINT );

  let mut lights = create_lights( 100 );
  let lights_ubo = gl::buffer::create( &gl ).unwrap();
  gl::ubo::upload
  (
    &gl,
    &lights_ubo,
    BINDING_POINT,
    bytemuck::cast_slice::< _, u8 >( &lights ),
    GL::DYNAMIC_DRAW
  );

  let positionbuffer = tex_storage( &gl, GL::RGBA16F, width, height );
  let normalbuffer = tex_storage( &gl, GL::RGBA16F, width, height );
  let depthbuffer = gl.create_renderbuffer();
  gl.bind_renderbuffer( GL::RENDERBUFFER, depthbuffer.as_ref() );
  gl.renderbuffer_storage( GL::RENDERBUFFER, GL::DEPTH_COMPONENT16, width, height );

  let framebuffer = gl.create_framebuffer();
  gl.bind_framebuffer( GL::FRAMEBUFFER, framebuffer.as_ref() );
  gl.framebuffer_texture_2d( GL::FRAMEBUFFER, GL::COLOR_ATTACHMENT0, GL::TEXTURE_2D, positionbuffer.as_ref(), 0 );
  gl.framebuffer_texture_2d( GL::FRAMEBUFFER, GL::COLOR_ATTACHMENT1, GL::TEXTURE_2D, normalbuffer.as_ref(), 0 );
  gl.framebuffer_renderbuffer( GL::FRAMEBUFFER, GL::DEPTH_ATTACHMENT, GL::RENDERBUFFER, depthbuffer.as_ref() );

  // draw data into framebuffer
  gl.use_program( Some( &object_shader ) );

  let colors = [ JsValue::from_f64( GL::COLOR_ATTACHMENT0 as f64 ), JsValue::from_f64( GL::COLOR_ATTACHMENT1 as f64 ) ];
  gl.draw_buffers( &Array::from_iter( colors.iter() ) );

  // draw trees
  for transform in transforms
  {
    gl::uniform::matrix_upload
    (
      &gl,
      mvp_location.clone(),
      ( projection * transform ).raw_slice(),
      true
    ).unwrap();
    gl::uniform::matrix_upload
    (
      &gl,
      model_location.clone(),
      transform.raw_slice(),
      true
    ).unwrap();
    draw_meshes( &tree_mesh, &gl );
  }

  // draw plane
  gl::uniform::matrix_upload
  (
    &gl,
    mvp_location.clone(),
    ( projection * plane_transform ).raw_slice(),
    true
  ).unwrap();
  gl::uniform::matrix_upload
  (
    &gl,
    model_location.clone(),
    plane_transform.raw_slice(),
    true
  ).unwrap();
  draw_meshes( &plane_mesh, &gl );


  gl.use_program( Some( &deferred_shader ) );
  gl.bind_framebuffer( GL::FRAMEBUFFER, None );

  gl.active_texture( GL::TEXTURE0 );
  gl.bind_texture( GL::TEXTURE_2D, positionbuffer.as_ref() );

  gl.active_texture( GL::TEXTURE1 );
  gl.bind_texture( GL::TEXTURE_2D, normalbuffer.as_ref() );

  let mut dir = 1.;
  let mut radius = 3.;
  let mut last_time = 0.;
  let draw_loop = move | t |
  {
    let time = ( t / 1000. ) as f32;
    let delta_time = time - last_time;
    last_time = time;

    if radius > 10.
    {
      dir = -1.;
    }
    if radius < 3.
    {
      dir = 1.;
    }
    radius += delta_time * dir * 2.;

    const PI2 : f32 = f32::consts::PI * 2.;
    let angle = time * f32::consts::FRAC_PI_2;
    let len = lights.len() as f32;
    for ( i, light ) in lights.iter_mut().enumerate()
    {
      let angle_offset = PI2 / len * i as f32;
      let x = ( angle + angle_offset ).sin() * radius;
      let z = -9. + ( angle + angle_offset ).cos() * radius;
      light.position[ 0 ] = x;
      light.position[ 2 ] = z;
    }
    gl.buffer_sub_data_with_f64_and_u8_array( GL::UNIFORM_BUFFER, 0., bytemuck::cast_slice( &lights ) );

    gl.clear( GL::COLOR_BUFFER_BIT );
    gl.draw_arrays( GL::TRIANGLES, 0, 3 );

    true
  };

  gl::exec_loop::run( draw_loop );

  Ok( () )
}

fn tex_storage( gl : &GL, format : u32, width : i32, height : i32 ) -> Option< WebGlTexture >
{
  let tex = gl.create_texture();
  gl.bind_texture( GL::TEXTURE_2D, tex.as_ref() );
  gl.tex_storage_2d( GL::TEXTURE_2D, 1, format, width, height );
  gl.tex_parameteri( GL::TEXTURE_2D, GL::TEXTURE_MIN_FILTER, GL::NEAREST as i32 );
  gl.tex_parameteri( GL::TEXTURE_2D, GL::TEXTURE_MAG_FILTER, GL::NEAREST as i32 );
  tex
}

fn load_meshes( models: &[ tobj::Model ], gl: &GL )
->
Result< Vec< ( WebGlVertexArrayObject, i32 ) >, minwebgl::WebglError >
{
  let mut meshes = vec![];
  for model in models
  {
    let vao = gl::vao::create( gl )?;
    gl.bind_vertex_array( Some( &vao ) );

    let position_buffer = gl::buffer::create( gl )?;
    let normal_buffer = gl::buffer::create( gl )?;
    let index_buffer = gl::buffer::create( gl )?;

    gl::buffer::upload( gl, &position_buffer, &model.mesh.positions, GL::STATIC_DRAW );
    gl::buffer::upload( gl, &normal_buffer, &model.mesh.normals, GL::STATIC_DRAW );
    gl::index::upload( gl, &index_buffer, &model.mesh.indices, GL::STATIC_DRAW );

    gl::BufferDescriptor::new::< [ f32; 3 ] >()
    .offset( 0 )
    .stride( 0 )
    .attribute_pointer( gl, 0, &position_buffer )
    .unwrap();

    gl::BufferDescriptor::new::< [ f32; 3 ] >()
    .offset( 0 )
    .stride( 0 )
    .attribute_pointer( gl, 1, &normal_buffer )
    .unwrap();

    meshes.push( ( vao, model.mesh.indices.len() as i32 ) );
  }
  Ok( meshes )
}

fn draw_meshes( meshes : &[ ( WebGlVertexArrayObject, i32 ) ], gl : &GL )
{
  for ( vao, count ) in meshes
  {
    gl.bind_vertex_array( Some( &vao ) );
    gl.draw_elements_with_i32( GL::TRIANGLES, *count, GL::UNSIGNED_INT, 0 );
  }
}

fn create_lights( num : usize ) -> Box< [ PointLight ] >
{
  ( 0 .. num ).map
  (
    | _ |
    {
      let position = [ 0., 1., 0., 0. ];

      let color =
      [
        rand::random::< bool >() as i32 as f32,
        rand::random::< bool >() as i32 as f32,
        rand::random::< bool >() as i32 as f32,
        0.,
      ];

      PointLight { position, color }
    }
  ).collect()
}

fn create_transforms( num : usize ) -> Box< [ Mat4 ] >
{
  ( 0 .. num ).map
  (
    | i |
    {
      let x = -4.5 + ( i % 10 ) as f32;
      let y = -0.7;
      let z = -3. - ( i / 10 ) as f32;
      let scale = 0.015;

      Mat4::from_row_major
      (
        [
          scale, 0.,    0.,    x,
          0.,    scale, 0.,    y,
          0.,    0.,    scale, z,
          0.,    0.,    0.,    1.,
        ]
      )
    }
  ).collect()
}

#[ repr( C ) ]
#[ derive( Clone, Copy, NoUninit ) ]
struct PointLight
{
  position : [ f32; 4 ],
  color : [ f32; 4 ],
}
