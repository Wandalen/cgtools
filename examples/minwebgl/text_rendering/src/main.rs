//! Text surface rendering example demonstrating curve-based text rendering with WebGL.
#![ doc( html_root_url = "https://docs.rs/text_rendering/latest/text_rendering/" ) ]
#![ cfg_attr( doc, doc = include_str!( concat!( env!( "CARGO_MANIFEST_DIR" ), "/", "readme.md" ) ) ) ]
#![ cfg_attr( not( doc ), doc = "Renders text as set of 3D glyph meshes" ) ]

use std::cell::RefCell;
use minwebgl as gl;
use gl::
{
  GL,
  WebGl2RenderingContext,
  web_sys::HtmlCanvasElement,
  VectorDataType
};
use renderer::webgl::
{
  Mesh,
  Object3D,
  Node,
  Geometry,
  IndexInfo,
  geometry::AttributeInfo,
  loaders::gltf::GLTF,
  post_processing::
  {
    self, Pass, SwapFramebuffer
  },
  Camera,
  Material,
  Primitive,
  Renderer,
  Scene,
  material::PbrMaterial
};
use std::rc::Rc;
use std::any::type_name_of_val;

mod text;
mod style;

fn buffer_attribute_info_make
(
  buffer : &web_sys::WebGlBuffer,
  offset : i32,
  stride : i32,
  slot : u32,
  normalized : bool,
  vector: gl::VectorDataType
) -> Result< AttributeInfo, gl::WebglError >
{
  let descriptor = match vector.scalar
  {
    gl::DataType::U8 => gl::BufferDescriptor::new::< [ u8; 1 ] >(),
    gl::DataType::I8 => gl::BufferDescriptor::new::< [ i8; 1 ] >(),
    gl::DataType::U16 => gl::BufferDescriptor::new::< [ u16; 1 ] >(),
    gl::DataType::I16 => gl::BufferDescriptor::new::< [ i16; 1 ] >(),
    gl::DataType::U32 => gl::BufferDescriptor::new::< [ u32; 1 ] >(),
    gl::DataType::F32 => gl::BufferDescriptor::new::< [ f32; 1 ] >(),
    _ => return Err( gl::WebglError::NotSupportedForType( type_name_of_val( &vector.scalar ) ) )
  };

  let descriptor = descriptor
  .offset( offset )
  .normalized( normalized )
  .stride( stride )
  .vector( vector );

  Ok(
    AttributeInfo
    {
      slot,
      buffer : buffer.clone(),
      descriptor,
      bounding_box : mingl::geometry::BoundingBox::default()
    }
  )
}

