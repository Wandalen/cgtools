//! Direct-call tests for `shader_chunks_params`'s command logic — no
//! subprocess; see `tests/cli_subprocess_test.rs` in the aggregator for
//! end-to-end argv and exit-code coverage.

use shader_chunks_params::{ ParamsCliError, tunables, tunables_of_chunk };

/// Mirrors `shader_chunks_params_core/tests/discovery_test.rs`'s own
/// `LOCAL_GLOW` fixture — a test-local chunk carrying `//@ param:` lines,
/// since no bundled chunk declares any (out of scope for this task to
/// change). It shares the name of the bundled `glow` chunk but never
/// touches the registry — it is only ever passed to `tunables_of_chunk`
/// directly.
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
    matches!( &err, ParamsCliError::UnknownChunk( name ) if name == "bogus_chunk" ),
    "expected UnknownChunk(\"bogus_chunk\"), got {err:?}"
  );
  assert_eq!( err.exit_code(), 1 );
}

// test_kind: bug_reproducer(BUG-XXX-B)
/// ## Root Cause
/// See `shader_chunks_params_core/tests/discovery_test.rs`'s own BUG-XXX-B tests -- same defect,
/// copy-pasted across 3 files. `shader_chunks_params/readme.md` and
/// `shader_chunks_params/docs/cli/command/01_tunables.md` are the other 2 copies: both omitted
/// `palette_cosine` from the leaf/infrastructure exception list, and the CLI doc additionally
/// claimed "46 of the 50" instead of the real 45.
/// ## Why Not Caught
/// No test in this crate read either doc file's own text -- `tunables_test.rs` exercised only
/// `tunables`/`tunables_of_chunk`'s runtime behavior against a real bundled chunk (`hash21`) and a
/// local fixture, never the prose describing that behavior.
/// ## Fix Applied
/// Added `palette_cosine` to `readme.md`'s exception list, and corrected
/// `docs/cli/command/01_tunables.md`'s count to "45 of the 50" plus its own exception list.
/// ## Prevention
/// Every copy of a restated fact needs its own direct doc-text assertion -- fixing 2 of 3 copies
/// and trusting the third "probably matches" is exactly how this drifted in the first place.
/// ## Pitfall
/// `shader_chunks_params_core/tests/discovery_test.rs`'s regression test only covers its own
/// crate's readme; without a test here, this crate's 2 copies (readme + CLI doc) could silently
/// re-diverge from the corrected fact independently of that other crate's test ever noticing.
#[ test ]
fn docs_reflect_palette_cosine_and_corrected_count()
{
  let readme = include_str!( "../readme.md" );
  assert!
  (
    readme.contains( "palette_cosine" ),
    "shader_chunks_params/readme.md must list `palette_cosine` among the chunks declaring zero \
    `//@ param:` lines (BUG-XXX-B)"
  );

  let tunables_doc = include_str!( "../docs/cli/command/01_tunables.md" );
  assert!
  (
    tunables_doc.contains( "palette_cosine" ),
    "docs/cli/command/01_tunables.md must list `palette_cosine` among the remaining \
    leaf/infrastructure chunks (BUG-XXX-B)"
  );
  assert!
  (
    tunables_doc.contains( "45 of the 50" ) && !tunables_doc.contains( "46 of the 50" ),
    "docs/cli/command/01_tunables.md must state 45 (not 46) of the 50 bundled chunks carry \
    `//@ param:` lines (BUG-XXX-B)"
  );
}
