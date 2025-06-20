use std::cell::RefCell;
use minwebgl as gl;
use gl::
{
  JsCast,
  F32x3,
  math::vector::cross,
  GL,
  WebGl2RenderingContext,
  web_sys::
  {
    HtmlCanvasElement,
    wasm_bindgen::closure::Closure,
    WebGlTexture
  },
  VectorDataType
};
use renderer::webgl::
{
  geometry::AttributeInfo, loaders::gltf::GLTF, post_processing::
  {
    self, Pass, SwapFramebuffer
  }, MinFilterMode, MagFilterMode, WrappingMode, Camera, Geometry, IndexInfo, Material, Mesh, Node, Object3D, Primitive, Renderer, Scene, Texture, TextureInfo, Sampler
};
use std::rc::Rc;
use csgrs::CSG;

mod camera_controls;
mod loaders;

fn upload_texture( gl : &WebGl2RenderingContext, src : Rc< String > ) -> WebGlTexture
{
  let window = web_sys::window().unwrap();
  let document =  window.document().unwrap();

  let texture = gl.create_texture().expect( "Failed to create a texture" );

  let img_element = document.create_element( "img" ).unwrap()
  .dyn_into::< gl::web_sys::HtmlImageElement >().unwrap();
  img_element.style().set_property( "display", "none" ).unwrap();
  let load_texture : Closure< dyn Fn() > = Closure::new
  (
    {
      let gl = gl.clone();
      let img = img_element.clone();
      let texture = texture.clone();
      let src = src.clone();
      move ||
      {
        gl.bind_texture( gl::TEXTURE_2D, Some( &texture ) );
        gl.tex_image_2d_with_u32_and_u32_and_html_image_element
        (
          gl::TEXTURE_2D,
          0,
          gl::RGBA as i32,
          gl::RGBA,
          gl::UNSIGNED_BYTE,
          &img
        ).expect( "Failed to upload data to texture" );

        gl.generate_mipmap( gl::TEXTURE_2D );

        //match
        gl::web_sys::Url::revoke_object_url( &src ).unwrap();
        img.remove();
      }
    }
  );

  img_element.set_onload( Some( load_texture.as_ref().unchecked_ref() ) );
  img_element.set_src( &src );
  load_texture.forget();

  texture
}

async fn create_texture( 
  gl : &WebGl2RenderingContext,
  image_path : &str
) -> Option< TextureInfo >
{
  let texture_id = upload_texture( gl, Rc::new( image_path.to_string() ) );

  let sampler = Sampler::former()
  .min_filter( MinFilterMode::Linear )
  .mag_filter( MagFilterMode::Linear )
  .wrap_s( WrappingMode::Repeat )
  .wrap_t( WrappingMode::Repeat )
  .end();

  let texture = Texture::former()
  .target( GL::TEXTURE_2D )
  .source( texture_id )
  .sampler( sampler )
  .end();

  let texture_info = TextureInfo
  {
    texture : Rc::new( RefCell::new( texture ) ),
    uv_position : 0,
  };

  Some( texture_info )
}

fn make_buffer_attibute_info
( 
  buffer : &web_sys::WebGlBuffer,
  descriptor : gl::BufferDescriptor, 
  offset : i32, 
  stride : i32, 
  slot : u32,
  normalized : bool,
  vector: gl::VectorDataType
) -> Result< AttributeInfo, gl::WebglError >
{
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
      bounding_box : Default::default()
    }
  )
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
  fn set_node_transform( &self, node : Rc< RefCell< Node > > )
  {
    let t = self.translation;
    let r = self.rotation;
    let s = self.scale;
    let mut node_mut = node.borrow_mut();
    node_mut.set_translation( [ t[ 0 ], t[ 1 ], t[ 2 ] ] );
    let q = glam::Quat::from_euler( glam::EulerRot::XYZ, r[ 0 ], r[ 1 ], r[ 2 ] );
    node_mut.set_rotation( q );
    node_mut.set_scale( [ s[ 0 ], s[ 1 ], s[ 2 ] ] );
    node_mut.update_local_matrix();
  }
}

struct AttributesData
{
  positions : Vec< [ f32; 3 ] >,
  normals : Vec< [ f32; 3 ] >,
  uvs : Vec< [ f32; 2 ] >,
  indices : Vec< u32 >
}

#[ derive( Clone ) ]
struct PrimitiveData 
{
  attributes : Rc< RefCell< AttributesData > >,
  material : Rc< RefCell< Material > >,
  transform : Transform
}

