//! Query engine over `shader_chunks_core`'s bundled WGSL chunks. Each
//! public function here takes already-parsed arguments and returns the
//! exact string the `shader_chunks_query` CLI prints — keeping rendering
//! inside these functions, rather than in the CLI wiring layer, is what
//! makes the direct-call test tier possible ( see `tests/` of
//! `shader_chunks_query` ): no subprocess is needed to assert on output
//! content.

mod private
{
  use core::fmt;
  use core::fmt::Write as _;
  use core::str::FromStr;
  use error_tools::Error;
  use data_fmt::
  {
    ColumnData, DecoratedText, ExpandedFormatter, Format, Heading, JsonFormatter, RowBuilder,
    TableConfig, TableFormatter, TreeFormatter, TreeNode, WrapConfig, WrapFormatter, YamlFormatter,
  };

  /// Error returned by every query function.
  #[ derive( Debug, Error ) ]
  pub enum QueryError
  {
    /// A query named a chunk not present in [`shader_chunks_core::CHUNKS`].
    UnknownChunk( String ),
    /// `fields::` named a field outside [`QUERY_FIELDS`].
    UnknownField( String ),
    /// A parameter value fell outside its allowed set or range.
    InvalidParam
    {
      /// The parameter's `key::` name as typed on the command line.
      param : &'static str,
      /// The offending value as typed.
      value : String,
      /// Human-readable statement of the allowed values or range.
      allowed : &'static str,
    },
    /// A `data_fmt` render call failed.
    Render( String ),
  }

  impl fmt::Display for QueryError
  {
    fn fmt( &self, f : &mut fmt::Formatter< '_ > ) -> fmt::Result
    {
      match self
      {
        Self::UnknownChunk( name ) => write!( f, "unknown chunk: `{name}` (see `list` for valid names)" ),
        Self::UnknownField( field ) => write!( f, "unknown field: `{field}` (valid fields: {})", QUERY_FIELDS.join( ", " ) ),
        Self::InvalidParam { param, value, allowed } => write!( f, "invalid `{param}` value: `{value}` (allowed: {allowed})" ),
        Self::Render( msg ) => write!( f, "render error: {msg}" ),
      }
    }
  }

  impl QueryError
  {
    /// Maps this error to a process exit code: `1` for a bad chunk name or
    /// a bad field or parameter value ( validation-style, caller-fixable by
    /// passing different arguments ), `2` for a render failure ( internal ).
    #[ must_use ]
    pub fn exit_code( &self ) -> i32
    {
      match self
      {
        Self::UnknownChunk( _ ) | Self::UnknownField( _ ) | Self::InvalidParam { .. } => 1,
        Self::Render( _ ) => 2,
      }
    }
  }

  fn chunk_find( name : &str ) -> Result< &'static shader_chunks_core::ChunkDescriptor, QueryError >
  {
    shader_chunks_core::chunk_get( name )
    .ok_or_else( || QueryError::UnknownChunk( name.to_string() ) )
  }

  fn tags_string( chunk : &shader_chunks_core::ChunkDescriptor ) -> String
  {
    chunk.tags
    .iter()
    .map( | ( group, tag ) | format!( "{group}:{tag}" ) )
    .collect::< Vec< _ > >()
    .join( ", " )
  }

  fn depends_on_string( chunk : &shader_chunks_core::ChunkDescriptor ) -> String
  {
    if chunk.depends_on.is_empty() { "(none)".to_string() } else { chunk.depends_on.join( ", " ) }
  }

  /// Names of every chunk some other chunk directly depends on — the
  /// complement of the `roots::1` filter and of the no-argument `tree` forest.
  fn depended_on_set() -> std::collections::HashSet< &'static str >
  {
    shader_chunks_core::CHUNKS.iter()
    .flat_map( | chunk | chunk.depends_on.iter().copied() )
    .collect()
  }

  /// Every bundled chunk nothing else depends on — the roots of the
  /// no-argument `tree` forest.
  fn dependents_free_roots() -> Vec< &'static shader_chunks_core::ChunkDescriptor >
  {
    let depended_on = depended_on_set();
    shader_chunks_core::CHUNKS.iter()
    .filter( | chunk | !depended_on.contains( chunk.name ) )
    .collect()
  }

