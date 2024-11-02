mod framebuffer;
mod shaders;

use minwebgl as gl;
use gl::GL;
use framebuffer::*;
use rand::Rng as _;
use web_sys::
{
  wasm_bindgen::prelude::*,
  HtmlImageElement,
  MouseEvent,
  WebGlRenderbuffer,
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

  let obj = gl::file::load( "cat/Cat.obj" ).await.unwrap();
  let ( models, materials ) = gl::obj::load_model_from_slice( &obj, "cat", &tobj::GPU_LOAD_OPTIONS )
  .await
  .expect( "Can't read model" );
  let materials = materials.expect( "Can't load materials" );
  let meshes : Box< [ _ ] > = load_meshes( &models, &materials, &gl ).await.into();

  // create framebuffer for id texture
  let mut framebuffer = Framebuffer::new( &gl ).unwrap();
  let id_texture = texture2d( &gl, GL::R32I, width, height );
  let depthbuffer = depthbuffer( &gl, width, height );
  framebuffer.attach_texture( id_texture, &gl );
  framebuffer.renderbuffer( depthbuffer, RenderbufferAttachment::Depth, &gl );

  // shader for drawing a single object
  let object_shader = shaders::ObjectShader::new( &gl );
  // shader for drawing object's id into texture
  let id_shader = shaders::IdShader::new( &gl );
  // shader for drawing outline
  let outline_shader = shaders::OutlineShader::new( &gl );

  let objects = create_objects();

  let aspect_ratio = width as f32 / height as f32;
  let projection = glam::Mat4::perspective_rh( 45.0_f32.to_radians(), aspect_ratio, 0.1, 1000.0 );

  // draw ids into texture
  gl.use_program( Some( &id_shader.program ) );

  framebuffer.bind_drawbuffers( &gl );
  gl.clear_bufferiv_with_i32_array( gl::COLOR, 0, [ -1, -1, -1, -1 ].as_slice() );
  gl.clear( GL::DEPTH_BUFFER_BIT );

  for object in &objects
  {
    let mvp = projection * object.transform;
    gl::uniform::matrix_upload( &gl, id_shader.mvp.clone(), mvp.to_cols_array().as_slice(), true ).unwrap();
    gl::uniform::upload( &gl, id_shader.id.clone(), &object.id ).unwrap();
    draw_meshes( &meshes, &gl );
  }

  gl.bind_framebuffer( GL::FRAMEBUFFER, None );

  // set projection to object shader once
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

    draw_meshes( meshes.as_ref(), &gl );
  }

  let id = web_sys::js_sys::Int32Array::new_with_length( 1 );

  let draw_closure = move | e : MouseEvent |
  {
    // draw all the objects
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

      draw_meshes( meshes.as_ref(), &gl );
    }

    // click position
    let x = e.client_x();
    let y = height - e.client_y();
    let pos = [ x, y ];

    // read id of selected object from texture
    framebuffer.bind( &gl );
    gl.read_buffer( GL::COLOR_ATTACHMENT0 );
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
    gl.bind_framebuffer( GL::FRAMEBUFFER, None );

    let selected = id.to_vec()[ 0 ];

    // draw the object if it is selected
    if selected != -1
    {
      let transform = objects[ selected as usize ].transform;
      let nmat = transform.matrix3.inverse().transpose();
      let model : glam::Mat4 = transform.into();

      // draw outline
      gl.use_program( Some( &outline_shader.program ) );
      gl::uniform::matrix_upload
      (
        &gl,
        outline_shader.mvp.clone(),
        ( projection * model ).to_cols_array().as_slice(),
        true
      ).unwrap();

      gl.disable( GL::DEPTH_TEST );
      draw_meshes( meshes.as_ref(), &gl );

      // draw object
      gl.use_program( Some( &object_shader.program ) );
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

      gl.enable( GL::DEPTH_TEST );
      gl.clear( GL::DEPTH_BUFFER_BIT );
      draw_meshes( meshes.as_ref(), &gl );
    }
  };
  let closure = Closure::< dyn Fn( _ ) >::new( Box::new( draw_closure ) );
  canvas.set_onclick( Some( closure.as_ref().unchecked_ref() ) );
  closure.forget();

  Ok( () )
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

