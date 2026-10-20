//! Off-GPU checks for loading meshopt-compressed, quantized glTF
//! ( `EXT_meshopt_compression` / `KHR_mesh_quantization` ) : parsing and validating
//! each buffer view's extension object, the range checks run before decoding,
//! fallback-buffer detection, validation that lets these extensions through, and the
//! dequantized position bounds and accessor reads. The decode itself runs meshoptimizer's JS decoder, so
//! its tests, and those of `load`'s buffer-index rules, are in `gltf_meshopt_browser_test.rs`.

use renderer::webgl::loaders::gltf_quantization::position_bounds_dequantize;
use renderer::webgl::loaders::gltf_extensions::{ document_validate, required_extensions_check };
use renderer::webgl::loaders::meshopt::internal::{ buffer_is_fallback, decodes_overlap_check, MeshoptFilter, MeshoptMode, MeshoptViewSpec };
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

  assert!( without( "buffer" ).is_err() );
  assert!( without( "byteLength" ).is_err() );
  assert!( without( "byteStride" ).is_err() );
  assert!( without( "count" ).is_err() );
  assert_eq!( without( "mode" ), Err( "missing `mode`".to_owned() ) );
  assert_eq!( with( "mode", json!( 1 ) ), Err( "`mode` is not a string".to_owned() ), "a mode that is there is not reported missing" );
  assert_eq!( with( "filter", json!( 2 ) ), Err( "`filter` is not a string".to_owned() ), "a non-string filter is not read as no filter" );
  assert!( with( "count", json!( -1 ) ).is_err(), "negative count" );
  assert!( with( "count", json!( 1.5 ) ).is_err(), "fractional count" );
  assert!( with( "mode", json!( "STRIPS" ) ).is_err() );
  assert!( with( "filter", json!( "WAVELET" ) ).is_err() );
  assert!( with( "byteStride", json!( 6 ) ).is_err(), "ATTRIBUTES stride must be a multiple of 4" );
  assert!( with( "byteStride", json!( 260 ) ).is_err(), "ATTRIBUTES stride is at most 256" );
  assert!( with( "filter", json!( "QUATERNION" ) ).is_err(), "QUATERNION needs stride 8" );
  assert!( with( "filter", json!( "OCTAHEDRAL" ) ).is_ok() );

  let indices = json!( { "buffer": 0, "byteLength": 8, "byteStride": 4, "count": 6, "mode": "TRIANGLES" } );
  assert!( MeshoptViewSpec::parse( EXT, &indices ).is_ok() );
  let mut odd = indices.clone();
  odd[ "count" ] = json!( 7 );
  assert!( MeshoptViewSpec::parse( EXT, &odd ).is_err(), "TRIANGLES count must be a multiple of 3" );
  let mut filtered = indices.clone();
  filtered[ "filter" ] = json!( "OCTAHEDRAL" );
  assert!( MeshoptViewSpec::parse( EXT, &filtered ).is_err(), "index data takes no filter" );
  let mut wide = indices;
  wide[ "byteStride" ] = json!( 8 );
  assert!( MeshoptViewSpec::parse( EXT, &wide ).is_err(), "index stride is 2 or 4" );
}

#[ test ]
fn the_color_filter_is_khr_only()
{
  let color = json!( { "buffer": 0, "byteLength": 8, "byteStride": 4, "count": 4, "mode": "ATTRIBUTES", "filter": "COLOR" } );
  assert_eq!( MeshoptViewSpec::parse( "KHR_meshopt_compression", &color ).unwrap().filter(), MeshoptFilter::Color );
  assert!( MeshoptViewSpec::parse( EXT, &color ).is_err(), "EXT_meshopt_compression defines no COLOR filter" );
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
  assert!( decodes_overlap_check( &placed ).is_ok() );
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

  assert!( decodes_overlap_check( &[ ( 0, &a, 1, 0 ), ( 1, &b, 1, 16 ) ] ).is_ok(), "decoded ranges touch end to start" );
  assert!( decodes_overlap_check( &[ ( 0, &a, 1, 0 ), ( 1, &b, 1, 8 ) ] ).is_err(), "decoded ranges overlap" );
  assert!( decodes_overlap_check( &[ ( 0, &a, 1, 0 ), ( 1, &b, 2, 0 ) ] ).is_ok(), "the same offset in different buffers" );

  // View 1 decodes into buffer 0 at 4..20, over view 0's compressed 0..12 and its own
  // compressed 12..24 : its own overlap is `ranges_check`'s case, so move its source away.
  let b_far = attributes_spec( 0, 40, 12, 4 );
  assert!( decodes_overlap_check( &[ ( 0, &a, 1, 0 ), ( 1, &b_far, 0, 4 ) ] ).is_err(), "a later view decodes over an earlier view's compressed bytes" );
  assert!( decodes_overlap_check( &[ ( 1, &b_far, 0, 4 ), ( 0, &a, 1, 0 ) ] ).is_err(), "an earlier view decodes over a later view's compressed bytes" );
  assert!( decodes_overlap_check( &[ ( 0, &a, 1, 0 ), ( 1, &b_far, 0, 12 ) ] ).is_ok(), "the decoded range starts where the other view's compressed bytes end" );

  let a_again = attributes_spec( 0, 0, 12, 4 );
  assert!( decodes_overlap_check( &[ ( 0, &a, 1, 0 ), ( 1, &a_again, 1, 16 ) ] ).is_ok(), "two views may read the same compressed bytes" );
}

