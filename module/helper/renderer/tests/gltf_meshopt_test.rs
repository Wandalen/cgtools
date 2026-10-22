//! Off-GPU checks for loading meshopt-compressed glTF ( `EXT_meshopt_compression` /
//! `KHR_meshopt_compression` ) : parsing and validating each buffer view's extension object,
//! the range checks run before decoding, fallback-buffer detection, and validation that lets
//! these extensions ( and `KHR_mesh_quantization` ) through. The decode itself runs
//! meshoptimizer's JS decoder, so its tests are in `gltf_meshopt_browser_test.rs`; the
//! dequantization of `KHR_mesh_quantization` data is in `gltf_quantization_test.rs`.

use renderer::webgl::loaders::gltf_extensions::{ document_validate, required_extensions_check };
use renderer::webgl::loaders::meshopt::internal::{ buffer_is_fallback, decodes_overlap_check, MeshoptFilter, MeshoptMode, MeshoptViewSpec, MESHOPT_EXTENSIONS };
use serde_json::json;

/// The layout glTF-Transform writes : compressed bytes in buffer 0, and a data-less
/// fallback buffer 1 that the views decode into. Indices, octahedral normals and
/// quantized positions, like `rara_585_solune_8.5.glb`.
const MESHOPT_FIXTURE : &str = r#"
{
  "asset": { "version": "2.0" },
  "extensionsUsed": [ "EXT_meshopt_compression", "KHR_mesh_quantization" ],
  "extensionsRequired": [ "EXT_meshopt_compression", "KHR_mesh_quantization" ],
  "accessors":
  [
    { "bufferView": 0, "componentType": 5125, "count": 6, "type": "SCALAR" },
    { "bufferView": 1, "componentType": 5120, "normalized": true, "count": 4, "type": "VEC3" },
    { "bufferView": 2, "componentType": 5122, "normalized": true, "count": 4, "type": "VEC3",
      "min": [ -32767, -16384, 0 ], "max": [ 32767, 16384, 32767 ] }
  ],
  "bufferViews":
  [
    { "buffer": 1, "byteOffset": 0, "byteLength": 24, "target": 34963,
      "extensions": { "EXT_meshopt_compression":
        { "buffer": 0, "byteOffset": 0, "byteLength": 10, "byteStride": 4, "count": 6, "mode": "TRIANGLES" } } },
    { "buffer": 1, "byteOffset": 24, "byteLength": 16, "byteStride": 4, "target": 34962,
      "extensions": { "EXT_meshopt_compression":
        { "buffer": 0, "byteOffset": 12, "byteLength": 12, "byteStride": 4, "count": 4, "mode": "ATTRIBUTES", "filter": "OCTAHEDRAL" } } },
    { "buffer": 1, "byteOffset": 40, "byteLength": 32, "byteStride": 8, "target": 34962,
      "extensions": { "EXT_meshopt_compression":
        { "buffer": 0, "byteOffset": 24, "byteLength": 20, "byteStride": 8, "count": 4, "mode": "ATTRIBUTES" } } }
  ],
  "buffers":
  [
    { "byteLength": 44, "uri": "placeholder.bin" },
    { "byteLength": 72, "extensions": { "EXT_meshopt_compression": { "fallback": true } } }
  ]
}
"#;

/// The extension name the fixtures parse under, unless a test says otherwise.
const EXT : &str = "EXT_meshopt_compression";

fn fixture() -> gltf::Gltf
{
  gltf::Gltf::from_slice_without_validation( MESHOPT_FIXTURE.as_bytes() ).expect( "fixture is well-formed JSON" )
}

/// An unfiltered `ATTRIBUTES` spec with a 4-byte stride, built through
/// `MeshoptViewSpec::parse` : its fields are private, so a test reaches a spec the way the
/// loader does.
fn attributes_spec( buffer : u64, byte_offset : u64, byte_length : u64, count : u64 ) -> MeshoptViewSpec
{
  let value = json!( { "buffer": buffer, "byteOffset": byte_offset, "byteLength": byte_length, "byteStride": 4, "count": count, "mode": "ATTRIBUTES" } );
  MeshoptViewSpec::parse( EXT, &value ).expect( "a well-formed ATTRIBUTES spec" )
}

