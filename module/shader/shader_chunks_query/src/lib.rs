//! Query utility CLI: the `list`, `get`, `tags`, and `tree` commands over
//! `shader_chunks_query_core`'s engine. Exposes its command set, help
//! groups, and help examples as data — parameterized by binary name — so
//! the `shader_chunks` aggregator folds them in unchanged, while
//! [`run`] serves the same commands as the standalone
//! `shader_chunks_query` binary.

mod private
{
  use unilang::prelude::*;
  use cli_fmt::prelude::*;
  use shader_chunks_cli_core::
  {
    CliApp, CommandSet, arg_bool_checked, arg_list, arg_string_checked, arg_usize, error_report, named_arg, text_output,
  };
  use shader_chunks_query_core::{ QueryError, QueryParams, TreeFormat };

  /// This utility's standalone binary name.
  pub const BINARY : &str = "shader_chunks_query";

  fn query_error( err : &QueryError ) -> ErrorData
  {
    let code = match err
    {
      QueryError::UnknownChunk( _ )
      | QueryError::UnknownField( _ )
      | QueryError::InvalidParam { .. } => ErrorCode::ValidationRuleFailed,
      QueryError::Render( _ ) => ErrorCode::InternalError,
    };
    error_report( err.exit_code(), code, err.to_string() )
  }

  /// The full parameter surface `list` and `get` both expose — identical
  /// arguments; the per-command defaults ( baked into each definition, and
  /// therefore into its help screen ) come from the `defaults` struct, which
  /// is the only thing distinguishing the two commands. `names` is
  /// positional and greedy ( `names_optional` is `true` for `list`, `false`
  /// for `get` ); every other parameter is passed as `key::value`.
  fn query_arguments( names_optional : bool, defaults : &QueryParams ) -> Vec< ArgumentDefinition >
  {
    vec!
    [
      ArgumentDefinition::former()
      .name( "names" )
      .kind( Kind::List( Box::new( Kind::String ), None ) )
      .hint( "Chunk names to select, in order (see `list`); duplicates allowed." )
      .attributes( ArgumentAttributes { optional : names_optional, multiple : true, ..ArgumentAttributes::default() } )
      .end(),
      named_arg( "pattern", Kind::String, "Substring filter on chunk names.", None ),
      named_arg( "case", Kind::Boolean, "Case-sensitive `pattern::`/`exports::`/`source::` matching.", Some( "false".to_string() ) ),
      named_arg( "tag", Kind::List( Box::new( Kind::String ), Some( ',' ) ), "Tag selectors, comma-separated: `group:tag` exact pair, bare `tag` any group.", None ),
      named_arg( "tags_mode", Kind::String, "Combine `tag::` selectors: any | all.", Some( defaults.tags_mode.as_str().to_string() ) ),
      named_arg( "stage", Kind::String, "Stage filter: any | none | a stage name.", Some( defaults.stage.clone() ) ),
      named_arg( "depends_on", Kind::String, "Keep only chunks depending on this chunk.", None ),
      named_arg( "transitive", Kind::Boolean, "Widen `depends_on::` to the transitive closure.", Some( "false".to_string() ) ),
      named_arg( "exports", Kind::String, "Substring filter over export signatures.", None ),
      named_arg( "source", Kind::String, "Substring filter over the chunk's raw WGSL body.", None ),
      named_arg( "roots", Kind::Boolean, "Keep only chunks nothing else depends on.", Some( "false".to_string() ) ),
      named_arg( "leaves", Kind::Boolean, "Keep only chunks with no dependencies.", Some( "false".to_string() ) ),
      named_arg( "fields", Kind::List( Box::new( Kind::String ), Some( ',' ) ), "Columns to project: name, description, stage, tags, depends_on, exports, source.", Some( defaults.fields.join( "," ) ) ),
      named_arg( "count", Kind::Boolean, "Print only the matched-chunk count.", Some( "false".to_string() ) ),
      named_arg( "format", Kind::String, "Output: table | markdown | expanded | json | yaml | names.", Some( defaults.format.as_str().to_string() ) ),
      named_arg( "sort", Kind::String, "Sort key: input | name | stage | description.", Some( defaults.sort.as_str().to_string() ) ),
      named_arg( "order", Kind::String, "Sort direction: asc | desc.", Some( defaults.order.as_str().to_string() ) ),
      named_arg( "limit", Kind::Integer, "Keep at most N chunks; 0 = unlimited.", Some( "0".to_string() ) ),
      named_arg( "offset", Kind::Integer, "Skip the first N chunks.", Some( "0".to_string() ) ),
      named_arg( "heading", Kind::String, "Heading line above the table (table/markdown formats only).", None ),
      named_arg( "width", Kind::Integer, "Max column width (table/markdown formats only); 0 = auto.", Some( "0".to_string() ) ),
    ]
  }

