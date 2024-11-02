mod controls;
mod framebuffer;
mod programs;

use minwebgl as gl;
use gl::GL;
use framebuffer::*;
use rand::Rng as _;
use std::
{
  cell::RefCell,
  f32,
  ops::RangeInclusive,
  rc::Rc
};
use web_sys::
{
  wasm_bindgen::prelude::*,
  HtmlImageElement,
  WebGlTexture
};

fn main()
{
  gl::browser::setup( Default::default() );
  gl::spawn_local( async { gl::info!( "{:?}", run().await ) } );
}

async fn run() -> Result< (), gl::WebglError >
{
  let window = web_sys::window().unwrap();
  let width = window.inner_width().unwrap().as_f64().unwrap() as i32;
  let height = window.inner_height().unwrap().as_f64().unwrap() as i32;

  let canvas = gl::canvas::retrieve().unwrap();
  canvas.set_width( width as u32 );
  canvas.set_height( height as u32 );

  let gl = gl::context::from_canvas( &canvas ).unwrap();
  gl.viewport( 0, 0, width, height );
  gl.enable( GL::DEPTH_TEST );
  gl.enable( GL::CULL_FACE );

  let mut framebuffer : Framebuffer< 2 > = Framebuffer::new( &gl ).unwrap();
  let color_texture = texture2d( &gl, GL::RGBA8, width, height );
  let id_texture = texture2d( &gl, GL::R32I, width, height );
  let renderbuffer = depth_buffer( &gl, width, height );
  framebuffer.attach_texture( 0, color_texture, &gl );
  framebuffer.attach_texture( 1, id_texture, &gl );
  framebuffer.attach_renderbuffer( renderbuffer, GL::DEPTH_ATTACHMENT, &gl );

  // shader for drawing a single object
  let object_shader = programs::single::Single::new( &gl );
  // shader for drawing outline
  let outline_shader = programs::outline::Outline::new( &gl );
  // shader for rasterizing the whole screen
  let rasterize_shader = programs::rasterize::Rasterize::new( &gl );

  let obj = gl::file::load( "cat/Cat.obj" ).await.unwrap();
  let ( models, materials ) = gl::obj::load_model_from_slice( &obj, "cat", &tobj::GPU_LOAD_OPTIONS )
  .await
  .expect( "Can't read model" );
  let materials = materials.expect( "Can't load materials" );
  let meshes : Box< [ _ ] > = load_meshes( &models, &materials, &gl ).await.into();

  let object_count = 50;
  let objects = create_objects( object_count );

  let aspect_ratio = gl.drawing_buffer_width() as f32 / gl.drawing_buffer_height() as f32;
  let projection = glam::Mat4::perspective_rh( 45.0_f32.to_radians(), aspect_ratio, 0.1, 1000.0 );


  // let cursor_controls = controls::CursorControls { sensitivity : 2.0 };
  // let cursor_controls = Rc::new( RefCell::new( cursor_controls ) );
  let click_pos = Rc::new( RefCell::new( [ 0; 2 ] ) );

  let mut selected = -1;
  let id = web_sys::js_sys::Int32Array::new_with_length( 1 );

  framebuffer.bind_all( &gl );

  gl.clear_bufferfv_with_f32_array( gl::COLOR, 0, [ 1.0, 1.0, 1.0, 1.0 ].as_slice() );
  gl.clear_bufferiv_with_i32_array( gl::COLOR, 1, [ -1, -1, -1, -1 ].as_slice() );
  gl.clear( GL::DEPTH_BUFFER_BIT );

  gl.use_program( Some( &object_shader.program ) );

  gl::uniform::matrix_upload
  (
    &gl,
    object_shader.projection_view.clone(),
    projection.to_cols_array().as_slice(),
    true
  ).unwrap();

  for object in &objects
  {
    let model = object.transform;
    let nmat = model.matrix3.inverse().transpose();
    let model : glam::Mat4 = model.into();

    gl::uniform::matrix_upload
    (
      &gl,
      object_shader.model.clone(),
      model.to_cols_array().as_slice(),
      true
    ).unwrap();
    gl::uniform::matrix_upload
    (
      &gl,
      object_shader.norm_mat.clone(),
      nmat.to_cols_array().as_slice(),
      true
    ).unwrap();
    gl::uniform::upload
    (
      &gl,
      object_shader.id.clone(),
      &object.id
    ).unwrap();

    draw_meshes( meshes.as_ref(), &gl );
  }

  let pos = [ 0, 0 ];

  if pos != [ -1; 2 ]
  {
    gl.read_buffer( GL::COLOR_ATTACHMENT1 );
    gl.read_pixels_with_array_buffer_view_and_dst_offset
    (
      pos[ 0 ],
      pos[ 1 ],
      1,
      1,
      GL::RED_INTEGER,
      GL::INT,
      &id,
      0
    ).unwrap();

    selected = id.to_vec()[ 0 ];
    gl::info!( "{selected}" );
  }

  // if selected != -1
  // {
  //   let model = objects[ selected as usize ].transform;
  //   let nmat = model.matrix3.inverse().transpose();
  //   let model : glam::Mat4 = model.into();

  //   gl.use_program( Some( &outline_shader.program ) );
  //   // gl::uniform::matrix_upload
  //   // (
  //   //   &gl,
  //   //   outline_shader.mvp_location.clone(),
  //   //   ( projection_view * model ).to_cols_array().as_slice(),
  //   //   true
  //   // ).unwrap();

  //   gl.disable( GL::DEPTH_TEST );
  //   framebuffer.bind_nth( 0, &gl );
  //   draw_meshes( meshes.as_ref(), &gl );

  //   gl.use_program( Some( &object_shader.program ) );
  //   gl::uniform::matrix_upload
  //   (
  //     &gl,
  //     object_shader.model.clone(),
  //     model.to_cols_array().as_slice(),
  //     true
  //   ).unwrap();
  //   // gl::uniform::matrix_upload
  //   // (
  //   //   &gl,
  //   //   single_shader.projection_view.clone(),
  //   //   projection_view.to_cols_array().as_slice(),
  //   //   true
  //   // ).unwrap();
  //   gl::uniform::matrix_upload
  //   (
  //     &gl,
  //     object_shader.norm_mat.clone(),
  //     nmat.to_cols_array().as_slice(),
  //     true
  //   ).unwrap();

  //   gl.enable( GL::DEPTH_TEST );
  //   gl.clear( GL::DEPTH_BUFFER_BIT );
  //   draw_meshes( meshes.as_ref(), &gl );
  // }

  gl.use_program( Some( &rasterize_shader.program ) );

  gl.bind_framebuffer( GL::FRAMEBUFFER, None );
  gl.bind_texture( GL::TEXTURE_2D, framebuffer.get_color_attachment( 0 ) );
  gl.draw_arrays( GL::TRIANGLES, 0, 3 );


  Ok( () )
}

