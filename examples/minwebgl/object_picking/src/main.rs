//! Object picking example — selects rendered objects under the mouse cursor with WebGL2.

mod shaders;

use minwebgl as gl;
use gl::{ GL, JsFuture };
use ndarray_cg::
{
  d2,
  mat::DescriptorOrderColumnMajor,
  vector::F32x3,
};
use d2::mat3x3h;
use rand::RngExt as _;
use std::{ cell::RefCell, rc::Rc };
use web_sys::
{
  js_sys,
  wasm_bindgen::prelude::*,
  MouseEvent,
  WebGlFramebuffer,
  WebGlRenderbuffer,
  WebGlTexture
};

fn main()
{
  gl::browser::setup( gl::browser::Config::default() );
  gl::spawn_local( async { gl::info!( "{:?}", app_run().await ) } );
}

async fn app_run() -> Result< (), gl::WebglError >
{
  let width =  1280;
  let height = 720;

  let canvas = gl::canvas::retrieve().unwrap();
  canvas.set_width( width as u32 );
  canvas.set_height( height as u32 );

  let gl = gl::context::from_canvas( &canvas ).unwrap();
  gl.viewport( 0, 0, width, height );
  gl.enable( GL::DEPTH_TEST );
  gl.enable( GL::CULL_FACE );

  let obj = gl::file::load( "static/cat/Cat.obj" ).await
  .map_err( | e | gl::dom::Error::BindgenError( "Failed to load static/cat/Cat.obj", format!( "{e:?}" ) ) )?;
  let ( models, materials ) = gl::model::model_load_from_slice( &obj, "static/cat", &tobj::GPU_LOAD_OPTIONS )
  .await
  .expect( "Can't read model" );
  let materials = materials.expect( "Can't load materials" );
  let meshes : Box< [ _ ] > = meshes_load( &models, &materials, &gl ).await.into();

  // create framebuffer for id texture
  let id_texture = empty_texture2d( &gl, GL::R32I, width, height );
  let depthbuffer = depthbuffer( &gl, width, height );
  let framebuffer = gl.create_framebuffer();
  gl.bind_framebuffer( GL::FRAMEBUFFER, framebuffer.as_ref() );
  gl.framebuffer_texture_2d( GL::FRAMEBUFFER, GL::COLOR_ATTACHMENT0, GL::TEXTURE_2D, id_texture.as_ref(), 0 );
  gl.framebuffer_renderbuffer( GL::FRAMEBUFFER, GL::DEPTH_ATTACHMENT, GL::RENDERBUFFER, depthbuffer.as_ref() );

  // shader for drawing a single object
  let object_shader = shaders::ObjectShader::new( &gl );
  // shader for drawing object's id into texture
  let id_shader = shaders::IdShader::new( &gl );
  // shader for drawing outline
  let outline_shader = shaders::OutlineShader::new( &gl );

  let objects : Box< [ _ ] > = objects_create().into();

  let aspect_ratio = width as f32 / height as f32;
  let fovy = 45.0_f32.to_radians();
  let projection = mat3x3h::perspective_rh_gl( fovy, aspect_ratio, 0.1, 1000.0 );

  let ctx = Rc::new
  (
    RenderCtx
    {
      gl,
      canvas,
      framebuffer,
      id_shader,
      object_shader,
      outline_shader,
      meshes,
      projection,
      width : width as f32,
      height : height as f32,
      tan_half_fovy : ( fovy / 2.0 ).tan(),
      aspect : aspect_ratio,
      pick_id : js_sys::Int32Array::new_with_length( 1 ),
    }
  );
  let state = Rc::new( RefCell::new( AppState { objects, selected : -1, dragging : None } ) );

  // seed the picking texture and draw the first frame
  ids_render( &ctx, &state.borrow().objects );

  // projection never changes after this, so object_shader only needs it uploaded once
  ctx.gl.use_program( Some( &ctx.object_shader.program ) );
  gl::uniform::matrix_upload
  (
    &ctx.gl,
    ctx.object_shader.projection_view.clone(),
    ctx.projection.raw_slice(),
    true
  ).unwrap();

  scene_redraw( &ctx, &state.borrow().objects, -1 );

  let window = web_sys::window().expect( "no global `window` exists" );

  // `mousedown` stays canvas-scoped ( a press must start over a rendered object ), but
  // `mousemove`/`mouseup` are attached to `window` instead of `canvas` so a drag survives the
  // cursor leaving canvas bounds mid-drag rather than getting stuck holding the object.
  setup_mousedown( &ctx, &state );
  setup_mousemove( &ctx, &state, &window );
  setup_mouseup( &state, &window );

  Ok( () )
}