  // Fix(BUG-285): every `arg_string`/`arg_bool` call in this function and in
  // `cmd_tree`'s routine switched to `arg_string_checked`/`arg_bool_checked`.
  // Root cause: same defect class as BUG-283 (`shader_chunks_cli_core`'s
  // `arg_string`/`arg_bool` catch-all arms cannot tell "argument absent"
  // apart from "argument supplied twice", since `unilang` binds ANY
  // repeated named argument to `Value::List` regardless of its declared
  // `multiple` attribute) -- BUG-283 fixed `shader_chunks_compose`'s two
  // call sites but left every other CLI crate's call sites unchecked; this
  // is the first of those, `shader_chunks_query`'s 17 (15 in this function,
  // 2 in `cmd_tree`). Every named parameter here was previously silently
  // misread on a duplicate: `pattern::a pattern::b` fell through to "no
  // filter" and matched every chunk instead of erroring.
  // Pitfall: `arg_usize` has the identical catch-all shape (`_ => Ok(0)`,
  // `shader_chunks_cli_core/src/lib.rs`) and is NOT yet fixed here or
  // anywhere else -- still a known gap, same as BUG-283 left it.
  /// Builds [`QueryParams`] from a verified command's bound arguments on
  /// top of `params` ( a per-command defaults struct ). Enum-style values
  /// ( `tags_mode`/`format`/`sort`/`order` ) and negative integers fail
  /// loudly here as `InvalidParam`.
  fn query_params_from
  (
    cmd : &VerifiedCommand,
    mut params : QueryParams,
  ) -> Result< QueryParams, ErrorData >
  {
    params.names = arg_list( cmd, "names" );
    if let Some( pattern ) = arg_string_checked( cmd, "pattern" )? { params.pattern = pattern; }
    params.case_sensitive = arg_bool_checked( cmd, "case", params.case_sensitive )?;
    params.tags = arg_list( cmd, "tag" );
    if let Some( mode ) = arg_string_checked( cmd, "tags_mode" )?
    { params.tags_mode = mode.parse().map_err( | e | query_error( &e ) )?; }
    if let Some( stage ) = arg_string_checked( cmd, "stage" )? { params.stage = stage; }
    if let Some( depends_on ) = arg_string_checked( cmd, "depends_on" )? { params.depends_on = depends_on; }
    params.transitive = arg_bool_checked( cmd, "transitive", params.transitive )?;
    if let Some( exports ) = arg_string_checked( cmd, "exports" )? { params.exports = exports; }
    if let Some( source ) = arg_string_checked( cmd, "source" )? { params.source = source; }
    params.roots = arg_bool_checked( cmd, "roots", params.roots )?;
    params.leaves = arg_bool_checked( cmd, "leaves", params.leaves )?;
    let fields = arg_list( cmd, "fields" );
    if !fields.is_empty() { params.fields = fields; }
    params.count = arg_bool_checked( cmd, "count", params.count )?;
    if let Some( format ) = arg_string_checked( cmd, "format" )?
    { params.format = format.parse().map_err( | e | query_error( &e ) )?; }
    if let Some( sort ) = arg_string_checked( cmd, "sort" )?
    { params.sort = sort.parse().map_err( | e | query_error( &e ) )?; }
    if let Some( order ) = arg_string_checked( cmd, "order" )?
    { params.order = order.parse().map_err( | e | query_error( &e ) )?; }
    params.limit = arg_usize( cmd, "limit" )?;
    params.offset = arg_usize( cmd, "offset" )?;
    if let Some( heading ) = arg_string_checked( cmd, "heading" )? { params.heading = heading; }
    params.width = arg_usize( cmd, "width" )?;
    Ok( params )
  }