#[ test ]
fn reads_each_views_extension()
{
  let gltf = fixture();
  let specs = gltf.views()
  .map( | v | MeshoptViewSpec::from_view( &v ).expect( "every fixture view is compressed" ).unwrap() )
  .collect::< Vec< _ > >();

  let first = &specs[ 0 ];
  assert_eq!
  (
    ( first.buffer(), first.byte_offset(), first.byte_length(), first.byte_stride(), first.count() ),
    ( 0, 0, 10, 4, 6 )
  );
  assert_eq!( ( first.mode(), first.filter() ), ( MeshoptMode::Triangles, MeshoptFilter::None ) );
  assert_eq!( specs[ 1 ].filter(), MeshoptFilter::Octahedral );
  assert_eq!( specs[ 2 ].byte_stride(), 8 );
  assert_eq!( specs[ 2 ].decoded_length(), Some( 32 ) );
}

#[ test ]
fn a_plain_view_is_not_compressed()
{
  let gltf = gltf::Gltf::from_slice_without_validation
  (
    br#"{ "asset": { "version": "2.0" }, "bufferViews": [ { "buffer": 0, "byteLength": 4 } ], "buffers": [ { "byteLength": 4, "uri": "a.bin" } ] }"#
  ).unwrap();
  assert!( MeshoptViewSpec::from_view( &gltf.views().next().unwrap() ).is_none() );
  assert!( !buffer_is_fallback( &gltf.buffers().next().unwrap() ) );
}

#[ test ]
fn the_khr_name_is_read_too()
{
  let gltf = gltf::Gltf::from_slice_without_validation
  (
    br#"{ "asset": { "version": "2.0" },
      "bufferViews": [ { "buffer": 0, "byteLength": 16, "extensions": { "KHR_meshopt_compression":
        { "buffer": 0, "byteLength": 8, "byteStride": 4, "count": 4, "mode": "ATTRIBUTES", "filter": "COLOR" } } } ],
      "buffers": [ { "byteLength": 16, "uri": "a.bin", "extensions": { "KHR_meshopt_compression": { "fallback": true } } } ] }"#
  ).unwrap();
  let spec = MeshoptViewSpec::from_view( &gltf.views().next().unwrap() ).unwrap().unwrap();
  assert_eq!( spec.filter(), MeshoptFilter::Color );
  assert_eq!( spec.byte_offset(), 0, "byteOffset defaults to 0" );
  assert!( buffer_is_fallback( &gltf.buffers().next().unwrap() ), "a fallback with a uri is still not fetched" );
}

/// `KHR_meshopt_compression` must not be used on a buffer view that also uses
/// `EXT_meshopt_compression`, so a view carrying both is refused rather than read as one.
#[ test ]
fn a_view_carrying_both_extensions_is_refused()
{
  let gltf = gltf::Gltf::from_slice_without_validation
  (
    br#"{ "asset": { "version": "2.0" },
      "bufferViews": [ { "buffer": 0, "byteLength": 16, "extensions": {
        "EXT_meshopt_compression": { "buffer": 0, "byteLength": 8, "byteStride": 4, "count": 4, "mode": "ATTRIBUTES" },
        "KHR_meshopt_compression": { "buffer": 0, "byteLength": 8, "byteStride": 4, "count": 4, "mode": "ATTRIBUTES", "filter": "COLOR" } } } ],
      "buffers": [ { "byteLength": 16, "uri": "a.bin" } ] }"#
  ).unwrap();
  assert_eq!
  (
    MeshoptViewSpec::from_view( &gltf.views().next().unwrap() ),
    Some( Err( "uses both EXT_meshopt_compression and KHR_meshopt_compression, which KHR_meshopt_compression forbids".to_owned() ) )
  );
}

#[ test ]
fn finds_the_fallback_buffer()
{
  let gltf = fixture();
  let fallback = gltf.buffers().map( | b | buffer_is_fallback( &b ) ).collect::< Vec< _ > >();
  assert_eq!( fallback, [ false, true ] );
}

