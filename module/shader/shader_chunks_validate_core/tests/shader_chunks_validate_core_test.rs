//! Direct-call tests for `shader_chunks_validate_core`'s five checks,
//! exercised only through the public `validate`/`validate_registry` surface
//! ( the `check_*` functions are private, matching every other `_core`
//! crate's own test-through-the-public-API convention — see
//! `shader_chunks_params_core/tests/`'s identical shape ). Each fixture
//! isolates one check by construction: self-consistent in every other
//! respect, so the only finding(s) `validate` can return come from the one
//! check under test — assertions filter by `check` rather than asserting
//! exact counts, so an incidental finding from an unrelated check would
//! still surface as a visible extra rather than a silent false pass.

use shader_chunks_core::ChunkDescriptor;
use shader_chunks_validate_core::validate;

/// Self-consistent, dependency-free, entry-point-free fixture — every
/// descriptor field matches what `manifest_mismatches` would parse from its
/// own `wgsl` text, so this fixture alone never produces a finding under
/// any check. Used as a clean baseline and as a duplicate-name pair source.
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

/// Descriptor `description` deliberately disagrees with the manifest's own
/// `//@ description:` line — every other field matches.
const LOCAL_DRIFT_WGSL : &str = "\
//@ name: local_drift
//@ description: Correct description, lives only in the manifest text.
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
  name : "local_drift",
  description : "Wrong description, lives only in the descriptor field.",
  tags : &[ ( "category", "test" ) ],
  stage : None,
  depends_on : &[],
  exports : &[ "fn local_drift() -> f32" ],
  wgsl : LOCAL_DRIFT_WGSL,
};

/// `depends_on` names a chunk absent from the fixture set passed to
/// `validate` — self-consistent otherwise, so `check_missing_dependencies`
/// is the only check with anything to say about it.
const LOCAL_MISSING_DEP_WGSL : &str = "\
//@ name: local_missing_dep
//@ description: Depends on a chunk absent from the fixture set.
//@ tags: category:test
//@ depends_on: nonexistent_chunk
//@ export: fn local_missing_dep() -> f32

fn local_missing_dep() -> f32
{
  return 1.0;
}
";

const LOCAL_MISSING_DEP : ChunkDescriptor = ChunkDescriptor
{
  name : "local_missing_dep",
  description : "Depends on a chunk absent from the fixture set.",
  tags : &[ ( "category", "test" ) ],
  stage : None,
  depends_on : &[ "nonexistent_chunk" ],
  exports : &[ "fn local_missing_dep() -> f32" ],
  wgsl : LOCAL_MISSING_DEP_WGSL,
};

/// A `depends_on` cycle: A -> B -> A. Self-consistent otherwise, so
/// `check_dependency_cycle` is the only check with anything to say — and,
/// since both names resolve within the pair, `check_missing_dependencies`
/// stays silent too.
const LOCAL_CYCLE_A_WGSL : &str = "\
//@ name: local_cycle_a
//@ description: Cyclic fixture, half A.
//@ tags: category:test
//@ depends_on: local_cycle_b
//@ export: fn local_cycle_a() -> f32

fn local_cycle_a() -> f32
{
  return local_cycle_b();
}
";

const LOCAL_CYCLE_A : ChunkDescriptor = ChunkDescriptor
{
  name : "local_cycle_a",
  description : "Cyclic fixture, half A.",
  tags : &[ ( "category", "test" ) ],
  stage : None,
  depends_on : &[ "local_cycle_b" ],
  exports : &[ "fn local_cycle_a() -> f32" ],
  wgsl : LOCAL_CYCLE_A_WGSL,
};

const LOCAL_CYCLE_B_WGSL : &str = "\
//@ name: local_cycle_b
//@ description: Cyclic fixture, half B.
//@ tags: category:test
//@ depends_on: local_cycle_a
//@ export: fn local_cycle_b() -> f32

fn local_cycle_b() -> f32
{
  return local_cycle_a();
}
";

const LOCAL_CYCLE_B : ChunkDescriptor = ChunkDescriptor
{
  name : "local_cycle_b",
  description : "Cyclic fixture, half B.",
  tags : &[ ( "category", "test" ) ],
  stage : None,
  depends_on : &[ "local_cycle_a" ],
  exports : &[ "fn local_cycle_b() -> f32" ],
  wgsl : LOCAL_CYCLE_B_WGSL,
};

/// A second `depends_on` cycle, structurally independent of
/// `LOCAL_CYCLE_A`/`LOCAL_CYCLE_B` ( no shared chunk between the two
/// pairs ): C -> D -> C. Used to prove `check_dependency_cycle` reports
/// every disjoint cycle in the input set, not just whichever one
/// `shader_chunks_core::set_try_compose`'s single topological-sort pass
/// happens to hit first.
const LOCAL_CYCLE_C_WGSL : &str = "\
//@ name: local_cycle_c
//@ description: Cyclic fixture, half C ( independent of A/B ).
//@ tags: category:test
//@ depends_on: local_cycle_d
//@ export: fn local_cycle_c() -> f32