  /// The one routine body behind both `list` and `get` — both commands
  /// execute exactly this function over
  /// [`shader_chunks_query_core::chunks_query`], differing only in the
  /// `defaults` constructor passed in.
  fn query_routine( defaults : fn() -> QueryParams ) -> CommandRoutine
  {
    Box::new( move | cmd, _ctx |
    {
      let params = query_params_from( &cmd, defaults() )?;
      let content = shader_chunks_query_core::chunks_query( &params ).map_err( | e | query_error( &e ) )?;
      Ok( text_output( content ) )
    })
  }

  fn cmd_list( binary : &str ) -> ( CommandDefinition, CommandRoutine )
  {
    let defaults = QueryParams::list_defaults();
    let def = CommandDefinition::former()
    .name( ".list" )
    .namespace( String::new() )
    .description( "Query bundled chunks: filter, sort, project, and format — every chunk by default.".to_string() )
    .hint( "chunk query with overview columns and a plain table by default" )
    .status( "stable" )
    .version( "1.0.0".to_string() )
    .aliases( vec![] )
    .tags( vec![] )
    .permissions( vec![] )
    .idempotent( true )
    .deprecation_message( String::new() )
    .http_method_hint( String::new() )
    .examples( vec!
    [
      format!( "{binary} list" ),
      format!( "{binary} list pattern::noise" ),
      format!( "{binary} list tag::noise format::json" ),
      format!( "{binary} list roots::1 fields::name,exports" ),
      format!( "{binary} list source::fract format::names" ),
    ])
    .arguments( query_arguments( true, &defaults ) )
    .end();

    ( def, query_routine( QueryParams::list_defaults ) )
  }

  fn cmd_get( binary : &str ) -> ( CommandDefinition, CommandRoutine )
  {
    let defaults = QueryParams::get_defaults();
    let def = CommandDefinition::former()
    .name( ".get" )
    .namespace( String::new() )
    .description( "Query named chunks with the same engine and parameters as `list` — detail columns, expanded records by default.".to_string() )
    .hint( "same query engine as `list`; names required, expanded detail by default" )
    .status( "stable" )
    .version( "1.0.0".to_string() )
    .aliases( vec![] )
    .tags( vec![] )
    .permissions( vec![] )
    .idempotent( true )
    .deprecation_message( String::new() )
    .http_method_hint( String::new() )
    .examples( vec!
    [
      format!( "{binary} get hash21" ),
      format!( "{binary} get hash21 fbm3 fields::name,source format::yaml" ),
    ])
    .arguments( query_arguments( false, &defaults ) )
    .end();

    ( def, query_routine( QueryParams::get_defaults ) )
  }

  fn cmd_tags( binary : &str ) -> ( CommandDefinition, CommandRoutine )
  {
    let def = CommandDefinition::former()
    .name( ".tags" )
    .namespace( String::new() )
    .description( "List every distinct tag and the chunks carrying it.".to_string() )
    .hint( "group:tag pairs and their carrying chunk(s)" )
    .status( "stable" )
    .version( "1.0.0".to_string() )
    .aliases( vec![] )
    .tags( vec![] )
    .permissions( vec![] )
    .idempotent( true )
    .deprecation_message( String::new() )
    .http_method_hint( String::new() )
    .examples( vec![ format!( "{binary} tags" ) ] )
    .arguments( vec![] )
    .end();

    let routine : CommandRoutine = Box::new( | _cmd, _ctx |
    {
      let content = shader_chunks_query_core::tags_list().map_err( | e | query_error( &e ) )?;
      Ok( text_output( content ) )
    });

    ( def, routine )
  }

