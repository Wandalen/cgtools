//! Shared CLI wiring for the `shader_chunks` utility family. Every utility
//! CLI ( `shader_chunks_query`, `shader_chunks_compose`,
//! `shader_chunks_params`, `shader_chunks_preview` ) and the aggregator
//! ( `shader_chunks` / `sch` ) runs through this one layer: it builds the
//! `unilang` `CommandRegistry` from a supplied command set, dispatches via
//! `Pipeline`, routes the conventional help spellings ( `help`, `.`,
//! `<command> help` ) to `cli_fmt`-rendered help screens, prints `Pipeline`
//! outputs centrally through pipe-safe write helpers, and maps stored exit
//! codes to the process exit. Everything here is parameterized by
//! [`CliApp`] — binary name, tagline, help groups/examples, and commands —
//! so the aggregator and each standalone utility binary get byte-identical
//! behavior from the same code path.

mod private
{
  use std::sync::atomic::{ AtomicI32, Ordering };
  use unilang::prelude::*;
  use cli_fmt::prelude::*;

  /// One utility's registrable command surface: each entry pairs a
  /// `unilang` [`CommandDefinition`] with its routine. Utilities return
  /// this from their `commands( binary )` constructors; the aggregator
  /// concatenates the sets of every utility it folds in.
  pub type CommandSet = Vec< ( CommandDefinition, CommandRoutine ) >;

  /// Everything [`run`] needs to behave as one concrete binary: the name
  /// help screens and usage lines print, the top-level tagline, the help
  /// groups/examples, and the command set to register.
  pub struct CliApp
  {
    /// Binary name rendered in `Usage:` lines and the top-level help
    /// screen ( e.g. `"shader_chunks"`, `"shader_chunks_query"` ).
    pub binary : String,
    /// One-line description under the binary name on the help screen.
    pub tagline : String,
    /// Help-screen command groups, in render order.
    pub groups : Vec< CommandGroup >,
    /// Help-screen example invocations, in render order.
    pub examples : Vec< ExampleEntry >,
    /// Commands to register, each with its routine.
    pub commands : CommandSet,
  }

  /// Exit code stashed by a routine just before it returns an error wrapped
  /// as [`ErrorData`]. `CommandResult.error` only ever carries a flattened
  /// `String` — `ErrorData`'s `Display` impl prints `message` alone, never
  /// `code` — so this is the only channel through which a specific
  /// per-utility exit code can reach [`run`] after `Pipeline` dispatch
  /// completes. Defaults to `1`, matching the framework-level failures
  /// ( unknown command, missing argument ) that never touch a utility error
  /// at all — both are caller-fixable invocation mistakes, the same class
  /// the utilities map to `1`.
  static EXIT_CODE : AtomicI32 = AtomicI32::new( 1 );

  // Fix(BUG-108): every user-facing write goes through `stdout_print` /
  // `stderr_print` instead of `println!`/`eprintln!`.
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
  /// stderr ( best-effort ) and exits 2.
  pub fn stdout_print( content : &str )
  {
    use std::io::Write;
    let mut stdout = std::io::stdout().lock();
    if let Err( error ) = writeln!( stdout, "{content}" )
    {
      if error.kind() == std::io::ErrorKind::BrokenPipe
      {
        std::process::exit( 0 );
      }
      stderr_print( &format!( "failed writing to stdout: {error}" ) );
      std::process::exit( 2 );
    }
  }

  /// Writes one line to stderr, discarding write errors — error reporting
  /// must never itself become a second failure path ( see BUG-108 ).
  pub fn stderr_print( content : &str )
  {
    use std::io::Write;
    let _ = writeln!( std::io::stderr(), "{content}" );
  }

  /// Builds the [`ErrorData`] a routine returns for a utility error,
  /// stashing `exit_code` for [`run`] to exit with after `Pipeline`
  /// dispatch flattens the error to a message string. Each utility's error
  /// enum supplies its own `exit_code()` / `ErrorCode` mapping and calls
  /// this at every routine error edge.
  pub fn error_report( exit_code : i32, code : ErrorCode, message : String ) -> ErrorData
  {
    EXIT_CODE.store( exit_code, Ordering::Relaxed );
    ErrorData::new( code, message )
  }

