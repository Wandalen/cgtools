
use std::{cell::RefCell, collections::{HashMap, HashSet}};

use minwebgl as gl;
use gl::
{
  WebGl2RenderingContext,
  web_sys::HtmlCanvasElement
};
use parley::FontContext;
use rand::Rng;
use renderer::webgl::
{
  geometry::AttributeInfo, loaders::gltf::GLTF, post_processing::
  {
    self, add_attributes, add_buffer, outline::narrow_outline::
    { 
      NarrowOutlinePass, 
      MAX_OBJECT_COUNT 
    }, GBuffer, GBufferAttachment, Pass, SwapFramebuffer
  }, Camera, Material, Primitive, Renderer
};

mod camera_controls;
mod loaders;
mod text;

fn generate_object_colors() -> Vec< [ f32; 4 ] > 
{
  let mut rng = rand::rng();

  let range = 0.2..1.0;
  let object_colors = ( 0..MAX_OBJECT_COUNT )
  .map
  ( 
    | _ |
    {
      [ 
        rng.random_range( range.clone() ), 
        rng.random_range( range.clone() ),
        rng.random_range( range.clone() ),
        1.0
      ]
    } 
  )
  .collect::< Vec< _ > >();

  object_colors
}

fn get_attributes( gltf : &GLTF ) -> Result< HashMap< Box< str >, AttributeInfo >, gl::WebglError >
{
  for mesh in &gltf.meshes
  {
    let mesh_ref = mesh.as_ref().borrow();
    for primitive in &mesh_ref.primitives
    {
      let primitive_ref = primitive.as_ref().borrow();
      return Ok( primitive_ref.geometry.as_ref().borrow().get_attributes() );
    }
  }

  Err( gl::WebglError::MissingDataError( "Primitive" ) )
}

#[ derive( Debug, Clone, Copy ) ]
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
  fn set_node_transform( node : Rc< RefCell< Node > > )
  {
    let mut node_mut = node.borrow_mut();
    node_mut.set_translation( [ t[ 0 ], t[ 1 ], t[ 2 ] ] );
    let q = glam::Quat::from_euler( glam::EulerRot::XYZ, t[ 3 ], t[ 4 ], t[ 5 ] );
    node_mut.set_rotation( q );
    node_mut.set_scale( [ t[ 6 ], t[ 7 ], t[ 8 ] ] );
    node_mut.update_local_matrix();
  }
}

struct AttributesData
{
  positions : Vec< [ f32; 3 ] >,
  normals : Vec< [ f32; 3 ] >,
  indices : Vec< u32 >
}

struct PrimitiveData 
{
  attributes : Rc< RefCell< AttributesData > >,
  material : Rc< RefCell< Material > >,
  transform : Transform
}

fn primitives_data_to_gltf
( 
  primitives_data : Vec< PrimitiveData >, 
  materials : Vec< Rc< RefCell< Material > > > 
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

  let attribute_infos = 
  [
    ( 
      "position", 
      make_buffer_attibute_info( 
        &position_buffer, 
        0, 
        3, 
        0, 
        false,
        VectorDataType::new( mingl::DataType::F32, 3, 1 )
      ).unwrap() 
    ),
    ( 
      "normal", 
      make_buffer_attibute_info( 
        &normal_buffer, 
        0, 
        3, 
        1, 
        false,
        VectorDataType::new( mingl::DataType::F32, 3, 1 )
      ).unwrap() 
    )
  ];

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
    let last_indices_count = indices.len() as u32;

    positions.extend( primitive_data.positions.clone() );
    normals.extend( primitive_data.normals.clone() );
    let primitive_indices = primitive_data.indices.iter()
    .map( | i | i + last_indices_count )
    .collect::< Vec< _ > >();
    indices.extend( primitive_indices );

    index_info.offset = last_indices_count * 4;
    index_info.count = primitive_data.indices.len();

    let Ok( mut geometry ) = Geometry::new( gl ) else
    {
      continue;
    };

    for ( name, info ) in &attribute_infos
    {
      geometry.add_attribute( gl, *name, info.clone(), false ).unwrap();
    }

    geometry.add_index( gl, index_info.clone() ).unwrap();
    geometry.vertex_count = primitive_data.positions.len();

    let primitive = Primitive
    {
      geometry : Rc::new( RefCell::new( geometry ) ),
      material : primitive_data.material.clone(),
    };

    let mesh = Rc::new( RefCell::new( Mesh::new() ) );
    mesh.borrow_mut().add_primitive( Rc::new( RefCell::new( primitive ) ) );

    let node = Rc::new( RefCell::new( Node::new() ) );
    node.borrow_mut().object = Object3D::Mesh( mesh );
    primitive_data.transform.set_node_transform( node.clone() );

    nodes.push( node.clone() );
    scenes[ 0 ].borrow_mut().children.push( node );
  }

  gl::buffer::upload( &gl, &position_buffer, &positions, GL::STATIC_DRAW );
  gl::buffer::upload( &gl, &normal_buffer, &normals, GL::STATIC_DRAW );
  gl::index::upload( &gl, &index_buffer, &indices, GL::STATIC_DRAW );
  
  GLTF
  {
    scenes,
    nodes,
    gl_buffers,
    images : vec![],
    textures : Rc::new( RefCell::new( vec![] ) ),
    materials,
    meshes
  }
}