fn primitives_data_to_gltf
( 
  gl : &GL,
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
  let uv_buffer = gl.create_buffer().unwrap();

  for buffer in 
  [
    position_buffer.clone(),
    normal_buffer.clone(),
    uv_buffer.clone()
  ]
  {
    gl_buffers.push( buffer );
  }

  let attribute_infos = 
  [
    ( 
      "positions", 
      make_buffer_attibute_info( 
        &position_buffer, 
        gl::BufferDescriptor::new::< [ f32; 3 ] >(),
        0, 
        3, 
        0, 
        false,
        VectorDataType::new( mingl::DataType::F32, 3, 1 )
      ).unwrap() 
    ),
    ( 
      "normals", 
      make_buffer_attibute_info( 
        &normal_buffer, 
        gl::BufferDescriptor::new::< [ f32; 3 ] >(),
        0, 
        3, 
        1, 
        false,
        VectorDataType::new( mingl::DataType::F32, 3, 1 )
      ).unwrap() 
    ),
    ( 
      "uvs", 
      make_buffer_attibute_info( 
        &normal_buffer, 
        gl::BufferDescriptor::new::< [ f32; 2 ] >(),
        0, 
        2, 
        2, 
        true,
        VectorDataType::new( mingl::DataType::F32, 2, 1 )
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
  let mut uvs = vec![];
  let mut indices = vec![];

  for primitive_data in primitives_data
  {
    let last_positions_count = positions.len() as u32;
    positions.extend( primitive_data.attributes.borrow().positions.clone() );
    normals.extend( primitive_data.attributes.borrow().normals.clone() );
    uvs.extend( primitive_data.attributes.borrow().uvs.clone() );
    let primitive_indices = primitive_data.attributes.borrow().indices.iter()
    .map( | i | i + last_positions_count )
    .collect::< Vec< _ > >();
    let offset = indices.len() as u32 * 4;
    indices.extend( primitive_indices );

    index_info.offset = offset;
    index_info.count = primitive_data.attributes.borrow().indices.len() as u32;

    let Ok( mut geometry ) = Geometry::new( gl ) else
    {
      continue;
    };

    for ( name, info ) in &attribute_infos
    {
      geometry.add_attribute( gl, *name, info.clone(), false ).unwrap();
    }

    geometry.add_index( gl, index_info.clone() ).unwrap();
    geometry.vertex_count = primitive_data.attributes.borrow().positions.len() as u32;

    let primitive = Primitive
    {
      geometry : Rc::new( RefCell::new( geometry ) ),
      material : primitive_data.material.clone(),
    };

    let mesh = Rc::new( RefCell::new( Mesh::new() ) );
    mesh.borrow_mut().add_primitive( Rc::new( RefCell::new( primitive ) ) );

    let node = Rc::new( RefCell::new( Node::new() ) );
    node.borrow_mut().object = Object3D::Mesh( mesh.clone() );
    primitive_data.transform.set_node_transform( node.clone() );

    nodes.push( node.clone() );
    meshes.push( mesh );
    scenes[ 0 ].borrow_mut().children.push( node );
  }

  gl::buffer::upload( &gl, &position_buffer, &positions, GL::STATIC_DRAW );
  gl::buffer::upload( &gl, &normal_buffer, &normals, GL::STATIC_DRAW );
  gl::buffer::upload( &gl, &uv_buffer, &uvs, GL::STATIC_DRAW );
  gl::index::upload( &gl, &index_buffer, &indices, GL::STATIC_DRAW );
  
  GLTF
  {
    scenes,
    nodes,
    gl_buffers,
    images : Rc::new( RefCell::new( vec![] ) ),
    textures : vec![],
    materials,
    meshes
  }
}

fn generate_primitive_data( primitive : &CSG< () > ) -> PrimitiveData
{
  let mesh = primitive.to_trimesh();
  let mesh = mesh.as_trimesh().unwrap();

  let positions = mesh.vertices()
  .iter()
  .map( | p | [ p.coords.x as f32, p.coords.y as f32, p.coords.z as f32 ] )
  .collect::< Vec< _ > >();

  let indices = mesh.indices()
  .iter()
  .flatten()
  .cloned()
  .collect::< Vec< _ > >();

  let remap = | value, low1, high1, low2, high2 | 
  {
    low2 + (value - low1) * (high2 - low2) / (high1 - low1)
  };
  let half_pi = std::f32::consts::PI / 2.0;

  let max = positions.iter()
  .map( | [ _, _, z ] | z )
  .max_by(| x, y| x.partial_cmp( &y ).unwrap() )
  .cloned()
  .unwrap_or_default();
  let min = positions.iter()
  .map( | [ _, _, z ] | z )
  .min_by(| x, y| x.partial_cmp( &y ).unwrap() )
  .cloned()
  .unwrap_or_default();

  let uvs = positions.iter()
  .map(
    | [ x, y, z ] |
    {
      let phi = ( y / x ).atan();
      let u = remap( phi, - half_pi, half_pi, 0.0_f32, 1.0_f32 );
      let v = remap( *z, min, max, 0.0, 1.0 );
      [ u, v ]
    }
  )
  .collect::< Vec< _ > >();

  //gl::info!( "{:#?}", positions.iter().zip( uvs.iter() ).collect::< Vec< _ > >() );

  let vertices_count = mesh.vertices().len();

  // Calculating normals for primitives using this article: https://iquilezles.org/articles/normals/
  let mut normals = vec![ [ 0.0; 3 ]; vertices_count ];
  indices.chunks( 3 )
  .for_each
  ( 
    | ids | 
    {
      let p = | i | F32x3::from( positions[ ids[ i ] as usize ] );
      let [ a, b, c ] = [ p( 0 ), p( 1 ), p( 2 ) ];
      let e1 = a - b;
      let e2 = c - b;
      let c = cross( &e1, &e2 );
      ( 0..3 ).for_each
      (
        | i | normals[ ids[ i ] as usize ] = c.normalize().0
      );
    }
  );

  let attributes = AttributesData
  {
    positions,
    normals,
    uvs,
    indices
  };

  PrimitiveData
  {
    attributes : Rc::new( RefCell::new( attributes ) ),
    material : Rc::new( RefCell::new( Material::default() ) ),
    transform : Transform::default()
  }
}

fn get_primitives_data() -> Vec< PrimitiveData >
{
  [
    CSG::sphere( 1.0, 128, 128, None ),
    CSG::sphere( 500000.0, 16, 8, None )
  ]
  .iter()
  .map( | primitive | generate_primitive_data( &primitive ) )
  .collect::< Vec< _ > >()
}

fn init_context() -> ( WebGl2RenderingContext, HtmlCanvasElement )
{
  gl::browser::setup( Default::default() );
  let options = gl::context::ContexOptions::default().antialias( false );

  let canvas = gl::canvas::make().unwrap();
  let gl = gl::context::from_canvas_with( &canvas, options ).unwrap();

  let _ = gl.get_extension( "EXT_color_buffer_float" ).expect( "Failed to enable EXT_color_buffer_float extension" );

  ( gl, canvas )
}

fn init_camera( canvas : &HtmlCanvasElement ) -> Camera
{
  let width = canvas.width() as f32;
  let height = canvas.height() as f32;

  // Camera setup
  let eye = gl::math::F32x3::from( [ 0.0, 1.0, 1.0 ] );
  let up = gl::math::F32x3::from( [ 0.0, 1.0, 0.0 ] );
  let center = gl::math::F32x3::from( [ 0.0, 0.0, 0.0 ] );

  let aspect_ratio = width / height;
  let fov = 70.0f32.to_radians();
  let near = 0.1;
  let far = 10000000.0;

  let mut camera = Camera::new( eye, up, center, aspect_ratio, fov, near, far );
  camera.set_window_size( [ width, height ].into() );

  camera_controls::setup_controls( &canvas, &camera.get_controls() );

  camera
}

async fn run() -> Result< (), gl::WebglError >
{
  let ( gl, canvas ) = init_context();

  let image_path = "static/curve_surface_rendering/chess.png";
  let texture0 = create_texture( &gl, image_path ).await;

  let image_path = "static/curve_surface_rendering/uv_test.jpg";
  let texture1 = create_texture( &gl, image_path ).await;

  let mut primitives_data = get_primitives_data();
  primitives_data.pop();
  if let Some( first ) = primitives_data.get( 0 )
  {
    first.material.borrow_mut().base_color_texture = texture0;
  }
  if let Some( second ) = primitives_data.get( 1 )
  {
    second.material.borrow_mut().base_color_texture = texture1;
  }

  let mut materials = vec![];
  for primitive_data in &primitives_data
  {
    primitive_data.material.borrow_mut().base_color_factor = [ 5.0, 5.0, 5.0, 1.0 ].into();
    materials.push( primitive_data.material.clone() );
  }

  let gltf = primitives_data_to_gltf( &gl, primitives_data, materials );
  let scenes = gltf.scenes.clone();

  scenes[ 0 ].borrow_mut().update_world_matrix();
  let camera = init_camera( &canvas );

  let mut renderer = Renderer::new( &gl, canvas.width(), canvas.height(), 4 )?;
  renderer.set_ibl( loaders::ibl::load( &gl, "environment_maps/gltf_viewer_ibl_unreal" ).await );

  let mut swap_buffer = SwapFramebuffer::new( &gl, canvas.width(), canvas.height() );

  let tonemapping = post_processing::ToneMappingPass::< post_processing::ToneMappingAces >::new( &gl )?;
  let to_srgb = post_processing::ToSrgbPass::new( &gl, true )?;

  // Define the update and draw logic
  let update_and_draw =
  {
    move | t : f64 |
    {
      // If textures are of different size, gl.view_port needs to be called
      let _time = t as f32 / 1000.0;

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