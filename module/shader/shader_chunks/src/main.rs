//! Thin entry point: builds the `unilang` `CommandRegistry`, dispatches via
//! `Pipeline`, and maps a [`shader_chunks::CliError`] to a process exit
//! code. All rendering and business logic lives in `src/lib.rs` — this file
//! only wires it to `unilang`'s dispatch and argv/exit-code plumbing, maps
//! the conventional help spellings (`help`, `.`, `<command> help`) onto
//! `unilang`'s help machinery, and prints `Pipeline` outputs centrally.

use std::sync::atomic::{ AtomicI32, Ordering };
use unilang::prelude::*;
use cli_fmt::prelude::*;

/// Exit code stashed by a routine just before it returns a [`CliError`]
/// wrapped as [`ErrorData`]. `CommandResult.error` only ever carries a
/// flattened `String` — `ErrorData`'s `Display` impl prints `message` alone,
/// never `code` — so this is the only channel through which a specific
/// [`CliError::exit_code`] can reach `main` after `Pipeline` dispatch
/// completes. Defaults to `1`, matching the framework-level failures
/// (unknown command, missing argument) that never touch a [`CliError`] at
/// all — both are caller-fixable invocation mistakes, the same class
/// [`shader_chunks::CliError::exit_code`] maps to `1`.
static EXIT_CODE : AtomicI32 = AtomicI32::new( 1 );

fn cli_error( err : &shader_chunks::CliError ) -> ErrorData
{
  EXIT_CODE.store( err.exit_code(), Ordering::Relaxed );
  let code = match err
  {
    shader_chunks::CliError::UnknownChunk( _ ) | shader_chunks::CliError::Compose( _ ) => ErrorCode::ValidationRuleFailed,
    shader_chunks::CliError::Render( _ ) => ErrorCode::InternalError,
  };
  ErrorData::new( code, err.to_string() )
}

fn text_output( content : String ) -> OutputData
{
  OutputData { content, format : "text".to_string(), execution_time_ms : None }
}

/// Recursively flattens a bound `names` argument value into plain strings.
///
/// `unilang`'s positional binding for a `multiple: true` + `Kind::List`
/// argument (see `cmd_compose`) coerces each raw argv token against the
/// *outer* `Kind::List` type rather than its inner `Kind::String` element
/// type, so a two-token invocation like `compose hash21 value_noise` binds
/// as `List([List([String("hash21")]), List([String("value_noise")])])`
/// rather than the flat `List([String("hash21"), String("value_noise")])`
/// one might expect — flattening here is what makes both shapes resolve to
/// the same `Vec<String>`.
fn flatten_names( value : &Value ) -> Vec< String >
{
  match value
  {
    Value::String( s ) => vec![ s.clone() ],
    Value::List( values ) => values.iter().flat_map( flatten_names ).collect(),
    _ => vec![],
  }
}

fn cmd_list() -> ( CommandDefinition, CommandRoutine )
{
  let def = CommandDefinition::former()
  .name( ".list" )
  .namespace( String::new() )
  .description( "List every bundled shader chunk.".to_string() )
  .hint( "name / description / tags / depends_on, one row per bundled chunk" )
  .status( "stable" )
  .version( "1.0.0".to_string() )
  .aliases( vec![] )
  .tags( vec![] )
  .permissions( vec![] )
  .idempotent( true )
  .deprecation_message( String::new() )
  .http_method_hint( String::new() )
  .examples( vec![] )
  .arguments( vec![] )
  .end();

  let routine : CommandRoutine = Box::new( | _cmd, _ctx |
  {
    let content = shader_chunks::list_chunks().map_err( | e | cli_error( &e ) )?;
    Ok( text_output( content ) )
  });

  ( def, routine )
}