/// The sweep in `decodes_overlap_check` agrees with the plain pairwise rule on many small
/// random layouts : a decoded range may overlap no other range, a compressed range may
/// overlap other compressed ranges. Ranges lie in two buffers, so both same-buffer and
/// cross-buffer pairs occur.
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

    let ranges = layout.iter().flat_map( | &( b, o, l, c, tb, to ) | [ ( ( tb, to, to + c * 4 ), true ), ( ( b, o, o + l ), false ) ] ).collect::< Vec< _ > >();
    let pairwise_ok = ranges.iter().enumerate().all( | ( i, &( a, a_decoded ) ) |
      ranges[ i + 1 .. ].iter().all( | &( b, b_decoded ) | !( a_decoded || b_decoded ) || !overlap( a, b ) ) );

    let swept = decodes_overlap_check( &placed );
    assert_eq!( swept.is_ok(), pairwise_ok, "layout {layout:?}: sweep says {swept:?}" );
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
  assert!( decodes_overlap_check( &placed ).is_ok() );
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

#[ test ]
#[ allow( clippy::float_cmp, reason = "each expected value is the same single f32 division the dequantization does ( or an exact 0, ±1 or pass-through literal ), so the results are bit-identical" ) ]
fn normalized_position_bounds_are_dequantized()
{
  use gltf::accessor::DataType;

  let gltf = fixture();
  let acc = gltf.accessors().nth( 2 ).unwrap();
  let raw = gltf::mesh::BoundingBox { min : [ -32767.0, -16384.0, 0.0 ], max : [ 32767.0, 16384.0, 32767.0 ] };

  let b = position_bounds_dequantize( raw, acc.data_type(), acc.normalized() );
  assert_eq!( b.min, [ -1.0, -16384.0 / 32767.0, 0.0 ] );
  assert_eq!( b.max, [ 1.0, 16384.0 / 32767.0, 1.0 ] );

  let one = | v : f32, t, n | position_bounds_dequantize( gltf::mesh::BoundingBox { min : [ v; 3 ], max : [ v; 3 ] }, t, n ).min[ 0 ];
  assert_eq!( one( -128.0, DataType::I8, true ), -1.0, "the signed minimum clamps to -1" );
  assert_eq!( one( 127.0, DataType::I8, true ), 1.0 );
  assert_eq!( one( 255.0, DataType::U8, true ), 1.0 );
  assert_eq!( one( 65535.0, DataType::U16, true ), 1.0 );
  assert_eq!( one( 300.0, DataType::I16, false ), 300.0, "non-normalized integers are already in shader units" );
  assert_eq!( one( 2.5, DataType::F32, false ), 2.5 );
}

/// One `VEC3` accessor per component type `accessor_vec3_read` matches on, plus a
/// `SCALAR` one. The `u8` and `u16` views pad each element with 0x7f to a 4- and an
/// 8-byte stride, so a reader that ignores `byteStride` picks the padding up.
const COMPONENT_TYPES_FIXTURE : &str = r#"
{
  "asset": { "version": "2.0" },
  "extensionsUsed": [ "KHR_mesh_quantization" ],
  "accessors":
  [
    { "bufferView": 0, "componentType": 5121, "normalized": true, "count": 2, "type": "VEC3" },
    { "bufferView": 0, "componentType": 5121, "count": 2, "type": "VEC3" },
    { "bufferView": 1, "componentType": 5123, "normalized": true, "count": 2, "type": "VEC3" },
    { "bufferView": 1, "componentType": 5123, "count": 2, "type": "VEC3" },
    { "bufferView": 2, "componentType": 5125, "count": 1, "type": "VEC3" },
    { "bufferView": 3, "componentType": 5126, "count": 1, "type": "VEC3" },
    { "bufferView": 4, "componentType": 5120, "normalized": true, "count": 1, "type": "VEC3" },
    { "bufferView": 5, "componentType": 5122, "normalized": true, "count": 1, "type": "VEC3" },
    { "bufferView": 3, "componentType": 5126, "count": 3, "type": "SCALAR" }
  ],
  "bufferViews":
  [
    { "buffer": 0, "byteOffset": 0, "byteLength": 8, "byteStride": 4 },
    { "buffer": 0, "byteOffset": 8, "byteLength": 16, "byteStride": 8 },
    { "buffer": 0, "byteOffset": 24, "byteLength": 12 },
    { "buffer": 0, "byteOffset": 36, "byteLength": 12 },
    { "buffer": 0, "byteOffset": 48, "byteLength": 3 },
    { "buffer": 0, "byteOffset": 52, "byteLength": 6 }
  ],
  "buffers": [ { "byteLength": 58, "uri": "placeholder.bin" } ]
}
"#;

