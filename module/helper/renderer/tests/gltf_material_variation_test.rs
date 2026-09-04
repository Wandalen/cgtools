//! Verifies the glTF loader's material-variation cache
//! ( `renderer::webgl::loaders::gltf::material_variation_resolve` ) — the pure lookup-or-insert
//! pairing extracted out of `primitive_material_resolve` specifically so it could be tested here
//! without a live `WebGl2RenderingContext` ( which the rest of the loader needs, per
//! `gltf_loader_tests.rs`'s own scope note ). `TestMaterial` below is a minimal, zero-GL-call
//! stand-in for the real `PbrMaterial` : every required `Material` trait method that takes a
//! `&WebGl2RenderingContext` is a stub that is never invoked by these tests.
//
// test_kind: bug_reproducer(BUG-245)
//
// ## Root Cause
// `primitive_material_resolve` looked up `material_variation_map` to reuse an existing clone,
// but on a cache miss it pushed the newly-created clone only into `used_materials`, never back
// into `material_variation_map` itself — the map parameter's type was `&FxHashMap<...>` ( shared
// reference ), so there was no way to write to it at all. Every entry `materials_create` seeded
// as an empty `Vec::new()` therefore stayed empty forever, and the lookup permanently missed on
// every call, for every primitive.
//
// ## Why Not Caught
// No test exercised `material_variation_map` across two sequential resolutions of the same
// `( material_id, vertex_defines )` pair. The defect has no crash and no visible artifact under
// casual testing — the loader still produces a *correct-looking* scene, just with more distinct
// `Rc` instances than intended ( unshared clones instead of shared ones ), which only matters to
// a consumer that mutates one shared instance and expects every primitive using it to observe
// the change.
//
// ## Fix Applied
// Extracted the lookup-or-insert pairing into its own function, `material_variation_resolve`,
// taking `material_variation_map` as `&mut` and performing the insert immediately after
// constructing a new variation — in the same place, so the two halves can't drift apart again.
// `primitive_material_resolve` now delegates to it, passing the GPU-dependent clone-and-configure
// step as a lazily-invoked closure so a cache hit still never touches `dyn_clone`/GL state.
//
// ## Prevention
// Any lookup-or-insert ( "get or create and remember" ) cache pairing should be a single function
// that owns both halves, not two call sites relying on the caller to keep them in sync — and
// should take its backing map by `&mut`, never `&`, since a cache that cannot be written to is
// not a cache.
//
// ## Pitfall
// A cache parameter typed `&FxHashMap<...>` compiles fine and looks identical to callers that
// only ever read from it — the missing mutability is invisible until you specifically test that
// a *second* lookup for the same key returns the *first* lookup's freshly-inserted value.

use renderer::webgl::loaders::gltf::{ material_variation_resolve, SharedMaterial };
use renderer::webgl::{ Material, MaterialUploadContext, ShaderProgram };
use minwebgl as gl;
use rustc_hash::FxHashMap;
use std::{ cell::{ Cell, RefCell }, rc::Rc };

/// Zero-GL-call stand-in for `PbrMaterial`, sized only for exercising
/// `material_variation_resolve`'s cache logic. Every method taking `&WebGl2RenderingContext`
/// is unreachable in these tests and stubbed accordingly.
#[ derive( Debug ) ]
struct TestMaterial
{
  id : uuid::Uuid,
  vertex_defines_str : String,
}