fn draw_meshes_instanced( meshes : &[ Mesh ], instance_count : i32, gl : &GL )
{
  for mesh in meshes.iter()
  {
    gl.bind_vertex_array( Some( &mesh.vao ) );
    gl.bind_texture( GL::TEXTURE_2D, mesh.diffuse_texture.as_ref() );
    gl.draw_elements_instanced_with_i32( GL::TRIANGLES, mesh.index_count, GL::UNSIGNED_INT, 0, instance_count );
  }
}

fn draw_meshes( meshes : &[ Mesh ], gl : &GL )
{
  for mesh in meshes
  {
    gl.bind_vertex_array( Some( &mesh.vao ) );
    gl.bind_texture( GL::TEXTURE_2D, mesh.diffuse_texture.as_ref() );
    gl.draw_elements_with_i32( GL::TRIANGLES, mesh.index_count, GL::UNSIGNED_INT, 0 );
  }
}

async fn load_meshes( models : &[ tobj::Model ], materials : &[ tobj::Material ], gl : &GL ) -> Vec< Mesh >
{
  let mut meshes = vec![];
  for ( model, material ) in models.iter().zip( materials )
  {
    let position_buffer = gl::buffer::create( gl ).unwrap();
    gl::buffer::upload( gl, &position_buffer, model.mesh.positions.as_slice(), GL::STATIC_DRAW );
    let normal_buffer = gl::buffer::create( gl ).unwrap();
    gl::buffer::upload( gl, &normal_buffer, model.mesh.normals.as_slice(), GL::STATIC_DRAW );
    let texcoord_buffer = gl::buffer::create( gl ).unwrap();
    gl::buffer::upload( gl, &texcoord_buffer, model.mesh.texcoords.as_slice(), GL::STATIC_DRAW );

    let vao = gl::vao::create( gl ).unwrap();
    gl.bind_vertex_array( Some( &vao ) );

    let index_buffer = gl::buffer::create( gl ).unwrap();
    gl::index::upload( gl, &index_buffer, model.mesh.indices.as_slice(), GL::STATIC_DRAW );

    gl::BufferDescriptor::new::< [ f32; 3 ] >()
    .stride( 0 )
    .offset( 0 )
    .attribute_pointer( gl, 0, &position_buffer)
    .unwrap();

    gl::BufferDescriptor::new::< [ f32; 3 ] >()
    .stride( 0 )
    .offset( 0 )
    .attribute_pointer( gl, 1, &normal_buffer)
    .unwrap();

    gl::BufferDescriptor::new::< [ f32; 2 ] >()
    .stride( 0 )
    .offset( 0 )
    .attribute_pointer( gl, 2, &texcoord_buffer)
    .unwrap();

    let texture = if let Some( name ) = &material.diffuse_texture
    {
      load_image( &format!( "static/cat/{}", name ) ).await.map_or
      (
        None,
        | img |
        {
          let texture = gl::texture::d2::upload( gl, &img );
          gl.generate_mipmap( GL::TEXTURE_2D );
          gl.tex_parameteri( GL::TEXTURE_2D, GL::TEXTURE_MIN_FILTER, GL::LINEAR_MIPMAP_LINEAR as i32 );
          gl.tex_parameteri( GL::TEXTURE_2D, GL::TEXTURE_MAG_FILTER, GL::LINEAR as i32 );
          img.remove();
          texture
        }
      )
    }
    else
    {
      None
    };

    meshes.push( Mesh { vao, index_count : model.mesh.indices.len() as i32, diffuse_texture: texture } );
  }

  meshes
}

