//! Tests for [`discover`] and [`chunk_discover`] — `//@ param:` line
//! parsing, file-order/empty-input handling, malformed-directive panics,
//! and declared-range precedence over inference. All fixtures are
//! self-contained WGSL strings owned by this file — no real bundled
//! `shader/*.wgsl` chunk is read or annotated.

use shader_chunks_params_core::
{
  discover, chunk_discover, Parameter, ParameterKind, Range, RangeSource, ValueType,
};

#[ test ]
fn discover_parses_declared_range_argument_u32()
{
  let wgsl = "//@ param: octaves argument u32 range(1, 8)\n";
  let expected = vec!
  [
    Parameter
    {
      name : "octaves".to_string(),
      kind : ParameterKind::Argument,
      value_type : ValueType::U32,
      range : Some( ( Range { min : 1.0, max : 8.0 }, RangeSource::Declared ) ),
    },
  ];
  assert_eq!( discover( wgsl ), expected );
}

#[ test ]
fn discover_infers_range_for_define_kind_via_seed_name_pattern()
{
  let wgsl = "//@ param: seed define u32\n";
  let expected = vec!
  [
    Parameter
    {
      name : "seed".to_string(),
      kind : ParameterKind::Define,
      value_type : ValueType::U32,
      range : Some( ( Range { min : 0.0, max : 65535.0 }, RangeSource::Inferred ) ),
    },
  ];
  assert_eq!( discover( wgsl ), expected );
}

#[ test ]
fn discover_parses_declared_range_for_all_five_kinds()
{
  let wgsl = "\
//@ param: a argument u32 range(1, 2)
//@ param: b define i32 range(3, 4)
//@ param: c uniform f32 range(5, 6)
//@ param: d attribute vec2f range(7, 8)
//@ param: e texture texture_2d range(9, 10)
";
  let expected = vec!
  [
    Parameter { name : "a".to_string(), kind : ParameterKind::Argument, value_type : ValueType::U32, range : Some( ( Range { min : 1.0, max : 2.0 }, RangeSource::Declared ) ) },
    Parameter { name : "b".to_string(), kind : ParameterKind::Define, value_type : ValueType::I32, range : Some( ( Range { min : 3.0, max : 4.0 }, RangeSource::Declared ) ) },
    Parameter { name : "c".to_string(), kind : ParameterKind::Uniform, value_type : ValueType::F32, range : Some( ( Range { min : 5.0, max : 6.0 }, RangeSource::Declared ) ) },
    Parameter { name : "d".to_string(), kind : ParameterKind::Attribute, value_type : ValueType::Vec2F, range : Some( ( Range { min : 7.0, max : 8.0 }, RangeSource::Declared ) ) },
    Parameter { name : "e".to_string(), kind : ParameterKind::Texture, value_type : ValueType::Texture2d, range : Some( ( Range { min : 9.0, max : 10.0 }, RangeSource::Declared ) ) },
  ];
  assert_eq!( discover( wgsl ), expected );
}

#[ test ]
fn discover_infers_range_for_argument_kind_with_no_declared_range()
{
  let wgsl = "//@ param: radius argument f32\n";
  let expected = vec!
  [
    Parameter
    {
      name : "radius".to_string(),
      kind : ParameterKind::Argument,
      value_type : ValueType::F32,
      range : Some( ( Range { min : 0.0, max : 100.0 }, RangeSource::Inferred ) ),
    },
  ];
  assert_eq!( discover( wgsl ), expected );
}

#[ test ]
fn discover_returns_multiple_params_in_file_order()
{
  let wgsl = "\
//@ param: first argument u32 range(1, 2)
//@ param: second uniform f32 range(3, 4)
";
  let result = discover( wgsl );
  assert_eq!( result.len(), 2 );
  assert_eq!( result[ 0 ].name, "first" );
  assert_eq!( result[ 1 ].name, "second" );
}

#[ test ]
fn discover_returns_empty_vec_when_no_param_lines()
{
  let wgsl = "//@ name: no_params\n//@ description: has no tunable parameters\n";
  assert_eq!( discover( wgsl ), Vec::< Parameter >::new() );
}

#[ test ]
#[ should_panic( expected = "unknown WGSL type token" ) ]
fn discover_panics_on_unknown_type_token()
{
  let _ = discover( "//@ param: x argument bogus_type\n" );
}

#[ test ]
#[ should_panic( expected = "unknown kind token" ) ]
fn discover_panics_on_unknown_kind_token()
{
  let _ = discover( "//@ param: x bogus_kind u32\n" );
}

