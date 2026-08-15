//! Direct-call tests for `shader_chunks_query_core`'s query engine — no
//! subprocess; see `tests/cli_subprocess_test.rs` in the aggregator for
//! end-to-end argv and exit-code coverage. The `query_*` tests cover
//! [`chunks_query`], the single engine behind both `list` and `get`:
//! selection, every filter, projection, every output format, sorting,
//! paging, and each loud error. `tree_*` and `list_tags_*` cover the graph
//! and tag-listing engines.

use shader_chunks_query_core::
{
  QueryError, OutputFormat, QUERY_FIELDS, QueryParams, SortKey, SortOrder, TagsMode,
  tags_list, chunks_query, chunk_tree,
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
fn query_list_defaults_renders_every_chunk_as_plain_table()
{
  let output = chunks_query( &QueryParams::list_defaults() ).expect( "default list query should succeed" );
  for name in
  [
    "hash21", "value_noise", "fbm3", "fullscreen_triangle",
    "hash22", "hash13", "hash33", "value_noise3", "gradient_noise", "voronoi", "domain_warp",
    "d2_sdf_circle", "d2_sdf_ring", "d3_sdf_sphere", "sdf_op_union",
    "glow", "aa_step", "rot2", "palette_cosine", "srgb", "tonemap_aces", "gaussian_weight",
  ]
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
    matches!( &err, QueryError::UnknownChunk( name ) if name == "bogus_chunk" ),
    "expected UnknownChunk(\"bogus_chunk\"), got {err:?}"
  );
  assert_eq!( err.exit_code(), 1 );
}

#[ test ]
fn query_pattern_matches_case_insensitively_by_default()
{
  let mut params = QueryParams::list_defaults();
  params.pattern = "NOISE".to_string();
  assert_eq!( names_of( &params ), [ "value_noise", "value_noise3", "gradient_noise" ] );
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
  assert_eq!( names_of( &params ), [ "value_noise", "fbm3", "value_noise3", "gradient_noise", "voronoi", "domain_warp" ] );
}

#[ test ]
fn query_pair_tag_selector_demands_the_exact_group()
{
  let mut params = QueryParams::list_defaults();
  params.tags = vec![ "category:noise".to_string() ];
  assert_eq!( names_of( &params ), [ "value_noise", "fbm3", "value_noise3", "gradient_noise", "voronoi", "domain_warp" ] );

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
  assert_eq!
  (
    names_of( &params ),
    [ "hash21", "value_noise", "fbm3", "hash22", "hash13", "hash33", "value_noise3", "gradient_noise", "voronoi", "domain_warp" ]
  );

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
  assert_eq!
  (
    names_of( &params ),
    [
      "hash21", "value_noise", "fbm3",
      "hash22", "hash13", "hash33", "value_noise3", "gradient_noise", "voronoi", "domain_warp",
      "d2_sdf_circle", "d2_sdf_ring", "d2_sdf_box", "d2_sdf_round_box", "d2_sdf_segment",
      "d2_sdf_equilateral_triangle", "d2_sdf_hexagon", "d2_sdf_arc", "d2_sdf_pie", "d2_sdf_vesica",
      "d2_sdf_star5", "d2_sdf_cross",
      "d3_sdf_sphere", "d3_sdf_box", "d3_sdf_round_box", "d3_sdf_torus", "d3_sdf_capsule",
      "d3_sdf_capped_cylinder", "d3_sdf_capped_cone", "d3_sdf_plane", "d3_sdf_octahedron",
      "d3_sdf_ellipsoid", "d3_sdf_hex_prism", "d3_sdf_round_cone",
      "sdf_op_union", "sdf_op_subtract", "sdf_op_intersect", "sdf_op_union_smooth",
      "sdf_op_subtract_smooth", "sdf_op_intersect_smooth", "sdf_op_round", "sdf_op_onion",
      "glow", "aa_step", "rot2", "palette_cosine", "srgb", "tonemap_aces", "gaussian_weight",
    ]
  );

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
  assert_eq!( names_of( &params ), [ "value_noise", "fbm3", "domain_warp" ] );
}

