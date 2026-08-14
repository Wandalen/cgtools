//! Direct-call tests for `shader_chunks`'s command logic — no
//! subprocess; see `tests/cli_subprocess_test.rs` for end-to-end argv and
//! exit-code coverage. The `query_*` tests cover [`chunks_query`], the
//! single engine behind both `list` and `get`: selection, every filter,
//! projection, every output format, sorting, paging, and each loud error.

use shader_chunks::
{
  CliError, OutputFormat, QUERY_FIELDS, QueryParams, SortKey, SortOrder, TagsMode,
  chunks_compose, tags_list, chunks_query, chunk_tree, wgsl_try_compose, tunables, tunables_of_chunk,
};

/// Runs `params` with `format::names` forced and returns the matched chunk
/// names in output order — the projection/format-independent way to assert
/// on selection, filtering, sorting, and paging.
fn names_of( params : &QueryParams ) -> Vec< String >
{
  let mut params = params.clone();
  params.format = OutputFormat::Names;
  let output = chunks_query( &params ).expect( "names query should succeed" );
  output.lines().map( str::to_string ).collect()
}

#[ test ]
fn query_list_defaults_renders_all_four_chunks_as_plain_table()
{
  let output = chunks_query( &QueryParams::list_defaults() ).expect( "default list query should succeed" );
  for name in [ "hash21", "value_noise", "fbm3", "fullscreen_triangle" ]
  {
    assert!( output.contains( name ), "list output missing chunk `{name}`:\n{output}" );
  }
  for header in [ "name", "description", "tags", "depends_on" ]
  {
    assert!( output.contains( header ), "list output missing column `{header}`:\n{output}" );
  }
  assert!( !output.contains( "-[ RECORD" ), "list default must be a table, not expanded records:\n{output}" );
}

#[ test ]
fn query_get_defaults_renders_expanded_records_with_detail_fields()
{
  let mut params = QueryParams::get_defaults();
  params.names = vec![ "hash21".to_string() ];
  let output = chunks_query( &params ).expect( "get query should succeed" );
  assert!( output.contains( "-[ RECORD 1 ]" ), "{output}" );
  assert!( output.contains( "| hash21" ), "{output}" );
  assert!( output.contains( "stage" ), "{output}" );
  assert!( output.contains( "| (none)" ), "{output}" );
  assert!( output.contains( "fn hash21(p: vec2f) -> f32" ), "expanded detail must include exports:\n{output}" );
}

#[ test ]
fn query_names_selects_in_given_order_and_allows_duplicates()
{
  let mut params = QueryParams::list_defaults();
  params.names = vec![ "fbm3".to_string(), "hash21".to_string(), "fbm3".to_string() ];
  assert_eq!( names_of( &params ), [ "fbm3", "hash21", "fbm3" ] );
}

#[ test ]
fn query_unknown_name_reports_unknown_chunk_error()
{
  let mut params = QueryParams::list_defaults();
  params.names = vec![ "bogus_chunk".to_string() ];
  let err = chunks_query( &params ).expect_err( "unknown chunk name should fail" );
  assert!
  (
    matches!( &err, CliError::UnknownChunk( name ) if name == "bogus_chunk" ),
    "expected UnknownChunk(\"bogus_chunk\"), got {err:?}"
  );
  assert_eq!( err.exit_code(), 1 );
}

#[ test ]
fn query_pattern_matches_case_insensitively_by_default()
{
  let mut params = QueryParams::list_defaults();
  params.pattern = "NOISE".to_string();
  assert_eq!( names_of( &params ), [ "value_noise" ] );
}

#[ test ]
fn query_pattern_with_case_switch_demands_exact_case()
{
  let mut params = QueryParams::list_defaults();
  params.pattern = "NOISE".to_string();
  params.case_sensitive = true;
  assert!( names_of( &params ).is_empty() );
}

#[ test ]
fn query_bare_tag_selector_matches_the_tag_under_any_group()
{
  let mut params = QueryParams::list_defaults();
  params.tags = vec![ "noise".to_string() ];
  assert_eq!( names_of( &params ), [ "value_noise", "fbm3" ] );
}

#[ test ]
fn query_pair_tag_selector_demands_the_exact_group()
{
  let mut params = QueryParams::list_defaults();
  params.tags = vec![ "category:noise".to_string() ];
  assert_eq!( names_of( &params ), [ "value_noise", "fbm3" ] );

  // `fractal` exists only under `technique:` — the `category:` pair must not match.
  params.tags = vec![ "category:fractal".to_string() ];
  assert!( names_of( &params ).is_empty() );
}