/// `COMPONENT_TYPES_FIXTURE`'s bytes, laid out view by view.
fn component_types_buffer() -> Vec< u8 >
{
  let mut bytes = Vec::new();
  for v in [ [ 255u8, 0, 51 ], [ 0, 255, 102 ] ]
  {
    bytes.extend( v );
    bytes.push( 0x7f );
  }
  for v in [ [ 65535u16, 0, 13107 ], [ 0, 65535, 26214 ] ]
  {
    for c in v { bytes.extend( c.to_le_bytes() ); }
    bytes.extend( [ 0x7f, 0x7f ] );
  }
  for c in [ 1u32, 2, 70000 ] { bytes.extend( c.to_le_bytes() ); }
  for c in [ 0.5f32, -1.5, 2.0 ] { bytes.extend( c.to_le_bytes() ); }
  bytes.extend( [ -128i8, 127, 0 ].map( | c | c.to_le_bytes()[ 0 ] ) );
  bytes.push( 0 );
  for c in [ -32768i16, 32767, 16384 ] { bytes.extend( c.to_le_bytes() ); }
  assert_eq!( bytes.len(), 58 );
  bytes
}

/// Every integer arm of `accessor_vec3_read` reads its own type through a real accessor,
/// with the view's stride, and dequantizes only a normalized one. Morph targets may only
/// be `i8` / `i16` ( `gltf_skeleton_displacements_test.rs` reads those through
/// `skeleton_displacements_data_load` ), so `u8` / `u16` / `u32` are reached here directly.
#[ test ]
fn accessor_vec3_read_reads_every_component_type()
{
  use renderer::webgl::loaders::gltf_quantization::internal::accessor_vec3_read;

  let gltf = gltf::Gltf::from_slice_without_validation( COMPONENT_TYPES_FIXTURE.as_bytes() ).expect( "fixture is well-formed JSON" );
  let buffers = vec![ component_types_buffer() ];
  let read = | i : usize | accessor_vec3_read( gltf.accessors().nth( i ).expect( "fixture accessor" ), &buffers );
  let near = | actual : Vec< [ f32; 3 ] >, expected : &[ [ f32; 3 ] ], what : &str |
  {
    assert_eq!( actual.len(), expected.len(), "{what}: {actual:?}" );
    for ( a, e ) in actual.iter().zip( expected )
    {
      assert!( a.iter().zip( e ).all( | ( a, e ) | ( a - e ).abs() < 1e-6 ), "{what}: {actual:?} vs {expected:?}" );
    }
  };

  near( read( 0 ).expect( "u8" ), &[ [ 1.0, 0.0, 0.2 ], [ 0.0, 1.0, 0.4 ] ], "normalized u8" );
  near( read( 1 ).expect( "u8" ), &[ [ 255.0, 0.0, 51.0 ], [ 0.0, 255.0, 102.0 ] ], "plain u8" );
  near( read( 2 ).expect( "u16" ), &[ [ 1.0, 0.0, 0.2 ], [ 0.0, 1.0, 0.4 ] ], "normalized u16" );
  near( read( 3 ).expect( "u16" ), &[ [ 65535.0, 0.0, 13107.0 ], [ 0.0, 65535.0, 26214.0 ] ], "plain u16" );
  near( read( 4 ).expect( "u32" ), &[ [ 1.0, 2.0, 70000.0 ] ], "u32" );
  near( read( 5 ).expect( "f32" ), &[ [ 0.5, -1.5, 2.0 ] ], "f32" );
  near( read( 6 ).expect( "i8" ), &[ [ -1.0, 1.0, 0.0 ] ], "normalized i8, the minimum clamped to -1" );
  near( read( 7 ).expect( "i16" ), &[ [ -1.0, 1.0, 16384.0 / 32767.0 ] ], "normalized i16, the minimum clamped to -1" );
  assert!( read( 8 ).is_none(), "a SCALAR accessor is not read as VEC3" );
  assert!( accessor_vec3_read( gltf.accessors().next().unwrap(), &[] ).is_none(), "missing buffer data reads as None" );
}