/// Wires the pick-and-grab handler: on a left-press over an object, re-renders the picking
/// texture ( positions may have changed since the last render ), hit-tests the cursor, and — on a
/// hit — records a `DragState` capturing the grab-point offset and the depth plane to drag along.
fn setup_mousedown( ctx : &Rc< RenderCtx >, state : &Rc< RefCell< AppState > > )
{
  let canvas = ctx.canvas.clone();
  let state = state.clone();
  let ctx = ctx.clone();

  let handler = move | e : MouseEvent |
  {
    if e.button() != 0
    {
      return;
    }
    e.prevent_default();

    let pixel = cursor_canvas_pixel( &e, &ctx.canvas, ctx.height );

    let mut state = state.borrow_mut();
    ids_render( &ctx, &state.objects );
    let id = pick_object_id( &ctx, pixel );

    if id != -1
    {
      let object = &state.objects[ id as usize ];
      let depth = object.translation.0[ 2 ];
      let ( world_x, world_y ) = cursor_to_world_at_depth
      (
        pixel, ( ctx.width, ctx.height ), ctx.tan_half_fovy, ctx.aspect, depth
      );
      let grab_offset = ( object.translation.0[ 0 ] - world_x, object.translation.0[ 1 ] - world_y );

      state.selected = id;
      state.dragging = Some( DragState { id, depth, grab_offset } );
      scene_redraw( &ctx, &state.objects, state.selected );
    }
  };
  let closure = Closure::< dyn FnMut( _ ) >::new( Box::new( handler ) );
  canvas.set_onmousedown( Some( closure.as_ref().unchecked_ref() ) );
  closure.forget();
}

/// While a drag is active, unprojects the cursor onto the drag's fixed depth plane and moves the
/// held object so the originally-grabbed point stays under the cursor. A no-op when not dragging.
fn setup_mousemove( ctx : &Rc< RenderCtx >, state : &Rc< RefCell< AppState > >, window : &web_sys::Window )
{
  let state = state.clone();
  let ctx = ctx.clone();

  let handler = move | e : MouseEvent |
  {
    let mut state = state.borrow_mut();
    let Some( drag ) = state.dragging else { return };

    let pixel = cursor_canvas_pixel( &e, &ctx.canvas, ctx.height );
    let ( world_x, world_y ) = cursor_to_world_at_depth
    (
      pixel, ( ctx.width, ctx.height ), ctx.tan_half_fovy, ctx.aspect, drag.depth
    );

    let object = &mut state.objects[ drag.id as usize ];
    object.translation.0[ 0 ] = world_x + drag.grab_offset.0;
    object.translation.0[ 1 ] = world_y + drag.grab_offset.1;

    let selected = state.selected;
    scene_redraw( &ctx, &state.objects, selected );
  };
  let closure = Closure::< dyn FnMut( _ ) >::new( Box::new( handler ) );
  window.set_onmousemove( Some( closure.as_ref().unchecked_ref() ) );
  closure.forget();
}

