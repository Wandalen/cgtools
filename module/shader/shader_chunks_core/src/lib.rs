//! Manifest-driven WGSL shader-chunk composer. Each bundled chunk under the
//! repo-root `shader/` directory is exactly one WGSL function (or, for the
//! vertex stage, one entry point plus the struct type it returns) with its
//! manifest embedded as a leading `//@`-prefixed comment block — the same
//! machine-parsable-attribute convention this ecosystem's playbooks use
//! ( see `playbook.rulebook.md § Structure : Attribute Block` ), with `//`
//! standing in for `#` since that is WGSL's line-comment token. [`compose`]
//! reads every chunk's `//@ depends_on:` line, topologically sorts the given
//! chunks, and concatenates their WGSL bodies dependency-before-dependent —
//! regardless of the order they were passed in — panicking immediately, and
//! naming the offending chunk, on a cyclic or unresolvable dependency.
//! [`try_compose`] is the same sort, non-panicking, for callers ( e.g. a CLI )
//! taking untrusted chunk sets.
//!
//! This crate intentionally has no Rust reimplementation of any chunk's
//! math ( no `hash21`/`value_noise`/`fbm3` Rust ports ). Chunks are a
//! shader-side concept only; the manifest, not a parallel Rust body, is what
//! makes a chunk's interface legible — and since the manifest lives inside
//! the same file as the code it describes, there is only ever one file to
//! open, not two.

mod private
{

  /// `hash21` chunk: 2D-point -> pseudo-random scalar hash.
  pub const HASH21 : &str = include_str!( "../../../../shader/hash21.wgsl" );
  /// `value_noise` chunk: bilinear value noise over [`HASH21`].
  pub const VALUE_NOISE : &str = include_str!( "../../../../shader/value_noise.wgsl" );
  /// `fbm3` chunk: three-octave fractal Brownian motion over [`VALUE_NOISE`].
  pub const FBM3 : &str = include_str!( "../../../../shader/fbm3.wgsl" );
  /// `fullscreen_triangle` chunk: vertex entry point emitting one screen-covering triangle.
  pub const FULLSCREEN_TRIANGLE : &str = include_str!( "../../../../shader/fullscreen_triangle.wgsl" );

  /// Every bundled chunk, in declaration order — the full set a caller can
  /// pass to [`compose`]/[`try_compose`] or enumerate for inspection.
  pub const ALL_CHUNKS : &[ &str ] = &[ HASH21, VALUE_NOISE, FBM3, FULLSCREEN_TRIANGLE ];

  // A chunk's `//@ key: value` header lines are its manifest. `manifest_field`
  // reads a mandatory single-line field ( `name`, `depends_on`, `description`,
  // `tags` ); `manifest_field_opt` reads an optional single-line field
  // ( `stage`, vertex-only ); `manifest_field_all` collects every line for a
  // repeatable field ( `export`, one or more per chunk ). This crate's tests
  // ( `tests/shader_chunks_core_test.rs` ) cross-check `export` against the real
  // WGSL body, so it can't silently drift.