fn local_cycle_c() -> f32
{
  return local_cycle_d();
}
";

const LOCAL_CYCLE_C : ChunkDescriptor = ChunkDescriptor
{
  name : "local_cycle_c",
  description : "Cyclic fixture, half C ( independent of A/B ).",
  tags : &[ ( "category", "test" ) ],
  stage : None,
  depends_on : &[ "local_cycle_d" ],
  exports : &[ "fn local_cycle_c() -> f32" ],
  wgsl : LOCAL_CYCLE_C_WGSL,
};

const LOCAL_CYCLE_D_WGSL : &str = "\
//@ name: local_cycle_d
//@ description: Cyclic fixture, half D ( independent of A/B ).
//@ tags: category:test
//@ depends_on: local_cycle_c
//@ export: fn local_cycle_d() -> f32

fn local_cycle_d() -> f32
{
  return local_cycle_c();
}
";

const LOCAL_CYCLE_D : ChunkDescriptor = ChunkDescriptor
{
  name : "local_cycle_d",
  description : "Cyclic fixture, half D ( independent of A/B ).",
  tags : &[ ( "category", "test" ) ],
  stage : None,
  depends_on : &[ "local_cycle_c" ],
  exports : &[ "fn local_cycle_d() -> f32" ],
  wgsl : LOCAL_CYCLE_D_WGSL,
};

/// Self-consistent manifest ( no drift ), but a body that is not valid WGSL
/// at all — the only check with anything to say is `check_wgsl_compiles`.
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

#[ test ]
fn clean_fixture_produces_no_findings()
{
  let findings = validate( &[ LOCAL_CLEAN ] );
  assert!( findings.is_empty(), "a fully self-consistent, dependency-free chunk should have nothing to report: {findings:?}" );
}

#[ test ]
fn manifest_drift_is_reported_for_a_mismatched_field()
{
  let findings = validate( &[ LOCAL_DRIFT ] );
  let drift : Vec< _ > = findings.iter().filter( | f | f.check == "manifest_drift" ).collect();
  assert_eq!( drift.len(), 1, "exactly the description field should drift: {findings:?}" );
  assert!( drift[ 0 ].message.contains( "description" ), "{:?}", drift[ 0 ] );
}

#[ test ]
fn duplicate_name_is_reported_for_two_chunks_sharing_a_name()
{
  let findings = validate( &[ LOCAL_CLEAN, LOCAL_CLEAN ] );
  let duplicates : Vec< _ > = findings.iter().filter( | f | f.check == "duplicate_name" ).collect();
  assert_eq!( duplicates.len(), 1, "the second occurrence only should be flagged: {findings:?}" );
  assert_eq!( duplicates[ 0 ].chunk, "local_clean" );
}

#[ test ]
fn missing_dependency_is_reported_and_not_duplicated_as_a_cycle()
{
  let findings = validate( &[ LOCAL_MISSING_DEP ] );
  assert_eq!( findings.len(), 1, "missing_dependency alone, with no derivative dependency_cycle/wgsl_compile noise: {findings:?}" );
  assert_eq!( findings[ 0 ].check, "missing_dependency" );
  assert!( findings[ 0 ].message.contains( "nonexistent_chunk" ), "{:?}", findings[ 0 ] );
}

#[ test ]
fn dependency_cycle_is_reported_and_not_duplicated_as_wgsl_compile_failure()
{
  let findings = validate( &[ LOCAL_CYCLE_A, LOCAL_CYCLE_B ] );
  assert_eq!( findings.len(), 1, "dependency_cycle alone, with no derivative wgsl_compile noise: {findings:?}" );
  assert_eq!( findings[ 0 ].check, "dependency_cycle" );
  assert!( findings[ 0 ].message.contains( "local_cycle_a" ) || findings[ 0 ].message.contains( "local_cycle_b" ), "{:?}", findings[ 0 ] );
}

