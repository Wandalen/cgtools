//! Browser tests for `EngravingSession`: how `build` binds a config to a scene, and what
//! `set_text` accepts.
//!
//! These need a real WebGL2 context (the mask texture) and a DOM (the offscreen canvas), so they
//! run in a headless browser like `engraving_shader_tests.rs`.

#[ cfg( target_arch = "wasm32" ) ]
#[ cfg( test ) ]
mod tests
{
  use wasm_bindgen_test::wasm_bindgen_test;
  use std::{ cell::RefCell, rc::Rc };
  use minwebgl as gl;
  use gl::GL;
  use renderer::webgl::
  {
    Geometry, Material, Mesh, Node, Object3D, Primitive, Scene, Texture,
    engraving::{ EngravingConfig, EngravingSession },
    material::PbrMaterial,
  };

  wasm_bindgen_test::wasm_bindgen_test_configure!( run_in_browser );

  type SharedMaterial = Rc< RefCell< Box< dyn Material > > >;

  fn gl_init() -> GL
  {
    gl::browser::setup( gl::browser::Config::default() );
    let options = gl::context::ContextOptions::default().antialias( false );
    let canvas = gl::canvas::make().unwrap();
    gl::context::from_canvas_with( &canvas, options ).unwrap()
  }

  fn pbr_material( gl : &GL ) -> SharedMaterial
  {
    Rc::new( RefCell::new( Box::new( PbrMaterial::new( gl ) ) ) )
  }

  /// A mesh with one primitive per entry of `materials`, each over its own empty geometry.
  fn mesh_make( gl : &GL, materials : &[ &SharedMaterial ] ) -> Rc< RefCell< Mesh > >
  {
    let mut mesh = Mesh::new();
    for material in materials
    {
      let geometry = Geometry::new( gl ).expect( "Geometry::new on a valid context" );
      let primitive = Primitive { geometry : Rc::new( RefCell::new( geometry ) ), material : ( *material ).clone() };
      mesh.primitive_add( Rc::new( RefCell::new( primitive ) ) );
    }
    Rc::new( RefCell::new( mesh ) )
  }

  /// Adds a root node named `name` holding `object` to `scene`.
  fn node_add( scene : &mut Scene, name : &str, object : Object3D ) -> Rc< RefCell< Node > >
  {
    let mut node = Node::new();
    node.name_set( name );
    node.object = object;
    let node = Rc::new( RefCell::new( node ) );
    scene.add( node.clone() );
    node
  }