#[ test ]
#[ should_panic( expected = "malformed `//@ param:` line" ) ]
fn discover_panics_on_wrong_token_count()
{
  let _ = discover( "//@ param: x argument\n" );
}

#[ test ]
fn discover_declared_range_overrides_name_pattern_inference()
{
  let wgsl = "//@ param: octaves argument u32 range(1, 8)\n";
  let result = discover( wgsl );
  assert_eq!( result[ 0 ].range, Some( ( Range { min : 1.0, max : 8.0 }, RangeSource::Declared ) ) );
}

#[ test ]
fn discover_declared_range_overrides_type_fallback_inference()
{
  // "workgroup_x" matches no name pattern, so a `u32` param with no
  // declared range would infer the type-fallback `[0, 16]` ( see
  // `infer_range_attribute_workgroup_x_falls_through_to_type_fallback` in
  // `range_inference_test.rs` ). Declaring `range(2, 4)` here must win
  // outright rather than blend with or defer to that fallback.
  let wgsl = "//@ param: workgroup_x attribute u32 range(2, 4)\n";
  let result = discover( wgsl );
  assert_eq!( result[ 0 ].range, Some( ( Range { min : 2.0, max : 4.0 }, RangeSource::Declared ) ) );
}

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
  return value_noise( p, 0.0 ) * 2.0;
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
fn discover_chunk_matches_discover_on_wgsl_field()
{
  assert_eq!( chunk_discover( &LOCAL_GLOW ), discover( LOCAL_GLOW_WGSL ) );
}

// test_kind: bug_reproducer(BUG-293)
/// ## Root Cause
/// `param_lines` recognized a `//@ param:` line even when preceded by leading whitespace, via
/// `line.trim_start().strip_prefix( "//@ param:" )` -- unlike `shader_chunks_core::manifest_field`
/// / `manifest_field_opt` / `manifest_field_all`, which read every sibling header field
/// ( `name`/`description`/`tags`/`depends_on`/`export`/`stage` ) via a bare
/// `line.strip_prefix( prefix )` with no `trim_start`, requiring the `//@ ` prefix at column 0.
/// This crate's own docs (`docs/api/001_tunable_parameter_taxonomy.md`'s Abstract) explicitly
/// claim `//@ param:` lives in "the same" flat header block `shader_chunks_core` already reads,
/// under "the same trust model" -- a claim the `trim_start()` leniency contradicted.
/// ## Why Not Caught
/// All 100% of real bundled `shader/*.wgsl` manifest headers are flush-left by disciplined
/// convention (verified: zero indented `//@ ` lines of any kind across all 50 real chunk files),
/// so the divergence was never triggered by real content either way, and every existing test
/// fixture in this file also starts its `//@ param:` lines at column 0 -- nothing ever exercised
/// the leniency gap directly.
/// ## Fix Applied
/// Removed `.trim_start()` from `param_lines`, matching `manifest_field`'s exact recognition rule.
/// ## Prevention
/// When a crate's own docs claim it mirrors another module's established convention, the
/// implementation must be checked line-for-line against that module's actual source, not just
/// assumed consistent from the shared vocabulary ( "the same header block" ) alone.
/// ## Pitfall
/// A manifest system built on "malformed authored content panics loudly" depends on every field
/// sharing one predictable recognition rule -- a lone lenient field can silently accept content
/// (e.g. an indented illustrative `//@ param:` line inside a doc-comment example) that every
/// other field would correctly ignore, undermining that guarantee for one field only.
#[ test ]
fn param_line_requires_column_zero_prefix_matching_manifest_field_convention()
{
  let indented = "  //@ param: octaves argument u32 range(1, 8)\n";
  assert_eq!
  (
    discover( indented ), Vec::< Parameter >::new(),
    "an indented `//@ param:` line must be ignored, exactly like shader_chunks_core::manifest_field \
    ignores an indented `//@ name:`/`//@ description:`/etc. line (BUG-293)"
  );

  let flush_left = "//@ param: octaves argument u32 range(1, 8)\n";
  assert_eq!( discover( flush_left ).len(), 1, "a column-0 `//@ param:` line must still be recognized" );
}