#[ test ]
fn query_tags_mode_any_unions_and_all_intersects_selectors()
{
  let mut params = QueryParams::list_defaults();
  params.tags = vec![ "noise".to_string(), "hash".to_string() ];
  params.tags_mode = TagsMode::Any;
  assert_eq!( names_of( &params ), [ "hash21", "value_noise", "fbm3" ] );

  params.tags_mode = TagsMode::All;
  assert!( names_of( &params ).is_empty() );

  // fbm3 carries both category:noise and technique:fractal.
  params.tags = vec![ "noise".to_string(), "fractal".to_string() ];
  assert_eq!( names_of( &params ), [ "fbm3" ] );
}

#[ test ]
fn query_stage_filter_selects_none_literal_and_any()
{
  let mut params = QueryParams::list_defaults();
  params.stage = "none".to_string();
  assert_eq!( names_of( &params ), [ "hash21", "value_noise", "fbm3" ] );

  params.stage = "vertex".to_string();
  assert_eq!( names_of( &params ), [ "fullscreen_triangle" ] );

  params.stage = "fragment".to_string();
  assert!( names_of( &params ).is_empty() );
}

#[ test ]
fn query_depends_on_selects_direct_dependents_and_transitive_widens()
{
  let mut params = QueryParams::list_defaults();
  params.depends_on = "hash21".to_string();
  assert_eq!( names_of( &params ), [ "value_noise" ] );

  params.transitive = true;
  assert_eq!( names_of( &params ), [ "value_noise", "fbm3" ] );
}

#[ test ]
fn query_depends_on_unknown_chunk_fails_loudly()
{
  let mut params = QueryParams::list_defaults();
  params.depends_on = "bogus_chunk".to_string();
  let err = chunks_query( &params ).expect_err( "unknown depends_on target should fail" );
  assert!( matches!( err, CliError::UnknownChunk( _ ) ), "expected UnknownChunk, got {err:?}" );
}

#[ test ]
fn query_roots_and_leaves_select_graph_extremes()
{
  let mut params = QueryParams::list_defaults();
  params.roots = true;
  assert_eq!( names_of( &params ), [ "fbm3", "fullscreen_triangle" ] );

  let mut params = QueryParams::list_defaults();
  params.leaves = true;
  assert_eq!( names_of( &params ), [ "hash21", "fullscreen_triangle" ] );
}

#[ test ]
fn query_exports_filter_matches_signatures_with_case_switch()
{
  let mut params = QueryParams::list_defaults();
  params.exports = "FN HASH21".to_string();
  assert_eq!( names_of( &params ), [ "hash21" ] );

  params.case_sensitive = true;
  assert!( names_of( &params ).is_empty() );
}

#[ test ]
fn query_count_reports_filtered_total_before_paging()
{
  let mut params = QueryParams::list_defaults();
  params.count = true;
  params.limit = 1;
  assert_eq!( chunks_query( &params ).expect( "count query should succeed" ), "4" );

  params.pattern = "noise".to_string();
  assert_eq!( chunks_query( &params ).expect( "filtered count should succeed" ), "1" );
}

#[ test ]
fn query_fields_projects_only_the_named_columns()
{
  let mut params = QueryParams::get_defaults();
  params.names = vec![ "hash21".to_string() ];
  params.fields = vec![ "name".to_string() ];
  let output = chunks_query( &params ).expect( "projection query should succeed" );
  assert!( output.contains( "| hash21" ), "{output}" );
  assert!( !output.contains( "description" ), "projection must drop unrequested fields:\n{output}" );
}

#[ test ]
fn query_every_declared_field_renders_including_source()
{
  let mut params = QueryParams::get_defaults();
  params.names = vec![ "hash21".to_string() ];
  params.fields = QUERY_FIELDS.iter().map( | field | ( *field ).to_string() ).collect();
  let output = chunks_query( &params ).expect( "all-fields query should succeed" );
  for field in QUERY_FIELDS
  {
    assert!( output.contains( field ), "all-fields output missing `{field}`:\n{output}" );
  }
  assert!( output.contains( "fn hash21" ), "source field must carry WGSL:\n{output}" );
}

#[ test ]
fn query_unknown_field_fails_loudly()
{
  let mut params = QueryParams::list_defaults();
  params.fields = vec![ "bogus".to_string() ];
  let err = chunks_query( &params ).expect_err( "unknown field should fail" );
  assert!
  (
    matches!( &err, CliError::UnknownField( field ) if field == "bogus" ),
    "expected UnknownField(\"bogus\"), got {err:?}"
  );
  assert_eq!( err.exit_code(), 1 );
}

