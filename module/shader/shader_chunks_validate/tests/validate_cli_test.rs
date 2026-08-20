//! Direct-call tests for `shader_chunks_validate`'s CLI-wiring layer: the
//! all-clear message shape, the `FindingsPresent` report shape, and its
//! exit code. `shader_chunks_validate_core`'s own tests already prove each
//! check's correctness in isolation ( `Vec<Finding>` comparisons ) — these
//! tests instead prove this crate's own responsibility, the rendered
//! *string* a user actually sees, using local fixtures so no bundled chunk
//! needs to be broken to exercise the dirty path ( same split
//! `shader_chunks_params/tests/tunables_test.rs` makes with its own
//! `LOCAL_GLOW` fixture, for the same reason ).

use shader_chunks_core::ChunkDescriptor;
use shader_chunks_validate::{ ValidateCliError, validate, validate_chunks };

const LOCAL_CLEAN_WGSL : &str = "\
//@ name: local_clean
//@ description: A clean, self-consistent fixture chunk.
//@ tags: category:test
//@ depends_on:
//@ export: fn local_clean() -> f32

fn local_clean() -> f32
{
  return 1.0;
}
";

const LOCAL_CLEAN : ChunkDescriptor = ChunkDescriptor
{
  name : "local_clean",
  description : "A clean, self-consistent fixture chunk.",
  tags : &[ ( "category", "test" ) ],
  stage : None,
  depends_on : &[],
  exports : &[ "fn local_clean() -> f32" ],
  wgsl : LOCAL_CLEAN_WGSL,
};

const LOCAL_BROKEN_WGSL : &str = "\
//@ name: local_broken
//@ description: Deliberately invalid WGSL body.
//@ tags: category:test
//@ depends_on:
//@ export: fn local_broken() -> f32

this is not valid wgsl at all !!!
";

const LOCAL_BROKEN : ChunkDescriptor = ChunkDescriptor
{
  name : "local_broken",
  description : "Deliberately invalid WGSL body.",
  tags : &[ ( "category", "test" ) ],
  stage : None,
  depends_on : &[],
  exports : &[ "fn local_broken() -> f32" ],
  wgsl : LOCAL_BROKEN_WGSL,
};

/// Descriptor `name` deliberately disagrees with the manifest's own
/// `//@ name:` line.
const LOCAL_DRIFT_WGSL : &str = "\
//@ name: local_drift_manifest_name
//@ description: A second, independent drift fixture.
//@ tags: category:test
//@ depends_on:
//@ export: fn local_drift() -> f32

fn local_drift() -> f32
{
  return 1.0;
}
";

const LOCAL_DRIFT : ChunkDescriptor = ChunkDescriptor
{
  name : "local_drift_descriptor_name",
  description : "A second, independent drift fixture.",
  tags : &[ ( "category", "test" ) ],
  stage : None,
  depends_on : &[],
  exports : &[ "fn local_drift() -> f32" ],
  wgsl : LOCAL_DRIFT_WGSL,
};

#[ test ]
fn clean_fixture_produces_the_all_clear_message()
{
  let report = validate_chunks( &[ LOCAL_CLEAN ] ).expect( "a clean fixture should not error" );
  assert!( report.contains( "clean" ), "{report}" );
  assert!( report.contains( "0 findings" ), "{report}" );
}

#[ test ]
fn the_real_bundled_registry_is_reported_clean_through_the_cli_wiring()
{
  let report = validate().expect( "the bundled registry is expected to be clean" );
  assert!( report.contains( "clean" ), "{report}" );
  assert!( report.contains( "0 findings" ), "{report}" );
}

#[ test ]
fn one_finding_produces_a_readable_report_with_exit_code_one()
{
  let err = validate_chunks( &[ LOCAL_BROKEN ] ).expect_err( "broken WGSL should be reported" );
  let ValidateCliError::FindingsPresent( report ) = &err;
  assert!( report.starts_with( "1 finding(s):" ), "{report}" );
  assert!( report.contains( "[local_broken] wgsl_compile:" ), "{report}" );
  assert_eq!( err.exit_code(), 1 );
}

#[ test ]
fn multiple_findings_are_joined_as_separate_blank_line_separated_blocks()
{
  let err = validate_chunks( &[ LOCAL_BROKEN, LOCAL_DRIFT ] ).expect_err( "two independent problems should both be reported" );
  let ValidateCliError::FindingsPresent( report ) = &err;
  assert!( report.starts_with( "2 finding(s):" ), "{report}" );
  assert!( report.contains( "[local_broken] wgsl_compile:" ), "{report}" );
  assert!( report.contains( "[local_drift_descriptor_name] manifest_drift:" ), "{report}" );
  assert!( report.contains( "\n\n" ), "blocks should be blank-line separated:\n{report}" );
}