  /// Wraps rendered command output as `unilang` [`OutputData`] with the
  /// `text` format tag.
  #[ must_use ]
  pub fn text_output( content : String ) -> OutputData
  {
    OutputData { content, format : "text".to_string(), execution_time_ms : None }
  }

  /// Recursively flattens a bound list argument value into plain strings.
  ///
  /// `unilang`'s positional binding for a `multiple: true` + `Kind::List`
  /// argument ( `names` on `list`/`get`/`compose` ) coerces each raw argv
  /// token against the *outer* `Kind::List` type rather than its inner
  /// `Kind::String` element type, so a two-token invocation like
  /// `compose hash21 value_noise` binds as
  /// `List([List([String("hash21")]), List([String("value_noise")])])`
  /// rather than the flat `List([String("hash21"), String("value_noise")])`
  /// one might expect — flattening here is what makes both shapes ( and the
  /// comma-delimited named lists `tag::`/`fields::` ) resolve to the same
  /// `Vec<String>`.
  #[ must_use ]
  pub fn names_flatten( value : &Value ) -> Vec< String >
  {
    match value
    {
      Value::String( s ) => vec![ s.clone() ],
      Value::List( values ) => values.iter().flat_map( names_flatten ).collect(),
      _ => vec![],
    }
  }

  /// One optional named ( `key::value` ) parameter definition.
  #[ must_use ]
  pub fn named_arg( name : &str, kind : Kind, hint : &str, default : Option< String > ) -> ArgumentDefinition
  {
    ArgumentDefinition::former()
    .name( name )
    .kind( kind )
    .hint( hint )
    .attributes( ArgumentAttributes { optional : true, default, ..ArgumentAttributes::default() } )
    .end()
  }

  /// Extracts a string-valued parameter ( plain or enum-spelled ).
  #[ must_use ]
  pub fn arg_string( cmd : &VerifiedCommand, key : &str ) -> Option< String >
  {
    match cmd.arguments.get( key )
    {
      Some( Value::String( s ) | Value::Enum( s ) ) => Some( s.clone() ),
      _ => None,
    }
  }

  // Fix(BUG-283): new `_checked` extractor -- `arg_string`'s catch-all
  // `_ => None` arm silently swallowed a duplicated named key. `unilang`
  // binds ANY repeated named argument to `Value::List` regardless of the
  // argument's own `multiple` attribute (`bind_argument_values`'s
  // `if parser_args.len() > 1` check runs before it ever consults
  // `arg_def.attributes.multiple`), so `out::a out::b` was indistinguishable
  // from `out::` never having been supplied at all -- `shader_chunks_compose`
  // silently printed to stdout instead of writing either file.
  // Root cause: a `Value` match's catch-all arm cannot tell "argument
  // absent" apart from "argument supplied in an unexpected shape."
  // Pitfall: never add a new single-value extractor here with a bare `_`
  // catch-all over `Value` -- always match `Value::List` explicitly and
  // fail loudly, since any named key can be repeated regardless of its
  // declared `multiple` attribute.
  /// Extracts a string-valued parameter ( plain or enum-spelled ), failing
  /// loudly instead of silently defaulting to `None` when `key` was
  /// supplied more than once ( see [`arg_string`]'s doc comment gap -- its
  /// catch-all arm cannot distinguish "absent" from "duplicated" ).
  ///
  /// # Errors
  ///
  /// Returns [`ErrorData`] naming `key` when it was bound to more than one
  /// value.
  pub fn arg_string_checked( cmd : &VerifiedCommand, key : &str ) -> Result< Option< String >, ErrorData >
  {
    match cmd.arguments.get( key )
    {
      Some( Value::String( s ) | Value::Enum( s ) ) => Ok( Some( s.clone() ) ),
      Some( Value::List( values ) ) => Err( error_report
      (
        1,
        ErrorCode::ValidationRuleFailed,
        format!( "`{key}` was given {} times; a single value is required", values.len() ),
      )),
      _ => Ok( None ),
    }
  }

