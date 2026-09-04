//! Native coverage for `browser_log::DebugLog` -- proves each convenience method reports the
//! REAL external caller's `file:line` (BUG-167) and `target`/`module_path` (BUG-229), not
//! `debug_log.rs`'s own internal location/module. Lives in its own file: `log::set_logger` is
//! process-global and callable once per process, so this installs its own captor without
//! racing `basic_test.rs`/`panic_hook_test.rs`, neither of which installs a `log::Log`
//! implementation. A single `#[ test ]` covers all 5 methods so nothing else in this binary can
//! install a second competing logger mid-run -- this is also why BUG-229 coverage extends this
//! same test instead of adding a second `#[ test ]` fn (which would need its own logger install).

use browser_log::DebugLog;
use log::{ Level, Log, Metadata, Record };
use std::sync::Mutex;

#[ derive( Debug ) ]
struct Sample
{
  value : i32,
}

struct CapturingLogger;

/// `( level, file, line, target, module_path, formatted args )` -- one entry per captured record.
type CapturedRecord = ( Level, Option< String >, Option< u32 >, String, Option< String >, String );

static CAPTURED : Mutex< Vec< CapturedRecord > > = Mutex::new( Vec::new() );

impl Log for CapturingLogger
{
  fn enabled( &self, _metadata : &Metadata< '_ > ) -> bool
  {
    true
  }

  fn log( &self, record : &Record< '_ > )
  {
    CAPTURED.lock().unwrap().push((
      record.level(),
      record.file().map( str::to_string ),
      record.line(),
      record.target().to_string(),
      record.module_path().map( str::to_string ),
      record.args().to_string(),
    ));
  }

  fn flush( &self ) {}
}

static LOGGER : CapturingLogger = CapturingLogger;

/// `bug_reproducer(BUG-167)`, `bug_reproducer(BUG-229)`
///
/// BUG-167 root cause: `debug_trace`/`debug_info`/`debug_warn`/`debug_error`/`debug_log` used
/// to call `log::trace!`/`log::info!`/etc. directly inside their own trait-default bodies, so
/// `file!()`/`line!()` always resolved to `debug_log.rs`'s own location -- never the real
/// caller's, regardless of `#[inline]`, since these macros are lexical, not dynamic.
///
/// BUG-229 root cause: the BUG-167 fix left `module_path!()` itself unaddressed -- it is
/// lexical exactly like `file!()`/`line!()`, but unlike them has no `#[track_caller]`-equivalent
/// in stable Rust, so `target`/`module_path` on every emitted `Record` still resolved to this
/// trait's OWN defining module (`browser_log::log::debug_log::private`), silently defeating
/// `Config::target_filter` (`metadata.target().starts_with(prefix)`) for every consumer that
/// set one. Why not caught by the BUG-167 test above: that test only ever asserted on
/// `file()`/`line()`/`args()` -- it captured records without inspecting `target()` or
/// `module_path()` at all, so a record silently mistagged with the trait's own module sailed
/// through unnoticed underneath an otherwise-green suite.
///
/// This pins both fixes: each method, called from a KNOWN line in THIS file, must produce a
/// record whose `file()`/`line()` point at THIS file and THIS call site (BUG-167), and whose
/// `target()`/`module_path()` equal the caller-supplied `module_path!()` (BUG-229) -- not
/// `debug_log.rs`'s own internal location or module.
#[ test ]
fn debug_log_methods_report_the_real_caller_location_and_module()
{
  log::set_logger( &LOGGER ).expect( "set_logger must succeed -- this is the only test in this binary that installs a logger" );
  log::set_max_level( log::LevelFilter::Trace );

  let sample = Sample { value : 7 };
  let this_module = module_path!();

  let trace_line = line!() + 1;
  sample.debug_trace( this_module );
  let info_line = line!() + 1;
  sample.debug_info( this_module );
  let warn_line = line!() + 1;
  sample.debug_warn( this_module );
  let error_line = line!() + 1;
  sample.debug_error( this_module );
  let log_line = line!() + 1;
  sample.debug_log( Level::Debug, this_module );

  let captured = CAPTURED.lock().unwrap();
  assert_eq!( captured.len(), 5, "all 5 calls must have reached the logger: {captured:?}" );

  let expected =
  [
    ( Level::Trace, trace_line ),
    ( Level::Info, info_line ),
    ( Level::Warn, warn_line ),
    ( Level::Error, error_line ),
    ( Level::Debug, log_line ),
  ];

  for ( ( level, file, line, target, module_path, args ), ( expected_level, expected_line ) ) in captured.iter().zip( expected )
  {
    assert_eq!( *level, expected_level, "wrong level captured: {captured:?}" );
    assert!(
      file.as_deref().unwrap_or( "" ).ends_with( "debug_log_test.rs" ),
      "must report THIS file, not debug_log.rs's own internal location: {file:?}"
    );
    assert_eq!(
      *line, Some( expected_line ),
      "must report the real caller's line ({expected_line}), not debug_log.rs's internal location: {line:?}"
    );
    assert!(
      args.contains( "value" ) && args.contains( &sample.value.to_string() ),
      "must still format the real Debug body: {args:?}"
    );
    // Fix(BUG-229): target/module_path must be the caller-supplied module, not browser_log's own.
    assert_eq!( target, this_module, "target must be the caller's module_path!(), not browser_log's own: {target:?}" );
    assert_eq!( module_path.as_deref(), Some( this_module ), "module_path must match the caller-supplied target: {module_path:?}" );
    assert!(
      !target.starts_with( "browser_log" ),
      "target must not resolve to browser_log's own internal trait-defining module: {target:?}"
    );
  }
}
