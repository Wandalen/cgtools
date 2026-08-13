//! CLI wiring layer shared by the `shader_chunks` and `sch` binaries:
//! builds the `unilang` `CommandRegistry`, dispatches via `Pipeline`, and
//! maps a [`crate::CliError`] to a process exit code. All rendering and
//! business logic lives in the crate root — this layer only wires it to
//! `unilang`'s dispatch and argv/exit-code plumbing, routes the
//! conventional help spellings (`help`, `.`, `<command> help`) to
//! `cli_fmt`-rendered help screens, and prints `Pipeline` outputs centrally
//! through pipe-safe write helpers.

mod private
{
  use std::sync::atomic::{ AtomicI32, Ordering };
  use unilang::prelude::*;
  use cli_fmt::prelude::*;

  /// Exit code stashed by a routine just before it returns a [`CliError`]
  /// wrapped as [`ErrorData`]. `CommandResult.error` only ever carries a
  /// flattened `String` — `ErrorData`'s `Display` impl prints `message` alone,
  /// never `code` — so this is the only channel through which a specific
  /// [`CliError::exit_code`] can reach [`run`] after `Pipeline` dispatch
  /// completes. Defaults to `1`, matching the framework-level failures
  /// (unknown command, missing argument) that never touch a [`CliError`] at
  /// all — both are caller-fixable invocation mistakes, the same class
  /// [`CliError::exit_code`] maps to `1`.
  ///
  /// [`CliError`]: crate::CliError
  /// [`CliError::exit_code`]: crate::CliError::exit_code
  static EXIT_CODE : AtomicI32 = AtomicI32::new( 1 );

  // Fix(BUG-108): every user-facing write goes through `print_stdout` /
  // `print_stderr` instead of `println!`/`eprintln!`.
  // Root cause: the std print macros panic on any write error — including
  // `EPIPE` once a pipeline reader like `head` has exited — and `Stdout`'s
  // line buffering makes the first post-hangup write fail deterministically,
  // so `sch list | true` aborted with a backtrace and exit 101 against the
  // crate's documented "never a panic" contract.
  // Pitfall: a broken pipe is not an application error — the reader chose to
  // stop reading; quiet exit 0 is the Unix convention (`head` closing `cat`).
  // Only a non-`EPIPE` stdout failure is real, and stderr reporting must
  // swallow its own write errors or it becomes a second panic path.

  /// Writes one line to stdout, pipe-safely: a closed pipe (`EPIPE`) ends
  /// the process quietly with exit 0; any other write failure reports on
  /// stderr (best-effort) and exits 2.
  fn print_stdout( content : &str )
  {
    use std::io::Write;
    let mut stdout = std::io::stdout().lock();
    if let Err( error ) = writeln!( stdout, "{content}" )
    {
      if error.kind() == std::io::ErrorKind::BrokenPipe
      {
        std::process::exit( 0 );
      }
      print_stderr( &format!( "failed writing to stdout: {error}" ) );
      std::process::exit( 2 );
    }
  }

  /// Writes one line to stderr, discarding write errors — error reporting
  /// must never itself become a second failure path (see BUG-108).
  fn print_stderr( content : &str )
  {
    use std::io::Write;
    let _ = writeln!( std::io::stderr(), "{content}" );
  }

  fn cli_error( err : &crate::CliError ) -> ErrorData
  {
    EXIT_CODE.store( err.exit_code(), Ordering::Relaxed );
    let code = match err
    {
      crate::CliError::UnknownChunk( _ )
      | crate::CliError::UnknownField( _ )
      | crate::CliError::InvalidParam { .. }
      | crate::CliError::Compose( _ ) => ErrorCode::ValidationRuleFailed,
      crate::CliError::Render( _ ) => ErrorCode::InternalError,
    };
    ErrorData::new( code, err.to_string() )
  }

  fn text_output( content : String ) -> OutputData
  {
    OutputData { content, format : "text".to_string(), execution_time_ms : None }
  }

