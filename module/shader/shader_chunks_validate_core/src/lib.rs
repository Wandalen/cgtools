//! Registry-wide integrity checks over [`shader_chunks_core::CHUNKS`] — the
//! engine behind `shader_chunks validate`. Five independent, non-panicking
//! checks run across every bundled chunk in one pass and report every
//! problem found, rather than failing loudly ( `compose`'s panic ) or
//! stopping at the first one:
//!
//! - **Manifest drift** — a chunk's compiled-in descriptor fields disagree
//!   with what [`shader_chunks_core::manifest_mismatches`] freshly parses
//!   from the chunk's own `wgsl` text.
//! - **Duplicate names** — two bundled chunks share a `//@ name:`, which
//!   would silently shadow one of them behind
//!   [`shader_chunks_core::chunk_get`]'s first-match lookup.
//! - **Missing dependencies** — a `//@ depends_on:` entry names a chunk not
//!   present anywhere in the bundled registry.
//! - **Dependency cycles** — the registry cannot be topologically sorted.
//! - **WGSL compilation** — a chunk's own transitive dependency closure,
//!   composed, fails naga parse or validation ( the same front end `wgpu`
//!   uses, reused from [`shader_chunks_preview`]'s `bundle_prepare` check
//!   but scoped to raw composed text rather than a full preview bundle, so
//!   a dependency-only chunk with no previewable export is still checked ).
//!
//! Deliberately out of scope: `//@ param:` line malformation. Discovering
//! that requires [`shader_chunks_params_core::discover`], which panics
//! rather than returning a `Result` on a malformed line ( by design — chunk
//! manifests are trusted authored content, not adversarial input, matching
//! `shader_chunks_core`'s own `manifest_field` panic-on-malformed idiom ).
//! Catching that panic here would need `std::panic::catch_unwind`, a
//! pattern this codebase uses nowhere else; the clean fix — a non-panicking
//! `try_discover` twin, mirroring `compose`/`try_compose` — belongs to
//! `shader_chunks_params_core` itself, not duplicated or worked around here.

mod private
{
  use shader_chunks_core::ChunkDescriptor;

  /// One registry problem: which chunk it concerns ( or `"(registry)"` for
  /// a whole-registry problem like a dependency cycle spanning several
  /// chunks ), which check found it, and a human-readable detail message.
  #[ derive( Debug, Clone, PartialEq, Eq ) ]
  pub struct Finding
  {
    /// The offending chunk's `//@ name:`, or `"(registry)"` when the
    /// problem is not attributable to one single chunk.
    pub chunk : String,
    /// Short, stable check identifier ( e.g. `"manifest_drift"` ) — see the
    /// `check_*` functions below for the fixed set this crate produces.
    pub check : &'static str,
    /// Human-readable detail.
    pub message : String,
  }

  /// A chunk's compiled-in descriptor fields disagree with what
  /// [`shader_chunks_core::manifest_mismatches`] freshly parses from its
  /// own `wgsl` text — a stale generated `chunks.rs` or a `build.rs`
  /// parsing bug, not an authoring mistake in the chunk file itself.
  fn check_manifest_drift( chunks : &[ ChunkDescriptor ] ) -> Vec< Finding >
  {
    chunks.iter()
    .flat_map( | chunk | shader_chunks_core::manifest_mismatches( chunk ).into_iter()
      .map( | message | Finding { chunk : chunk.name.to_string(), check : "manifest_drift", message } ) )
    .collect()
  }

  /// Two bundled chunks share a `//@ name:` — [`shader_chunks_core::chunk_get`]'s
  /// linear first-match lookup would silently resolve every by-name lookup
  /// to whichever one appears first in [`shader_chunks_core::CHUNKS`],
  /// shadowing the rest.
  fn check_duplicate_names( chunks : &[ ChunkDescriptor ] ) -> Vec< Finding >
  {
    let mut seen = std::collections::HashSet::new();
    chunks.iter()
    .filter( | chunk | !seen.insert( chunk.name ) )
    .map( | chunk | Finding
    {
      chunk : chunk.name.to_string(),
      check : "duplicate_name",
      message : format!( "chunk name `{}` appears more than once in the registry — by-name lookup silently resolves to the first occurrence", chunk.name ),
    })
    .collect()
  }

  /// A `//@ depends_on:` entry names a chunk not present anywhere in
  /// `chunks` — every instance across the whole set, not just the first
  /// one [`shader_chunks_core::dependency_closed`]'s boolean would find.
  fn check_missing_dependencies( chunks : &[ ChunkDescriptor ] ) -> Vec< Finding >
  {
    let names : std::collections::HashSet< &str > = chunks.iter().map( | chunk | chunk.name ).collect();
    chunks.iter()
    .flat_map( | chunk | chunk.depends_on.iter()
      .filter( | &&dep | !names.contains( dep ) )
      .map( move | dep | Finding
      {
        chunk : chunk.name.to_string(),
        check : "missing_dependency",
        message : format!( "depends_on names `{dep}`, which is not in the bundled registry" ),
      }) )
    .collect()
  }

