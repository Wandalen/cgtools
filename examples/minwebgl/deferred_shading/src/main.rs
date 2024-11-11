mod framebuffer;

use minwebgl as gl;
use gl::{ GL, JsCast as _ };
use framebuffer::*;
use bytemuck::NoUninit;
use ndarray_cg::d2::Mat4;
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
  gl.enable( GL::DEPTH_TEST );
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

  let meshes = load_meshes( &models, &gl )?;
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

  const BINDING_POINT : u32 = 0;
  gl.uniform_block_binding( &deferred_shader, lights_index, BINDING_POINT );

  let lights = create_lights( 100 );
  let lights_ubo = gl::buffer::create( &gl ).unwrap();
  gl::ubo::upload
  (
    &gl,
    &lights_ubo,
    BINDING_POINT,
    bytemuck::cast_slice::< _, u8 >( &lights ),
    GL::STATIC_DRAW
  );

  let positionbuffer = create_tex( &gl, GL::RGBA16F, width, height );
  let normalbuffer = create_tex( &gl, GL::RGBA16F, width, height );
  let depthbuffer = gl.create_renderbuffer().unwrap();
  gl.bind_renderbuffer( GL::RENDERBUFFER, Some( &depthbuffer ) );
  gl.renderbuffer_storage( GL::RENDERBUFFER, GL::DEPTH_COMPONENT16, width, height );

  let framebuffer = gl.create_framebuffer();
  gl.bind_framebuffer( GL::DRAW_FRAMEBUFFER, framebuffer.as_ref() );
  // gl.framebuffer_
  // gl.framebuffer_texture_2d( GL::FRAMEBUFFER, GL::COLOR_ATTACHMENT0, GL::TEXTURE_2D, Some( &positionbuffer ), 0 );
  // gl.framebuffer_texture_2d( GL::FRAMEBUFFER, GL::COLOR_ATTACHMENT1, GL::TEXTURE_2D, Some( &normalbuffer ), 0 );
  // gl.framebuffer_texture_layer(target, attachment, texture, level, layer);
  gl.framebuffer_renderbuffer( GL::FRAMEBUFFER, GL::DEPTH_ATTACHMENT, GL::RENDERBUFFER, Some( &depthbuffer ) );

  // let framebuffer = FramebufferBuilder::new()
  // .color_attachment( ColorAttachment::N0, Attachment::Texture( positionbuffer ) )
  // .color_attachment( ColorAttachment::N1, Attachment::Texture( normalbuffer ) )
  // .depth_attachment( DepthAttachment::Depth, Attachment::Renderbuffer( depthbuffer ) )
  // .build( &gl )?;

  // draw data into framebuffer
  gl.use_program( Some( &object_shader ) );
  gl.draw_buffers
  (
    &web_sys::js_sys::Array::from_iter
    (
      [
        gl::JsValue::from_f64( GL::COLOR_ATTACHMENT0 as f64 ),
        gl::JsValue::from_f64( GL::COLOR_ATTACHMENT1 as f64 )
      ].into_iter()
    )
  );
  // framebuffer.bind_draw( &gl );

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
    draw_meshes( &meshes, &gl );
  }

  gl.use_program( Some( &deferred_shader ) );
  gl.bind_framebuffer( GL::FRAMEBUFFER, None );
  gl.bind_buffer( GL::UNIFORM_BUFFER, Some( &lights_ubo ) );
  gl.active_texture( GL::TEXTURE0 );
  // gl.bind_texture( GL::TEXTURE_2D, Some( framebuffer[ ColorAttachment::N0 ].unwrap_texture() ) );
  gl.bind_texture( GL::TEXTURE_2D, Some( &positionbuffer ) );
  gl.active_texture( GL::TEXTURE1 );
  // gl.bind_texture( GL::TEXTURE_2D, Some( framebuffer[ ColorAttachment::N1 ].unwrap_texture() ) );
  gl.bind_texture( GL::TEXTURE_2D, Some( &normalbuffer ) );

  let draw_loop = move | t |
  {
    let time = ( t / 1000. ) as f32;
    let mut lights = lights.clone();
    for light in &mut lights
    {
      light.position[ 2 ] += 80. + -time % 8. * 20.;
    }
    gl.buffer_sub_data_with_f64_and_u8_array( GL::UNIFORM_BUFFER, 0., bytemuck::cast_slice( &lights ) );
    gl.draw_arrays( GL::TRIANGLES, 0, 3 );

    true
  };

  gl::exec_loop::run( draw_loop );

  Ok( () )
}

fn create_tex( gl : &GL, format : u32, width : i32, height : i32 ) -> WebGlTexture
{
  let tex = gl.create_texture().unwrap();
  gl.bind_texture( GL::TEXTURE_2D, Some( &tex ) );
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

#[ repr( C ) ]
#[ derive( Clone, Copy, NoUninit ) ]
struct PointLight
{
  position : [ f32; 4 ],
  color : [ f32; 4 ],
}

fn create_lights( num : usize ) -> Vec< PointLight >
{
  let mut lights = vec![];
  for i in 0..num
  {
    let z = ( i / 5 + 2 ) as f32 * -4.;
    let x = ( -2. + ( i % 5 ) as f32 ) * 2.;
    let position = [ x, 1., z, 0. ];

    let color =
    [
      rand::random::< bool >() as i32 as f32,
      rand::random::< bool >() as i32 as f32,
      rand::random::< bool >() as i32 as f32,
      0.,
    ];

    lights.push( PointLight { position, color } );
  }

  lights
}

fn create_transforms( num : usize ) -> Vec< Mat4< f32 > >
{
  let mut objects = vec![];
  for i in 0 .. num
  {
    let z = ( i / 5 + 2 ) as f32 * -2.;
    let x = ( -2. + ( i % 5 ) as f32 ) * 1.3;

    let translation = Mat4::from_row_major
    (
      [
        1., 0., 0.,  x,
        0., 1., 0., -1.5,
        0., 0., 1.,  z,
        0., 0., 0.,  1.,
      ]
    );
    let scale = Mat4::from_row_major
    (
      [
        0.03, 0.,   0.,   0.,
        0.,   0.03, 0.,   0.,
        0.,   0.,   0.03, 0.,
        0.,   0.,   0.,   1.,
      ]
    );

    let model = translation * scale;

    objects.push( model );
  }

  objects
}
