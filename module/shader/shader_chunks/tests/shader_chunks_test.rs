//! Direct-call tests for `shader_chunks`'s command logic — no
//! subprocess; see `tests/cli_subprocess_test.rs` for end-to-end argv and
//! exit-code coverage.

use shader_chunks::{ CliError, compose_chunks, get_chunk, list_chunks, list_tags, tree_chunk, try_compose_wgsl };

#[ test ]
fn list_chunks_lists_all_four_bundled_chunks_with_expected_columns()
{
  let output = list_chunks().expect( "list_chunks should not fail" );
  for name in [ "hash21", "value_noise", "fbm3", "fullscreen_triangle" ]
  {
    assert!( output.contains( name ), "list output missing chunk `{name}`:\n{output}" );
  }
  assert!( output.contains( "hash" ), "list output missing tag `hash`:\n{output}" );
}

#[ test ]
fn get_chunk_reports_full_detail_for_hash21()
{
  let output = get_chunk( "hash21" ).expect( "get_chunk should succeed for a real chunk" );
  assert!( output.contains( "name: hash21" ), "{output}" );
  assert!( output.contains( "description: Single-value hash of a 2D point into [0, 1)." ), "{output}" );
  assert!( output.contains( "stage: None" ), "{output}" );
  assert!( output.contains( "tags: category:hash" ), "{output}" );
  assert!( output.contains( "depends_on: (none)" ), "{output}" );
  assert!( output.contains( "fn hash21(p: vec2f) -> f32" ), "{output}" );
}

#[ test ]
fn get_chunk_reports_unknown_chunk_error_for_bogus_name()
{
  let err = get_chunk( "bogus_chunk" ).expect_err( "get_chunk should fail for an unknown name" );
  assert!
  (
    matches!( &err, CliError::UnknownChunk( name ) if name == "bogus_chunk" ),
    "expected UnknownChunk(\"bogus_chunk\"), got {err:?}"
  );
  assert_eq!( err.exit_code(), 1 );
}

#[ test ]
fn list_tags_lists_every_distinct_group_tag_pair_and_its_chunks()
{
  let output = list_tags().expect( "list_tags should not fail" );
  for pair in [ "category:hash", "category:noise", "technique:fractal", "category:vertex" ]
  {
    assert!( output.contains( pair ), "tags output missing `{pair}`:\n{output}" );
  }
  assert!( output.contains( "hash21" ), "{output}" );
  assert!( output.contains( "fbm3" ), "{output}" );
}

#[ test ]
fn tree_chunk_shows_fbm3_dependency_chain_in_order()
{
  let output = tree_chunk( Some( "fbm3" ) ).expect( "tree_chunk should succeed for a real chunk" );
  let fbm3_pos = output.find( "fbm3" ).expect( "fbm3 present" );
  let value_noise_pos = output.find( "value_noise" ).expect( "value_noise present" );
  let hash21_pos = output.find( "hash21" ).expect( "hash21 present" );
  assert!( fbm3_pos < value_noise_pos, "fbm3 should precede value_noise in the tree:\n{output}" );
  assert!( value_noise_pos < hash21_pos, "value_noise should precede hash21 in the tree:\n{output}" );
}

#[ test ]
fn tree_chunk_with_no_name_shows_forest_of_every_root_chunk()
{
  let output = tree_chunk( None ).expect( "tree_chunk should succeed with no name" );
  assert!( output.contains( "fbm3" ), "forest missing root `fbm3`:\n{output}" );
  assert!( output.contains( "fullscreen_triangle" ), "forest missing root `fullscreen_triangle`:\n{output}" );
}

#[ test ]
fn tree_chunk_reports_unknown_chunk_error_for_bogus_name()
{
  let err = tree_chunk( Some( "bogus_chunk" ) ).expect_err( "tree_chunk should fail for an unknown name" );
  assert!( matches!( err, CliError::UnknownChunk( _ ) ), "expected UnknownChunk, got {err:?}" );
}

#[ test ]
fn compose_chunks_orders_hash21_before_value_noise_regardless_of_input_order()
{
  let output = compose_chunks( &[ "value_noise".to_string(), "hash21".to_string() ] ).expect( "compose_chunks should succeed" );
  let hash21_pos = output.find( "fn hash21" ).expect( "hash21 present" );
  let value_noise_pos = output.find( "fn value_noise" ).expect( "value_noise present" );
  assert!( hash21_pos < value_noise_pos, "hash21 must precede value_noise:\n{output}" );
}

#[ test ]
fn compose_chunks_reports_unknown_chunk_error_for_bogus_name()
{
  let err = compose_chunks( &[ "bogus_chunk".to_string() ] ).expect_err( "compose_chunks should fail for an unknown name" );
  assert!
  (
    matches!( &err, CliError::UnknownChunk( name ) if name == "bogus_chunk" ),
    "expected UnknownChunk(\"bogus_chunk\"), got {err:?}"
  );
}

#[ test ]
fn compose_chunks_reports_missing_dependency_error_when_hash21_is_omitted()
{
  let err = compose_chunks( &[ "value_noise".to_string() ] ).expect_err( "compose_chunks should fail on a missing dependency" );
  assert!
  (
    matches!( &err, CliError::Compose( shader_chunks_core::ComposeError::MissingDependency { .. } ) ),
    "expected Compose(MissingDependency), got {err:?}"
  );
  assert_eq!( err.exit_code(), 1 );
}

#[ test ]
fn try_compose_wgsl_reports_cyclic_dependency_error_on_synthetic_fixture()
{
  const A : &str = "//@ name: a\n//@ depends_on: b\nfn a() {}";
  const B : &str = "//@ name: b\n//@ depends_on: a\nfn b() {}";
  let err = try_compose_wgsl( &[ A, B ] ).expect_err( "try_compose_wgsl should fail on a cyclic dependency" );
  assert!
  (
    matches!( &err, CliError::Compose( shader_chunks_core::ComposeError::CyclicDependency( _ ) ) ),
    "expected Compose(CyclicDependency), got {err:?}"
  );
}
