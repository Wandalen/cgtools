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
    Geometry, Material, MaterialUploadContext, Mesh, Node, Object3D, Primitive, Scene, ShaderProgram, Texture,
    engraving::
    {
      EngravingBitmap, EngravingCanvasError, EngravingCanvasManager, EngravingConfig, EngravingError, EngravingSession,
      EngravingTextLayout, EngravingTexture, SizingMode, TextAlign, font_load, mip_dimensions,
    },
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

  /// A material that isn't a `PbrMaterial`, for `build`'s `MaterialNotPbr` check. Nothing here
  /// draws, so its GL-facing methods are never called.
  #[ derive( Debug ) ]
  struct OtherMaterial;

  impl Material for OtherMaterial
  {
    fn id( &self ) -> uuid::Uuid { uuid::Uuid::nil() }
    fn needs_update( &self ) -> bool { false }
    fn needs_update_set( &self, _value : bool ) {}
    fn shader_program_make( &self, _gl : &GL, _program : &gl::WebGlProgram ) -> Box< dyn ShaderProgram >
    {
      unreachable!( "never drawn" )
    }
    fn type_name( &self ) -> &'static str { "OtherMaterial" }
    fn vertex_shader( &self ) -> String { String::new() }
    fn fragment_shader( &self ) -> String { String::new() }
    fn configure( &self, _gl : &GL, _ctx : &MaterialUploadContext< '_ > ) {}
    fn upload_on_state_change( &self, _gl : &GL, _ctx : &MaterialUploadContext< '_ > ) -> Result< (), gl::WebglError > { Ok( () ) }
    fn bind( &self, _gl : &GL ) {}
    fn dyn_clone( &self ) -> Box< dyn Material > { Box::new( OtherMaterial ) }
    fn as_any( &self ) -> &dyn std::any::Any { self }
    fn as_any_mut( &mut self ) -> &mut dyn std::any::Any { self }
  }

  /// A text layout in `mode` on a strip `strip_mm` tall, aiming at `font_mm` letters.
  fn layout( mode : SizingMode, strip_mm : f32, font_mm : f32, min_mm : Option< f32 > ) -> EngravingTextLayout
  {
    EngravingTextLayout
    {
      max_characters : 64,
      padding : 0.1,
      align : TextAlign::Center,
      sizing_mode : Some( mode ),
      strip_height_mm : Some( strip_mm ),
      default_font_size_mm : Some( font_mm ),
      min_font_size_mm : min_mm,
      font_size_ratio : None,
    }
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

  /// `build` rejects a node whose texture the GPU can't hold, before touching the scene.
  #[ wasm_bindgen_test ]
  fn build_rejects_a_texture_above_max_texture_size()
  {
    let gl = gl_init();
    let mut scene = Scene::new();
    let material = pbr_material( &gl );
    let node = node_add( &mut scene, "Long", Object3D::Mesh( mesh_make( &gl, &[ &material ] ) ) );
    let config = EngravingConfig::from_json
    (
      r#"{ "nodes": [ { "nodeName": "Long", "uvChannel": 1, "aspectRatio": 100000.0, "maxCharacters": 8,
      "textureHeight": 16, "defaultFont": "monospace", "allowedFonts": [ "monospace" ] } ] }"#
    ).expect( "valid config" );

    let Err( err ) = EngravingSession::build( &gl, &scene, &config ) else { panic!( "a 1600000 px wide texture must be rejected" ) };
    assert!
    (
      matches!( &err, EngravingError::TextureTooLarge { node, width : 1_600_000, height : 16, .. } if &**node == "Long" ),
      "unexpected error: {err:?}"
    );
    assert!( Rc::ptr_eq( &node_materials( &node )[ 0 ], &material ), "a failed build leaves the scene untouched" );
  }

  /// `set_text` and its synchronous half `text_render` only accept the node's `allowedFonts`.
  #[ wasm_bindgen_test( async ) ]
  async fn set_text_rejects_a_font_outside_the_whitelist()
  {
    let gl = gl_init();
    let mut scene = Scene::new();
    node_add( &mut scene, "A", Object3D::Mesh( mesh_make( &gl, &[ &pbr_material( &gl ) ] ) ) );
    let mut session = EngravingSession::build( &gl, &scene, &config_for( &[ "A" ] ) ).expect( "build succeeds" );

    let err = session.set_text( &gl, "A", "Hi", "serif" ).await.expect_err( "serif is not whitelisted" );
    assert!
    (
      matches!( &err, EngravingError::FontNotAllowed { node, font } if &**node == "A" && &**font == "serif" ),
      "unexpected error: {err:?}"
    );

    session.set_text( &gl, "A", "Hi", "monospace" ).await.expect( "a whitelisted font renders" );

    let err = session.text_render( &gl, "A", "Hi", "serif" ).expect_err( "text_render checks the whitelist too" );
    assert!( matches!( err, EngravingError::FontNotAllowed { .. } ), "unexpected error: {err:?}" );
    font_load( "monospace", "Hey" ).await.expect( "font loads" );
    session.text_render( &gl, "A", "Hey", "monospace" ).expect( "a loaded whitelisted font renders synchronously" );
  }

  /// `build` names the node it can't engrave, and why.
  #[ wasm_bindgen_test ]
  fn build_reports_nodes_it_cannot_engrave()
  {
    let gl = gl_init();
    let mut scene = Scene::new();
    node_add( &mut scene, "Empty", Object3D::Other );
    node_add( &mut scene, "NoPrimitives", Object3D::Mesh( Rc::new( RefCell::new( Mesh::new() ) ) ) );
    let other : SharedMaterial = Rc::new( RefCell::new( Box::new( OtherMaterial ) ) );
    node_add( &mut scene, "Custom", Object3D::Mesh( mesh_make( &gl, &[ &pbr_material( &gl ), &other ] ) ) );

    let error_for = | name : &str | match EngravingSession::build( &gl, &scene, &config_for( &[ name ] ) )
    {
      Ok( _ ) => panic!( "building for '{name}' must fail" ),
      Err( e ) => e,
    };

    assert!( matches!( error_for( "Missing" ), EngravingError::NodeNotFound( n ) if &*n == "Missing" ) );
    assert!( matches!( error_for( "Empty" ), EngravingError::NotAMesh( n ) if &*n == "Empty" ) );
    assert!( matches!( error_for( "NoPrimitives" ), EngravingError::NotAMesh( n ) if &*n == "NoPrimitives" ) );
    assert!( matches!( error_for( "Custom" ), EngravingError::MaterialNotPbr( n ) if &*n == "Custom" ) );
  }

  /// `EngravingTexture::update` takes exactly one bitmap per mip level, each at its level's size.
  #[ wasm_bindgen_test ]
  fn texture_update_rejects_a_mismatched_chain()
  {
    let gl = gl_init();
    let texture = EngravingTexture::new( &gl, 8, 2, 1 ).expect( "small texture allocates" );
    assert_eq!( texture.level_count(), 4 );

    let chain = | sizes : &[ ( u32, u32 ) ] | sizes.iter()
    .map( | &( width, height ) | EngravingBitmap { width, height, pixels : vec![ 0; ( width * height * 4 ) as usize ] } )
    .collect::< Vec< _ > >();

    let valid : Vec< _ > = mip_dimensions( 8, 2 ).collect();
    assert_eq!( valid, vec![ ( 8, 2 ), ( 4, 1 ), ( 2, 1 ), ( 1, 1 ) ] );
    texture.update( &gl, &chain( &valid ) ).expect( "a matching chain uploads" );

    assert!( texture.update( &gl, &chain( &valid[ ..3 ] ) ).is_err(), "one level short" );
    assert!( texture.update( &gl, &chain( &[ ( 8, 2 ), ( 4, 2 ), ( 2, 1 ), ( 1, 1 ) ] ) ).is_err(), "level 1 at the wrong size" );
  }

  /// The canvas rasterizes exactly the levels the texture allocated, at the same sizes, so the
  /// chain always uploads.
  #[ wasm_bindgen_test( async ) ]
  async fn canvas_mip_chain_matches_the_texture_levels()
  {
    let gl = gl_init();
    let ( width, height ) = ( 37, 5 );
    let texture = EngravingTexture::new( &gl, width, height, 1 ).expect( "texture allocates" );
    let mut canvas = EngravingCanvasManager::new( width, height ).expect( "canvas is created" );
    let relative = layout( SizingMode::Relative, 1.0, 0.5, None );

    let bitmaps = canvas.render_text_mip_chain( "Mip", "monospace", &relative ).await.expect( "text renders" );

    assert_eq!( bitmaps.len() as u32, texture.level_count() );
    let sizes : Vec< _ > = bitmaps.iter().map( | b | ( b.width, b.height ) ).collect();
    assert_eq!( sizes, mip_dimensions( width, height ).collect::< Vec< _ > >() );
    texture.update( &gl, &bitmaps ).expect( "the canvas chain uploads" );
  }

  /// `PHYSICAL` keeps the configured letter height and rejects text that overflows the strip;
  /// `HYBRID` shrinks the same text to fit, down to `minFontSizeMm`.
  #[ wasm_bindgen_test( async ) ]
  async fn physical_rejects_overflow_that_hybrid_shrinks_to_fit()
  {
    let mut canvas = EngravingCanvasManager::new( 64, 16 ).expect( "canvas is created" );
    // 0.8 of the strip height: 12.8 px letters, far too wide for 20 characters in 64 px.
    let long = "WWWWWWWWWWWWWWWWWWWW";

    let physical = layout( SizingMode::Physical, 2.0, 1.6, None );
    let err = canvas.render_text_mip_chain( long, "monospace", &physical ).await.expect_err( "PHYSICAL never shrinks" );
    assert!( matches!( err, EngravingCanvasError::TextOverflow { .. } ), "unexpected error: {err:?}" );
    canvas.render_text_mip_chain( "I", "monospace", &physical ).await.expect( "text that fits renders at full size" );

    let hybrid = layout( SizingMode::Hybrid, 2.0, 1.6, None );
    canvas.render_text_mip_chain( long, "monospace", &hybrid ).await.expect( "HYBRID shrinks to fit" );

    // A floor of 1.5 mm keeps the letters near full size, so the text still can't fit.
    let floored = layout( SizingMode::Hybrid, 2.0, 1.6, Some( 1.5 ) );
    let err = canvas.render_text_mip_chain( long, "monospace", &floored ).await.expect_err( "HYBRID stops at minFontSizeMm" );
    assert!( matches!( err, EngravingCanvasError::TextOverflow { .. } ), "unexpected error: {err:?}" );
  }
}
