//! Testable command logic for the `shader_chunks` binary. Each public
//! function here takes already-parsed arguments and returns the exact string
//! `main.rs` prints — keeping rendering inside these functions, rather than
//! in `main.rs` itself, is what makes the direct-call test tier possible
//! (see `tests/shader_chunks_test.rs`): no subprocess is needed to
//! assert on output content.

mod private
{
  use core::fmt;
  use error_tools::Error;
  use data_fmt::{ ColumnData, Format, RowBuilder, TableConfig, TableFormatter, TreeFormatter, TreeNode };

  /// Error returned by every `shader_chunks` command function.
  #[ derive( Debug, Error ) ]
  pub enum CliError
  {
    /// `get`/`tree`/`compose` named a chunk not present in [`shader_chunks_core::CHUNKS`].
    UnknownChunk( String ),
    /// `compose` failed the dependency resolution [`shader_chunks_core::try_compose`] performs.
    Compose( shader_chunks_core::ComposeError ),
    /// A `data_fmt` render call failed.
    Render( String ),
  }

  impl fmt::Display for CliError
  {
    fn fmt( &self, f : &mut fmt::Formatter< '_ > ) -> fmt::Result
    {
      match self
      {
        Self::UnknownChunk( name ) => write!( f, "unknown chunk: `{name}` (see `list` for valid names)" ),
        Self::Compose( err ) => write!( f, "{err}" ),
        Self::Render( msg ) => write!( f, "render error: {msg}" ),
      }
    }
  }

  impl CliError
  {
    /// Maps this error to a process exit code: `1` for a bad chunk name or a
    /// failed composition (validation-style, caller-fixable by passing
    /// different arguments), `2` for a render failure (internal).
    #[ must_use ]
    pub fn exit_code( &self ) -> i32
    {
      match self
      {
        Self::UnknownChunk( _ ) | Self::Compose( _ ) => 1,
        Self::Render( _ ) => 2,
      }
    }
  }

  fn find_chunk( name : &str ) -> Result< &'static shader_chunks_core::ChunkDescriptor, CliError >
  {
    shader_chunks_core::chunk_get( name )
    .ok_or_else( || CliError::UnknownChunk( name.to_string() ) )
  }

  fn tags_string( wgsl : &str ) -> String
  {
    shader_chunks_core::parse_tags( wgsl )
    .iter()
    .map( | ( group, tag ) | format!( "{group}:{tag}" ) )
    .collect::< Vec< _ > >()
    .join( ", " )
  }

  fn depends_on_string( wgsl : &str ) -> String
  {
    let deps = shader_chunks_core::parse_depends_on( wgsl );
    if deps.is_empty() { "(none)".to_string() } else { deps.join( ", " ) }
  }

  /// Every bundled chunk nothing else depends on — the roots of the
  /// no-argument `tree` forest.
  fn dependents_free_roots() -> Vec< &'static shader_chunks_core::ChunkDescriptor >
  {
    let depended_on : std::collections::HashSet< &str > = shader_chunks_core::CHUNKS.iter()
    .flat_map( | chunk | shader_chunks_core::parse_depends_on( chunk.wgsl ) )
    .collect();

    shader_chunks_core::CHUNKS.iter()
    .filter( | chunk | !depended_on.contains( chunk.name ) )
    .collect()
  }

  /// Builds one `name` tree node carrying its name and tags as aligned
  /// column data, recursing into each `depends_on` entry. `TreeFormatter`'s
  /// `format_aligned` renders a node's `ColumnData` (not its bare `name`
  /// field) for every non-root row, so `name` must be column 0 for it to
  /// appear at all — see `tree_chunk`, which also compensates for
  /// `format_aligned`'s default `show_root: false` by wrapping each root in
  /// an invisible parent so it renders as a normal (column-bearing) row
  /// rather than being skipped as the tree's root. A dependency name that
  /// can't be resolved against the bundled set is skipped rather than
  /// panicking — defensive only, since every real chunk's `depends_on` is
  /// validated at `compose` time and this bundled set is fixed and
  /// self-consistent.
  fn dep_tree( chunk : &shader_chunks_core::ChunkDescriptor ) -> TreeNode< ColumnData >
  {
    let name = chunk.name;
    let mut node = TreeNode::new( name.to_string(), Some( ColumnData::new( vec![ name.to_string(), tags_string( chunk.wgsl ) ] ) ) );
    for dep_name in shader_chunks_core::parse_depends_on( chunk.wgsl )
    {
      if let Ok( dep ) = find_chunk( dep_name )
      {
        node.children.push( dep_tree( dep ) );
      }
    }
    node
  }

  /// Table of every bundled chunk: name / description / tags / depends_on.
  ///
  /// # Errors
  ///
  /// Returns [`CliError::Render`] if the `data_fmt` table formatter fails.
  pub fn list_chunks() -> Result< String, CliError >
  {
    let mut builder = RowBuilder::new( vec!
    [
      "name".to_string(), "description".to_string(), "tags".to_string(), "depends_on".to_string(),
    ]);
    for chunk in shader_chunks_core::CHUNKS
    {
      builder.add_row_mut( vec!
      [
        chunk.name.into(),
        shader_chunks_core::parse_description( chunk.wgsl ).into(),
        tags_string( chunk.wgsl ).into(),
        depends_on_string( chunk.wgsl ).into(),
      ]);
    }
    let view = builder.build_view();
    Format::format( &TableFormatter::with_config( TableConfig::plain() ), &view )
    .map_err( | e | CliError::Render( e.to_string() ) )
  }