  /// The registry cannot be topologically sorted — reuses
  /// [`shader_chunks_core::set_try_compose`]'s own cycle-detecting walk
  /// over the whole set rather than re-implementing cycle detection here.
  /// A [`shader_chunks_core::ComposeError::MissingDependency`] result is
  /// deliberately not reported by this check — [`check_missing_dependencies`]
  /// already reports every instance of that class in full detail, so
  /// surfacing it here too would just be a less detailed duplicate.
  fn check_dependency_cycle( chunks : &[ ChunkDescriptor ] ) -> Vec< Finding >
  {
    match shader_chunks_core::set_try_compose( chunks )
    {
      Err( shader_chunks_core::ComposeError::CyclicDependency( trail ) ) => vec!
      [
        Finding { chunk : "(registry)".to_string(), check : "dependency_cycle", message : format!( "cyclic dependency: {trail}" ) },
      ],
      Ok( _ ) | Err( shader_chunks_core::ComposeError::MissingDependency { .. } ) => vec![],
    }
  }

  /// `chunk_name`'s own transitive dependency closure within `chunks` —
  /// `chunk_name` itself first, then every reachable `depends_on` chunk,
  /// each exactly once. `None` when any name in the closure ( direct or
  /// transitive ) is not present in `chunks`. Deliberately not
  /// [`shader_chunks_core::set_resolve`]: that function is hard-wired to
  /// resolve against the bundled [`shader_chunks_core::CHUNKS`] registry
  /// specifically ( via `chunk_get` ), regardless of what `chunks` a caller
  /// passes in — which would make [`check_wgsl_compiles`] silently ignore
  /// any fixture chunk not also present in the real bundled registry, an
  /// untestable check. This walk is the same seen/queue widening shape as
  /// `set_resolve`'s own, parameterized over an arbitrary slice via
  /// [`shader_chunks_core::chunk_get_from`] instead.
  fn transitive_closure<'a>( chunks : &'a [ ChunkDescriptor ], chunk_name : &str ) -> Option< Vec< &'a ChunkDescriptor > >
  {
    let root = shader_chunks_core::chunk_get_from( chunks, chunk_name )?;
    let mut selected = vec![ root ];
    let mut seen : std::collections::HashSet< &str > = std::iter::once( root.name ).collect();
    let mut queue : Vec< &str > = root.depends_on.to_vec();
    while let Some( dep_name ) = queue.pop()
    {
      if seen.insert( dep_name )
      {
        let dep = shader_chunks_core::chunk_get_from( chunks, dep_name )?;
        queue.extend( dep.depends_on.iter().copied() );
        selected.push( dep );
      }
    }
    Some( selected )
  }

  /// A chunk's own transitive dependency closure, composed, fails naga
  /// parse or validation — the same two calls
  /// `shader_chunks_preview::bundle_prepare` makes, applied to the chunk's
  /// raw composed text directly rather than a full preview bundle, so a
  /// dependency-only chunk with no previewable export ( most of the
  /// registry — plain building blocks like `hash21`/`value_noise` carry no
  /// entry point at all ) is still checked: a WGSL module with only helper
  /// functions and no `@vertex`/`@fragment`/`@compute` stage is itself
  /// valid WGSL, so `naga::valid::Validator::validate` does not require one
  /// ( confirmed by this crate's own `wgsl_compile_accepts_a_dependency_only_chunk_with_no_entry_point`
  /// test ). A chunk whose closure fails to resolve or compose ( a missing
  /// dependency or a cycle reachable from it ) is silently skipped here —
  /// [`check_missing_dependencies`] and [`check_dependency_cycle`] already
  /// report that problem; this check only ever adds a genuinely new class
  /// of finding ( WGSL the composer accepted but naga rejects ).
  fn check_wgsl_compiles( chunks : &[ ChunkDescriptor ] ) -> Vec< Finding >
  {
    chunks.iter()
    .filter_map( | chunk |
    {
      let closure = transitive_closure( chunks, chunk.name )?;
      let set : Vec< ChunkDescriptor > = closure.into_iter().copied().collect();
      let wgsl = shader_chunks_core::set_try_compose( &set ).ok()?;

      let module = match naga::front::wgsl::parse_str( &wgsl )
      {
        Ok( module ) => module,
        Err( err ) => return Some( Finding { chunk : chunk.name.to_string(), check : "wgsl_compile", message : err.emit_to_string( &wgsl ) } ),
      };
      naga::valid::Validator::new( naga::valid::ValidationFlags::all(), naga::valid::Capabilities::default() )
      .validate( &module )
      .err()
      .map( | err | Finding { chunk : chunk.name.to_string(), check : "wgsl_compile", message : err.emit_to_string( &wgsl ) } )
    })
    .collect()
  }

  /// Runs every check in this crate over `chunks`, collecting every finding
  /// from every check — an empty result means `chunks` is clean. Generic
  /// over an arbitrary set ( rather than the bundled registry alone ) so
  /// tests can exercise each check against self-contained fixtures; see
  /// [`validate_registry`] for the bundled-registry convenience wrapper the
  /// CLI actually calls.
  #[ must_use ]
  pub fn validate( chunks : &[ ChunkDescriptor ] ) -> Vec< Finding >
  {
    let mut findings = Vec::new();
    findings.extend( check_manifest_drift( chunks ) );
    findings.extend( check_duplicate_names( chunks ) );
    findings.extend( check_missing_dependencies( chunks ) );
    findings.extend( check_dependency_cycle( chunks ) );
    findings.extend( check_wgsl_compiles( chunks ) );
    findings
  }

  /// [`validate`] over the real bundled [`shader_chunks_core::CHUNKS`]
  /// registry — what `shader_chunks validate` actually runs.
  #[ must_use ]
  pub fn validate_registry() -> Vec< Finding >
  {
    validate( shader_chunks_core::CHUNKS )
  }
}

::mod_interface::mod_interface!
{
  own use Finding;
  own use validate;
  own use validate_registry;
}
