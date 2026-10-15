//! Tests for `shader::has_blocking_error` -- whether a batch of shader compilation diagnostics
//! contains at least one message severe enough to block a pipeline rebuild attempt.
//!
//! Pure data-in/data-out logic over `shader::CompilationMessage`/`CompilationMessageKind`, no
//! live `GpuDevice` involved -- same wasm32-gated shape as `bind_group_layout_entry_tests.rs`,
//! since minwebgpu's real API (including this one) only exists under `target_arch = "wasm32"`
//! (see `src/lib.rs`'s native `stub` module).

#[ cfg( target_arch = "wasm32" ) ]
#[ cfg( test ) ]
mod tests
{
  use wasm_bindgen_test::wasm_bindgen_test;
  use minwebgpu as gl;
  use gl::{ CompilationMessage, CompilationMessageKind };

  fn message( kind : CompilationMessageKind ) -> CompilationMessage
  {
    CompilationMessage { text : String::new(), kind, line : 0.0, column : 0.0 }
  }

  #[ wasm_bindgen_test ]
  fn empty_messages_has_no_blocking_error_test()
  {
    assert!( !gl::shader::has_blocking_error( &[] ) );
  }

  #[ wasm_bindgen_test ]
  fn only_warnings_and_info_has_no_blocking_error_test()
  {
    let messages = [ message( CompilationMessageKind::Warning ), message( CompilationMessageKind::Info ) ];

    assert!( !gl::shader::has_blocking_error( &messages ) );
  }

  #[ wasm_bindgen_test ]
  fn one_error_among_others_has_blocking_error_test()
  {
    let messages =
    [
      message( CompilationMessageKind::Info ),
      message( CompilationMessageKind::Error ),
      message( CompilationMessageKind::Warning ),
    ];

    assert!( gl::shader::has_blocking_error( &messages ) );
  }
}