  /// Full detail text for one chunk: name, description, stage, tags,
  /// `depends_on`, exports.
  ///
  /// # Errors
  ///
  /// Returns [`CliError::UnknownChunk`] when `name` is not in [`shader_chunks_core::CHUNKS`].
  pub fn get_chunk( name : &str ) -> Result< String, CliError >
  {
    let chunk = find_chunk( name )?;
    let exports = shader_chunks_core::parse_exports( chunk.wgsl ).join( "\n  " );
    Ok( format!
    (
      "name: {name}\ndescription: {description}\nstage: {stage:?}\ntags: {tags}\ndepends_on: {depends_on}\nexports:\n  {exports}\n",
      name = chunk.name,
      description = shader_chunks_core::parse_description( chunk.wgsl ),
      stage = shader_chunks_core::parse_stage( chunk.wgsl ),
      tags = tags_string( chunk.wgsl ),
      depends_on = depends_on_string( chunk.wgsl ),
    ))
  }

  /// Table of every distinct `group:tag` pair and the chunk(s) carrying it.
  ///
  /// # Errors
  ///
  /// Returns [`CliError::Render`] if the `data_fmt` table formatter fails.
  pub fn list_tags() -> Result< String, CliError >
  {
    let mut pairs : Vec< ( String, Vec< &'static str > ) > = Vec::new();
    for chunk in shader_chunks_core::CHUNKS
    {
      for ( group, tag ) in shader_chunks_core::parse_tags( chunk.wgsl )
      {
        let key = format!( "{group}:{tag}" );
        if let Some( entry ) = pairs.iter_mut().find( | entry | entry.0 == key )
        {
          entry.1.push( chunk.name );
        }
        else
        {
          pairs.push( ( key, vec![ chunk.name ] ) );
        }
      }
    }

    let mut builder = RowBuilder::new( vec![ "tag".to_string(), "chunks".to_string() ] );
    for ( tag, chunks ) in pairs
    {
      builder.add_row_mut( vec![ tag.into(), chunks.join( ", " ).into() ] );
    }
    let view = builder.build_view();
    Format::format( &TableFormatter::with_config( TableConfig::plain() ), &view )
    .map_err( | e | CliError::Render( e.to_string() ) )
  }

  /// Dependency tree for one chunk, or — with `name` absent — a forest of
  /// every chunk nothing else depends on.
  ///
  /// # Errors
  ///
  /// Returns [`CliError::UnknownChunk`] when `name` is `Some` and not found.
  pub fn tree_chunk( name : Option< &str > ) -> Result< String, CliError >
  {
    let roots : Vec< &'static shader_chunks_core::ChunkDescriptor > = match name
    {
      Some( name ) => vec![ find_chunk( name )? ],
      None => dependents_free_roots(),
    };

    let formatter = TreeFormatter::new();
    Ok( roots.iter().map( | &chunk |
    {
      // `format_aligned` never renders its own argument's `name`/`data` (only
      // `show_root: true` would, and even then via bare `name` with no
      // column alignment) — it only renders `children`. Wrapping each real
      // root as the sole child of an invisible, data-less parent makes that
      // root itself appear as a normal aligned row instead of being skipped.
      let mut invisible_parent = TreeNode::new( String::new(), None );
      invisible_parent.children.push( dep_tree( chunk ) );
      formatter.format_aligned( &invisible_parent )
    }).collect::< Vec< _ > >().join( "\n" ) )
  }

  /// Composes already-resolved WGSL chunk bodies via
  /// [`shader_chunks_core::try_compose`]. Exposed separately from
  /// [`compose_chunks`] so tests can exercise cyclic/missing-dependency
  /// failures with synthetic fixtures — the real bundled chunk set is fixed
  /// and acyclic, so it can never produce a `CyclicDependency` through the
  /// name-based [`compose_chunks`] path.
  ///
  /// # Errors
  ///
  /// Returns [`CliError::Compose`] on a cyclic or unresolved dependency.
  pub fn try_compose_wgsl( chunks : &[ &str ] ) -> Result< String, CliError >
  {
    shader_chunks_core::try_compose( chunks ).map_err( CliError::Compose )
  }

  /// Resolves `names` via [`shader_chunks_core::chunk_get`] and composes them.
  ///
  /// # Errors
  ///
  /// Returns [`CliError::UnknownChunk`] if any name is not bundled, or
  /// [`CliError::Compose`] on a missing dependency.
  pub fn compose_chunks( names : &[ String ] ) -> Result< String, CliError >
  {
    let chunks : Vec< &str > = names.iter()
    .map( | name | find_chunk( name ).map( | chunk | chunk.wgsl ) )
    .collect::< Result< _, _ > >()?;
    try_compose_wgsl( &chunks )
  }
}

::mod_interface::mod_interface!
{
  own use CliError;
  own use list_chunks;
  own use get_chunk;
  own use list_tags;
  own use tree_chunk;
  own use try_compose_wgsl;
  own use compose_chunks;
}