#[ test ]
fn query_depends_on_unknown_chunk_fails_loudly()
{
  let mut params = QueryParams::list_defaults();
  params.depends_on = "bogus_chunk".to_string();
  let err = chunks_query( &params ).expect_err( "unknown depends_on target should fail" );
  assert!( matches!( err, QueryError::UnknownChunk( _ ) ), "expected UnknownChunk, got {err:?}" );
}

#[ test ]
fn query_roots_and_leaves_select_graph_extremes()
{
  let mut params = QueryParams::list_defaults();
  params.roots = true;
  assert_eq!
  (
    names_of( &params ),
    [
      "fullscreen_triangle", "hash33", "value_noise3", "gradient_noise", "voronoi", "domain_warp",
      "d2_sdf_ring", "d2_sdf_round_box", "d2_sdf_segment", "d2_sdf_equilateral_triangle",
      "d2_sdf_hexagon", "d2_sdf_arc", "d2_sdf_pie", "d2_sdf_vesica", "d2_sdf_star5", "d2_sdf_cross",
      "d3_sdf_sphere", "d3_sdf_round_box", "d3_sdf_torus", "d3_sdf_capsule", "d3_sdf_capped_cylinder",
      "d3_sdf_capped_cone", "d3_sdf_plane", "d3_sdf_octahedron", "d3_sdf_ellipsoid", "d3_sdf_hex_prism",
      "d3_sdf_round_cone",
      "sdf_op_union", "sdf_op_subtract", "sdf_op_intersect", "sdf_op_union_smooth",
      "sdf_op_subtract_smooth", "sdf_op_intersect_smooth", "sdf_op_round", "sdf_op_onion",
      "glow", "aa_step", "rot2", "palette_cosine", "srgb", "tonemap_aces", "gaussian_weight",
    ]
  );

  // `leaves` (unlike `roots`) is keyed on each chunk's OWN depends_on being
  // empty, not on whether anything else depends on it — so it excludes the
  // 2 round_box variants and the 8 sdf_op_* operators (all compose other
  // primitives) even though those are still `roots`, and it still includes
  // hash21/hash22/hash13/d2_sdf_box/d3_sdf_box/d2_sdf_circle (each depended
  // on by something, but zero-dep themselves).
  let mut params = QueryParams::list_defaults();
  params.leaves = true;
  assert_eq!
  (
    names_of( &params ),
    [
      "hash21", "fullscreen_triangle", "hash22", "hash13", "hash33",
      "d2_sdf_circle", "d2_sdf_ring", "d2_sdf_box", "d2_sdf_segment", "d2_sdf_equilateral_triangle",
      "d2_sdf_hexagon", "d2_sdf_arc", "d2_sdf_pie", "d2_sdf_vesica", "d2_sdf_star5", "d2_sdf_cross",
      "d3_sdf_sphere", "d3_sdf_box", "d3_sdf_torus", "d3_sdf_capsule", "d3_sdf_capped_cylinder",
      "d3_sdf_capped_cone", "d3_sdf_plane", "d3_sdf_octahedron", "d3_sdf_ellipsoid", "d3_sdf_hex_prism",
      "d3_sdf_round_cone",
      "glow", "aa_step", "rot2", "palette_cosine", "srgb", "tonemap_aces", "gaussian_weight",
    ]
  );
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
  assert_eq!( chunks_query( &params ).expect( "count query should succeed" ), "50" );

  params.pattern = "noise".to_string();
  assert_eq!( chunks_query( &params ).expect( "filtered count should succeed" ), "3" );
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
    matches!( &err, QueryError::UnknownField( field ) if field == "bogus" ),
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
  assert_eq!
  (
    names_of( &params ),
    [
      "aa_step", "d2_sdf_arc", "d2_sdf_box", "d2_sdf_circle", "d2_sdf_cross", "d2_sdf_equilateral_triangle",
      "d2_sdf_hexagon", "d2_sdf_pie", "d2_sdf_ring", "d2_sdf_round_box", "d2_sdf_segment", "d2_sdf_star5",
      "d2_sdf_vesica", "d3_sdf_box", "d3_sdf_capped_cone", "d3_sdf_capped_cylinder", "d3_sdf_capsule",
      "d3_sdf_ellipsoid", "d3_sdf_hex_prism", "d3_sdf_octahedron", "d3_sdf_plane", "d3_sdf_round_box",
      "d3_sdf_round_cone", "d3_sdf_sphere", "d3_sdf_torus", "domain_warp", "fbm3", "fullscreen_triangle",
      "gaussian_weight", "glow", "gradient_noise", "hash13", "hash21", "hash22", "hash33", "palette_cosine",
      "rot2", "sdf_op_intersect", "sdf_op_intersect_smooth", "sdf_op_onion", "sdf_op_round", "sdf_op_subtract",
      "sdf_op_subtract_smooth", "sdf_op_union", "sdf_op_union_smooth", "srgb", "tonemap_aces", "value_noise",
      "value_noise3", "voronoi",
    ]
  );

  // Stage-less chunks sort first (empty stage key), name-tiebroken — every
  // new SDF chunk is stage-less, so only fullscreen_triangle (the sole
  // vertex-stage chunk) moves to the end.
  params.sort = SortKey::Stage;
  assert_eq!
  (
    names_of( &params ),
    [
      "aa_step", "d2_sdf_arc", "d2_sdf_box", "d2_sdf_circle", "d2_sdf_cross", "d2_sdf_equilateral_triangle",
      "d2_sdf_hexagon", "d2_sdf_pie", "d2_sdf_ring", "d2_sdf_round_box", "d2_sdf_segment", "d2_sdf_star5",
      "d2_sdf_vesica", "d3_sdf_box", "d3_sdf_capped_cone", "d3_sdf_capped_cylinder", "d3_sdf_capsule",
      "d3_sdf_ellipsoid", "d3_sdf_hex_prism", "d3_sdf_octahedron", "d3_sdf_plane", "d3_sdf_round_box",
      "d3_sdf_round_cone", "d3_sdf_sphere", "d3_sdf_torus", "domain_warp", "fbm3",
      "gaussian_weight", "glow", "gradient_noise", "hash13", "hash21", "hash22", "hash33", "palette_cosine",
      "rot2", "sdf_op_intersect", "sdf_op_intersect_smooth", "sdf_op_onion", "sdf_op_round", "sdf_op_subtract",
      "sdf_op_subtract_smooth", "sdf_op_union", "sdf_op_union_smooth", "srgb", "tonemap_aces", "value_noise",
      "value_noise3", "voronoi", "fullscreen_triangle",
    ]
  );

  // Byte-wise over descriptions — real engine output, not hand-derived;
  // see this test's own doc comment convention (regenerate via a failing
  // assert_eq! rather than re-sorting by hand if this list ever needs to
  // change again).
  params.sort = SortKey::Description;
  assert_eq!
  (
    names_of( &params ),
    [
      "rot2", "tonemap_aces", "glow", "aa_step", "value_noise", "voronoi", "palette_cosine", "fbm3",
      "fullscreen_triangle", "srgb", "gradient_noise", "sdf_op_round", "sdf_op_intersect", "sdf_op_subtract",
      "sdf_op_union", "d3_sdf_ellipsoid", "d2_sdf_star5", "d2_sdf_round_box", "d2_sdf_circle", "d2_sdf_pie",
      "d2_sdf_cross", "d2_sdf_hexagon", "d2_sdf_vesica", "d2_sdf_box", "d2_sdf_equilateral_triangle",
      "d3_sdf_round_box", "d3_sdf_capped_cone", "d3_sdf_capsule", "d3_sdf_capped_cylinder", "d3_sdf_hex_prism",
      "d3_sdf_round_cone", "d3_sdf_sphere", "d3_sdf_torus", "d3_sdf_box", "d3_sdf_plane", "d3_sdf_octahedron",
      "hash21", "hash13", "sdf_op_intersect_smooth", "sdf_op_subtract_smooth", "sdf_op_union_smooth", "hash33",
      "value_noise3", "sdf_op_onion", "hash22", "gaussian_weight", "d2_sdf_ring", "d2_sdf_arc", "d2_sdf_segment",
      "domain_warp",
    ]
  );
}