// test_kind: bug_reproducer(BUG-294)
/// ## Root Cause
/// `shader_chunks_params_core/readme.md`, `shader_chunks_params/readme.md`, and
/// `shader_chunks_params/docs/cli/command/01_tunables.md` all claimed "46 of the 50" bundled
/// chunks declare `//@ param:` lines today, naming exactly 4 exceptions (`hash21`, `hash22`,
/// `srgb`, `fullscreen_triangle`). The real count is 45, with a 5th exception, `palette_cosine`,
/// missing from all 3 copies of the list.
/// ## Why Not Caught
/// No test cross-checked the docs' specific count/exception-list claim against
/// `shader_chunks_core::CHUNKS` -- this crate's own tests deliberately use only self-contained
/// fixture WGSL (`docs/api/001_tunable_parameter_taxonomy.md`'s own Out of Scope note: "annotating
/// any real bundled `shader/*.wgsl` chunk is a `shader/` collection concern, not this crate's"),
/// so real-chunk adoption state was never asserted anywhere in this crate's test suite.
/// ## Fix Applied
/// Corrected all 3 doc copies to "45 of the 50" / the 5-name exception list including
/// `palette_cosine`, and added this test as a standing regression guard against the fact itself
/// drifting out of sync with the docs again.
/// ## Prevention
/// A specific, quotable count in documentation ("46 of the 50") is a factual claim about the
/// codebase's current state, not prose -- it decays the moment a bundled chunk's own annotation
/// state changes, exactly like a doc comment claiming a return type or mutability. Back a
/// standing count claim with a real test against the actual source of truth, not just a
/// point-in-time correctness check at authoring time.
/// ## Pitfall
/// The same wrong count was copy-pasted across 3 separate files (2 readmes + 1 CLI command doc)
/// -- a single source-of-truth fact restated in multiple places will drift in all of them
/// identically unless something other than human memory keeps them in sync.
#[ test ]
fn exactly_5_bundled_chunks_declare_no_tunable_params()
{
  let mut without_params : Vec< &str > = shader_chunks_core::CHUNKS
  .iter()
  .filter( | chunk | chunk_discover( chunk ).is_empty() )
  .map( | chunk | chunk.name )
  .collect();
  without_params.sort_unstable();

  assert_eq!
  (
    without_params,
    vec![ "fullscreen_triangle", "hash21", "hash22", "palette_cosine", "srgb" ],
    "bundled chunks declaring zero `//@ param:` lines changed (BUG-294) -- update this list AND \
    the matching count/list in shader_chunks_params_core/readme.md, shader_chunks_params/readme.md, \
    and shader_chunks_params/docs/cli/command/01_tunables.md"
  );
}

// test_kind: bug_reproducer(BUG-294)
/// ## Root Cause
/// See `exactly_5_bundled_chunks_declare_no_tunable_params` above -- same defect. This test
/// targets the other half of it: the codebase-fact test proves `discover` returns empty for
/// exactly 5 named chunks, but never reads `shader_chunks_params_core/readme.md`'s own prose, so
/// it cannot prove the doc text itself was actually corrected rather than merely intended to be.
/// ## Why Not Caught
/// This session's established precedent for doc-only defects (BUG-287/288/290) is a direct
/// `include_str!` read of the specific defective file with a text/substring assertion -- the
/// codebase-fact test above was written first and, while a valid future-drift guard, does not
/// follow that precedent, leaving the readme's actual text unverified by any test.
/// ## Fix Applied
/// Added a direct `include_str!( "../readme.md" )` read asserting the corrected substrings
/// (`palette_cosine` present, "45 of" present, "46 of" absent).
/// ## Prevention
/// A doc-text defect's regression test must read the actual doc file's text, not only a codebase
/// fact the doc is supposed to describe -- the two can diverge again independently.
/// ## Pitfall
/// A test that re-derives and checks a codebase fact (e.g. "exactly these 5 chunks") can pass
/// forever even if the prose describing that fact is never actually fixed, silently defeating the
/// purpose of a doc-only bug's regression test.
#[ test ]
fn readme_chunk_annotations_reflect_palette_cosine_and_corrected_count()
{
  let readme = include_str!( "../readme.md" );
  assert!
  (
    readme.contains( "palette_cosine" ),
    "shader_chunks_params_core/readme.md's Chunk annotations section must list `palette_cosine` \
    among the chunks declaring zero `//@ param:` lines (BUG-294)"
  );
  assert!
  (
    readme.contains( "45 of" ) && !readme.contains( "46 of" ),
    "shader_chunks_params_core/readme.md's Chunk annotations section must state 45 (not 46) of \
    the 50 bundled chunks carry `//@ param:` lines (BUG-294)"
  );
}