  /// A config engraving each of `names` on UV set 1, with a small texture so the tests stay fast.
  fn config_for( names : &[ &str ] ) -> EngravingConfig
  {
    let nodes = names.iter()
    .map
    (
      | name | format!
      (
        r#"{{ "nodeName": "{name}", "uvChannel": 1, "aspectRatio": 4.0, "maxCharacters": 8,
        "textureHeight": 16, "defaultFont": "monospace", "allowedFonts": [ "monospace" ] }}"#
      )
    )
    .collect::< Vec< _ > >()
    .join( ", " );
    EngravingConfig::from_json( &format!( r#"{{ "nodes": [ {nodes} ] }}"# ) ).expect( "valid config" )
  }

  /// The materials of `node`'s primitives, in primitive order.
  fn node_materials( node : &Rc< RefCell< Node > > ) -> Vec< SharedMaterial >
  {
    let node = node.borrow();
    let Object3D::Mesh( mesh ) = &node.object else { panic!( "test node is a mesh" ) };
    let materials = mesh.borrow().primitives.iter().map( | p | p.borrow().material.clone() ).collect();
    materials
  }

  /// The engraving mask bound to `material`, if any.
  fn engraving_of( material : &SharedMaterial ) -> Option< Rc< RefCell< Texture > > >
  {
    let material = material.borrow();
    let pbr = material.as_any().downcast_ref::< PbrMaterial >().expect( "test materials are PbrMaterial" );
    pbr.engraving_texture().map( | info | info.texture.clone() )
  }

  /// Two configured nodes sharing one material each get their own copy with their own mask; an
  /// unconfigured node sharing it keeps the untouched original.
  #[ wasm_bindgen_test ]
  fn build_gives_each_configured_node_its_own_material_copy()
  {
    let gl = gl_init();
    let shared = pbr_material( &gl );
    let mut scene = Scene::new();
    // `a` uses the shared material on two primitives: they must keep sharing one copy.
    let a = node_add( &mut scene, "A", Object3D::Mesh( mesh_make( &gl, &[ &shared, &shared ] ) ) );
    let b = node_add( &mut scene, "B", Object3D::Mesh( mesh_make( &gl, &[ &shared ] ) ) );
    let c = node_add( &mut scene, "C", Object3D::Mesh( mesh_make( &gl, &[ &shared ] ) ) );

    let _session = EngravingSession::build( &gl, &scene, &config_for( &[ "A", "B" ] ) ).expect( "build succeeds" );

    let a_materials = node_materials( &a );
    let b_materials = node_materials( &b );
    let c_materials = node_materials( &c );

    assert!( Rc::ptr_eq( &a_materials[ 0 ], &a_materials[ 1 ] ), "primitives of one node that shared a material share one copy" );
    assert!( !Rc::ptr_eq( &a_materials[ 0 ], &shared ), "A renders with its own copy" );
    assert!( !Rc::ptr_eq( &b_materials[ 0 ], &shared ), "B renders with its own copy" );
    assert!( !Rc::ptr_eq( &a_materials[ 0 ], &b_materials[ 0 ] ), "A and B don't share a copy" );

    let a_mask = engraving_of( &a_materials[ 0 ] ).expect( "A is engraved" );
    let b_mask = engraving_of( &b_materials[ 0 ] ).expect( "B is engraved" );
    assert!( !Rc::ptr_eq( &a_mask, &b_mask ), "A and B sample different masks" );

    assert!( Rc::ptr_eq( &c_materials[ 0 ], &shared ), "the unconfigured node keeps the original material" );
    assert!( engraving_of( &shared ).is_none(), "the shared original is never engraved" );
  }

  /// A configured node whose mesh another node also uses gets its own mesh, so the other node
  /// keeps the unengraved primitives.
  #[ wasm_bindgen_test ]
  fn build_detaches_a_mesh_shared_with_an_unconfigured_node()
  {
    let gl = gl_init();
    let material = pbr_material( &gl );
    let mesh = mesh_make( &gl, &[ &material ] );
    let mut scene = Scene::new();
    let engraved = node_add( &mut scene, "Engraved", Object3D::Mesh( mesh.clone() ) );
    let plain = node_add( &mut scene, "Plain", Object3D::Mesh( mesh.clone() ) );

    let _session = EngravingSession::build( &gl, &scene, &config_for( &[ "Engraved" ] ) ).expect( "build succeeds" );

    {
      let engraved_ref = engraved.borrow();
      let Object3D::Mesh( engraved_mesh ) = &engraved_ref.object else { panic!( "still a mesh" ) };
      assert!( !Rc::ptr_eq( engraved_mesh, &mesh ), "the engraved node got its own mesh" );
      let engraved_primitive = engraved_mesh.borrow().primitives[ 0 ].clone();
      let original_primitive = mesh.borrow().primitives[ 0 ].clone();
      assert!
      (
        Rc::ptr_eq( &engraved_primitive.borrow().geometry, &original_primitive.borrow().geometry ),
        "the detached mesh shares the original geometry"
      );
    }

    assert!( engraving_of( &node_materials( &engraved )[ 0 ] ).is_some(), "the configured node is engraved" );
    assert!( engraving_of( &node_materials( &plain )[ 0 ] ).is_none(), "the other user of the mesh is not" );
  }
}