fn create_objects() -> Vec< Object >
{
  let transforms =
  [
    glam::Affine3A::from_rotation_translation( random_rotation() , glam::vec3( -200.0,  100.0, -400.0 ) ),
    glam::Affine3A::from_rotation_translation( random_rotation() , glam::vec3( -100.0,  100.0, -400.0 ) ),
    glam::Affine3A::from_rotation_translation( random_rotation() , glam::vec3(    0.0,  100.0, -400.0 ) ),
    glam::Affine3A::from_rotation_translation( random_rotation() , glam::vec3(  100.0,  100.0, -400.0 ) ),
    glam::Affine3A::from_rotation_translation( random_rotation() , glam::vec3(  200.0,  100.0, -400.0 ) ),
    glam::Affine3A::from_rotation_translation( random_rotation() , glam::vec3( -200.0,    0.0, -400.0 ) ),
    glam::Affine3A::from_rotation_translation( random_rotation() , glam::vec3( -100.0,    0.0, -400.0 ) ),
    glam::Affine3A::from_rotation_translation( random_rotation() , glam::vec3(    0.0,    0.0, -400.0 ) ),
    glam::Affine3A::from_rotation_translation( random_rotation() , glam::vec3(  100.0,    0.0, -400.0 ) ),
    glam::Affine3A::from_rotation_translation( random_rotation() , glam::vec3(  200.0,    0.0, -400.0 ) ),
    glam::Affine3A::from_rotation_translation( random_rotation() , glam::vec3( -200.0, -100.0, -400.0 ) ),
    glam::Affine3A::from_rotation_translation( random_rotation() , glam::vec3( -100.0, -100.0, -400.0 ) ),
    glam::Affine3A::from_rotation_translation( random_rotation() , glam::vec3(    0.0, -100.0, -400.0 ) ),
    glam::Affine3A::from_rotation_translation( random_rotation() , glam::vec3(  100.0, -100.0, -400.0 ) ),
    glam::Affine3A::from_rotation_translation( random_rotation() , glam::vec3(  200.0, -100.0, -400.0 ) ),
  ];

  transforms
  .into_iter()
  .enumerate()
  .map( | ( i, t ) | Object { transform : t, id : i as i32 } )
  .collect()
}

fn random_rotation() -> glam::Quat
{
  let rot_x = rand::thread_rng().gen_range( 0.0..std::f32::consts::PI * 2.0 );
  let rot_y = rand::thread_rng().gen_range( 0.0..std::f32::consts::PI * 2.0 );
  let rot_z = rand::thread_rng().gen_range( 0.0..std::f32::consts::PI * 2.0 );

  glam::Quat::from_euler( glam::EulerRot::XYZ, rot_x, rot_y, rot_z )
}

fn texture2d( gl : &GL, internal_format : u32, width : i32, height : i32 ) -> Option< WebGlTexture >
{
  let texture = gl.create_texture();
  gl.bind_texture( GL::TEXTURE_2D, texture.as_ref() );
  gl.tex_storage_2d( GL::TEXTURE_2D, 1, internal_format, width, height );
  gl.tex_parameteri( GL::TEXTURE_2D, GL::TEXTURE_MIN_FILTER, GL::LINEAR as i32 );
  gl.tex_parameteri( GL::TEXTURE_2D, GL::TEXTURE_MAG_FILTER, GL::LINEAR as i32 );
  texture
}

fn depthbuffer( gl : &GL, width : i32, height : i32 ) -> Option< WebGlRenderbuffer >
{
  let renderbuffer = gl.create_renderbuffer();
  gl.bind_renderbuffer( GL::RENDERBUFFER, renderbuffer.as_ref() );
  gl.renderbuffer_storage( GL::RENDERBUFFER, GL::DEPTH_COMPONENT16, width, height );
  renderbuffer
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