fn cmd_get() -> ( CommandDefinition, CommandRoutine )
{
  let def = CommandDefinition::former()
  .name( ".get" )
  .namespace( String::new() )
  .description( "Show full detail for one shader chunk.".to_string() )
  .hint( "name, description, stage, tags, depends_on, exports for one chunk" )
  .status( "stable" )
  .version( "1.0.0".to_string() )
  .aliases( vec![] )
  .tags( vec![] )
  .permissions( vec![] )
  .idempotent( true )
  .deprecation_message( String::new() )
  .http_method_hint( String::new() )
  .examples( vec![] )
  .arguments( vec!
  [
    ArgumentDefinition::former()
    .name( "name" )
    .kind( Kind::String )
    .hint( "Chunk name (see `list`)." )
    .end(),
  ])
  .end();

  let routine : CommandRoutine = Box::new( | cmd, _ctx |
  {
    let Some( Value::String( name ) ) = cmd.arguments.get( "name" ) else
    {
      unreachable!( "argument 'name' is declared Kind::String and required" )
    };
    let content = shader_chunks::get_chunk( name ).map_err( | e | cli_error( &e ) )?;
    Ok( text_output( content ) )
  });

  ( def, routine )
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
  .examples( vec![] )
  .arguments( vec![] )
  .end();

  let routine : CommandRoutine = Box::new( | _cmd, _ctx |
  {
    let content = shader_chunks::list_tags().map_err( | e | cli_error( &e ) )?;
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
  .examples( vec![] )
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
    let content = shader_chunks::tree_chunk( name ).map_err( | e | cli_error( &e ) )?;
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
  .examples( vec![] )
  .arguments( vec!
  [
    ArgumentDefinition::former()
    .name( "names" )
    .kind( Kind::List( Box::new( Kind::String ), None ) )
    .hint( "One or more chunk names (see `list`)." )
    .attributes( ArgumentAttributes { multiple : true, ..ArgumentAttributes::default() } )
    .end(),
  ])
  .end();

  let routine : CommandRoutine = Box::new( | cmd, _ctx |
  {
    let names : Vec< String > = match cmd.arguments.get( "names" )
    {
      Some( value ) => flatten_names( value ),
      None => unreachable!( "argument 'names' is declared Kind::List, multiple, and required" ),
    };
    let content = shader_chunks::compose_chunks( &names ).map_err( | e | cli_error( &e ) )?;
    Ok( text_output( content ) )
  });

  ( def, routine )
}

fn build_registry() -> CommandRegistry
{
  let mut registry = CommandRegistry::new();
  for ( def, routine ) in [ cmd_list(), cmd_get(), cmd_tags(), cmd_tree(), cmd_compose() ]
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
    CommandGroup
    {
      name : "Commands".to_string(),
      entries : vec!
      [
        CommandEntry { name : "list".to_string(), desc : "List every bundled chunk.".to_string() },
        CommandEntry { name : "get <name>".to_string(), desc : "Show full detail for one chunk.".to_string() },
        CommandEntry { name : "tags".to_string(), desc : "List every distinct tag and its chunk(s).".to_string() },
        CommandEntry { name : "tree [name]".to_string(), desc : "Show a chunk's dependency tree, or the full forest.".to_string() },
        CommandEntry { name : "compose <name...>".to_string(), desc : "Preview composed WGSL for the given chunks.".to_string() },
      ],
    },
  ];
  data.examples = vec!
  [
    ExampleEntry { invocation : "shader_chunks list".to_string(), desc : None },
    ExampleEntry { invocation : "shader_chunks get hash21".to_string(), desc : None },
    ExampleEntry { invocation : "shader_chunks compose hash21 value_noise".to_string(), desc : None },
  ];

  let template = CliHelpTemplate::new( CliHelpStyle::default(), data );
  println!( "{}", template.render() );
}

fn main()
{
  let mut argv : Vec< String > = std::env::args().skip( 1 ).collect();

  // Fix(BUG-103): map conventional help spellings onto `unilang`'s help
  // machinery and print `result.outputs` centrally instead of leaving all
  // printing to command routines.
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
  // first, then spell help forms via the auto-registered `.{command}.help`
  // builtins so unknown commands still fail loudly.

  // `sch`, `sch .`, `sch help`, `sch .help` — all spell "top-level help".
  if argv.is_empty() || ( argv.len() == 1 && matches!( argv[ 0 ].as_str(), "." | "help" | ".help" ) )
  {
    print_help();
    return;
  }

  // `sch help <command>` / `sch <command> ... help` — per-command help via
  // the `.{command}.help` builtin `unilang` auto-registers per command; an
  // unknown command still fails loudly through the unknown-command path.
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

  if let Some( target ) = help_target
  {
    let target = target.strip_prefix( '.' ).unwrap_or( target.as_str() );
    if target == "help"
    {
      print_help();
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

  let pipeline = Pipeline::new( build_registry() );
  let result = pipeline.process_command_from_argv_simple( &argv );

  if !result.success
  {
    if let Some( error ) = &result.error
    {
      eprintln!( "{error}" );
    }
    std::process::exit( EXIT_CODE.load( Ordering::Relaxed ) );
  }

  for output in &result.outputs
  {
    if !output.content.is_empty()
    {
      println!( "{}", output.content );
    }
  }
}
