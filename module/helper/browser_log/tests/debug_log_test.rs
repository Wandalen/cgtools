//! Native coverage for `browser_log::DebugLog` -- proves each convenience method reports the
//! REAL external caller's `file:line` (BUG-167), not `debug_log.rs`'s own internal location.
//! Lives in its own file: `log::set_logger` is process-global and callable once per process,
//! so this installs its own captor without racing `basic_test.rs`/`panic_hook_test.rs`, neither
//! of which installs a `log::Log` implementation. A single `#[ test ]` covers all 5 methods so
//! nothing else in this binary can install a second competing logger mid-run.

use browser_log::DebugLog;
use log::{ Level, Log, Metadata, Record };
use std::sync::Mutex;

#[ derive( Debug ) ]
struct Sample
{
  value : i32,
}

struct CapturingLogger;

/// `( level, file, line, formatted args )` -- one entry per captured record.
type CapturedRecord = ( Level, Option< String >, Option< u32 >, String );

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
      record.args().to_string(),
    ));
  }

  fn flush( &self ) {}
}

static LOGGER : CapturingLogger = CapturingLogger;

/// `bug_reproducer(BUG-167)`
///
/// Root cause: `debug_trace`/`debug_info`/`debug_warn`/`debug_error`/`debug_log` used to call
/// `log::trace!`/`log::info!`/etc. directly inside their own trait-default bodies, so
/// `file!()`/`line!()` always resolved to `debug_log.rs`'s own location -- never the real
/// caller's, regardless of `#[inline]`, since these macros are lexical, not dynamic.
///
/// This pins the fix: each method, called from a KNOWN line in THIS file, must produce a
/// record whose `file()`/`line()` point at THIS file and THIS call site -- not `debug_log.rs`.
#[ test ]
fn debug_log_methods_report_the_real_caller_location()
{
  log::set_logger( &LOGGER ).expect( "set_logger must succeed -- this is the only test in this binary that installs a logger" );
  log::set_max_level( log::LevelFilter::Trace );

  let sample = Sample { value : 7 };

  let trace_line = line!() + 1;
  sample.debug_trace();
  let info_line = line!() + 1;
  sample.debug_info();
  let warn_line = line!() + 1;
  sample.debug_warn();
  let error_line = line!() + 1;
  sample.debug_error();
  let log_line = line!() + 1;
  sample.debug_log( Level::Debug );

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

  for ( ( level, file, line, args ), ( expected_level, expected_line ) ) in captured.iter().zip( expected )
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
  }
}