#[ test ]
fn query_order_desc_reverses_including_input_order()
{
  let mut params = QueryParams::list_defaults();
  params.sort = SortKey::Name;
  params.order = SortOrder::Desc;
  assert_eq!
  (
    names_of( &params ),
    [
      "voronoi", "value_noise3", "value_noise", "tonemap_aces", "srgb", "sdf_op_union_smooth", "sdf_op_union",
      "sdf_op_subtract_smooth", "sdf_op_subtract", "sdf_op_round", "sdf_op_onion", "sdf_op_intersect_smooth",
      "sdf_op_intersect", "rot2", "palette_cosine", "hash33", "hash22", "hash21", "hash13", "gradient_noise",
      "glow", "gaussian_weight", "fullscreen_triangle", "fbm3", "domain_warp",
      "d3_sdf_torus", "d3_sdf_sphere", "d3_sdf_round_cone", "d3_sdf_round_box", "d3_sdf_plane",
      "d3_sdf_octahedron", "d3_sdf_hex_prism", "d3_sdf_ellipsoid", "d3_sdf_capsule", "d3_sdf_capped_cylinder",
      "d3_sdf_capped_cone", "d3_sdf_box",
      "d2_sdf_vesica", "d2_sdf_star5", "d2_sdf_segment", "d2_sdf_round_box", "d2_sdf_ring", "d2_sdf_pie",
      "d2_sdf_hexagon", "d2_sdf_equilateral_triangle", "d2_sdf_cross", "d2_sdf_circle", "d2_sdf_box",
      "d2_sdf_arc", "aa_step",
    ]
  );

  // Exact reverse of registry order ( shader/readme.md's collection-index
  // table — load-bearing for build.rs, so this ordering is authoritative ).
  params.sort = SortKey::Input;
  assert_eq!
  (
    names_of( &params ),
    [
      "gaussian_weight", "tonemap_aces", "srgb", "palette_cosine", "rot2", "aa_step", "glow",
      "sdf_op_onion", "sdf_op_round", "sdf_op_intersect_smooth", "sdf_op_subtract_smooth", "sdf_op_union_smooth",
      "sdf_op_intersect", "sdf_op_subtract", "sdf_op_union",
      "d3_sdf_round_cone", "d3_sdf_hex_prism", "d3_sdf_ellipsoid", "d3_sdf_octahedron", "d3_sdf_plane",
      "d3_sdf_capped_cone", "d3_sdf_capped_cylinder", "d3_sdf_capsule", "d3_sdf_torus", "d3_sdf_round_box",
      "d3_sdf_box", "d3_sdf_sphere",
      "d2_sdf_cross", "d2_sdf_star5", "d2_sdf_vesica", "d2_sdf_pie", "d2_sdf_arc", "d2_sdf_hexagon",
      "d2_sdf_equilateral_triangle", "d2_sdf_segment", "d2_sdf_round_box", "d2_sdf_box", "d2_sdf_ring",
      "d2_sdf_circle",
      "domain_warp", "voronoi", "gradient_noise", "value_noise3", "hash33", "hash13", "hash22",
      "fullscreen_triangle", "fbm3", "value_noise", "hash21",
    ]
  );
}