/// bug_reproducer(BUG-510): `check_dependency_cycle` reused
/// `shader_chunks_core::set_try_compose`'s single topological-sort pass
/// wholesale, so it returned at most one `dependency_cycle` `Finding` per
/// `validate` call. `set_try_compose`'s own `entries_sort_and_join` aborts
/// its `for entry in entries { visit( ... )?; }` loop on the *first*
/// `Err` it hits ( `shader_chunks_core/src/lib.rs`'s `visit` function,
/// propagated via `?` ), so a second, structurally independent cycle
/// elsewhere in the same input set was silently never visited at all —
/// not merely de-duplicated, genuinely unreached. This directly
/// contradicts this crate's own documented contract ( readme.md: "report
/// every problem found ... rather than ... stopping at the first one";
/// `shader_chunks_validate/docs/cli/command_group/01_validate.md`'s
/// `Invariants`: "no check short-circuits ... [over] the full input set
/// in one pass" ), and the precedent set by `check_missing_dependencies`'s
/// own doc comment ( "every instance across the whole set, not just the
/// first one" ).
///
/// Root Cause: `check_dependency_cycle` called `set_try_compose` exactly
/// once over the whole chunk slice and mapped at most its single `Err`
/// into a `Finding` — it never retried after removing the culprit to look
/// for further, disjoint cycles.
///
/// Why Not Caught: the only existing dependency-cycle fixture
/// ( `LOCAL_CYCLE_A`/`LOCAL_CYCLE_B` ) exercised exactly one cycle in
/// isolation; no fixture ever combined two structurally independent
/// cycles in the same `validate` call, so the "stop after the first
/// `Err`" behavior of the reused `set_try_compose` engine was never
/// exercised through `check_dependency_cycle`'s own multi-problem
/// contract.
///
/// Fix Applied: `check_dependency_cycle` now loops, removing the specific
/// chunk that closed each detected cycle ( parsed from
/// `ComposeError::CyclicDependency`'s documented `"[...] -> name"` trail
/// — the trailing `name` is exactly the chunk whose re-visit closed the
/// loop ) and re-running `set_try_compose` on the shrunken set until it
/// returns `Ok`. A collateral `ComposeError::MissingDependency` surfaced
/// by that shrinking ( an innocent chunk that only depended on the
/// just-removed cyclic chunk ) is silently absorbed by removing the
/// affected dependent too — it is not a real registry problem and
/// `check_missing_dependencies` already reports every genuine instance
/// against the full, untouched input.
///
/// Prevention: any check in this crate that reuses a single-shot engine
/// function from `shader_chunks_core` must be verified against a fixture
/// combining *two* independent instances of the problem, not just one —
/// matching the "report every instance, not just the first" bar the other
/// four checks already meet.
///
/// Pitfall: re-introducing a bare, single-shot
/// `match shader_chunks_core::set_try_compose( chunks ) { ... }` ( no
/// retry loop ) would silently regress back to "first cycle only" with
/// every existing single-cycle test still green — this test is the only
/// one that would catch it.
#[ test ]
fn dependency_cycle_reports_every_independent_cycle_not_just_the_first()
{
  let findings = validate( &[ LOCAL_CYCLE_A, LOCAL_CYCLE_B, LOCAL_CYCLE_C, LOCAL_CYCLE_D ] );
  let cycles : Vec< _ > = findings.iter().filter( | f | f.check == "dependency_cycle" ).collect();
  assert_eq!
  (
    cycles.len(), 2,
    "two structurally independent cycles ( A<->B and C<->D, no shared chunk ) must each produce their own finding, \
    not just whichever one the underlying topological sort happens to hit first: {findings:?}"
  );
  assert!
  (
    cycles.iter().any( | f | f.message.contains( "local_cycle_a" ) || f.message.contains( "local_cycle_b" ) ),
    "the A<->B cycle must be one of the two reported findings: {cycles:?}"
  );
  assert!
  (
    cycles.iter().any( | f | f.message.contains( "local_cycle_c" ) || f.message.contains( "local_cycle_d" ) ),
    "the C<->D cycle must be one of the two reported findings: {cycles:?}"
  );
}

#[ test ]
fn wgsl_compile_is_reported_for_a_naga_parse_failure()
{
  let findings = validate( &[ LOCAL_BROKEN ] );
  assert_eq!( findings.len(), 1, "wgsl_compile alone: {findings:?}" );
  assert_eq!( findings[ 0 ].check, "wgsl_compile" );
}

/// The load-bearing empirical check behind `check_wgsl_compiles`'s design:
/// a dependency-only chunk with no `//@ stage:` entry point — most of the
/// bundled registry, e.g. `hash21` — must still naga-validate cleanly. A
/// WGSL module containing only free functions and no `@vertex` /
/// `@fragment` / `@compute` stage is itself valid WGSL; if this test ever
/// fails, `naga::valid::Validator::validate` has started requiring an entry
/// point and `check_wgsl_compiles` needs to skip entry-point-free closures
/// instead of validating them.
#[ test ]
fn wgsl_compile_accepts_a_dependency_only_chunk_with_no_entry_point()
{
  let hash21 = shader_chunks_core::chunk_get_from( shader_chunks_core::CHUNKS, "hash21" )
  .expect( "hash21 should be a bundled chunk" );
  assert!( hash21.stage.is_none(), "hash21 is expected to declare no //@ stage: entry point for this test to be meaningful" );
  assert!( hash21.depends_on.is_empty(), "hash21 is expected to be a leaf ( no depends_on ) for this test to isolate wgsl_compile alone" );

  let findings = validate( &[ *hash21 ] );
  assert!( findings.is_empty(), "hash21, a real bundled dependency-only chunk with no entry point, should validate cleanly: {findings:?}" );
}

/// End-to-end sanity check over the real bundled registry — the exact call
/// `shader_chunks validate` makes. A failure here is a genuine defect
/// somewhere in the currently bundled `shader/` collection, not a test bug.
#[ test ]
fn validate_registry_reports_nothing_for_the_current_bundled_registry()
{
  let findings = shader_chunks_validate_core::validate_registry();
  assert!( findings.is_empty(), "the bundled registry is expected to be clean under every check: {findings:#?}" );
}