/// Every malformed field is refused with its own reason. `parse` returns that reason as a
/// `String`, the one the loader logs, so each case is pinned exactly.
#[ test ]
fn rejects_malformed_extension_objects()
{
  let base = json!( { "buffer": 0, "byteLength": 8, "byteStride": 4, "count": 3, "mode": "ATTRIBUTES" } );
  assert!( MeshoptViewSpec::parse( EXT, &base ).is_ok() );

  let with = | key : &str, value : serde_json::Value |
  {
    let mut v = base.clone();
    v[ key ] = value;
    MeshoptViewSpec::parse( EXT, &v )
  };
  let without = | key : &str |
  {
    let mut v = base.clone();
    v.as_object_mut().unwrap().remove( key );
    MeshoptViewSpec::parse( EXT, &v )
  };
  let refused = | reason : &str | Err( reason.to_owned() );

  for key in [ "buffer", "byteLength", "byteStride", "count" ]
  {
    assert_eq!( without( key ), refused( &format!( "missing `{key}`" ) ) );
  }
  assert_eq!( without( "mode" ), refused( "missing `mode`" ) );
  assert_eq!( with( "mode", json!( 1 ) ), refused( "`mode` is not a string" ), "a mode that is there is not reported missing" );
  assert_eq!( with( "filter", json!( 2 ) ), refused( "`filter` is not a string" ), "a non-string filter is not read as no filter" );
  assert_eq!( with( "count", json!( -1 ) ), refused( "`count` is not a non-negative integer" ), "negative count" );
  assert_eq!( with( "count", json!( 1.5 ) ), refused( "`count` is not a non-negative integer" ), "fractional count" );
  assert_eq!( with( "mode", json!( "STRIPS" ) ), refused( "unknown mode `STRIPS`" ) );
  assert_eq!( with( "filter", json!( "WAVELET" ) ), refused( "unknown EXT_meshopt_compression filter `WAVELET`" ) );
  assert_eq!( with( "byteStride", json!( 6 ) ), refused( "ATTRIBUTES byteStride 6 must be a multiple of 4 up to 256" ) );
  assert_eq!( with( "byteStride", json!( 260 ) ), refused( "ATTRIBUTES byteStride 260 must be a multiple of 4 up to 256" ) );
  assert_eq!( with( "filter", json!( "QUATERNION" ) ), refused( "filter QUATERNION does not support byteStride 4" ) );
  assert!( with( "filter", json!( "OCTAHEDRAL" ) ).is_ok() );

  let indices = json!( { "buffer": 0, "byteLength": 8, "byteStride": 4, "count": 6, "mode": "TRIANGLES" } );
  assert!( MeshoptViewSpec::parse( EXT, &indices ).is_ok() );
  let mut odd = indices.clone();
  odd[ "count" ] = json!( 7 );
  assert_eq!( MeshoptViewSpec::parse( EXT, &odd ), refused( "TRIANGLES count 7 is not a multiple of 3" ) );
  let mut filtered = indices.clone();
  filtered[ "filter" ] = json!( "OCTAHEDRAL" );
  assert_eq!( MeshoptViewSpec::parse( EXT, &filtered ), refused( "TRIANGLES data cannot use a filter" ) );
  let mut wide = indices;
  wide[ "byteStride" ] = json!( 8 );
  assert_eq!( MeshoptViewSpec::parse( EXT, &wide ), refused( "TRIANGLES byteStride 8 must be 2 or 4" ) );
}