async fn load_image( src : &str ) -> Option< HtmlImageElement >
{
  let doc = web_sys::window().unwrap().document().unwrap();
  let img = doc.create_element( "img" ).unwrap().dyn_into::< HtmlImageElement >().unwrap();
  let ( sender, receiver ) = futures::channel::oneshot::channel();
  let onload_closure = Closure::once( move || sender.send( () ).unwrap() );
  img.set_onload( Some( onload_closure.as_ref().unchecked_ref() ) );
  onload_closure.forget();
  img.set_src( src );

  if let Err( _ ) = receiver.await
  {
    None
  }
  else
  {
    Some( img )
  }
}

fn create_objects( count : i32 ) -> Vec< Object >
{
  ( 0..count )
  .into_iter()
  .map( | i | Object { transform : random_transform(), id : i } )
  .collect()
}

fn random_transform() -> glam::Affine3A
{
  let pos_x = rand::thread_rng().gen_range( -50.0..=50.0 );
  let pos_y = rand::thread_rng().gen_range( -50.0..=50.0 );
  let pos_z = rand::thread_rng().gen_range( -100.0..=-50.0 );

  let rot_x = rand::thread_rng().gen_range( 0.0..std::f32::consts::PI * 2.0 );
  let rot_y = rand::thread_rng().gen_range( 0.0..std::f32::consts::PI * 2.0 );
  let rot_z = rand::thread_rng().gen_range( 0.0..std::f32::consts::PI * 2.0 );

  let scale = rand::thread_rng().gen_range( 0.2..=0.8 );

  glam::Affine3A::from_scale_rotation_translation
  (
    glam::vec3( scale, scale, scale ),
    glam::Quat::from_euler( glam::EulerRot::XYZ, rot_x, rot_y, rot_z ),
    glam::vec3( pos_x, pos_y, pos_z ),
  )
}

struct Object
{
  transform : glam::Affine3A,
  id : i32,
}

struct Mesh
{
  vao : gl::WebGlVertexArrayObject,
  diffuse_texture : Option< WebGlTexture >,
  index_count : i32,
}