/// Releases the current drag, if any. Window-scoped so a release outside canvas bounds still
/// ends the drag cleanly.
fn setup_mouseup( state : &Rc< RefCell< AppState > >, window : &web_sys::Window )
{
  let state = state.clone();

  let handler = move | _e : MouseEvent |
  {
    state.borrow_mut().dragging = None;
  };
  let closure = Closure::< dyn FnMut( _ ) >::new( Box::new( handler ) );
  window.set_onmouseup( Some( closure.as_ref().unchecked_ref() ) );
  closure.forget();
}

fn outline_draw
(
  objects : &[ Object ],
  object_shader : &shaders::ObjectShader,
  outline_shader : &shaders::OutlineShader,
  meshes : &[ Mesh ],
  selected : i32,
  projection : d2::Mat4< f32, DescriptorOrderColumnMajor >,
  gl : &GL,
)
{
  let transform = object_transform( &objects[ selected as usize ] );

  // this is not the optimal way to draw an outline
  // but it is done so for simplicity

  // basically just draw an extruded version of object
  // with some solid color and then overdraw the actual
  // object above it

  // draw outline
  gl.use_program( Some( &outline_shader.program ) );
  gl::uniform::matrix_upload
  (
    gl,
    outline_shader.mvp.clone(),
    ( projection * transform ).raw_slice(),
    true
  ).unwrap();

  gl.disable( GL::DEPTH_TEST );
  meshes_draw( meshes, gl );

  // draw object
  gl.use_program( Some( &object_shader.program ) );
  gl::uniform::matrix_upload
  (
    gl,
    object_shader.model.clone(),
    transform.raw_slice(),
    true
  ).unwrap();

  gl.enable( GL::DEPTH_TEST );
  gl.clear( GL::DEPTH_BUFFER_BIT );
  meshes_draw( meshes, gl );
}

fn objects_draw( objects : &[ Object ], object_shader : &shaders::ObjectShader, meshes : &[ Mesh ], gl : &GL )
{
  for object in objects
  {
    gl::uniform::matrix_upload
    (
      gl,
      object_shader.model.clone(),
      object_transform( object ).raw_slice(),
      true
    ).unwrap();

    meshes_draw( meshes, gl );
  }
}

/// Composes an object's fixed `rotation` with its current ( possibly drag-updated ) `translation`.
/// `M = T * R` has the same translation column as `T` alone ( `R`'s bottom row is `[0,0,0,1]` and
/// its translation column is zero ), so splitting the two and recomposing here — instead of storing
/// one baked `Mat4` and needing decomposition to move an object — is exact, not an approximation.
fn object_transform( object : &Object ) -> ndarray_cg::Mat4< f32, DescriptorOrderColumnMajor >
{
  mat3x3h::translation( object.translation ) * object.rotation
}

/// Re-renders every object's id into the offscreen picking texture at its current transform.
/// Objects move when dragged, so the id texture must be refreshed before every pick — the initial
/// startup pass alone would go stale the moment any object is moved.
fn ids_render( ctx : &RenderCtx, objects : &[ Object ] )
{
  ctx.gl.use_program( Some( &ctx.id_shader.program ) );

  ctx.gl.bind_framebuffer( GL::FRAMEBUFFER, ctx.framebuffer.as_ref() );
  ctx.gl.clear_bufferiv_with_i32_array( gl::COLOR, 0, [ -1, -1, -1, -1 ].as_slice() );
  ctx.gl.clear( GL::DEPTH_BUFFER_BIT );

  for object in objects
  {
    let mvp = ctx.projection * object_transform( object );
    gl::uniform::matrix_upload( &ctx.gl, ctx.id_shader.mvp.clone(), mvp.raw_slice(), true ).unwrap();
    gl::uniform::upload( &ctx.gl, ctx.id_shader.id.clone(), &object.id ).unwrap();
    meshes_draw( &ctx.meshes, &ctx.gl );
  }

  ctx.gl.bind_framebuffer( GL::FRAMEBUFFER, None );
}