/// The layouts the specs allow are accepted, not only the malformed ones refused : 16- and
/// 32-bit indices in both index modes, `QUATERNION` on 8-byte elements and `EXPONENTIAL` on
/// any multiple of 4. A check that refused one of them would disable that data silently.
#[ test ]
fn accepts_every_layout_the_specs_allow()
{
  let parse = | value : serde_json::Value | MeshoptViewSpec::parse( EXT, &value );
  for mode in [ "TRIANGLES", "INDICES" ]
  {
    for stride in [ 2, 4 ]
    {
      let spec = parse( json!( { "buffer": 0, "byteLength": 8, "byteStride": stride, "count": 6, "mode": mode } ) );
      assert!( spec.is_ok(), "{mode} with byteStride {stride}: {spec:?}" );
    }
  }
  let indices = parse( json!( { "buffer": 0, "byteLength": 8, "byteStride": 2, "count": 7, "mode": "INDICES" } ) );
  assert!( indices.is_ok(), "INDICES need no multiple of 3: {indices:?}" );

  let quaternion = parse( json!( { "buffer": 0, "byteLength": 8, "byteStride": 8, "count": 2, "mode": "ATTRIBUTES", "filter": "QUATERNION" } ) );
  assert_eq!( quaternion.map( | s | s.filter() ), Ok( MeshoptFilter::Quaternion ) );
  for stride in [ 4, 8, 12, 256 ]
  {
    let exponential = parse( json!( { "buffer": 0, "byteLength": 8, "byteStride": stride, "count": 2, "mode": "ATTRIBUTES", "filter": "EXPONENTIAL" } ) );
    assert_eq!( exponential.map( | s | s.filter() ), Ok( MeshoptFilter::Exponential ), "EXPONENTIAL with byteStride {stride}" );
  }
}

/// `OCTAHEDRAL` and `COLOR` decode 4- or 8-byte elements only, and `QUATERNION` 8-byte ones :
/// each is accepted at its strides and refused, with the stride named, at the others.
#[ test ]
fn filters_accept_only_their_strides()
{
  let parse = | extension : &str, filter : &str, stride : u64 |
    MeshoptViewSpec::parse( extension, &json!( { "buffer": 0, "byteLength": 8, "byteStride": stride, "count": 2, "mode": "ATTRIBUTES", "filter": filter } ) );
  for ( extension, filter ) in [ ( EXT, "OCTAHEDRAL" ), ( "KHR_meshopt_compression", "OCTAHEDRAL" ), ( "KHR_meshopt_compression", "COLOR" ) ]
  {
    for stride in [ 4, 8 ]
    {
      let spec = parse( extension, filter, stride );
      assert_eq!( spec.map( | s | s.filter().as_str() ), Ok( filter ), "{extension} {filter} with byteStride {stride}" );
    }
    for stride in [ 12, 16 ]
    {
      assert_eq!
      (
        parse( extension, filter, stride ),
        Err( format!( "filter {filter} does not support byteStride {stride}" ) ),
        "{extension} {filter} with byteStride {stride}"
      );
    }
  }
  assert_eq!( parse( EXT, "QUATERNION", 4 ), Err( "filter QUATERNION does not support byteStride 4".to_owned() ) );
}

/// Every mode and filter name parses to the variant whose `as_str` gives it back : the
/// decoder receives `as_str`, so a typo on either side would only show as a JS decode
/// failure. Names a variant has no `as_str` for, or the reverse, would fail here.
#[ test ]
fn mode_and_filter_names_round_trip()
{
  for name in [ "ATTRIBUTES", "TRIANGLES", "INDICES" ]
  {
    let stride = if name == "ATTRIBUTES" { 4 } else { 2 };
    let spec = MeshoptViewSpec::parse( EXT, &json!( { "buffer": 0, "byteLength": 8, "byteStride": stride, "count": 3, "mode": name } ) ).unwrap();
    assert_eq!( spec.mode().as_str(), name );
  }
  for ( extension, names ) in
  [
    ( EXT, &[ "NONE", "OCTAHEDRAL", "QUATERNION", "EXPONENTIAL" ][ .. ] ),
    ( "KHR_meshopt_compression", &[ "NONE", "OCTAHEDRAL", "QUATERNION", "EXPONENTIAL", "COLOR" ][ .. ] ),
  ]
  {
    for &name in names
    {
      let spec = MeshoptViewSpec::parse( extension, &json!( { "buffer": 0, "byteLength": 8, "byteStride": 8, "count": 1, "mode": "ATTRIBUTES", "filter": name } ) ).unwrap();
      assert_eq!( spec.filter().as_str(), name, "{extension}" );
    }
  }
}