  /// Recursively flattens a bound list argument value into plain strings.
  ///
  /// `unilang`'s positional binding for a `multiple: true` + `Kind::List`
  /// argument (`names` on `list`/`get`/`compose`) coerces each raw argv token
  /// against the *outer* `Kind::List` type rather than its inner
  /// `Kind::String` element type, so a two-token invocation like
  /// `compose hash21 value_noise` binds as
  /// `List([List([String("hash21")]), List([String("value_noise")])])`
  /// rather than the flat `List([String("hash21"), String("value_noise")])`
  /// one might expect — flattening here is what makes both shapes (and the
  /// comma-delimited named lists `tag::`/`fields::`) resolve to the same
  /// `Vec<String>`.
  fn flatten_names( value : &Value ) -> Vec< String >
  {
    match value
    {
      Value::String( s ) => vec![ s.clone() ],
      Value::List( values ) => values.iter().flat_map( flatten_names ).collect(),
      _ => vec![],
    }
  }

  /// One optional named (`key::value`) query parameter definition.
  fn named_arg( name : &str, kind : Kind, hint : &str, default : Option< String > ) -> ArgumentDefinition
  {
    ArgumentDefinition::former()
    .name( name )
    .kind( kind )
    .hint( hint )
    .attributes( ArgumentAttributes { optional : true, default, ..ArgumentAttributes::default() } )
    .end()
  }

  /// The full parameter surface `list` and `get` both expose — identical
  /// arguments; the per-command defaults (baked into each definition, and
  /// therefore into its help screen) come from the `defaults` struct, which is
  /// the only thing distinguishing the two commands. `names` is positional and
  /// greedy (`names_optional` is `true` for `list`, `false` for `get`); every
  /// other parameter is passed as `key::value`.
  fn query_arguments( names_optional : bool, defaults : &crate::QueryParams ) -> Vec< ArgumentDefinition >
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
      named_arg( "case", Kind::Boolean, "Case-sensitive `pattern::`/`exports::` matching.", Some( "false".to_string() ) ),
      named_arg( "tag", Kind::List( Box::new( Kind::String ), Some( ',' ) ), "Tag selectors, comma-separated: `group:tag` exact pair, bare `tag` any group.", None ),
      named_arg( "tags_mode", Kind::String, "Combine `tag::` selectors: any | all.", Some( defaults.tags_mode.as_str().to_string() ) ),
      named_arg( "stage", Kind::String, "Stage filter: any | none | a stage name.", Some( defaults.stage.clone() ) ),
      named_arg( "depends_on", Kind::String, "Keep only chunks depending on this chunk.", None ),
      named_arg( "transitive", Kind::Boolean, "Widen `depends_on::` to the transitive closure.", Some( "false".to_string() ) ),
      named_arg( "exports", Kind::String, "Substring filter over export signatures.", None ),
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

  fn arg_string( cmd : &VerifiedCommand, key : &str ) -> Option< String >
  {
    match cmd.arguments.get( key )
    {
      Some( Value::String( s ) | Value::Enum( s ) ) => Some( s.clone() ),
      _ => None,
    }
  }

  fn arg_bool( cmd : &VerifiedCommand, key : &str, default : bool ) -> bool
  {
    match cmd.arguments.get( key )
    {
      Some( Value::Boolean( flag ) ) => *flag,
      _ => default,
    }
  }