/// Draws every object, plus an outline around `selected` when an object is currently selected.
/// The single redraw entry point shared by startup and all three pointer-event handlers.
fn scene_redraw( ctx : &RenderCtx, objects : &[ Object ], selected : i32 )
{
  ctx.gl.use_program( Some( &ctx.object_shader.program ) );
  objects_draw( objects, &ctx.object_shader, &ctx.meshes, &ctx.gl );

  if selected != -1
  {
    outline_draw( objects, &ctx.object_shader, &ctx.outline_shader, &ctx.meshes, selected, ctx.projection, &ctx.gl );
  }
}

/// Converts a `MouseEvent`'s window-relative client coordinates into canvas-local, bottom-up
/// pixel space — the convention the id-picking texture's `read_pixels` call already uses. Shared
/// by the press handler ( hit-test ) and the move handler ( drag unprojection ) so both always
/// agree on the same cursor position; safe to call even while the cursor is outside canvas bounds,
/// since `mousemove`/`mouseup` are window-scoped so a drag survives that.
fn cursor_canvas_pixel( e : &MouseEvent, canvas : &web_sys::HtmlCanvasElement, height : f32 ) -> ( f64, f64 )
{
  let rect = canvas.get_bounding_client_rect();
  // Fix(BUG-053): `client_x`/`client_y` return `i32` or `f64` depending on whether
  // `web_sys_unstable_apis` is active (see minwebgl/src/texture/d2.rs); the explicit `f64`
  // annotation makes `.into()` resolve correctly in both cases (`i32: Into<f64>` widens,
  // `f64: Into<f64>` is identity), matching `rect.left()`/`rect.top()`.
  #[ allow( clippy::useless_conversion, reason = "cfg-dependent per the Fix(BUG-053) note above — the conversion is an identity only under the web_sys_unstable_apis f64 signature, so expect would be unfulfilled in the default i32 build" ) ]
  let x : f64 = e.client_x().into();
  #[ allow( clippy::useless_conversion, reason = "cfg-dependent per the Fix(BUG-053) note above — the conversion is an identity only under the web_sys_unstable_apis f64 signature, so expect would be unfulfilled in the default i32 build" ) ]
  let y : f64 = e.client_y().into();

  let x = x - rect.left();
  let y = f64::from( height ) - ( y - rect.top() );
  ( x, y )
}

/// Reads the id-picking texture at `pixel` ( canvas-local, bottom-up — `cursor_canvas_pixel`'s
/// convention ) and returns the object id found there, or `-1` if none. Caller must have already
/// re-rendered the picking texture via `ids_render` for this to reflect current object positions.
fn pick_object_id( ctx : &RenderCtx, pixel : ( f64, f64 ) ) -> i32
{
  ctx.gl.bind_framebuffer( GL::FRAMEBUFFER, ctx.framebuffer.as_ref() );
  ctx.gl.read_buffer( GL::COLOR_ATTACHMENT0 );
  ctx.gl.read_pixels_with_array_buffer_view_and_dst_offset
  (
    pixel.0 as i32,
    pixel.1 as i32,
    1,
    1,
    GL::RED_INTEGER,
    GL::INT,
    &ctx.pick_id,
    0
  ).unwrap();
  ctx.gl.bind_framebuffer( GL::FRAMEBUFFER, None );

  ctx.pick_id.to_vec()[ 0 ]
}