  /// Extracts a boolean parameter, falling back to `default` when absent.
  #[ must_use ]
  pub fn arg_bool( cmd : &VerifiedCommand, key : &str, default : bool ) -> bool
  {
    match cmd.arguments.get( key )
    {
      Some( Value::Boolean( flag ) ) => *flag,
      _ => default,
    }
  }

  // Fix(BUG-283): new `_checked` extractor -- same defect class as
  // `arg_string`'s (see the `Fix(BUG-283)` comment above `arg_string_checked`):
  // `arg_bool`'s catch-all `_ => default` arm silently absorbed a duplicated
  // `transitive::1 transitive::1` ( bound by `unilang` as
  // `Value::List([Boolean(true), Boolean(true)])` ), making it
  // indistinguishable from `transitive::` never having been supplied --
  // `shader_chunks_compose` silently composed with `transitive=false`.
  // Root cause: same as `arg_string`'s -- a bare `_` catch-all over `Value`
  // cannot tell "absent" apart from "duplicated."
  // Pitfall: same as `arg_string_checked`'s -- never add a single-value
  // extractor here without an explicit, loud `Value::List` arm.
  /// Extracts a boolean parameter, failing loudly instead of silently
  /// falling back to `default` when `key` was supplied more than once ( see
  /// [`arg_bool`]'s doc comment gap -- its catch-all arm cannot distinguish
  /// "absent" from "duplicated" ).
  ///
  /// # Errors
  ///
  /// Returns [`ErrorData`] naming `key` when it was bound to more than one
  /// value.
  pub fn arg_bool_checked( cmd : &VerifiedCommand, key : &str, default : bool ) -> Result< bool, ErrorData >
  {
    match cmd.arguments.get( key )
    {
      Some( Value::Boolean( flag ) ) => Ok( *flag ),
      Some( Value::List( values ) ) => Err( error_report
      (
        1,
        ErrorCode::ValidationRuleFailed,
        format!( "`{key}` was given {} times; a single value is required", values.len() ),
      )),
      _ => Ok( default ),
    }
  }

  /// Extracts a non-negative integer parameter; a negative value fails
  /// loudly ( exit 1, validation error ) rather than wrapping.
  ///
  /// # Errors
  ///
  /// Returns [`ErrorData`] naming the parameter and offending value when it
  /// is negative.
  pub fn arg_usize( cmd : &VerifiedCommand, key : &'static str ) -> Result< usize, ErrorData >
  {
    match cmd.arguments.get( key )
    {
      Some( Value::Integer( n ) ) => usize::try_from( *n ).map_err( | _ | error_report
      (
        1,
        ErrorCode::ValidationRuleFailed,
        format!( "invalid `{key}` value: `{n}` (allowed: a non-negative integer)" ),
      )),
      _ => Ok( 0 ),
    }
  }

  /// Extracts a list parameter as flat strings ( see [`names_flatten`] ).
  #[ must_use ]
  pub fn arg_list( cmd : &VerifiedCommand, key : &str ) -> Vec< String >
  {
    cmd.arguments.get( key ).map( names_flatten ).unwrap_or_default()
  }

  /// Folds a [`CommandSet`] into a `unilang` [`CommandRegistry`].
  ///
  /// # Panics
  ///
  /// Panics on a duplicate command name — two aggregated utilities
  /// declaring the same command is an integration mistake that must fail
  /// loudly at first dispatch, never silently shadow — and on a malformed
  /// hand-written definition.
  #[ must_use ]
  pub fn registry_build( commands : CommandSet ) -> CommandRegistry
  {
    let mut registry = CommandRegistry::new();
    let mut names = std::collections::HashSet::new();
    for ( def, routine ) in commands
    {
      assert!
      (
        names.insert( def.name().to_string() ),
        "duplicate command name `{}` across aggregated utilities", def.name()
      );
      registry.register_with_routine( &def, routine ).expect( "hand-written static command definitions are well-formed" );
    }
    registry
  }

  fn help_print( binary : &str, tagline : &str, groups : &[ CommandGroup ], examples : &[ ExampleEntry ] )
  {
    let mut data = CliHelpData::default();
    data.binary = binary.to_string();
    data.tagline = tagline.to_string();
    data.groups = groups.to_vec();
    data.examples = examples.to_vec();

    let template = CliHelpTemplate::new( CliHelpStyle::default(), data );
    stdout_print( &template.render() );
  }

  /// Renders per-command help for `command` ( bare name, no leading dot )
  /// from its registered [`CommandDefinition`], through the same `cli_fmt`
  /// template and style as the top-level help screen. The first declared
  /// argument is the command's positional one and is spelled by shape
  /// ( `<name>` required, `[name]` optional, `...` repeatable ); every
  /// further argument is named-only and rendered as a `key::` row carrying
  /// its hint and declared default, with a `[param::value ...]` marker
  /// appended to the usage line. Examples come from the definition itself.
  fn command_help_print( binary : &str, command : &str, def : &CommandDefinition )
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
    data.usage_lines = vec![ format!( "Usage: {binary} {syntax}" ) ];
    data.tagline = def.description().to_string();
    data.groups = vec![ CommandGroup { name : syntax, entries : argument_rows } ];
    data.examples = def.examples().iter()
    .map( | invocation | ExampleEntry { invocation : invocation.clone(), desc : None } )
    .collect();

    let template = CliHelpTemplate::new( CliHelpStyle::default(), data );
    stdout_print( &template.render() );
  }