#[ test ]
fn query_json_and_yaml_formats_carry_row_content()
{
  let mut params = QueryParams::list_defaults();
  params.names = vec![ "hash21".to_string() ];
  params.fields = vec![ "name".to_string(), "depends_on".to_string() ];

  params.format = OutputFormat::Json;
  let json = chunks_query( &params ).expect( "json query should succeed" );
  assert!( json.contains( "\"name\": \"hash21\"" ), "{json}" );
  assert!( json.contains( "\"depends_on\": \"(none)\"" ), "{json}" );

  params.format = OutputFormat::Yaml;
  let yaml = chunks_query( &params ).expect( "yaml query should succeed" );
  assert!( yaml.contains( "name: hash21" ), "{yaml}" );
  assert!( yaml.contains( "depends_on: (none)" ), "{yaml}" );
}

#[ test ]
fn query_markdown_format_renders_pipe_table_with_heading_and_width()
{
  let mut params = QueryParams::list_defaults();
  params.format = OutputFormat::Markdown;
  params.heading = "Chunks".to_string();
  params.width = 30;
  let output = chunks_query( &params ).expect( "markdown query should succeed" );
  assert!( output.contains( "| name" ), "{output}" );
  assert!( output.contains( "|---" ), "{output}" );
  assert!( output.contains( "Chunks" ), "heading line missing:\n{output}" );
  assert!( output.contains( "..." ), "width::30 must truncate long descriptions:\n{output}" );
}

#[ test ]
fn query_names_format_ignores_fields_projection()
{
  let mut params = QueryParams::list_defaults();
  params.names = vec![ "fbm3".to_string() ];
  params.fields = vec![ "source".to_string() ];
  params.format = OutputFormat::Names;
  assert_eq!( chunks_query( &params ).expect( "names query should succeed" ), "fbm3" );
}

#[ test ]
fn query_sort_keys_order_deterministically()
{
  let mut params = QueryParams::list_defaults();
  params.sort = SortKey::Name;
  assert_eq!( names_of( &params ), [ "fbm3", "fullscreen_triangle", "hash21", "value_noise" ] );

  // Stage-less chunks sort first (empty stage key), name-tiebroken.
  params.sort = SortKey::Stage;
  assert_eq!( names_of( &params ), [ "fbm3", "hash21", "value_noise", "fullscreen_triangle" ] );

  params.sort = SortKey::Description;
  assert_eq!( names_of( &params ), [ "value_noise", "fbm3", "fullscreen_triangle", "hash21" ] );
}

#[ test ]
fn query_order_desc_reverses_including_input_order()
{
  let mut params = QueryParams::list_defaults();
  params.sort = SortKey::Name;
  params.order = SortOrder::Desc;
  assert_eq!( names_of( &params ), [ "value_noise", "hash21", "fullscreen_triangle", "fbm3" ] );

  params.sort = SortKey::Input;
  assert_eq!( names_of( &params ), [ "fullscreen_triangle", "fbm3", "value_noise", "hash21" ] );
}

#[ test ]
fn query_offset_and_limit_page_the_result()
{
  let mut params = QueryParams::list_defaults();
  params.offset = 1;
  params.limit = 2;
  assert_eq!( names_of( &params ), [ "value_noise", "fbm3" ] );

  params.offset = 9;
  params.limit = 0;
  assert!( names_of( &params ).is_empty() );
}

#[ test ]
fn query_enum_params_round_trip_and_reject_bogus_values()
{
  for mode in [ TagsMode::Any, TagsMode::All ]
  {
    assert_eq!( mode.as_str().parse::< TagsMode >().expect( "round trip" ), mode );
  }
  for key in [ SortKey::Input, SortKey::Name, SortKey::Stage, SortKey::Description ]
  {
    assert_eq!( key.as_str().parse::< SortKey >().expect( "round trip" ), key );
  }
  for order in [ SortOrder::Asc, SortOrder::Desc ]
  {
    assert_eq!( order.as_str().parse::< SortOrder >().expect( "round trip" ), order );
  }
  for format in
  [
    OutputFormat::Table, OutputFormat::Markdown, OutputFormat::Expanded,
    OutputFormat::Json, OutputFormat::Yaml, OutputFormat::Names,
  ]
  {
    assert_eq!( format.as_str().parse::< OutputFormat >().expect( "round trip" ), format );
  }

  let err = "bogus".parse::< OutputFormat >().expect_err( "bogus format must fail" );
  assert!
  (
    matches!( &err, CliError::InvalidParam { param : "format", .. } ),
    "expected InvalidParam for format, got {err:?}"
  );
  assert_eq!( err.exit_code(), 1 );
  assert!( "bogus".parse::< SortKey >().is_err() );
  assert!( "bogus".parse::< SortOrder >().is_err() );
  assert!( "bogus".parse::< TagsMode >().is_err() );
}