#[ test ]
fn query_offset_and_limit_page_the_result()
{
  let mut params = QueryParams::list_defaults();
  params.offset = 1;
  params.limit = 2;
  assert_eq!( names_of( &params ), [ "value_noise", "fbm3" ] );

  // limit::0 means "unlimited" ( see chunks_query's paging step ), so this
  // only exercises offset overrunning the total chunk count — bump past it.
  params.offset = 100;
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
    matches!( &err, QueryError::InvalidParam { param : "format", .. } ),
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
  for pair in
  [
    "category:hash", "category:noise", "technique:fractal", "category:vertex",
    "technique:gradient", "technique:cellular", "technique:warp",
    "category:sdf", "category:shading", "category:antialiasing", "category:transform",
    "category:color", "category:filter",
  ]
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
  assert!( output.contains( "domain_warp" ), "forest missing root `domain_warp`:\n{output}" );
  assert!( output.contains( "fullscreen_triangle" ), "forest missing root `fullscreen_triangle`:\n{output}" );
  // fbm3 stopped being a root once domain_warp arrived — it must still show
  // up, but only nested inside domain_warp's tree.
  assert!( output.contains( "fbm3" ), "forest missing `fbm3` under domain_warp:\n{output}" );
}

#[ test ]
fn tree_chunk_reports_unknown_chunk_error_for_bogus_name()
{
  let err = chunk_tree( Some( "bogus_chunk" ) ).expect_err( "chunk_tree should fail for an unknown name" );
  assert!( matches!( err, QueryError::UnknownChunk( _ ) ), "expected UnknownChunk, got {err:?}" );
}
