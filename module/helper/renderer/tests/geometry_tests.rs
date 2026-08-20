//! Tests for `Geometry`'s attribute API ( `renderer::webgl::geometry` ).

#[ cfg( target_arch = "wasm32" ) ]
#[ cfg( test ) ]
mod tests
{
  use wasm_bindgen_test::wasm_bindgen_test;

  // Fix(BUG-110): this suite had no `wasm_bindgen_test_configure!( run_in_browser )` call, so
  // its one test binary defaulted to Node.js, where `web_sys::window()` is always `None`.
  // Root cause: file created without the configure! line every sibling suite in this directory
  // (animation_tests.rs, pmrem_tests.rs, skeleton_tests.rs) already carries.
  // Pitfall: a missing `run_in_browser` config doesn't fail to compile — it fails at runtime
  // with an unrelated-looking `CanvasRetrievingError("Failed to get window")`, which reads like
  // a `minwebgl`/`mingl` regression rather than the test's own harness misconfiguration.
  wasm_bindgen_test::wasm_bindgen_test_configure!( run_in_browser );
  use minwebgl as gl;
  use gl::GL;
  use mingl::geometry::BoundingBox;
  use renderer::webgl::{ Geometry, AttributeInfo };

  /// Creates a headless WebGL2 context for structural tests.
  fn gl_init() -> GL
  {
    gl::browser::setup( gl::browser::Config::default() );
    let canvas = gl::canvas::make().unwrap();
    gl::context::from_canvas( &canvas ).unwrap()
  }

  /// Builds a minimal, valid `AttributeInfo` for exercising `Geometry::add_attribute`.
  /// Contents are irrelevant to this structural test — only that the buffer/descriptor
  /// are well-formed enough for `AttributeInfo::upload` to succeed.
  fn make_attribute_info( gl : &GL ) -> AttributeInfo
  {
    let buffer = gl.create_buffer().unwrap();
    let attr = mingl::VertexAttribute::new( 0, mingl::VectorDataType::new( mingl::DataType::F32, 3, 1 ), 0 );
    let descriptor = gl::BufferDescriptor::from_vector( attr.vector ).offset( attr.offset ).stride( 0 );

    AttributeInfo
    {
      slot : 0,
      buffer,
      descriptor,
      bounding_box : BoundingBox::default()
    }
  }

  /// ## Root Cause
  /// `Geometry::add_attribute` (`src/webgl/geometry.rs`) already declared
  /// `-> Result< (), gl::WebglError >`, but its duplicate-attribute-name branch called
  /// `panic!( "An attribute {} already exists", name )` instead of returning `Err` — an
  /// ordinary, externally-reachable condition (e.g. malformed glTF re-declaring the same
  /// accessor semantic twice) treated as an unrecoverable abort.
  /// ## Why Not Caught
  /// No existing test called `add_attribute` twice with the same name — every prior caller
  /// either used unique names or only exercised the happy path, so the panic branch was never
  /// executed.
  /// ## Fix Applied
  /// The duplicate-name branch now returns
  /// `Err( gl::WebglError::Other( "An attribute with this name already exists" ) )`; the
  /// function's doc comment was updated from "It panics if..." to "Returns `Err` if...".
  /// ## Prevention
  /// Any fn declared `-> Result< _, _ >` must route every failure branch through `Err`, even
  /// ones that read like "should never happen" in normal use.
  /// ## Pitfall
  /// A `Result`-returning signature is a caller-facing contract — a `panic!` hidden inside one
  /// of its branches breaks that contract silently until the exact input that triggers it is
  /// exercised.
  #[ wasm_bindgen_test( async ) ]
  async fn add_attribute_duplicate_name_returns_err_not_panic()
  {
    let gl = gl_init();
    let mut geometry = Geometry::new( &gl ).expect( "Geometry::new should succeed" );

    geometry.attribute_add( &gl, "positions", make_attribute_info( &gl ) )
    .expect( "first add_attribute call with a fresh name should succeed" );

    let result = geometry.attribute_add( &gl, "positions", make_attribute_info( &gl ) );

    assert!
    (
      result.is_err(),
      "adding a duplicate attribute name must return Err, not panic — got {result:?}"
    );
  }
}