#[ test ]
fn query_list_and_get_defaults_share_engine_and_agree_under_equal_params()
{
  // The unification contract: with identical explicit parameters, the two
  // defaults structs are indistinguishable — same engine, same output.
  let mut list_params = QueryParams::list_defaults();
  let mut get_params = QueryParams::get_defaults();
  for params in [ &mut list_params, &mut get_params ]
  {
    params.names = vec![ "hash21".to_string() ];
    params.fields = vec![ "name".to_string(), "stage".to_string() ];
    params.format = OutputFormat::Expanded;
  }
  assert_eq!
  (
    chunks_query( &list_params ).expect( "list-defaults query should succeed" ),
    chunks_query( &get_params ).expect( "get-defaults query should succeed" ),
  );
}

#[ test ]
fn list_tags_lists_every_distinct_group_tag_pair_and_its_chunks()
{
  let output = tags_list().expect( "tags_list should not fail" );
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
  let output = chunk_tree( Some( "fbm3" ) ).expect( "chunk_tree should succeed for a real chunk" );
  let fbm3_pos = output.find( "fbm3" ).expect( "fbm3 present" );
  let value_noise_pos = output.find( "value_noise" ).expect( "value_noise present" );
  let hash21_pos = output.find( "hash21" ).expect( "hash21 present" );
  assert!( fbm3_pos < value_noise_pos, "fbm3 should precede value_noise in the tree:\n{output}" );
  assert!( value_noise_pos < hash21_pos, "value_noise should precede hash21 in the tree:\n{output}" );
}

#[ test ]
fn tree_chunk_with_no_name_shows_forest_of_every_root_chunk()
{
  let output = chunk_tree( None ).expect( "chunk_tree should succeed with no name" );
  assert!( output.contains( "fbm3" ), "forest missing root `fbm3`:\n{output}" );
  assert!( output.contains( "fullscreen_triangle" ), "forest missing root `fullscreen_triangle`:\n{output}" );
}

#[ test ]
fn tree_chunk_reports_unknown_chunk_error_for_bogus_name()
{
  let err = chunk_tree( Some( "bogus_chunk" ) ).expect_err( "chunk_tree should fail for an unknown name" );
  assert!( matches!( err, CliError::UnknownChunk( _ ) ), "expected UnknownChunk, got {err:?}" );
}

#[ test ]
fn compose_chunks_orders_hash21_before_value_noise_regardless_of_input_order()
{
  let output = chunks_compose( &[ "value_noise".to_string(), "hash21".to_string() ], false ).expect( "chunks_compose should succeed" );
  let hash21_pos = output.find( "fn hash21" ).expect( "hash21 present" );
  let value_noise_pos = output.find( "fn value_noise" ).expect( "value_noise present" );
  assert!( hash21_pos < value_noise_pos, "hash21 must precede value_noise:\n{output}" );
}

#[ test ]
fn compose_chunks_reports_unknown_chunk_error_for_bogus_name()
{
  let err = chunks_compose( &[ "bogus_chunk".to_string() ], false ).expect_err( "chunks_compose should fail for an unknown name" );
  assert!
  (
    matches!( &err, CliError::UnknownChunk( name ) if name == "bogus_chunk" ),
    "expected UnknownChunk(\"bogus_chunk\"), got {err:?}"
  );
}

#[ test ]
fn compose_chunks_reports_missing_dependency_error_when_hash21_is_omitted()
{
  let err = chunks_compose( &[ "value_noise".to_string() ], false ).expect_err( "chunks_compose should fail on a missing dependency" );
  assert!
  (
    matches!( &err, CliError::Compose( shader_chunks_core::ComposeError::MissingDependency { .. } ) ),
    "expected Compose(MissingDependency), got {err:?}"
  );
  assert_eq!( err.exit_code(), 1 );
}

#[ test ]
fn compose_chunks_transitive_closure_equals_the_explicit_full_set()
{
  let closure = chunks_compose( &[ "fbm3".to_string() ], true )
  .expect( "transitive compose of a single root should pull its whole chain" );
  let explicit = chunks_compose
  (
    &[ "hash21".to_string(), "value_noise".to_string(), "fbm3".to_string() ],
    false,
  ).expect( "explicit full set should compose" );
  assert_eq!( closure, explicit, "closure and explicit full set must compose identically" );

  let hash21_pos = closure.find( "fn hash21" ).expect( "hash21 pulled in transitively" );
  let value_noise_pos = closure.find( "fn value_noise" ).expect( "value_noise pulled in transitively" );
  let fbm3_pos = closure.find( "fn fbm3" ).expect( "fbm3 present" );
  assert!
  (
    hash21_pos < value_noise_pos && value_noise_pos < fbm3_pos,
    "closure must compose in dependency order:\n{closure}"
  );
}