/// Builds the shared position and normal attribute descriptors for the scene buffers.
fn scene_attribute_infos
(
  position_buffer : &web_sys::WebGlBuffer,
  normal_buffer : &web_sys::WebGlBuffer
) -> [ ( &'static str, AttributeInfo ); 2 ]
{
  [
    (
      "positions",
      buffer_attribute_info_make
      (
        position_buffer,
        0,
        3,
        0,
        false,
        VectorDataType::new( mingl::DataType::F32, 3, 1 )
      ).unwrap()
    ),
    (
      "normals",
      buffer_attribute_info_make
      (
        normal_buffer,
        0,
        3,
        1,
        false,
        VectorDataType::new( mingl::DataType::F32, 3, 1 )
      ).unwrap()
    )
  ]
}

#[ derive( Debug, Clone ) ]
struct Transform
{
  translation : [ f32; 3 ],
  rotation : [ f32; 3 ],
  scale : [ f32; 3 ],
}

impl Default for Transform
{
  fn default() -> Self
  {
    Self
    {
      translation : [ 0.0; 3 ],
      rotation : [ 0.0; 3 ],
      scale : [ 1.0; 3 ]
    }
  }
}

impl Transform
{
  fn node_transform_set( &self, node : &Rc< RefCell< Node > > )
  {
    let t = self.translation;
    let r = self.rotation;
    let s = self.scale;
    let mut node_mut = node.borrow_mut();
    node_mut.translation_set( [ t[ 0 ], t[ 1 ], t[ 2 ] ] );
    let q = gl::QuatF32::from_euler_xyz( r );
    node_mut.rotation_set( q );
    node_mut.scale_set( [ s[ 0 ], s[ 1 ], s[ 2 ] ] );
    node_mut.local_matrix_update();
  }
}

struct AttributesData
{
  positions : Vec< [ f32; 3 ] >,
  normals : Vec< [ f32; 3 ] >,
  indices : Vec< u32 >
}

#[ derive( Clone ) ]
struct PrimitiveData
{
  attributes : Rc< RefCell< AttributesData > >,
  material : Rc< RefCell< Box< dyn Material > > >,
  transform : Transform
}

fn primitives_data_to_gltf
(
  gl : &GL,
  primitives_data : Vec< PrimitiveData >,
  materials : Vec< Rc< RefCell< Box< dyn Material > > > >
) -> GLTF
{
  let mut scenes = vec![];
  let mut nodes = vec![];
  let mut gl_buffers = vec![];
  let mut meshes = vec![];

  scenes.push( Rc::new( RefCell::new( Scene::new() ) ) );

  let position_buffer = gl.create_buffer().unwrap();
  let normal_buffer = gl.create_buffer().unwrap();

  for buffer in
  [
    position_buffer.clone(),
    normal_buffer.clone()
  ]
  {
    gl_buffers.push( buffer );
  }

  let attribute_infos = scene_attribute_infos( &position_buffer, &normal_buffer );

  let index_buffer = gl.create_buffer().unwrap();
  gl_buffers.push( index_buffer.clone() );

  let mut index_info = IndexInfo
  {
    buffer : index_buffer.clone(),
    count : 0,
    offset : 0,
    data_type : GL::UNSIGNED_INT
  };

  let mut positions = vec![];
  let mut normals = vec![];
  let mut indices = vec![];

  for primitive_data in primitives_data
  {
    let last_positions_count = positions.len() as u32;
    positions.extend( primitive_data.attributes.borrow().positions.clone() );
    normals.extend( primitive_data.attributes.borrow().normals.clone() );
    let primitive_indices = primitive_data.attributes.borrow().indices.iter()
    .map( | i | i + last_positions_count )
    .collect::< Vec< _ > >();
    let offset = indices.len() as u32 * 4;
    indices.extend( primitive_indices );

    index_info.offset = offset;
    index_info.count = primitive_data.attributes.borrow().indices.len() as u32;

    let Ok( mut geometry ) = Geometry::new( gl ) else
    {
      panic!( "Can't create new Geometry struct" );
    };

    for ( name, info ) in &attribute_infos
    {
      geometry.attribute_add( gl, *name, info.clone() ).unwrap();
    }

    geometry.index_add( gl, index_info.clone() ).unwrap();
    geometry.vertex_count = primitive_data.attributes.borrow().positions.len() as u32;

    let primitive = Primitive
    {
      geometry : Rc::new( RefCell::new( geometry ) ),
      material : primitive_data.material.clone(),
    };

    let mesh = Rc::new( RefCell::new( Mesh::new() ) );
    mesh.borrow_mut().primitive_add( Rc::new( RefCell::new( primitive ) ) );

    let node = Rc::new( RefCell::new( Node::new() ) );
    node.borrow_mut().object = Object3D::Mesh( mesh.clone() );
    primitive_data.transform.node_transform_set( &node );

    nodes.push( node.clone() );
    meshes.push( mesh );
    scenes[ 0 ].borrow_mut().children.push( node );
  }

  gl::buffer::upload( gl, &position_buffer, &positions, GL::STATIC_DRAW );
  gl::buffer::upload( gl, &normal_buffer, &normals, GL::STATIC_DRAW );
  gl::index::upload( gl, &index_buffer, &indices, GL::STATIC_DRAW );

  GLTF
  {
    scenes,
    nodes,
    gl_buffers,
    images : Rc::new( RefCell::new( vec![] ) ),
    textures : vec![],
    materials,
    meshes,
    animations : vec![],
    lights : vec![]
  }
}

/// Builds a fresh [`PbrMaterial`] with `color` as its base color factor —
/// used to give each demonstrated string its own material instead of
/// cloning one shared material into every glyph.
fn material_make( gl : &GL, color : [ f32; 4 ] ) -> Rc< RefCell< Box< dyn Material > > >
{
  let mut pbr = PbrMaterial::new( gl );
  pbr.base_color_factor = color.into();
  Rc::new( RefCell::new( Box::new( pbr ) as Box< dyn Material > ) )
}

/// Section 2 : size -- `transform.scale` swept across 4 values. Previously
/// `text_to_mesh` silently discarded any caller-supplied scale ; it is now
/// multiplied into the internal base scale instead of overwriting it. The
/// largest sample ( 1.5x ) is taller than the other rows' fixed 1.0x, so the
/// gap down to the color row is widened 1.5x to match ( see its own
/// translation ) rather than reusing the plain 1.0 gap the other rows prove
/// safe at scale 1.0.
fn size_row_primitives_build
(
  text : &str,
  font : &text::ttf::Font3D,
  material : &Rc< RefCell< Box< dyn Material > > >
) -> Vec< PrimitiveData >
{
  let mut primitives_data = vec![];
  for ( i, scale ) in [ 0.6_f32, 0.9, 1.2, 1.5 ].into_iter().enumerate()
  {
    let mut t = Transform::default();
    t.translation = [ -3.3 + i as f32 * 2.2, -2.0, 0.0 ];
    t.scale = [ scale, scale, scale ];
    let mut mesh = text::ttf::text_to_mesh( text, font, &t );
    for p in &mut mesh
    {
      p.material = material.clone();
    }
    primitives_data.extend( mesh );
  }
  primitives_data
}

/// Section 3 : color -- `PrimitiveData.material` is already per-primitive ;
/// the font gallery just always cloned one shared material into every glyph.
/// Assigning a distinct material per string is all real color needs.
fn color_row_primitives_build
(
  gl : &GL,
  text : &str,
  font : &text::ttf::Font3D,
  materials : &mut Vec< Rc< RefCell< Box< dyn Material > > > >
) -> Vec< PrimitiveData >
{
  let colors =
  [
    [ 1.0, 1.0, 1.0, 1.0 ],
    [ 0.85, 0.15, 0.15, 1.0 ],
    [ 0.15, 0.75, 0.25, 1.0 ],
    [ 0.2, 0.4, 0.9, 1.0 ],
    [ 0.95, 0.65, 0.05, 1.0 ],
  ];

  let mut primitives_data = vec![];
  for ( i, color ) in colors.into_iter().enumerate()
  {
    let mut t = Transform::default();
    t.translation = [ -4.4 + i as f32 * 2.2, -3.5, 0.0 ];
    let color_material = material_make( gl, color );
    materials.push( color_material.clone() );
    let mut mesh = text::ttf::text_to_mesh( text, font, &t );
    for p in &mut mesh
    {
      p.material = color_material.clone();
    }
    primitives_data.extend( mesh );
  }
  primitives_data
}

/// Section 4 : style modifiers -- bold ( synthetic double-pass, no bold font
/// asset exists ), italic ( synthetic shear, same reason ), underline ( a
/// measured quad ), and all three combined -- proving the modifiers compose
/// rather than being mutually exclusive modes.
fn style_row_primitives_build
(
  gl : &GL,
  text : &str,
  font : &text::ttf::Font3D,
  materials : &mut Vec< Rc< RefCell< Box< dyn Material > > > >
) -> Vec< PrimitiveData >
{
  let style_material = material_make( gl, [ 0.85, 0.85, 0.9, 1.0 ] );
  materials.push( style_material.clone() );

  let mut primitives_data = vec![];
  for ( i, ( bold, italic, underline ) ) in
  [
    ( false, false, false ),
    ( true, false, false ),
    ( false, true, false ),
    ( false, false, true ),
    ( true, true, true ),
  ]
  .into_iter()
  .enumerate()
  {
    let mut t = Transform::default();
    t.translation = [ -4.4 + i as f32 * 2.2, -4.5, 0.0 ];
    let mut mesh = text::ttf::text_to_mesh( text, font, &t );
    for p in &mut mesh
    {
      p.material = style_material.clone();
    }
    if bold
    {
      style::mesh_bold_apply( &mut mesh, 1.15 );
    }
    if italic
    {
      style::mesh_shear_x( &mut mesh, 0.3 );
    }
    if underline
    {
      let width = text::ttf::text_advance_width( text, font, &t );
      let mut underline_quad = style::underline_quad_make( width, 0.06 * t.scale[ 1 ], style_material.clone() );
      // `text_max_height` is the worst-case drop to any glyph's own baseline
      // that `text_to_mesh` could place ( derived from the font's union
      // bounding box, not eyeballed -- see its own doc comment ) ; the extra
      // 0.1 keeps the line clear of the baseline itself rather than touching it.
      let below_baseline = text::ttf::text_max_height( font, &t ) + 0.1 * t.scale[ 1 ];
      underline_quad.transform.translation = [ t.translation[ 0 ], t.translation[ 1 ] - below_baseline, 0.0 ];
      primitives_data.push( underline_quad );
    }
    primitives_data.extend( mesh );
  }
  primitives_data
}

fn context_init() -> ( WebGl2RenderingContext, HtmlCanvasElement )
{
  gl::browser::setup( gl::browser::Config::default() );
  let options = gl::context::ContextOptions::default().antialias( false );

  let canvas = gl::canvas::make().unwrap();
  let gl = gl::context::from_canvas_with( &canvas, options ).unwrap();

  let _ = gl.get_extension( "EXT_color_buffer_float" ).expect( "Failed to enable EXT_color_buffer_float extension" );

  ( gl, canvas )
}

fn camera_init( canvas : &HtmlCanvasElement ) -> Camera
{
  let width = canvas.width() as f32;
  let height = canvas.height() as f32;

  // Camera setup -- pulled back from the original ( 0, 1, 1 )/origin framing
  // to fit the taller scene ( font gallery plus the size/color/style rows
  // added below it ). Eye position and fov were solved by projecting the
  // scene's own world-space bounding box corners into camera space and
  // checking they land inside the fov cone at every corner's own depth
  // ( not eyeballed, and not the weaker "shoot a ray from each frustum edge"
  // check -- that one turned out to under-count a nearer, wide row's
  // horizontal extent ). Bounding box used ( generous on x since exact
  // per-glyph advance widths aren't available without a live render ) :
  // x in [ -8.0, 8.0 ], y in [ -5.9, 3.2 ], holding a 10-65 % margin at
  // every corner across landscape aspect ratios ( 1.0 to 2.4 -- this canvas
  // is always browser-window-sized, so portrait is not a realistic case
  // here ). This repo's browser-verification tooling can't produce real
  // pixel output for this renderer's HDR tonemapping pipeline in this
  // environment, so the frustum math stands in for a screenshot.
  let eye = gl::math::F32x3::from( [ 0.0, 10.0, 10.0 ] );
  let up = gl::math::F32x3::from( [ 0.0, 1.0, 0.0 ] );
  let center = gl::math::F32x3::from( [ 0.0, -1.0, 0.0 ] );

  let aspect_ratio = width / height;
  let fov = 80.0f32.to_radians();
  let near = 0.1;
  let far = 1000.0;

  let mut camera = Camera::new( eye, up, center, aspect_ratio, fov, near, far ).expect( "camera parameters are valid" );
  camera.window_size_set( [ width, height ].into() );

  camera.controls_bind( canvas );

  camera
}

async fn app_run() -> Result< (), gl::WebglError >
{
  let ( gl, canvas ) = context_init();

  let font_names = [
    "Roboto-Regular".to_string(),
    "Caveat".to_string(),
    "HennyPenny-Regular".to_string(),
    "Parisienne-Regular".to_string()
  ];

  let fonts_ufo_3d = text::ufo::fonts_3d_load( &gl, font_names.as_slice() ).await;
  let fonts_ttf_3d = text::ttf::fonts_3d_load( &gl, font_names.as_slice() ).await;

  let text = "CGTools".to_string();

  let mut materials : Vec< Rc< RefCell< Box< dyn Material > > > > = vec![];
  let mut primitives_data = vec![];

  // Section 1 : font families -- the same text through both geometry
  // pipelines ( UFO glyph outlines vs TTF extrusion ), across all 4
  // typefaces. Unchanged from the original example.
  let default_material = material_make( &gl, [ 1.0, 1.0, 1.0, 1.0 ] );
  materials.push( default_material.clone() );
  let mut transform_ufo = Transform::default();
  transform_ufo.translation[ 1 ] += f32::midpoint( font_names.len() as f32, 1.0 ) + 0.5;
  transform_ufo.translation[ 0 ] -= 1.8;
  let mut transform_ttf = Transform::default();
  transform_ttf.translation[ 1 ] += f32::midpoint( font_names.len() as f32, 1.0 ) + 0.5;
  transform_ttf.translation[ 0 ] += 1.8;
  for font_name in &font_names
  {
    transform_ufo.translation[ 1 ] -= 1.0;
    let mut text_mesh = text::ufo::text_to_mesh( &text, fonts_ufo_3d.get( font_name ).unwrap(), &transform_ufo );
    for p in &mut text_mesh
    {
      p.material = default_material.clone();
    }
    primitives_data.extend( text_mesh );

    transform_ttf.translation[ 1 ] -= 1.0;
    let mut text_mesh = text::ttf::text_to_mesh( &text, fonts_ttf_3d.get( font_name ).unwrap(), &transform_ttf );
    for p in &mut text_mesh
    {
      p.material = default_material.clone();
    }
    primitives_data.extend( text_mesh );
  }

  // Every row below reuses the TTF pipeline and the Roboto-Regular font, so
  // font choice stays fixed while each row isolates one styling parameter.
  let style_font = fonts_ttf_3d.get( "Roboto-Regular" ).unwrap();

  primitives_data.extend( size_row_primitives_build( &text, style_font, &default_material ) );
  primitives_data.extend( color_row_primitives_build( &gl, &text, style_font, &mut materials ) );
  primitives_data.extend( style_row_primitives_build( &gl, &text, style_font, &mut materials ) );

  let gltf = primitives_data_to_gltf( &gl, primitives_data, materials );
  let scenes = gltf.scenes.clone();

  scenes[ 0 ].borrow_mut().world_matrix_update();
  let camera = camera_init( &canvas );

  let mut renderer = Renderer::new( &gl, canvas.width(), canvas.height(), 4 )?;

  let mut swap_buffer = SwapFramebuffer::new( &gl, canvas.width(), canvas.height() );

  let tonemapping = post_processing::ToneMappingPass::< post_processing::ToneMappingAces >::new( &gl )?;
  let to_srgb = post_processing::ToSrgbPass::new( &gl, true )?;

  // Define the update and draw logic
  let update_and_draw =
  {
    move | _ : f64 |
    {
      // If textures are of different size, gl.view_port needs to be called
      renderer.render( &gl, &mut scenes[ 0 ].borrow_mut(), &camera )
      .expect( "Failed to render" );

      swap_buffer.reset();
      swap_buffer.bind( &gl );
      swap_buffer.input_set( renderer.main_texture() );

      let t = tonemapping.render( &gl, swap_buffer.input_get(), swap_buffer.output_get() )
      .expect( "Failed to render tonemapping pass" );

      swap_buffer.output_set( t );
      swap_buffer.swap();

      let _t = to_srgb.render( &gl, swap_buffer.input_get(), swap_buffer.output_get() )
      .expect( "Failed to render to srgb pass" );

      true
    }
  };

  // Run the render loop
  gl::exec_loop::run( update_and_draw );

  Ok( () )
}

fn main()
{
  gl::spawn_local( async move { app_run().await.unwrap() } );
}