#[ test ]
fn the_color_filter_is_khr_only()
{
  let color = json!( { "buffer": 0, "byteLength": 8, "byteStride": 4, "count": 4, "mode": "ATTRIBUTES", "filter": "COLOR" } );
  assert_eq!( MeshoptViewSpec::parse( "KHR_meshopt_compression", &color ).unwrap().filter(), MeshoptFilter::Color );
  assert_eq!
  (
    MeshoptViewSpec::parse( EXT, &color ),
    Err( "unknown EXT_meshopt_compression filter `COLOR`".to_owned() ),
    "EXT_meshopt_compression defines no COLOR filter"
  );
}

#[ test ]
fn checks_ranges_against_the_loaded_buffers()
{
  let spec = attributes_spec( 0, 12, 12, 4 );
  let lengths = [ 44, 72 ];

  assert!( spec.ranges_check( &lengths, 1, 24, 16, None ).is_ok() );
  assert!( spec.ranges_check( &[ 20, 72 ], 1, 24, 16, None ).is_err(), "compressed bytes run past buffer 0" );
  assert!( spec.ranges_check( &lengths, 1, 64, 16, None ).is_err(), "view runs past buffer 1" );
  assert!( spec.ranges_check( &lengths, 1, 24, 12, None ).is_err(), "decoded 16 bytes do not fit a 12-byte view" );
  assert_eq!
  (
    spec.ranges_check( &lengths, 1, 24, 20, None ),
    Err( "4 x 4 decoded bytes do not match the view's byteLength 20".to_owned() ),
    "the parent byteLength is exactly count * byteStride, not merely enough"
  );
  assert!( spec.ranges_check( &lengths, 2, 0, 16, None ).is_err(), "view names a missing buffer" );
  assert!( attributes_spec( 5, 12, 12, 4 ).ranges_check( &lengths, 1, 24, 16, None ).is_err(), "source names a missing buffer" );
  assert!( attributes_spec( 0, u64::MAX, 12, 4 ).ranges_check( &lengths, 1, 24, 16, None ).is_err(), "overflow" );
  assert!( attributes_spec( 0, 12, 12, u64::MAX ).ranges_check( &lengths, 1, 24, 16, None ).is_err(), "overflow" );
}

/// `EXT_meshopt_compression` requires a parent bufferView that declares `byteStride` to declare
/// the extension's; `KHR_meshopt_compression` dropped the rule, and a view may leave it out.
#[ test ]
fn the_parent_byte_stride_matches_under_ext_only()
{
  let lengths = [ 44, 72 ];
  let ext = attributes_spec( 0, 12, 12, 4 );
  assert!( ext.ranges_check( &lengths, 1, 24, 16, Some( 4 ) ).is_ok() );
  assert!( ext.ranges_check( &lengths, 1, 24, 16, None ).is_ok(), "no byteStride on the view" );
  assert_eq!
  (
    ext.ranges_check( &lengths, 1, 24, 16, Some( 8 ) ),
    Err( "the view's byteStride 8 is not the extension's 4".to_owned() )
  );

  let value = json!( { "buffer": 0, "byteOffset": 12, "byteLength": 12, "byteStride": 4, "count": 4, "mode": "ATTRIBUTES" } );
  let khr = MeshoptViewSpec::parse( "KHR_meshopt_compression", &value ).unwrap();
  assert!( khr.ranges_check( &lengths, 1, 24, 16, Some( 8 ) ).is_ok(), "KHR_meshopt_compression has no such rule" );
}