  fn cmd_tree( binary : &str ) -> ( CommandDefinition, CommandRoutine )
  {
    let def = CommandDefinition::former()
    .name( ".tree" )
    .namespace( String::new() )
    .description( "Show the dependency tree for one chunk, or a forest of every root chunk; reverse::1 walks dependents instead.".to_string() )
    .hint( "dependency tree for one chunk, or every root chunk with no argument" )
    .status( "stable" )
    .version( "1.0.0".to_string() )
    .aliases( vec![] )
    .tags( vec![] )
    .permissions( vec![] )
    .idempotent( true )
    .deprecation_message( String::new() )
    .http_method_hint( String::new() )
    .examples( vec!
    [
      format!( "{binary} tree fbm3" ),
      format!( "{binary} tree" ),
      format!( "{binary} tree hash21 reverse::1" ),
      format!( "{binary} tree fbm3 shape::dot" ),
    ])
    .arguments( vec!
    [
      ArgumentDefinition::former()
      .name( "name" )
      .kind( Kind::String )
      .hint( "Chunk name (see `list`); omit for the full forest." )
      .attributes( ArgumentAttributes { optional : true, ..ArgumentAttributes::default() } )
      .end(),
      named_arg( "reverse", Kind::Boolean, "Walk dependents instead of dependencies: what (transitively) depends on this chunk.", Some( "false".to_string() ) ),
      named_arg( "shape", Kind::String, "Rendering shape: aligned (indented text), dot (Graphviz digraph), mermaid (Mermaid graph TD).", Some( "aligned".to_string() ) ),
    ])
    .end();

    let routine : CommandRoutine = Box::new( | cmd, _ctx |
    {
      let name = match cmd.arguments.get( "name" )
      {
        Some( Value::String( name ) ) => Some( name.as_str() ),
        _ => None,
      };
      let reverse = arg_bool_checked( &cmd, "reverse", false )?;
      let shape = match arg_string_checked( &cmd, "shape" )?
      {
        Some( s ) => s.parse().map_err( | e | query_error( &e ) )?,
        None => TreeFormat::Aligned,
      };
      let content = shader_chunks_query_core::chunk_tree( name, reverse, shape ).map_err( | e | query_error( &e ) )?;
      Ok( text_output( content ) )
    });

    ( def, routine )
  }

  /// This utility's command set — `list`, `get`, `tags`, `tree` — with
  /// example invocations spelled against `binary`.
  #[ must_use ]
  pub fn commands( binary : &str ) -> CommandSet
  {
    vec![ cmd_list( binary ), cmd_get( binary ), cmd_tags( binary ), cmd_tree( binary ) ]
  }

  /// This utility's help-screen groups: `Query` ( `list`/`get`/`tags` ) and
  /// `Graph` ( `tree` ) — same names and membership as
  /// `docs/cli/command_group/` documents for the aggregator.
  #[ must_use ]
  pub fn help_groups() -> Vec< CommandGroup >
  {
    vec!
    [
      CommandGroup
      {
        name : "Query".to_string(),
        entries : vec!
        [
          CommandEntry { name : "list [names...]".to_string(), desc : "Query chunks: filter, sort, project, format (plain table).".to_string() },
          CommandEntry { name : "get <names...>".to_string(), desc : "Same query engine, detail fields, expanded records.".to_string() },
          CommandEntry { name : "tags".to_string(), desc : "List every distinct tag and its chunk(s).".to_string() },
        ],
      },
      CommandGroup
      {
        name : "Graph".to_string(),
        entries : vec!
        [
          CommandEntry { name : "tree [name]".to_string(), desc : "Show a chunk's dependency tree, or the full forest.".to_string() },
        ],
      },
    ]
  }

  /// This utility's help-screen example invocations, spelled against
  /// `binary`.
  #[ must_use ]
  pub fn help_examples( binary : &str ) -> Vec< ExampleEntry >
  {
    vec!
    [
      ExampleEntry { invocation : format!( "{binary} list tag::noise format::json" ), desc : None },
      ExampleEntry { invocation : format!( "{binary} get hash21" ), desc : None },
    ]
  }

  /// Standalone entry point for the `shader_chunks_query` binary.
  pub fn run()
  {
    shader_chunks_cli_core::run( CliApp
    {
      binary : BINARY.to_string(),
      tagline : "Query and inspect shader_chunks_core's bundled WGSL chunks.".to_string(),
      groups : help_groups(),
      examples : help_examples( BINARY ),
      commands : commands( BINARY ),
    });
  }
}

::mod_interface::mod_interface!
{
  own use BINARY;
  own use commands;
  own use help_groups;
  own use help_examples;
  own use run;
}