fn init_context() -> ( WebGl2RenderingContext, HtmlCanvasElement )
{
  gl::browser::setup( Default::default() );
  let options = gl::context::ContexOptions::default().antialias( false );

  let canvas = gl::canvas::make()?;
  let gl = gl::context::from_canvas_with( &canvas, options )?;
  let window = gl::web_sys::window().unwrap();
  let document = window.document().unwrap();

  let _ = gl.get_extension( "EXT_color_buffer_float" ).expect( "Failed to enable EXT_color_buffer_float extension" );

  ( gl, canvas )
}

fn init_camera( canvas : &HtmlCanvasElement, scene : Rc< RefCell< Scene > > ) -> Camera
{
  let width = canvas.width() as f32;
  let height = canvas.height() as f32;

  // Camera setup
  let scene_bounding_box = scene.borrow().bounding_box();
  let diagonal = ( scene_bounding_box.max - scene_bounding_box.min ).mag();
  let dist = scene_bounding_box.max.mag();
  let exponent = 
  {
    let bits = diagonal.to_bits();
    let exponent_field = ( ( bits >> 23 ) & 0xFF ) as i32;
    exponent_field - 127
  };

  // Camera setup
  let mut eye = gl::math::F32x3::from( [ 0.0, 1.0, 1.0 ] );
  eye *= dist;
  let up = gl::math::F32x3::from( [ 0.0, 1.0, 0.0 ] );
  let center = scene_bounding_box.center();

  let aspect_ratio = width / height;
  let fov = 70.0f32.to_radians();
  let near = 1.0 * 10.0f32.powi( exponent ).min( 1.0 );
  let far = near * 10.0f32.powi( exponent.abs() );

  let mut camera = Camera::new( eye, up, center, aspect_ratio, fov, near, far );
  camera.set_window_size( [ width, height ].into() );

  camera_controls::setup_controls( &canvas, &camera.get_controls() );

  camera
}

async fn run() -> Result< (), gl::WebglError >
{
  let ( gl, canvas ) = init();

  let font_names = 
  [
    "Caveat".to_string(),
    "HennyPenny-Regular".to_string(),
    "Parisienne-Regular".to_string(),
    "Roboto-Regular".to_string()
  ];

  let fonts_3d = text::norad::load_fonts_3d( font_names );

  let text = "Hello world".to_string();

  let material = Rc::new( RefCell::new( Material::default() ) );
  let materials = vec![ material.clone() ];

  let primitives_data = vec![];
  for font_name in font_names
  {
    let mut text_mesh = text::norad::text_to_mesh( &text, fonts_3d.get( &font_name ).unwrap() );
    for p in text_mesh.iter_mut()
    {
      p.material = material.clone()
    }
    primitives_data.extend( text_mesh );
  }

  let gltf = primitives_data_to_gltf( primitives_data, materials );
  let scenes = gltf.scenes.clone();
  scenes[ 0 ].borrow_mut().update_world_matrix();

  let camera = init_camera( &canvas, scenes[ 0 ].clone() );

  let mut renderer = Renderer::new( &gl, canvas.width(), canvas.height(), 4 )?;
  renderer.set_ibl( loaders::ibl::load( &gl, "envMap" ).await );

  let mut swap_buffer = SwapFramebuffer::new( &gl, canvas.width(), canvas.height() );

  let tonemapping = post_processing::ToneMappingPass::< post_processing::ToneMappingAces >::new( &gl )?;
  let to_srgb = post_processing::ToSrgbPass::new( &gl, true )?;

  // Define the update and draw logic
  let update_and_draw =
  {
    move | t : f64 |
    {
      // If textures are of different size, gl.view_port needs to be called
      let time = t as f32 / 1000.0;

      renderer.render( &gl, &mut scenes[ 0 ].borrow_mut(), &camera )
      .expect( "Failed to render" );

      swap_buffer.reset();
      swap_buffer.bind( &gl );
      swap_buffer.set_input( renderer.get_main_texture() );

      let t = tonemapping.render( &gl, swap_buffer.get_input(), swap_buffer.get_output() )
      .expect( "Failed to render tonemapping pass" );

      swap_buffer.set_output( t );
      swap_buffer.swap();
    
      let _t = to_srgb.render( &gl, swap_buffer.get_input(), swap_buffer.get_output() )
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
  gl::spawn_local( async move { run().await.unwrap() } );
}