/// The decoder writes the view while reading the compressed bytes, so the two may share a
/// buffer only if their ranges are disjoint; touching ends are fine.
#[ test ]
fn rejects_a_decode_target_overlapping_its_source()
{
  // 12 compressed bytes at 0..12 of buffer 0, decoding 16 bytes.
  let spec = attributes_spec( 0, 0, 12, 4 );
  let lengths = [ 64 ];

  assert!( spec.ranges_check( &lengths, 0, 12, 16, None ).is_ok(), "the view starts where the compressed bytes end" );
  assert!( spec.ranges_check( &lengths, 0, 32, 16, None ).is_ok(), "disjoint ranges in one buffer" );
  assert!( spec.ranges_check( &lengths, 0, 8, 16, None ).is_err(), "the view starts inside the compressed bytes" );
  assert!( spec.ranges_check( &lengths, 0, 0, 16, None ).is_err(), "the view starts where the compressed bytes start" );
  let after = attributes_spec( 0, 20, 12, 4 );
  assert!( after.ranges_check( &lengths, 0, 8, 16, None ).is_err(), "the decoded bytes run into compressed bytes after them" );
  assert!( after.ranges_check( &lengths, 0, 4, 16, None ).is_ok(), "the decoded bytes end where the compressed bytes start" );
}

/// The fixture's three views decode side by side into the fallback buffer from disjoint
/// compressed bytes, so no view can corrupt another.
#[ test ]
fn decodes_overlap_check_accepts_the_fixture_layout()
{
  let gltf = fixture();
  let specs = gltf.views().map( | v | MeshoptViewSpec::from_view( &v ).unwrap().unwrap() ).collect::< Vec< _ > >();
  let placed = gltf.views().zip( &specs )
  .map( | ( v, spec ) | ( v.index(), spec, v.buffer().index(), v.offset() ) )
  .collect::< Vec< _ > >();
  assert!( decodes_overlap_check( &placed, &[] ).is_ok() );
}

/// Two views sharing a fallback buffer : each passes `ranges_check` alone, but a
/// decoded range that overlaps another view's would overwrite its output, and one that
/// overlaps another view's compressed bytes would overwrite its input. Ranges that only
/// touch, and compressed ranges shared between views ( they are only read ), are fine.
#[ test ]
fn decodes_overlap_check_rejects_views_corrupting_each_other()
{
  // Each decodes 16 bytes ( 4 x 4 ) from 12 compressed bytes.
  let a = attributes_spec( 0, 0, 12, 4 );
  let b = attributes_spec( 0, 12, 12, 4 );

  assert!( decodes_overlap_check( &[ ( 0, &a, 1, 0 ), ( 1, &b, 1, 16 ) ], &[] ).is_ok(), "decoded ranges touch end to start" );
  assert!( decodes_overlap_check( &[ ( 0, &a, 1, 0 ), ( 1, &b, 1, 8 ) ], &[] ).is_err(), "decoded ranges overlap" );
  assert!( decodes_overlap_check( &[ ( 0, &a, 1, 0 ), ( 1, &b, 2, 0 ) ], &[] ).is_ok(), "the same offset in different buffers" );

  // View 1 decodes into buffer 0 at 4..20, over view 0's compressed 0..12 and its own
  // compressed 12..24 : its own overlap is `ranges_check`'s case, so move its source away.
  let b_far = attributes_spec( 0, 40, 12, 4 );
  assert!( decodes_overlap_check( &[ ( 0, &a, 1, 0 ), ( 1, &b_far, 0, 4 ) ], &[] ).is_err(), "a later view decodes over an earlier view's compressed bytes" );
  assert!( decodes_overlap_check( &[ ( 1, &b_far, 0, 4 ), ( 0, &a, 1, 0 ) ], &[] ).is_err(), "an earlier view decodes over a later view's compressed bytes" );
  assert!( decodes_overlap_check( &[ ( 0, &a, 1, 0 ), ( 1, &b_far, 0, 12 ) ], &[] ).is_ok(), "the decoded range starts where the other view's compressed bytes end" );

  let a_again = attributes_spec( 0, 0, 12, 4 );
  assert!( decodes_overlap_check( &[ ( 0, &a, 1, 0 ), ( 1, &a_again, 1, 16 ) ], &[] ).is_ok(), "two views may read the same compressed bytes" );
}