  /// Extracts a non-negative integer parameter; a negative value fails loudly
  /// as [`crate::CliError::InvalidParam`] rather than wrapping.
  fn arg_usize( cmd : &VerifiedCommand, key : &'static str ) -> Result< usize, ErrorData >
  {
    match cmd.arguments.get( key )
    {
      Some( Value::Integer( n ) ) => usize::try_from( *n ).map_err( | _ | cli_error(
        &crate::CliError::InvalidParam
        {
          param : key,
          value : n.to_string(),
          allowed : "a non-negative integer",
        })),
      _ => Ok( 0 ),
    }
  }

  fn arg_list( cmd : &VerifiedCommand, key : &str ) -> Vec< String >
  {
    cmd.arguments.get( key ).map( flatten_names ).unwrap_or_default()
  }

  /// Builds [`crate::QueryParams`] from a verified command's bound
  /// arguments on top of `params` (a per-command defaults struct). Enum-style
  /// values (`tags_mode`/`format`/`sort`/`order`) and negative integers fail
  /// loudly here as `InvalidParam`.
  fn query_params_from
  (
    cmd : &VerifiedCommand,
    mut params : crate::QueryParams,
  ) -> Result< crate::QueryParams, ErrorData >
  {
    params.names = arg_list( cmd, "names" );
    if let Some( pattern ) = arg_string( cmd, "pattern" ) { params.pattern = pattern; }
    params.case_sensitive = arg_bool( cmd, "case", params.case_sensitive );
    params.tags = arg_list( cmd, "tag" );
    if let Some( mode ) = arg_string( cmd, "tags_mode" )
    { params.tags_mode = mode.parse().map_err( | e | cli_error( &e ) )?; }
    if let Some( stage ) = arg_string( cmd, "stage" ) { params.stage = stage; }
    if let Some( depends_on ) = arg_string( cmd, "depends_on" ) { params.depends_on = depends_on; }
    params.transitive = arg_bool( cmd, "transitive", params.transitive );
    if let Some( exports ) = arg_string( cmd, "exports" ) { params.exports = exports; }
    params.roots = arg_bool( cmd, "roots", params.roots );
    params.leaves = arg_bool( cmd, "leaves", params.leaves );
    let fields = arg_list( cmd, "fields" );
    if !fields.is_empty() { params.fields = fields; }
    params.count = arg_bool( cmd, "count", params.count );
    if let Some( format ) = arg_string( cmd, "format" )
    { params.format = format.parse().map_err( | e | cli_error( &e ) )?; }
    if let Some( sort ) = arg_string( cmd, "sort" )
    { params.sort = sort.parse().map_err( | e | cli_error( &e ) )?; }
    if let Some( order ) = arg_string( cmd, "order" )
    { params.order = order.parse().map_err( | e | cli_error( &e ) )?; }
    params.limit = arg_usize( cmd, "limit" )?;
    params.offset = arg_usize( cmd, "offset" )?;
    if let Some( heading ) = arg_string( cmd, "heading" ) { params.heading = heading; }
    params.width = arg_usize( cmd, "width" )?;
    Ok( params )
  }

  /// The one routine body behind both `list` and `get` — both commands execute
  /// exactly this function over [`crate::query_chunks`], differing
  /// only in the `defaults` constructor passed in.
  fn query_routine( defaults : fn() -> crate::QueryParams ) -> CommandRoutine
  {
    Box::new( move | cmd, _ctx |
    {
      let params = query_params_from( &cmd, defaults() )?;
      let content = crate::query_chunks( &params ).map_err( | e | cli_error( &e ) )?;
      Ok( text_output( content ) )
    })
  }

  fn cmd_list() -> ( CommandDefinition, CommandRoutine )
  {
    let defaults = crate::QueryParams::list_defaults();
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
      "shader_chunks list".to_string(),
      "shader_chunks list pattern::noise".to_string(),
      "shader_chunks list tag::noise format::json".to_string(),
      "shader_chunks list roots::1 fields::name,exports".to_string(),
    ])
    .arguments( query_arguments( true, &defaults ) )
    .end();