/// Unprojects a canvas-local pixel position ( bottom-up, `cursor_canvas_pixel`'s convention ) onto
/// the world-space plane at camera-space `depth` ( negative — in front of the camera ), inverting
/// exactly the x/y terms `perspective_rh_gl` encodes for that depth: `ndc = (f * view) / (-view_z)`
/// where `f = 1/tan(fovy/2)` ( x additionally divided by `aspect` ), so `view = ndc * (-view_z) / f`.
/// This scene has no separate view matrix ( `mvp = projection * object_transform`, camera fixed at
/// the origin looking down -Z ), so camera space and world space coincide and this is a direct
/// world-space unprojection. Dragging keeps `depth` fixed at the value captured when the object was
/// grabbed, matching the 2D `cursor_to_arena` mapping used for the flecs_bouncing_circles example.
fn cursor_to_world_at_depth
(
  pixel : ( f64, f64 ),
  canvas_size : ( f32, f32 ),
  tan_half_fovy : f32,
  aspect : f32,
  depth : f32,
) -> ( f32, f32 )
{
  let ndc_x = ( pixel.0 as f32 / canvas_size.0 ) * 2.0 - 1.0;
  let ndc_y = ( pixel.1 as f32 / canvas_size.1 ) * 2.0 - 1.0;

  let world_x = ndc_x * -depth * aspect * tan_half_fovy;
  let world_y = ndc_y * -depth * tan_half_fovy;
  ( world_x, world_y )
}

fn meshes_draw( meshes : &[ Mesh ], gl : &GL )
{
  for mesh in meshes
  {
    gl.bind_vertex_array( Some( &mesh.vao ) );
    gl.bind_texture( GL::TEXTURE_2D, mesh.diffuse_texture.as_ref() );
    gl.draw_elements_with_i32( GL::TRIANGLES, mesh.index_count, GL::UNSIGNED_INT, 0 );
  }
}

async fn meshes_load( models : &[ tobj::Model ], materials : &[ tobj::Material ], gl : &GL ) -> Vec< Mesh >
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
    .attribute_pointer( gl, 0, &position_buffer )
    .unwrap();

    gl::BufferDescriptor::new::< [ f32; 3 ] >()
    .stride( 0 )
    .offset( 0 )
    .attribute_pointer( gl, 1, &normal_buffer )
    .unwrap();

    gl::BufferDescriptor::new::< [ f32; 2 ] >()
    .stride( 0 )
    .offset( 0 )
    .attribute_pointer( gl, 2, &texcoord_buffer )
    .unwrap();

    let texture = if let Some( name ) = &material.diffuse_texture
    {
      let img = gl::dom::image_element_create( &format!( "static/cat/{name}" ) ).unwrap();
      // tried to do texture uploading in on_load callback
      // but i had visual artifacts on the texture
      // like, some black spots for some reason
      // so i decided to await for image to load and that worked
      let load_promise = js_sys::Promise::new
      (
        &mut | resolve, reject |
        {
          let on_load = Closure::once_into_js
          (
            move || { resolve.call0( &JsValue::NULL ).unwrap() }
          );

          let on_error = Closure::once_into_js
          (
            move || { reject.call0( &JsValue::NULL ).unwrap() }
          );

          img.set_onload( Some( on_load.as_ref().unchecked_ref() ) );
          img.set_onerror( Some( on_error.as_ref().unchecked_ref() ) );
        }
      );
      JsFuture::from( load_promise ).await.unwrap();
      let texture = gl.create_texture();
      gl::texture::d2::upload( gl, texture.as_ref(), &img );
      gl.generate_mipmap( GL::TEXTURE_2D );
      gl.tex_parameteri( GL::TEXTURE_2D, GL::TEXTURE_MIN_FILTER, GL::LINEAR_MIPMAP_LINEAR as i32 );

      texture
    }
    else
    {
      None
    };

    meshes.push( Mesh { vao, index_count : model.mesh.indices.len() as i32, diffuse_texture : texture } );
  }

  meshes
}