/// An uncompressed view in the buffer a compressed view decodes into : its accessors would
/// read the decoded bytes of the other view, so a decode over it is refused, whichever comes
/// first. It may share bytes with compressed data, since both are only read.
#[ test ]
fn decodes_overlap_check_rejects_a_decode_over_an_uncompressed_view()
{
  // Decodes 16 bytes into buffer 1 at 0..16, from buffer 0 at 0..12.
  let a = attributes_spec( 0, 0, 12, 4 );
  let decode = [ ( 0, &a, 1, 0 ) ];

  assert_eq!
  (
    decodes_overlap_check( &decode, &[ ( 1, 1, 8, 16 ) ] ),
    Err( "bufferView 0 decodes over the data of bufferView 1".to_owned() ),
    "the uncompressed view starts inside the decoded range"
  );
  assert_eq!
  (
    decodes_overlap_check( &[ ( 1, &a, 1, 8 ) ], &[ ( 0, 1, 0, 16 ) ] ),
    Err( "bufferView 1 decodes over the data of bufferView 0".to_owned() ),
    "the decoded range starts inside the uncompressed view"
  );
  assert!( decodes_overlap_check( &decode, &[ ( 1, 1, 16, 8 ) ] ).is_ok(), "the uncompressed view starts where the decoded range ends" );
  assert!( decodes_overlap_check( &decode, &[ ( 1, 2, 0, 16 ) ] ).is_ok(), "the same range in another buffer" );
  assert!( decodes_overlap_check( &decode, &[ ( 1, 0, 4, 16 ), ( 2, 0, 0, 8 ) ] ).is_ok(), "uncompressed views may share bytes with compressed data and each other" );
  assert!( decodes_overlap_check( &decode, &[ ( 1, 1, usize::MAX, 1 ) ] ).is_err(), "an uncompressed range whose end overflows" );
}

/// The sweep in `decodes_overlap_check` agrees with the plain pairwise rule on many small
/// random layouts : a decoded range may overlap no other range, and the ranges only read (
/// compressed bytes and uncompressed views ) may overlap each other. Ranges lie in two
/// buffers, so both same-buffer and cross-buffer pairs occur.
#[ test ]
fn decodes_overlap_check_matches_the_pairwise_rule()
{
  // A small LCG, so the layouts are random but the same on every run.
  let mut state = 0x2545_f491_u64;
  let mut next = | bound : u64 | { state = state.wrapping_mul( 6_364_136_223_846_793_005 ).wrapping_add( 1 ); ( state >> 33 ) % bound };

  let overlap = | a : ( u64, u64, u64 ), b : ( u64, u64, u64 ) | a.0 == b.0 && a.1 < b.2 && b.1 < a.2;
  let ( mut accepted, mut refused ) = ( 0, 0 );
  for _ in 0..2000
  {
    let views = 1 + next( 4 );
    let layout = ( 0..views ).map( | _ |
    {
      // ( compressed buffer, compressed offset, compressed length, element count, target buffer, target offset )
      ( next( 2 ), next( 64 ), 1 + next( 8 ), 1 + next( 4 ), next( 2 ), next( 64 ) )
    } ).collect::< Vec< _ > >();

    let specs = layout.iter().map( | &( b, o, l, c, _, _ ) | attributes_spec( b, o, l, c ) ).collect::< Vec< _ > >();
    let placed = layout.iter().zip( &specs ).enumerate()
    .map( | ( i, ( &( .., tb, to ), spec ) ) | ( i, spec, usize::try_from( tb ).unwrap(), usize::try_from( to ).unwrap() ) )
    .collect::< Vec< _ > >();

    // ( buffer, offset, length ) of each uncompressed view.
    let plain_layout = ( 0..next( 3 ) ).map( | _ | ( next( 2 ), next( 64 ), 1 + next( 8 ) ) ).collect::< Vec< _ > >();
    let plain = plain_layout.iter().enumerate()
    .map( | ( i, &( b, o, l ) ) | ( layout.len() + i, usize::try_from( b ).unwrap(), usize::try_from( o ).unwrap(), usize::try_from( l ).unwrap() ) )
    .collect::< Vec< _ > >();

    let mut ranges = layout.iter().flat_map( | &( b, o, l, c, tb, to ) | [ ( ( tb, to, to + c * 4 ), true ), ( ( b, o, o + l ), false ) ] ).collect::< Vec< _ > >();
    ranges.extend( plain_layout.iter().map( | &( b, o, l ) | ( ( b, o, o + l ), false ) ) );
    let pairwise_ok = ranges.iter().enumerate().all( | ( i, &( a, a_decoded ) ) |
      ranges[ i + 1 .. ].iter().all( | &( b, b_decoded ) | !( a_decoded || b_decoded ) || !overlap( a, b ) ) );

    let swept = decodes_overlap_check( &placed, &plain );
    assert_eq!( swept.is_ok(), pairwise_ok, "layout {layout:?}, uncompressed {plain_layout:?}: sweep says {swept:?}" );
    if pairwise_ok { accepted += 1; } else { refused += 1; }
  }
  assert!( accepted > 100 && refused > 100, "both outcomes are exercised: {accepted} accepted, {refused} refused" );
}