  /// The whole CLI: parses argv, routes help spellings, dispatches through
  /// `unilang`'s `Pipeline`, prints outputs, and exits non-zero on failure.
  /// Every `src/bin/` entry point in the utility family delegates here with
  /// its own [`CliApp`] — keeping the entry points byte-level trivial is
  /// what guarantees aggregated and standalone spellings behave identically.
  pub fn run( app : CliApp )
  {
    let CliApp { binary, tagline, groups, examples, commands } = app;
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
    // framework answers on its own ( help listings, per-command help ) is
    // invisible until the entry point prints `result.outputs` — wire that
    // first, then render known-command help via `cli_fmt` and route only
    // unknown targets through `.{target}.help` so they still fail loudly.

    // `sch`, `sch .`, `sch help`, `sch .help`, `sch --help`, `sch -h` — all
    // spell "top-level help".
    if argv.is_empty() || ( argv.len() == 1 && matches!( argv[ 0 ].as_str(), "." | "help" | ".help" | "--help" | "-h" ) )
    {
      help_print( &binary, &tagline, &groups, &examples );
      return;
    }

    // `sch help <command>` / `sch <command> ... help` — per-command help,
    // rendered by `cli_fmt` from the command's own registered definition; an
    // unknown target falls through to the `.{target}.help` rewrite so it
    // still fails loudly through the unknown-command path.
    // Only a token that is exactly `help`/`--help`/`-h` is a help request —
    // named-argument spellings (`name::help`) pass through untouched.
    let help_target = if matches!( argv[ 0 ].as_str(), "help" | ".help" | "--help" | "-h" )
    {
      Some( argv[ 1 ].clone() )
    }
    else if argv.len() >= 2 && matches!( argv[ argv.len() - 1 ].as_str(), "help" | "--help" | "-h" )
    {
      Some( argv[ 0 ].clone() )
    }
    else
    {
      None
    };

    let registry = registry_build( commands );

    if let Some( target ) = help_target
    {
      let target = target.strip_prefix( '.' ).unwrap_or( target.as_str() );
      if target.is_empty() || target == "help"
      {
        help_print( &binary, &tagline, &groups, &examples );
        return;
      }
      if let Some( def ) = registry.commands().get( &format!( ".{target}" ) )
      {
        command_help_print( &binary, target, def );
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
        stderr_print( error );
      }
      std::process::exit( EXIT_CODE.load( Ordering::Relaxed ) );
    }

    for output in &result.outputs
    {
      if !output.content.is_empty()
      {
        stdout_print( &output.content );
      }
    }
  }
}

::mod_interface::mod_interface!
{
  own use CommandSet;
  own use CliApp;
  own use stdout_print;
  own use stderr_print;
  own use error_report;
  own use text_output;
  own use names_flatten;
  own use named_arg;
  own use arg_string;
  own use arg_string_checked;
  own use arg_bool;
  own use arg_bool_checked;
  own use arg_usize;
  own use arg_list;
  own use registry_build;
  own use run;
}