fn objects_create() -> Vec< Object >
{
  let transforms =
  [
    ( random_rotation(), F32x3::new( -200.,  100., -400. ) ),
    ( random_rotation(), F32x3::new( -100.,  100., -400. ) ),
    ( random_rotation(), F32x3::new(    0.,  100., -400. ) ),
    ( random_rotation(), F32x3::new(  100.,  100., -400. ) ),
    ( random_rotation(), F32x3::new(  200.,  100., -400. ) ),
    ( random_rotation(), F32x3::new( -200.,    0., -400. ) ),
    ( random_rotation(), F32x3::new( -100.,    0., -400. ) ),
    ( random_rotation(), F32x3::new(    0.,    0., -400. ) ),
    ( random_rotation(), F32x3::new(  100.,    0., -400. ) ),
    ( random_rotation(), F32x3::new(  200.,    0., -400. ) ),
    ( random_rotation(), F32x3::new( -200., -100., -400. ) ),
    ( random_rotation(), F32x3::new( -100., -100., -400. ) ),
    ( random_rotation(), F32x3::new(    0., -100., -400. ) ),
    ( random_rotation(), F32x3::new(  100., -100., -400. ) ),
    ( random_rotation(), F32x3::new(  200., -100., -400. ) ),
  ];

  transforms
  .into_iter()
  .enumerate()
  .map( | ( i, ( r, t ) ) | Object { rotation : r, translation : t, id : i as i32 } )
  .collect()
}

fn random_rotation() -> ndarray_cg::Mat4< f32, DescriptorOrderColumnMajor >
{
  let rot_x = rand::rng().random_range( 0. .. std::f32::consts::PI * 2. );
  let rot_y = rand::rng().random_range( 0. .. std::f32::consts::PI * 2. );
  let rot_z = rand::rng().random_range( 0. .. std::f32::consts::PI * 2. );
  mat3x3h::rot( rot_x, rot_y, rot_z )
}

fn empty_texture2d( gl : &GL, internal_format : u32, width : i32, height : i32 ) -> Option< WebGlTexture >
{
  let texture = gl.create_texture();
  gl.bind_texture( GL::TEXTURE_2D, texture.as_ref() );
  gl.tex_storage_2d( GL::TEXTURE_2D, 1, internal_format, width, height );
  gl.tex_parameteri( GL::TEXTURE_2D, GL::TEXTURE_MIN_FILTER, GL::NEAREST as i32 );
  gl.tex_parameteri( GL::TEXTURE_2D, GL::TEXTURE_MAG_FILTER, GL::NEAREST as i32 );
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
  rotation : ndarray_cg::Mat4< f32, DescriptorOrderColumnMajor >,
  translation : F32x3,
  id : i32,
}

struct Mesh
{
  vao : gl::WebGlVertexArrayObject,
  diffuse_texture : Option< WebGlTexture >,
  index_count : i32,
}

/// Everything a redraw or a pick needs that never changes after setup — bundled so all three
/// pointer-event closures can share one `Rc` clone instead of each capturing a dozen handles.
struct RenderCtx
{
  gl : GL,
  canvas : web_sys::HtmlCanvasElement,
  framebuffer : Option< WebGlFramebuffer >,
  id_shader : shaders::IdShader,
  object_shader : shaders::ObjectShader,
  outline_shader : shaders::OutlineShader,
  meshes : Box< [ Mesh ] >,
  projection : ndarray_cg::Mat4< f32, DescriptorOrderColumnMajor >,
  width : f32,
  height : f32,
  tan_half_fovy : f32,
  aspect : f32,
  pick_id : js_sys::Int32Array,
}

/// Mutable scene state shared, via `Rc<RefCell<_>>`, across the mousedown/mousemove/mouseup
/// closures.
struct AppState
{
  objects : Box< [ Object ] >,
  selected : i32,
  dragging : Option< DragState >,
}

#[ derive( Clone, Copy ) ]
struct DragState
{
  id : i32,
  /// Camera-space depth ( negative ) held fixed for the duration of the drag — matches how the
  /// object was grabbed rather than trying to resolve depth from a single 2D cursor sample.
  depth : f32,
  /// `object.translation.xy` minus the world-space cursor position at pick time — added back to
  /// the cursor's world position on every move so the exact point grabbed stays under the cursor
  /// instead of the object's center snapping to it.
  grab_offset : ( f32, f32 ),
}