  fn manifest_field<'a>( wgsl : &'a str, key : &str ) -> &'a str
  {
    let prefix = format!( "//@ {key}:" );
    wgsl.lines()
    .find_map( | line | line.strip_prefix( prefix.as_str() ) )
    .unwrap_or_else( || panic!( "chunk missing required `//@ {key}:` header line:\n{wgsl}" ) )
    .trim()
  }

  fn manifest_field_opt<'a>( wgsl : &'a str, key : &str ) -> Option< &'a str >
  {
    let prefix = format!( "//@ {key}:" );
    wgsl.lines()
    .find_map( | line | line.strip_prefix( prefix.as_str() ) )
    .map( str::trim )
  }

  fn manifest_field_all<'a>( wgsl : &'a str, key : &str ) -> Vec< &'a str >
  {
    let prefix = format!( "//@ {key}:" );
    wgsl.lines()
    .filter_map( | line | line.strip_prefix( prefix.as_str() ) )
    .map( str::trim )
    .collect()
  }

  /// Reads the chunk's `//@ name:` manifest line.
  ///
  /// # Panics
  ///
  /// Panics if the chunk has no `//@ name:` header line.
  #[ must_use ]
  pub fn parse_name( wgsl : &str ) -> &str
  {
    manifest_field( wgsl, "name" )
  }

  /// Reads the chunk's `//@ depends_on:` manifest line as a list of chunk
  /// names ( empty when the value is empty ).
  ///
  /// # Panics
  ///
  /// Panics if the chunk has no `//@ depends_on:` header line.
  #[ must_use ]
  pub fn parse_depends_on( wgsl : &str ) -> Vec< &str >
  {
    let raw = manifest_field( wgsl, "depends_on" );
    if raw.is_empty()
    {
      return Vec::new();
    }
    raw.split( ',' ).map( str::trim ).collect()
  }

  /// Reads the chunk's `//@ description:` manifest line.
  ///
  /// # Panics
  ///
  /// Panics if the chunk has no `//@ description:` header line.
  #[ must_use ]
  pub fn parse_description( wgsl : &str ) -> &str
  {
    manifest_field( wgsl, "description" )
  }

  /// Reads the chunk's `//@ stage:` manifest line, if present ( only the
  /// vertex-stage chunk declares one — ordinary function chunks have none ).
  #[ must_use ]
  pub fn parse_stage( wgsl : &str ) -> Option< &str >
  {
    manifest_field_opt( wgsl, "stage" )
  }

  /// Reads every `//@ export:` manifest line, in file order ( a chunk may
  /// export more than one symbol, e.g. a struct plus the function that
  /// returns it ).
  #[ must_use ]
  pub fn parse_exports( wgsl : &str ) -> Vec< &str >
  {
    manifest_field_all( wgsl, "export" )
  }

  /// Reads the chunk's `//@ tags:` manifest line as a list of `(group, tag)`
  /// pairs ( empty when the value is empty ).
  ///
  /// # Panics
  ///
  /// Panics if the chunk has no `//@ tags:` header line, or if an entry has
  /// no `:` separator.
  #[ must_use ]
  pub fn parse_tags( wgsl : &str ) -> Vec< ( &str, &str ) >
  {
    let raw = manifest_field( wgsl, "tags" );
    if raw.is_empty()
    {
      return Vec::new();
    }
    raw.split( ',' )
    .map( str::trim )
    .map( | entry | entry.split_once( ':' )
      .unwrap_or_else( || panic!( "malformed `//@ tags:` entry (expected `group:tag`): {entry:?}" ) ) )
    .collect()
  }

  struct ParsedChunk< 'a >
  {
    name : &'a str,
    depends_on : Vec< &'a str >,
    wgsl : &'a str,
  }

  /// Error returned by [`try_compose`] instead of panicking.
  #[ derive( Debug, Clone, PartialEq, Eq ) ]
  pub enum ComposeError
  {
    /// A dependency cycle was found among the given chunks. Carries the
    /// visiting-stack trail ( `"[...] -> name"` ) that closed the cycle.
    CyclicDependency( String ),
    /// A chunk's `//@ depends_on:` names a chunk not present in the given set.
    MissingDependency
    {
      /// The chunk whose `depends_on` line named the missing chunk.
      chunk : String,
      /// The depended-on chunk name that was not passed to `try_compose`.
      missing : String,
    },
  }

  impl std::fmt::Display for ComposeError
  {
    fn fmt( &self, f : &mut std::fmt::Formatter< '_ > ) -> std::fmt::Result
    {
      match self
      {
        Self::CyclicDependency( trail ) => write!( f, "cyclic shader-chunk dependency: {trail}" ),
        Self::MissingDependency { chunk, missing } =>
        write!( f, "chunk `{chunk}` depends on `{missing}`, which was not passed to compose()" ),
      }
    }
  }

  impl std::error::Error for ComposeError {}

  /// Topologically sorts `chunks` by each one's header-declared `depends_on`
  /// and concatenates their WGSL bodies in that order, dependency before
  /// dependent, regardless of the order they were passed in.
  ///
  /// # Panics
  ///
  /// Panics, naming the offending chunk, on a dependency cycle or a
  /// `depends_on` entry not present in `chunks` — both are authoring mistakes
  /// in a chunk's header ( or in the set passed to `compose` ), not states a
  /// correctly-authored composition can reach. Use [`try_compose`] instead
  /// when `chunks` is not already trusted to be internally consistent.
  #[ must_use ]
  pub fn compose( chunks : &[ &str ] ) -> String
  {
    try_compose( chunks ).unwrap_or_else( | err | panic!( "{err}" ) )
  }

  /// Non-panicking twin of [`compose`]: same topological sort over `chunks`,
  /// but reports a dependency cycle or unresolved dependency as an [`Err`]
  /// instead of panicking — for callers ( e.g. a CLI ) taking untrusted chunk
  /// sets where a panic is the wrong failure mode.
  ///
  /// # Errors
  ///
  /// Returns [`ComposeError::CyclicDependency`] on a dependency cycle, or
  /// [`ComposeError::MissingDependency`] when a `depends_on` entry names a
  /// chunk not present in `chunks`.
  pub fn try_compose( chunks : &[ &str ] ) -> Result< String, ComposeError >
  {
    let entries : Vec< ParsedChunk< '_ > > = chunks.iter()
    .map( | &wgsl | ParsedChunk { name : parse_name( wgsl ), depends_on : parse_depends_on( wgsl ), wgsl } )
    .collect();

    let mut ordered : Vec< &ParsedChunk< '_ > > = Vec::with_capacity( entries.len() );
    let mut visiting : Vec< &str > = Vec::new();

    for entry in &entries
    {
      visit( entry.name, None, &entries, &mut visiting, &mut ordered )?;
    }

    Ok( ordered.iter().map( | e | e.wgsl ).collect::< Vec< _ > >().join( "\n\n" ) )
  }

  // Two lifetimes: `'a` is the borrowed WGSL source text ( outlives this call,
  // e.g. `'static` for the bundled consts ); `'e` is the local `entries`
  // slice built in `try_compose`. `ordered` collects `&'e ParsedChunk<'a>`
  // directly instead of names, so the final join needs no second name->chunk
  // lookup ( which would be a spurious, clippy-flagged `.expect()` — every
  // name in `ordered` already has its chunk in hand at push time ).
  fn visit<'a, 'e>
  (
    name : &'a str,
    required_by : Option< &'a str >,
    entries : &'e [ ParsedChunk< 'a > ],
    visiting : &mut Vec< &'a str >,
    ordered : &mut Vec< &'e ParsedChunk< 'a > >,
  ) -> Result< (), ComposeError >
  {
    if ordered.iter().any( | e | e.name == name )
    {
      return Ok( () );
    }
    if visiting.contains( &name )
    {
      return Err( ComposeError::CyclicDependency( format!( "{visiting:?} -> {name}" ) ) );
    }

    let Some( entry ) = entries.iter().find( | e | e.name == name ) else
    {
      return Err( ComposeError::MissingDependency
      {
        chunk : required_by.unwrap_or( name ).to_string(),
        missing : name.to_string(),
      });
    };

    visiting.push( name );
    for &dep in &entry.depends_on
    {
      visit( dep, Some( name ), entries, visiting, ordered )?;
    }
    visiting.pop();
    ordered.push( entry );
    Ok( () )
  }

}

::mod_interface::mod_interface!
{
  own use HASH21;
  own use VALUE_NOISE;
  own use FBM3;
  own use FULLSCREEN_TRIANGLE;
  own use ALL_CHUNKS;
  own use parse_name;
  own use parse_depends_on;
  own use parse_description;
  own use parse_stage;
  own use parse_exports;
  own use parse_tags;
  own use ComposeError;
  own use compose;
  own use try_compose;
}