    ( def, query_routine( crate::QueryParams::list_defaults ) )
  }

  fn cmd_get() -> ( CommandDefinition, CommandRoutine )
  {
    let defaults = crate::QueryParams::get_defaults();
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
      "shader_chunks get hash21".to_string(),
      "shader_chunks get hash21 fbm3 fields::name,source format::yaml".to_string(),
    ])
    .arguments( query_arguments( false, &defaults ) )
    .end();

    ( def, query_routine( crate::QueryParams::get_defaults ) )
  }

  fn cmd_tags() -> ( CommandDefinition, CommandRoutine )
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
    .examples( vec![ "shader_chunks tags".to_string() ] )
    .arguments( vec![] )
    .end();

    let routine : CommandRoutine = Box::new( | _cmd, _ctx |
    {
      let content = crate::list_tags().map_err( | e | cli_error( &e ) )?;
      Ok( text_output( content ) )
    });

    ( def, routine )
  }

  fn cmd_tree() -> ( CommandDefinition, CommandRoutine )
  {
    let def = CommandDefinition::former()
    .name( ".tree" )
    .namespace( String::new() )
    .description( "Show the dependency tree for one chunk, or a forest of every root chunk.".to_string() )
    .hint( "dependency tree for one chunk, or every root chunk with no argument" )
    .status( "stable" )
    .version( "1.0.0".to_string() )
    .aliases( vec![] )
    .tags( vec![] )
    .permissions( vec![] )
    .idempotent( true )
    .deprecation_message( String::new() )
    .http_method_hint( String::new() )
    .examples( vec![ "shader_chunks tree fbm3".to_string(), "shader_chunks tree".to_string() ] )
    .arguments( vec!
    [
      ArgumentDefinition::former()
      .name( "name" )
      .kind( Kind::String )
      .hint( "Chunk name (see `list`); omit for the full forest." )
      .attributes( ArgumentAttributes { optional : true, ..ArgumentAttributes::default() } )
      .end(),
    ])
    .end();

    let routine : CommandRoutine = Box::new( | cmd, _ctx |
    {
      let name = match cmd.arguments.get( "name" )
      {
        Some( Value::String( name ) ) => Some( name.as_str() ),
        _ => None,
      };
      let content = crate::tree_chunk( name ).map_err( | e | cli_error( &e ) )?;
      Ok( text_output( content ) )
    });

    ( def, routine )
  }

  fn cmd_compose() -> ( CommandDefinition, CommandRoutine )
  {
    let def = CommandDefinition::former()
    .name( ".compose" )
    .namespace( String::new() )
    .description( "Preview WGSL composed from one or more chunks, dependency-ordered.".to_string() )
    .hint( "composed WGSL text for the given chunks, in dependency order" )
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
      "shader_chunks compose hash21 value_noise".to_string(),
      "shader_chunks compose fbm3 transitive::1".to_string(),
    ])
    .arguments( vec!
    [
      ArgumentDefinition::former()
      .name( "names" )
      .kind( Kind::List( Box::new( Kind::String ), None ) )
      .hint( "One or more chunk names (see `list`)." )
      .attributes( ArgumentAttributes { multiple : true, ..ArgumentAttributes::default() } )
      .end(),
      named_arg( "transitive", Kind::Boolean, "Widen the named set to its full dependency closure.", Some( "false".to_string() ) ),
    ])
    .end();

    let routine : CommandRoutine = Box::new( | cmd, _ctx |
    {
      let names : Vec< String > = match cmd.arguments.get( "names" )
      {
        Some( value ) => flatten_names( value ),
        None => unreachable!( "argument 'names' is declared Kind::List, multiple, and required" ),
      };
      let transitive = arg_bool( &cmd, "transitive", false );
      let content = crate::compose_chunks( &names, transitive ).map_err( | e | cli_error( &e ) )?;
      Ok( text_output( content ) )
    });

    ( def, routine )
  }

  fn cmd_tunables() -> ( CommandDefinition, CommandRoutine )
  {
    let def = CommandDefinition::former()
    .name( ".tunables" )
    .namespace( String::new() )
    .description( "List every tunable parameter a chunk declares via `//@ param:` lines.".to_string() )
    .hint( "name, kind, type, range, and range source for one chunk's declared tunables" )
    .status( "stable" )
    .version( "1.0.0".to_string() )
    .aliases( vec![] )
    .tags( vec![] )
    .permissions( vec![] )
    .idempotent( true )
    .deprecation_message( String::new() )
    .http_method_hint( String::new() )
    .examples( vec![ "shader_chunks tunables fbm3".to_string() ] )
    .arguments( vec!
    [
      ArgumentDefinition::former()
      .name( "name" )
      .kind( Kind::String )
      .hint( "Chunk name (see `list`)." )
      .attributes( ArgumentAttributes::default() )
      .end(),
    ])
    .end();

    let routine : CommandRoutine = Box::new( | cmd, _ctx |
    {
      let name = match cmd.arguments.get( "name" )
      {
        Some( Value::String( name ) ) => name.clone(),
        _ => unreachable!( "argument 'name' is declared Kind::String and required" ),
      };
      let content = crate::tunables( &name ).map_err( | e | cli_error( &e ) )?;
      Ok( text_output( content ) )
    });

    ( def, routine )
  }

  fn build_registry() -> CommandRegistry
  {
    let mut registry = CommandRegistry::new();
    for ( def, routine ) in [ cmd_list(), cmd_get(), cmd_tags(), cmd_tree(), cmd_compose(), cmd_tunables() ]
    {
      registry.register_with_routine( &def, routine ).expect( "hand-written static command definitions are well-formed" );
    }
    registry
  }

  fn print_help()
  {
    let mut data = CliHelpData::default();
    data.binary = "shader_chunks".to_string();
    data.tagline = "Inspect and compose shader_chunks_core's bundled WGSL chunks.".to_string();
    data.groups = vec!
    [
      // Grouping mirrors docs/cli/command_group/: one CommandGroup per doc
      // instance, same names, same membership.
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
      CommandGroup
      {
        name : "Compose".to_string(),
        entries : vec!
        [
          CommandEntry { name : "compose <names...>".to_string(), desc : "Preview composed WGSL for the given chunks.".to_string() },
        ],
      },
      CommandGroup
      {
        name : "Parameters".to_string(),
        entries : vec!
        [
          CommandEntry { name : "tunables <name>".to_string(), desc : "List a chunk's tunable parameters: kind, type, range, source.".to_string() },
        ],
      },
    ];
    data.examples = vec!
    [
      ExampleEntry { invocation : "shader_chunks list tag::noise format::json".to_string(), desc : None },
      ExampleEntry { invocation : "shader_chunks get hash21".to_string(), desc : None },
      ExampleEntry { invocation : "shader_chunks compose hash21 value_noise".to_string(), desc : None },
      ExampleEntry { invocation : "shader_chunks tunables fbm3".to_string(), desc : None },
    ];

    let template = CliHelpTemplate::new( CliHelpStyle::default(), data );
    print_stdout( &template.render() );
  }

  /// Renders per-command help for `command` (bare name, no leading dot) from
  /// its registered [`CommandDefinition`], through the same `cli_fmt` template
  /// and style as the top-level [`print_help`] screen. The first declared
  /// argument is the command's positional one and is spelled by shape
  /// (`<name>` required, `[name]` optional, `...` repeatable); every further
  /// argument is named-only and rendered as a `key::` row carrying its hint
  /// and declared default, with a `[param::value ...]` marker appended to the
  /// usage line. Examples come from the definition itself.
  fn print_command_help( command : &str, def : &CommandDefinition )
  {
    let mut syntax = command.to_string();
    let mut argument_rows = Vec::new();
    for ( index, arg ) in def.arguments().iter().enumerate()
    {
      if index == 0
      {
        let shape = match ( arg.attributes.multiple, arg.attributes.optional )
        {
          ( true, true ) => format!( "[{}...]", arg.name ),
          ( true, false ) => format!( "<{}...>", arg.name ),
          ( false, true ) => format!( "[{}]", arg.name ),
          ( false, false ) => format!( "<{}>", arg.name ),
        };
        syntax.push( ' ' );
        syntax.push_str( &shape );
        argument_rows.push( CommandEntry { name : shape, desc : arg.hint.clone() } );
      }
      else
      {
        let desc = match &arg.attributes.default
        {
          Some( default ) => format!( "{} [default: {default}]", arg.hint ),
          None => arg.hint.clone(),
        };
        argument_rows.push( CommandEntry { name : format!( "{}::", arg.name ), desc } );
      }
    }
    if def.arguments().len() > 1
    {
      syntax.push_str( " [param::value ...]" );
    }

    let mut data = CliHelpData::default();
    data.usage_lines = vec![ format!( "Usage: shader_chunks {syntax}" ) ];
    data.tagline = def.description().to_string();
    data.groups = vec![ CommandGroup { name : syntax, entries : argument_rows } ];
    data.examples = def.examples().iter()
    .map( | invocation | ExampleEntry { invocation : invocation.clone(), desc : None } )
    .collect();

    let template = CliHelpTemplate::new( CliHelpStyle::default(), data );
    print_stdout( &template.render() );
  }

  /// The whole CLI: parses argv, routes help spellings, dispatches through
  /// `unilang`'s `Pipeline`, prints outputs, and exits non-zero on failure.
  /// Both `src/bin/` entry points delegate here — keeping them byte-level
  /// trivial is what guarantees `sch` and `shader_chunks` behave identically.
  pub fn run()
  {
    let mut argv : Vec< String > = std::env::args().skip( 1 ).collect();

    // Fix(BUG-103): route conventional help spellings to `cli_fmt`-rendered
    // help screens and print `result.outputs` centrally instead of leaving
    // all printing to command routines.
    // Root cause: `main` never printed `result.outputs`, so every
    // framework-generated help path (`.`, `.help`, `?`/`??`, `.{command}.help`)
    // succeeded with zero bytes of output — routines only printed their own
    // success content — and no mapping existed from the conventional `help` /
    // `<command> help` spellings to those framework forms, so a trailing
    // `help` bound as an ordinary positional argument (`compose help` →
    // "unknown chunk: `help`").
    // Pitfall: in a routines-print-themselves `unilang` setup, anything the
    // framework answers on its own (help listings, per-command help) is
    // invisible until the entry point prints `result.outputs` — wire that
    // first, then render known-command help via `cli_fmt` and route only
    // unknown targets through `.{target}.help` so they still fail loudly.

    // `sch`, `sch .`, `sch help`, `sch .help` — all spell "top-level help".
    if argv.is_empty() || ( argv.len() == 1 && matches!( argv[ 0 ].as_str(), "." | "help" | ".help" ) )
    {
      print_help();
      return;
    }

    // `sch help <command>` / `sch <command> ... help` — per-command help,
    // rendered by `cli_fmt` from the command's own registered definition; an
    // unknown target falls through to the `.{target}.help` rewrite so it
    // still fails loudly through the unknown-command path.
    // Only a token that is exactly `help` is a help request — named-argument
    // spellings (`name::help`) pass through untouched.
    let help_target = if matches!( argv[ 0 ].as_str(), "help" | ".help" )
    {
      Some( argv[ 1 ].clone() )
    }
    else if argv.len() >= 2 && argv[ argv.len() - 1 ] == "help"
    {
      Some( argv[ 0 ].clone() )
    }
    else
    {
      None
    };

    let registry = build_registry();

    if let Some( target ) = help_target
    {
      let target = target.strip_prefix( '.' ).unwrap_or( target.as_str() );
      if target.is_empty() || target == "help"
      {
        print_help();
        return;
      }
      if let Some( def ) = registry.commands().get( &format!( ".{target}" ) )
      {
        print_command_help( target, def );
        return;
      }
      argv = vec![ format!( ".{target}.help" ) ];
    }

    // unilang's own argv-parsing tests (tests/system/argv_api.rs) only ever
    // exercise leading-dot command tokens (`.echo`, `.run`) — a bare `list`
    // typed by a CLI user is never dot-prefixed, so it must be normalized
    // before reaching the pipeline.
    if let Some( first ) = argv.first_mut()
      && !first.starts_with( '.' )
    {
      *first = format!( ".{first}" );
    }

    let pipeline = Pipeline::new( registry );
    let result = pipeline.process_command_from_argv_simple( &argv );

    if !result.success
    {
      if let Some( error ) = &result.error
      {
        print_stderr( error );
      }
      std::process::exit( EXIT_CODE.load( Ordering::Relaxed ) );
    }

    for output in &result.outputs
    {
      if !output.content.is_empty()
      {
        print_stdout( &output.content );
      }
    }
  }
}

crate::mod_interface!
{
  own use run;
}