/// Many views cost the check `O( n log n )`, not `O( n^2 )` : 100 000 disjoint views, which
/// the pairwise check would compare five billion times, go through at once.
#[ test ]
fn decodes_overlap_check_scales_to_many_views()
{
  let n = 100_000_u64;
  let specs = ( 0..n ).map( | i | attributes_spec( 0, i * 4, 4, 1 ) ).collect::< Vec< _ > >();
  let placed = specs.iter().enumerate().map( | ( i, spec ) | ( i, spec, 1, i * 4 ) ).collect::< Vec< _ > >();
  let start = std::time::Instant::now();
  assert!( decodes_overlap_check( &placed, &[] ).is_ok() );
  assert!( start.elapsed() < std::time::Duration::from_secs( 2 ), "took {:?}", start.elapsed() );
}

#[ test ]
fn meshopt_and_quantization_pass_validation()
{
  let gltf = fixture();
  assert!( gltf::Gltf::from_slice( MESHOPT_FIXTURE.as_bytes() ).is_err(), "the gltf crate alone refuses these extensions" );
  assert!( document_validate( &gltf ).is_ok() );
  assert!( required_extensions_check( &gltf ).is_ok() );
}

/// An asset may require any extension `meshopt` decodes, `KHR_meshopt_compression` as well as
/// `EXT_meshopt_compression`, and `KHR_mesh_quantization`. The loader's supported list is
/// written out by hand, so this keeps it from drifting from `MESHOPT_EXTENSIONS`.
#[ test ]
fn every_decoded_extension_may_be_required()
{
  for name in MESHOPT_EXTENSIONS.iter().chain( [ &"KHR_mesh_quantization" ] )
  {
    let json = format!( r#"{{ "asset": {{ "version": "2.0" }}, "extensionsUsed": [ "{name}" ], "extensionsRequired": [ "{name}" ] }}"# );
    let gltf = gltf::Gltf::from_slice_without_validation( json.as_bytes() ).unwrap();
    assert!( required_extensions_check( &gltf ).is_ok(), "{name} must be accepted in extensionsRequired" );
    assert!( document_validate( &gltf ).is_ok(), "{name}: document_validate applies the same rule" );
  }
}

#[ test ]
fn other_validation_errors_still_refuse_the_file()
{
  let broken = MESHOPT_FIXTURE.replace( r#"{ "bufferView": 0, "componentType""#, r#"{ "bufferView": 9, "componentType""# );
  let gltf = gltf::Gltf::from_slice_without_validation( broken.as_bytes() ).unwrap();
  assert!( document_validate( &gltf ).is_err(), "an accessor pointing at a missing bufferView" );
}

#[ test ]
fn draco_is_still_refused()
{
  let gltf = gltf::Gltf::from_slice_without_validation
  (
    br#"{ "asset": { "version": "2.0" }, "extensionsRequired": [ "KHR_draco_mesh_compression" ] }"#
  ).unwrap();
  assert!( required_extensions_check( &gltf ).is_err() );
  assert!( document_validate( &gltf ).is_err(), "document_validate applies the extensionsRequired rule as well" );
}