  /// Every bundled chunk with no dependencies of its own — the roots of
  /// the no-argument `tree reverse::1` forest ( a reverse walk has to
  /// start somewhere with nothing beneath it; the mirror image of
  /// [`dependents_free_roots`] ).
  fn leaf_roots() -> Vec< &'static shader_chunks_core::ChunkDescriptor >
  {
    shader_chunks_core::CHUNKS.iter()
    .filter( | chunk | chunk.depends_on.is_empty() )
    .collect()
  }

  /// Maps each chunk name to the chunks that directly depend on it — the
  /// reverse of `depends_on` — built fresh per call ( the registry is
  /// small and static, so caching would add complexity for no measurable
  /// gain ).
  fn reverse_adjacency() -> std::collections::HashMap< &'static str, Vec< &'static str > >
  {
    let mut map : std::collections::HashMap< &'static str, Vec< &'static str > > = std::collections::HashMap::new();
    for chunk in shader_chunks_core::CHUNKS
    {
      for &dep_name in chunk.depends_on
      {
        map.entry( dep_name ).or_default().push( chunk.name );
      }
    }
    map
  }

  /// Builds one `name` tree node carrying its name and tags as aligned
  /// column data, recursing via `children_of` — `depends_on` itself in
  /// forward mode, [`reverse_adjacency`]'s map in `reverse::1` mode — so
  /// one recursive walk backs both tree directions ( see `chunk_tree` ).
  /// `TreeFormatter`'s `format_aligned` renders a node's `ColumnData` ( not
  /// its bare `name` field ) for every non-root row, so `name` must be
  /// column 0 for it to appear at all — see `chunk_tree`, which also
  /// compensates for `format_aligned`'s default `show_root: false` by
  /// wrapping each root in an invisible parent so it renders as a normal
  /// ( column-bearing ) row rather than being skipped as the tree's root.
  /// A child name `children_of` returns that can't be resolved against the
  /// bundled set is skipped rather than panicking — defensive only, since
  /// every real chunk's `depends_on` is validated at `compose` time and
  /// this bundled set is fixed and self-consistent.
  fn dep_tree_node( chunk : &shader_chunks_core::ChunkDescriptor, children_of : &impl Fn( &str ) -> Vec< &'static str > ) -> TreeNode< ColumnData >
  {
    let name = chunk.name;
    let mut node = TreeNode::new( name.to_string(), Some( ColumnData::new( vec![ name.to_string(), tags_string( chunk ) ] ) ) );
    for dep_name in children_of( name )
    {
      if let Ok( dep ) = chunk_find( dep_name )
      {
        node.children.push( dep_tree_node( dep, children_of ) );
      }
    }
    node
  }

  /// Every field name `fields::` and `sort::` accept, in canonical column
  /// order. `source` is the chunk's raw WGSL body.
  pub const QUERY_FIELDS : &[ &str ] =
  &[ "name", "description", "stage", "tags", "depends_on", "exports", "source" ];

  /// How multiple `tag::` selectors combine.
  #[ derive( Debug, Clone, Copy, PartialEq, Eq ) ]
  pub enum TagsMode
  {
    /// A chunk matches when at least one selector matches.
    Any,
    /// A chunk matches only when every selector matches.
    All,
  }

  impl TagsMode
  {
    /// The `tags_mode::` spelling of this variant.
    #[ must_use ]
    pub fn as_str( self ) -> &'static str
    {
      match self { Self::Any => "any", Self::All => "all" }
    }
  }

  impl FromStr for TagsMode
  {
    type Err = QueryError;
    fn from_str( s : &str ) -> Result< Self, QueryError >
    {
      match s
      {
        "any" => Ok( Self::Any ),
        "all" => Ok( Self::All ),
        other => Err( QueryError::InvalidParam { param : "tags_mode", value : other.to_string(), allowed : "any, all" } ),
      }
    }
  }

  /// Sort key applied to the filtered chunk set.
  #[ derive( Debug, Clone, Copy, PartialEq, Eq ) ]
  pub enum SortKey
  {
    /// Keep selection order: `names::` order when given, registry order otherwise.
    Input,
    /// Sort by chunk name.
    Name,
    /// Sort by stage ( stage-less chunks first ), then name.
    Stage,
    /// Sort by description text, then name.
    Description,
  }

  impl SortKey
  {
    /// The `sort::` spelling of this variant.
    #[ must_use ]
    pub fn as_str( self ) -> &'static str
    {
      match self
      {
        Self::Input => "input",
        Self::Name => "name",
        Self::Stage => "stage",
        Self::Description => "description",
      }
    }
  }

  impl FromStr for SortKey
  {
    type Err = QueryError;
    fn from_str( s : &str ) -> Result< Self, QueryError >
    {
      match s
      {
        "input" => Ok( Self::Input ),
        "name" => Ok( Self::Name ),
        "stage" => Ok( Self::Stage ),
        "description" => Ok( Self::Description ),
        other => Err( QueryError::InvalidParam { param : "sort", value : other.to_string(), allowed : "input, name, stage, description" } ),
      }
    }
  }

  /// Sort direction. `Desc` reverses whatever `sort::` produced — including
  /// `input` order, so `sort::input order::desc` is the reversed registry.
  #[ derive( Debug, Clone, Copy, PartialEq, Eq ) ]
  pub enum SortOrder
  {
    /// Ascending ( the sort's natural direction ).
    Asc,
    /// Descending ( reverse of `Asc` ).
    Desc,
  }

  impl SortOrder
  {
    /// The `order::` spelling of this variant.
    #[ must_use ]
    pub fn as_str( self ) -> &'static str
    {
      match self { Self::Asc => "asc", Self::Desc => "desc" }
    }
  }

  impl FromStr for SortOrder
  {
    type Err = QueryError;
    fn from_str( s : &str ) -> Result< Self, QueryError >
    {
      match s
      {
        "asc" => Ok( Self::Asc ),
        "desc" => Ok( Self::Desc ),
        other => Err( QueryError::InvalidParam { param : "order", value : other.to_string(), allowed : "asc, desc" } ),
      }
    }
  }

  /// Output rendering for query results.
  #[ derive( Debug, Clone, Copy, PartialEq, Eq ) ]
  pub enum OutputFormat
  {
    /// Plain aligned table ( `data_fmt` plain config ).
    Table,
    /// Markdown pipe table.
    Markdown,
    /// One `-[ RECORD N ]` block per chunk, one line per field ( psql `\x` style ).
    Expanded,
    /// Pretty-printed JSON array of row objects.
    Json,
    /// YAML sequence of row mappings.
    Yaml,
    /// Bare chunk names, one per line — script/pipe friendly; ignores `fields::`.
    Names,
  }

  impl OutputFormat
  {
    /// The `format::` spelling of this variant.
    #[ must_use ]
    pub fn as_str( self ) -> &'static str
    {
      match self
      {
        Self::Table => "table",
        Self::Markdown => "markdown",
        Self::Expanded => "expanded",
        Self::Json => "json",
        Self::Yaml => "yaml",
        Self::Names => "names",
      }
    }
  }

  impl FromStr for OutputFormat
  {
    type Err = QueryError;
    fn from_str( s : &str ) -> Result< Self, QueryError >
    {
      match s
      {
        "table" => Ok( Self::Table ),
        "markdown" => Ok( Self::Markdown ),
        "expanded" => Ok( Self::Expanded ),
        "json" => Ok( Self::Json ),
        "yaml" => Ok( Self::Yaml ),
        "names" => Ok( Self::Names ),
        other => Err( QueryError::InvalidParam { param : "format", value : other.to_string(), allowed : "table, markdown, expanded, json, yaml, names" } ),
      }
    }
  }

  /// Rendering shape for [`chunk_tree`]'s output.
  #[ derive( Debug, Clone, Copy, PartialEq, Eq ) ]
  pub enum TreeFormat
  {
    /// Indented, column-aligned tree ( the original `tree` rendering ).
    Aligned,
    /// Graphviz `digraph` — one `"parent" -> "child";` edge statement per
    /// dependency, paste-able directly into Graphviz or an online renderer.
    Dot,
    /// Mermaid `graph TD` flowchart — one `parent --> child` edge per line,
    /// paste-able directly into a Mermaid Live Editor or Markdown viewer
    /// with Mermaid support.
    Mermaid,
  }

  impl TreeFormat
  {
    /// The `shape::` spelling of this variant.
    #[ must_use ]
    pub fn as_str( self ) -> &'static str
    {
      match self
      {
        Self::Aligned => "aligned",
        Self::Dot => "dot",
        Self::Mermaid => "mermaid",
      }
    }
  }

  impl FromStr for TreeFormat
  {
    type Err = QueryError;
    fn from_str( s : &str ) -> Result< Self, QueryError >
    {
      match s
      {
        "aligned" => Ok( Self::Aligned ),
        "dot" => Ok( Self::Dot ),
        "mermaid" => Ok( Self::Mermaid ),
        other => Err( QueryError::InvalidParam { param : "shape", value : other.to_string(), allowed : "aligned, dot, mermaid" } ),
      }
    }
  }

  /// The full parameter surface shared by `list` and `get` — one struct, one
  /// engine ( [`chunks_query`] ). The two commands differ only in the defaults
  /// [`QueryParams::list_defaults`] and [`QueryParams::get_defaults`] supply.
  #[ derive( Debug, Clone ) ]
  pub struct QueryParams
  {
    /// Chunk names to select, in the given order ( duplicates allowed );
    /// empty selects every bundled chunk in registry order.
    pub names : Vec< String >,
    /// Substring filter on chunk names; empty = off.
    pub pattern : String,
    /// `true` makes `pattern`/`exports` matching case-sensitive.
    pub case_sensitive : bool,
    /// Tag selectors — `group:tag` exact pair, or bare `tag` matching any
    /// group; empty = off. Selector matching is always case-sensitive.
    pub tags : Vec< String >,
    /// How multiple `tags` selectors combine.
    pub tags_mode : TagsMode,
    /// Stage filter: `any` ( off ), `none` ( stage-less chunks only ), or a
    /// literal stage name.
    pub stage : String,
    /// Keep only chunks that depend on this chunk ( name validated loudly );
    /// empty = off.
    pub depends_on : String,
    /// Widen `depends_on` from direct dependencies to the full transitive
    /// closure.
    pub transitive : bool,
    /// Substring filter over export signatures; empty = off.
    pub exports : String,
    /// Substring filter over the chunk's raw WGSL body ( the same text the
    /// `source` field renders — manifest header comments included ); empty = off.
    pub source : String,
    /// Keep only chunks nothing else depends on.
    pub roots : bool,
    /// Keep only chunks with no dependencies of their own.
    pub leaves : bool,
    /// Fields to project as columns, each from [`QUERY_FIELDS`].
    pub fields : Vec< String >,
    /// Print only the matched-chunk count — taken after all filters, before
    /// `offset`/`limit`.
    pub count : bool,
    /// Output rendering.
    pub format : OutputFormat,
    /// Sort key applied after filtering.
    pub sort : SortKey,
    /// Sort direction ( `Desc` reverses, including `input` order ).
    pub order : SortOrder,
    /// Keep at most this many chunks after `offset`; `0` = unlimited.
    pub limit : usize,
    /// Skip this many chunks before applying `limit`.
    pub offset : usize,
    /// Heading line above the table ( `table`/`markdown` formats only );
    /// empty = off.
    pub heading : String,
    /// Maximum column width ( `table`/`markdown` formats only ); `0` = auto.
    pub width : usize,
  }

  impl QueryParams
  {
    /// `list` defaults: every chunk, overview columns, plain table.
    #[ must_use ]
    pub fn list_defaults() -> Self
    {
      Self
      {
        names : Vec::new(),
        pattern : String::new(),
        case_sensitive : false,
        tags : Vec::new(),
        tags_mode : TagsMode::Any,
        stage : "any".to_string(),
        depends_on : String::new(),
        transitive : false,
        exports : String::new(),
        source : String::new(),
        roots : false,
        leaves : false,
        fields : vec![ "name".to_string(), "description".to_string(), "tags".to_string(), "depends_on".to_string() ],
        count : false,
        format : OutputFormat::Table,
        sort : SortKey::Input,
        order : SortOrder::Asc,
        limit : 0,
        offset : 0,
        heading : String::new(),
        width : 0,
      }
    }

    /// `get` defaults: same engine as `list`, detail columns, expanded
    /// per-record output.
    #[ must_use ]
    pub fn get_defaults() -> Self
    {
      let mut params = Self::list_defaults();
      params.fields = vec!
      [
        "name".to_string(), "description".to_string(), "stage".to_string(),
        "tags".to_string(), "depends_on".to_string(), "exports".to_string(),
      ];
      params.format = OutputFormat::Expanded;
      params
    }
  }

  /// Substring test honoring the `case::` switch — case-insensitive by
  /// default, exact when `case_sensitive` is set.
  fn pattern_contains( haystack : &str, needle : &str, case_sensitive : bool ) -> bool
  {
    if case_sensitive
    { haystack.contains( needle ) }
    else
    { haystack.to_lowercase().contains( &needle.to_lowercase() ) }
  }

  /// One `tag::` selector against one chunk: `group:tag` demands the exact
  /// pair; a bare `tag` matches that tag under any group.
  fn tag_selector_matches( chunk : &shader_chunks_core::ChunkDescriptor, selector : &str ) -> bool
  {
    match selector.split_once( ':' )
    {
      Some( ( group, tag ) ) => chunk.tags.iter().any( | &( g, t ) | g == group && t == tag ),
      None => chunk.tags.iter().any( | &( _, t ) | t == selector ),
    }
  }

  /// Whether `chunk` reaches `target` through its transitive `depends_on`
  /// closure ( breadth-agnostic walk; the chunk itself is not a member of its
  /// own closure ).
  fn reaches( chunk : &shader_chunks_core::ChunkDescriptor, target : &str ) -> bool
  {
    let mut queue : Vec< &str > = chunk.depends_on.to_vec();
    let mut seen = std::collections::HashSet::new();
    while let Some( name ) = queue.pop()
    {
      if name == target
      {
        return true;
      }
      if seen.insert( name )
        && let Some( dep ) = shader_chunks_core::chunk_get( name )
      {
        queue.extend( dep.depends_on.iter().copied() );
      }
    }
    false
  }

  /// Renders one [`QUERY_FIELDS`] field of one chunk as cell text.
  fn field_value( chunk : &shader_chunks_core::ChunkDescriptor, field : &str ) -> String
  {
    match field
    {
      "name" => chunk.name.to_string(),
      "description" => chunk.description.to_string(),
      "stage" => chunk.stage.map_or_else( || "(none)".to_string(), str::to_string ),
      "tags" => tags_string( chunk ),
      "depends_on" => depends_on_string( chunk ),
      "exports" => if chunk.exports.is_empty() { "(none)".to_string() } else { chunk.exports.join( "; " ) },
      "source" => chunk.wgsl.to_string(),
      _ => unreachable!( "field `{field}` validated against QUERY_FIELDS before projection" ),
    }
  }

  /// All filters of [`QueryParams`] applied to one chunk.
  fn chunk_matches
  (
    chunk : &shader_chunks_core::ChunkDescriptor,
    params : &QueryParams,
    depended_on : &std::collections::HashSet< &str >,
  ) -> bool
  {
    if !params.pattern.is_empty() && !pattern_contains( chunk.name, &params.pattern, params.case_sensitive )
    {
      return false;
    }
    if !params.tags.is_empty()
    {
      let matched = | selector : &String | tag_selector_matches( chunk, selector );
      let ok = match params.tags_mode
      {
        TagsMode::Any => params.tags.iter().any( matched ),
        TagsMode::All => params.tags.iter().all( matched ),
      };
      if !ok
      {
        return false;
      }
    }
    match params.stage.as_str()
    {
      "any" => {},
      "none" => if chunk.stage.is_some() { return false; },
      literal => if chunk.stage != Some( literal ) { return false; },
    }
    if !params.depends_on.is_empty()
    {
      let hit = if params.transitive
      { reaches( chunk, &params.depends_on ) }
      else
      { chunk.depends_on.contains( &params.depends_on.as_str() ) };
      if !hit
      {
        return false;
      }
    }
    if !params.exports.is_empty()
    {
      let hit = chunk.exports.iter()
      .any( | signature | pattern_contains( signature, &params.exports, params.case_sensitive ) );
      if !hit
      {
        return false;
      }
    }
    if !params.source.is_empty() && !pattern_contains( chunk.wgsl, &params.source, params.case_sensitive )
    {
      return false;
    }
    if params.roots && depended_on.contains( chunk.name )
    {
      return false;
    }
    if params.leaves && !chunk.depends_on.is_empty()
    {
      return false;
    }
    true
  }

  /// Renders an already-selected chunk sequence per the projection and
  /// formatting parameters.
  fn chunks_render
  (
    chunks : &[ &'static shader_chunks_core::ChunkDescriptor ],
    params : &QueryParams,
  ) -> Result< String, QueryError >
  {
    if params.format == OutputFormat::Names
    {
      return Ok( chunks.iter().map( | chunk | chunk.name ).collect::< Vec< _ > >().join( "\n" ) );
    }

    let rows : Vec< Vec< DecoratedText > > = chunks.iter().map( | chunk |
      params.fields.iter().map( | field | field_value( chunk, field ).into() ).collect()
    ).collect();

    let build_view = | rows : &[ Vec< DecoratedText > ] |
    {
      let mut builder = RowBuilder::new( params.fields.clone() );
      for row in rows
      {
        builder.add_row_mut( row.clone() );
      }
      builder.build_view()
    };
    let view = build_view( &rows );

    let with_heading = | mut config : TableConfig |
    {
      if !params.heading.is_empty()
      {
        config = config.with_heading( Heading::new( params.heading.clone() ) );
      }
      config
    };

    match params.format
    {
      OutputFormat::Table =>
      {
        let mut config = with_heading( TableConfig::plain() );
        // Fix(BUG-116): with_max_column_width alone doesn't guarantee wrap onto continuation lines
        // Root cause: data_fmt's auto_wrap only pre-wraps cells when the CAPPED total row width exceeds the resolved terminal width (120 fallback) — a cell exceeding max_column_width does NOT trigger wrap when the rest of the row stays narrow (e.g. a short `name` column), so table_plain's documented wrap contract (docs/cli/format/01_table_plain.md) silently degraded to truncate_cell's `...` truncation for exactly that case; pre-wrap every cell directly via WrapFormatter (the same primitive auto_wrap uses internally) to bypass the terminal-width gate entirely
        // Pitfall: a formatting library's "auto" behavior may be conditioned on more than the one config knob (`max_column_width`) that looks responsible — check the actual trigger condition in source, not just the knob's name, before trusting it
        let table_view = if params.width > 0
        {
          config = config.with_max_column_width( Some( params.width ) );
          let wrapper = WrapFormatter::with_config( WrapConfig::new().width( params.width ) );
          let wrapped_rows : Vec< Vec< DecoratedText > > = rows.iter().map( | row |
            row.iter().map( | cell |
            {
              let mut cell = cell.clone();
              cell.text = wrapper.wrap_joined( &cell.text );
              cell
            } ).collect()
          ).collect();
          build_view( &wrapped_rows )
        }
        else
        {
          build_view( &rows )
        };
        Format::format( &TableFormatter::with_config( config ), &table_view )
        .map_err( | e | QueryError::Render( e.to_string() ) )
      }
      OutputFormat::Markdown =>
      {
        let mut config = with_heading( TableConfig::markdown() );
        if params.width > 0
        {
          config = config.with_max_column_width( Some( params.width ) );
          // Fix(BUG-115): with_max_column_width alone doesn't guarantee truncation
          // Root cause: data_fmt's auto_wrap (default true) silently wraps instead of truncating once total capped row width exceeds the resolved terminal width (120 fallback) — markdown's documented contract (docs/cli/param/21_width.md) is truncate, so auto_wrap must be disabled here
          // Pitfall: a formatting library's independent config knobs can silently interact — always check whether disabling one (auto_wrap) is correct for every call site sharing the code path, not just the one that surfaced the bug
          config = config.with_auto_wrap( false );
        }
        Format::format( &TableFormatter::with_config( config ), &view )
        .map_err( | e | QueryError::Render( e.to_string() ) )
      }
      OutputFormat::Expanded => Format::format( &ExpandedFormatter::new(), &view )
      .map_err( | e | QueryError::Render( e.to_string() ) ),
      OutputFormat::Json => Format::format( &JsonFormatter::new(), &view )
      .map_err( | e | QueryError::Render( e.to_string() ) ),
      OutputFormat::Yaml => Format::format( &YamlFormatter::new(), &view )
      .map_err( | e | QueryError::Render( e.to_string() ) ),
      OutputFormat::Names => unreachable!( "names format returned before row building" ),
    }
  }

  /// The single query engine behind both `list` and `get`: selects chunks
  /// ( `names::`, or every bundled chunk ), applies every filter, sorts,
  /// pages, and renders. Pipeline order: select → filter → `count`
  /// short-circuit → sort/order → `offset`/`limit` → render.
  ///
  /// # Errors
  ///
  /// - [`QueryError::UnknownField`] — a `fields` entry outside [`QUERY_FIELDS`].
  /// - [`QueryError::UnknownChunk`] — a `names` entry or `depends_on` naming no
  ///   bundled chunk.
  /// - [`QueryError::Render`] — a `data_fmt` formatter failure.
  pub fn chunks_query( params : &QueryParams ) -> Result< String, QueryError >
  {
    for field in &params.fields
    {
      if !QUERY_FIELDS.contains( &field.as_str() )
      {
        return Err( QueryError::UnknownField( field.clone() ) );
      }
    }
    if !params.depends_on.is_empty()
    {
      chunk_find( &params.depends_on )?;
    }

    let mut chunks : Vec< &'static shader_chunks_core::ChunkDescriptor > = if params.names.is_empty()
    {
      shader_chunks_core::CHUNKS.iter().collect()
    }
    else
    {
      params.names.iter().map( | name | chunk_find( name ) ).collect::< Result< _, _ > >()?
    };

    let depended_on = depended_on_set();
    chunks.retain( | chunk | chunk_matches( chunk, params, &depended_on ) );

    if params.count
    {
      return Ok( chunks.len().to_string() );
    }

    match params.sort
    {
      SortKey::Input => {},
      SortKey::Name => chunks.sort_by_key( | chunk | chunk.name ),
      SortKey::Stage => chunks.sort_by_key( | chunk | ( chunk.stage.unwrap_or( "" ), chunk.name ) ),
      SortKey::Description => chunks.sort_by_key( | chunk | ( chunk.description, chunk.name ) ),
    }
    if params.order == SortOrder::Desc
    {
      chunks.reverse();
    }

    let limit = if params.limit == 0 { usize::MAX } else { params.limit };
    let chunks : Vec< _ > = chunks.into_iter().skip( params.offset ).take( limit ).collect();

    chunks_render( &chunks, params )
  }

  /// Table of every distinct `group:tag` pair and the chunk(s) carrying it.
  ///
  /// # Errors
  ///
  /// Returns [`QueryError::Render`] if the `data_fmt` table formatter fails.
  pub fn tags_list() -> Result< String, QueryError >
  {
    let mut pairs : Vec< ( String, Vec< &'static str > ) > = Vec::new();
    for chunk in shader_chunks_core::CHUNKS
    {
      for &( group, tag ) in chunk.tags
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
    .map_err( | e | QueryError::Render( e.to_string() ) )
  }

  /// Walks `chunk` via `children_of`, collecting every `(parent, child)`
  /// edge reachable from it exactly once. `expanded` guards a diamond
  /// dependency ( two parents converging on one child ) from having that
  /// child's own outgoing edges collected twice — shared across the whole
  /// `roots` walk in [`chunk_tree`], not reset per root.
  fn collect_edges<'a>( chunk : &'a shader_chunks_core::ChunkDescriptor, children_of : &impl Fn( &str ) -> Vec< &'static str >, expanded : &mut std::collections::HashSet< &'a str >, edges : &mut Vec< ( &'a str, &'a str ) > )
  {
    if !expanded.insert( chunk.name )
    {
      return;
    }
    for dep_name in children_of( chunk.name )
    {
      edges.push( ( chunk.name, dep_name ) );
      if let Ok( dep ) = chunk_find( dep_name )
      {
        collect_edges( dep, children_of, expanded, edges );
      }
    }
  }

  /// Renders `edges` ( from [`collect_edges`] ) plus any childless-root
  /// `isolated` names as a Graphviz `digraph`.
  fn dot_render( edges : &[ ( &str, &str ) ], isolated : &[ &str ] ) -> String
  {
    let mut out = String::from( "digraph chunks\n{\n" );
    for &( from, to ) in edges
    {
      let _ = writeln!( out, "  \"{from}\" -> \"{to}\";" );
    }
    for &name in isolated
    {
      let _ = writeln!( out, "  \"{name}\";" );
    }
    out.push( '}' );
    out
  }

  /// Renders `edges` ( from [`collect_edges`] ) plus any childless-root
  /// `isolated` names as a Mermaid `graph TD` flowchart. Chunk names are
  /// plain WGSL/Rust identifiers, so they need no quoting as Mermaid node
  /// IDs.
  fn mermaid_render( edges : &[ ( &str, &str ) ], isolated : &[ &str ] ) -> String
  {
    let mut out = String::from( "graph TD\n" );
    for &( from, to ) in edges
    {
      let _ = writeln!( out, "  {from} --> {to}" );
    }
    for &name in isolated
    {
      let _ = writeln!( out, "  {name}" );
    }
    out.trim_end().to_string()
  }

  /// Dependency tree for one chunk, or — with `name` absent — a forest of
  /// every chunk nothing else depends on. `reverse` flips the walk from
  /// "what this chunk depends on" to "what depends on this chunk": with
  /// `name` given, its dependents tree; with `name` absent, a forest
  /// rooted at every leaf chunk ( [`leaf_roots`] ) instead of every
  /// dependents-free root, since a reverse walk has to start somewhere
  /// with nothing beneath it. `format` selects the rendering shape —
  /// [`TreeFormat::Aligned`]'s indented text, or a
  /// [`TreeFormat::Dot`]/[`TreeFormat::Mermaid`] graph of the same
  /// roots/edges.
  ///
  /// # Errors
  ///
  /// Returns [`QueryError::UnknownChunk`] when `name` is `Some` and not found.
  pub fn chunk_tree( name : Option< &str >, reverse : bool, format : TreeFormat ) -> Result< String, QueryError >
  {
    let roots : Vec< &'static shader_chunks_core::ChunkDescriptor > = match name
    {
      Some( name ) => vec![ chunk_find( name )? ],
      None => if reverse { leaf_roots() } else { dependents_free_roots() },
    };

    let reverse_map = if reverse { Some( reverse_adjacency() ) } else { None };
    let children_of = | n : &str | -> Vec< &'static str >
    {
      match &reverse_map
      {
        Some( map ) => map.get( n ).cloned().unwrap_or_default(),
        None => shader_chunks_core::chunk_get( n ).map_or_else( Vec::new, | c | c.depends_on.to_vec() ),
      }
    };

    match format
    {
      TreeFormat::Aligned =>
      {
        let formatter = TreeFormatter::new();
        // Fix(BUG-284): render every forest root as a child of ONE shared invisible parent and
        // call format_aligned exactly once, instead of once per root joined by "\n".
        // Root cause: the old per-root loop called format_aligned separately for each root (each
        // wrapped as the SOLE child of its own invisible parent), then joined the resulting
        // strings with "\n" — but every per-root string already carried format_aligned's own
        // trailing "\n", so the join doubled it into a blank line between every pair of roots;
        // and since each root was always its invisible parent's only child, format_aligned always
        // rendered it with the "last sibling" connector (`└── `), even when more roots followed.
        // Pitfall: mapping a formatter call over each sibling and joining the strings defeats the
        // formatter's own sibling-position awareness (├── vs └──) and can double a trailing
        // separator the formatter already appends — give a formatter every sibling in one call
        // (one shared parent) whenever it is itself responsible for inter-sibling connectors.
        let mut invisible_parent = TreeNode::new( String::new(), None );
        for &chunk in &roots
        {
          // `format_aligned` never renders its own argument's `name`/`data` ( only
          // `show_root: true` would, and even then via bare `name` with no
          // column alignment ) — it only renders `children`. Wrapping every real
          // root as a child of ONE shared invisible, data-less parent makes each
          // root appear as a normal aligned row instead of being skipped.
          invisible_parent.children.push( dep_tree_node( chunk, &children_of ) );
        }
        Ok( formatter.format_aligned( &invisible_parent ) )
      },
      TreeFormat::Dot | TreeFormat::Mermaid =>
      {
        // Only a childless *root* needs an explicit bare-node declaration —
        // any other childless node reached during the walk already has an
        // incoming edge carrying it into the rendered graph.
        let isolated : Vec< &str > = roots.iter()
        .filter( | &&chunk | children_of( chunk.name ).is_empty() )
        .map( | chunk | chunk.name )
        .collect();

        let mut expanded = std::collections::HashSet::new();
        let mut edges = Vec::new();
        for &chunk in &roots
        {
          collect_edges( chunk, &children_of, &mut expanded, &mut edges );
        }
        Ok( if format == TreeFormat::Dot { dot_render( &edges, &isolated ) } else { mermaid_render( &edges, &isolated ) } )
      },
    }
  }
}

::mod_interface::mod_interface!
{
  own use QueryError;
  own use QUERY_FIELDS;
  own use TagsMode;
  own use SortKey;
  own use SortOrder;
  own use OutputFormat;
  own use TreeFormat;
  own use QueryParams;
  own use chunks_query;
  own use tags_list;
  own use chunk_tree;
}