#[ test ]
fn compose_chunks_transitive_reports_unknown_chunk_error_for_bogus_name()
{
  // The closure walk resolves every reachable dependency through the same
  // loud lookup as directly-named chunks — a bogus root fails identically
  // under both modes rather than the transitive path masking it.
  let err = chunks_compose( &[ "bogus_chunk".to_string() ], true )
  .expect_err( "transitive compose should fail for an unknown name" );
  assert!
  (
    matches!( &err, CliError::UnknownChunk( name ) if name == "bogus_chunk" ),
    "expected UnknownChunk(\"bogus_chunk\"), got {err:?}"
  );
}

#[ test ]
fn try_compose_wgsl_reports_cyclic_dependency_error_on_synthetic_fixture()
{
  const A : &str = "//@ name: a\n//@ depends_on: b\nfn a() {}";
  const B : &str = "//@ name: b\n//@ depends_on: a\nfn b() {}";
  let err = wgsl_try_compose( &[ A, B ] ).expect_err( "wgsl_try_compose should fail on a cyclic dependency" );
  assert!
  (
    matches!( &err, CliError::Compose( shader_chunks_core::ComposeError::CyclicDependency( _ ) ) ),
    "expected Compose(CyclicDependency), got {err:?}"
  );
}

/// Mirrors `shader_chunks_params/tests/discovery_test.rs`'s own `LOCAL_GLOW`
/// fixture — a test-local chunk carrying `//@ param:` lines, since none of
/// the 4 bundled chunks declare any (out of scope for this task to change).
const LOCAL_GLOW_WGSL : &str = "\
//@ name: glow
//@ description: Doubled value noise, a test-local chunk.
//@ tags: category:test
//@ depends_on: value_noise
//@ export: fn glow(p: vec2f) -> f32
//@ param: octaves argument u32 range(1, 8)
//@ param: seed define u32

fn glow( p : vec2f, octaves : u32, seed : u32 ) -> f32
{
  return value_noise( p ) * 2.0;
}
";

const LOCAL_GLOW : shader_chunks_core::ChunkDescriptor = shader_chunks_core::ChunkDescriptor
{
  name : "glow",
  description : "Doubled value noise, a test-local chunk.",
  tags : &[ ( "category", "test" ) ],
  stage : None,
  depends_on : &[ "value_noise" ],
  exports : &[ "fn glow(p: vec2f) -> f32" ],
  wgsl : LOCAL_GLOW_WGSL,
};

#[ test ]
fn tunables_of_chunk_lists_declared_and_inferred_parameters()
{
  let output = tunables_of_chunk( &LOCAL_GLOW ).expect( "tunables_of_chunk should succeed" );

  assert!( output.contains( "octaves" ), "{output}" );
  assert!( output.contains( "Argument" ), "{output}" );
  assert!( output.contains( "U32" ), "{output}" );
  assert!( output.contains( "1..8" ), "declared range should render verbatim:\n{output}" );
  assert!( output.contains( "Declared" ), "{output}" );

  assert!( output.contains( "seed" ), "{output}" );
  assert!( output.contains( "Define" ), "{output}" );
  assert!( output.contains( "0..65535" ), "inferred range for `seed` should be [0, 65535]:\n{output}" );
  assert!( output.contains( "Inferred" ), "{output}" );
}

#[ test ]
fn tunables_zero_declared_params_reports_explicit_message_not_blank_or_error()
{
  let output = tunables( "hash21" ).expect( "tunables should succeed for a bundled chunk with no declared params" );
  assert!( output.contains( "hash21" ), "{output}" );
  assert!( output.contains( "no tunable parameters" ), "empty case must be an explicit message, not blank:\n{output}" );
}

#[ test ]
fn tunables_unknown_chunk_reports_unknown_chunk_error()
{
  let err = tunables( "bogus_chunk" ).expect_err( "tunables should fail for an unknown chunk name" );
  assert!
  (
    matches!( &err, CliError::UnknownChunk( name ) if name == "bogus_chunk" ),
    "expected UnknownChunk(\"bogus_chunk\"), got {err:?}"
  );
  assert_eq!( err.exit_code(), 1 );
}