impl Material for TestMaterial
{
  fn id( &self ) -> uuid::Uuid { self.id }
  fn needs_update( &self ) -> bool { false }
  fn needs_update_set( &self, _value : bool ) {}
  fn shader_program_make( &self, _gl : &gl::WebGl2RenderingContext, _program : &gl::WebGlProgram ) -> Box< dyn ShaderProgram >
  {
    unreachable!( "not exercised by material_variation_resolve tests" )
  }
  fn type_name( &self ) -> &'static str { "TestMaterial" }
  fn vertex_shader( &self ) -> String { String::new() }
  fn fragment_shader( &self ) -> String { String::new() }
  fn vertex_defines_str( &self ) -> &str { &self.vertex_defines_str }
  fn configure( &self, _gl : &gl::WebGl2RenderingContext, _ctx : &MaterialUploadContext< '_ > ) {}
  fn upload_on_state_change( &self, _gl : &gl::WebGl2RenderingContext, _ctx : &MaterialUploadContext< '_ > ) -> Result< (), gl::WebglError >
  {
    Ok( () )
  }
  fn bind( &self, _gl : &gl::WebGl2RenderingContext ) {}
  fn dyn_clone( &self ) -> Box< dyn Material >
  {
    Box::new( TestMaterial { id : self.id, vertex_defines_str : self.vertex_defines_str.clone() } )
  }
}

fn test_material( id : uuid::Uuid, vertex_defines_str : &str ) -> SharedMaterial
{
  Rc::new( RefCell::new( Box::new( TestMaterial { id, vertex_defines_str : vertex_defines_str.to_string() } ) ) )
}

#[ test ]
fn first_lookup_for_a_material_id_creates_and_caches_a_new_variation()
{
  let mut map : FxHashMap< uuid::Uuid, Vec< SharedMaterial > > = FxHashMap::default();
  let material_id = uuid::Uuid::new_v4();

  let result = material_variation_resolve
  (
    &mut map,
    material_id,
    "DEFINE_A",
    || test_material( material_id, "DEFINE_A" )
  );

  assert_eq!( result.borrow().vertex_defines_str(), "DEFINE_A" );
  assert_eq!( map.get( &material_id ).map( Vec::len ), Some( 1 ), "the new variation must be recorded in the map" );
}

#[ test ]
fn second_lookup_with_the_same_vertex_defines_reuses_the_cached_variation()
{
  // BUG-245: pre-fix, `material_variation_map` was never written to on a cache miss, so this
  // second call would have missed the cache and invoked `new_material` again -- returning a
  // *different* `Rc` instead of the one the first call created and cached.
  let mut map : FxHashMap< uuid::Uuid, Vec< SharedMaterial > > = FxHashMap::default();
  let material_id = uuid::Uuid::new_v4();

  let first = material_variation_resolve
  (
    &mut map,
    material_id,
    "DEFINE_A",
    || test_material( material_id, "DEFINE_A" )
  );

  let second_call_constructed_new = Rc::new( Cell::new( false ) );
  let flag = second_call_constructed_new.clone();
  let second = material_variation_resolve
  (
    &mut map,
    material_id,
    "DEFINE_A",
    move || { flag.set( true ); test_material( material_id, "DEFINE_A" ) }
  );

  assert!( !second_call_constructed_new.get(), "cache hit must not invoke new_material" );
  assert!( Rc::ptr_eq( &first, &second ), "second lookup must return the exact instance the first call cached" );
  assert_eq!( map.get( &material_id ).map( Vec::len ), Some( 1 ), "no second variation should have been recorded" );
}

#[ test ]
fn different_vertex_defines_under_the_same_material_id_creates_a_second_independent_variation()
{
  let mut map : FxHashMap< uuid::Uuid, Vec< SharedMaterial > > = FxHashMap::default();
  let material_id = uuid::Uuid::new_v4();

  let a = material_variation_resolve( &mut map, material_id, "DEFINE_A", || test_material( material_id, "DEFINE_A" ) );
  let b = material_variation_resolve( &mut map, material_id, "DEFINE_B", || test_material( material_id, "DEFINE_B" ) );

  assert!( !Rc::ptr_eq( &a, &b ), "distinct vertex defines must not share an instance" );
  assert_eq!( map.get( &material_id ).map( Vec::len ), Some( 2 ), "both variations must be recorded under the same material id" );
}

#[ test ]
fn different_material_ids_are_cached_independently()
{
  let mut map : FxHashMap< uuid::Uuid, Vec< SharedMaterial > > = FxHashMap::default();
  let id_a = uuid::Uuid::new_v4();
  let id_b = uuid::Uuid::new_v4();

  let _ = material_variation_resolve( &mut map, id_a, "DEFINE_A", || test_material( id_a, "DEFINE_A" ) );
  let _ = material_variation_resolve( &mut map, id_b, "DEFINE_A", || test_material( id_b, "DEFINE_A" ) );

  assert_eq!( map.get( &id_a ).map( Vec::len ), Some( 1 ) );
  assert_eq!( map.get( &id_b ).map( Vec::len ), Some( 1 ) );
}
