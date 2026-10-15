//! Tests for `context::preferred_format` -- must return `Result`, never panic, when this
//! browser has no WebGPU support at all (`navigator.gpu` itself is `undefined`).
//!
//! `preferred_format` is exercised live -- a WebGPU-less test browser is exactly the reachable
//! case BUG-164 fixes (discovered as a side effect of BUG-162's own regression test failing one
//! call earlier than anything that bug touched). The new `ContextError::WebGpuUnsupported`
//! variant's `Display`/conversion wiring is additionally exercised in isolation, matching the
//! sibling test files' "pure data-in/data-out" scope where possible.

#[ cfg( target_arch = "wasm32" ) ]
#[ cfg( test ) ]
mod tests
{
  use wasm_bindgen_test::wasm_bindgen_test;

  // Fix(BUG-110 pitfall, pre-existing in this codebase): without this line the test binary
  // defaults to Node.js, where `web_sys::window()` is always `None` -- `preferred_format` calls
  // `navigator()`, which unwraps `web_sys::window()`, so this suite needs a real browser
  // context to exercise the live path at all.
  wasm_bindgen_test::wasm_bindgen_test_configure!( run_in_browser );

  use minwebgpu as gl;
  use gl::error::ContextError;

  // test_kind: bug_reproducer(BUG-164)
  /// ## Root Cause
  /// `preferred_format` called `navigator.gpu()` unconditionally and immediately invoked
  /// `.get_preferred_canvas_format()` on the result, panicking at the wasm-bindgen FFI boundary
  /// when the browser has no WebGPU support at all (`navigator.gpu` is JS `undefined`, not a
  /// `Gpu` object) -- a normal, reachable outcome in any WebGPU-less browser, not an exceptional
  /// caller error.
  /// ## Why Not Caught
  /// No existing test called `preferred_format` at all; every real call site assumed a
  /// WebGPU-capable browser as an implicit precondition. Discovered as a side effect of
  /// BUG-162's own regression test throwing this exact FFI error one call earlier than anything
  /// BUG-162 touched.
  /// ## Fix Applied
  /// `preferred_format` now returns `Result<GpuTextureFormat, WebGPUError>`, sharing a new
  /// private `gpu_or_unsupported` helper with `adapter_request` that checks `navigator.gpu()`'s
  /// result via `JsValue::is_undefined` before using it, returning
  /// `ContextError::WebGpuUnsupported` instead of panicking. `setup()` and all real call sites
  /// propagate via `?`.
  /// ## Prevention
  /// This test calls `preferred_format` for real against the test browser -- a headless test
  /// environment realistically has no WebGPU support, which is exactly the case that used to
  /// panic. It asserts the call returns `Ok` or the documented `WebGpuUnsupported` error, and
  /// never anything else (never panics, never a different error variant).
  /// ## Pitfall
  /// `web_sys` types a property getter like `Navigator::gpu()` as non-`Option` even when the
  /// underlying browser feature is experimental/optional -- the getter itself won't reveal that
  /// the feature is absent, it just returns `undefined` typed as if it were the real object.
  /// Callers must feature-detect explicitly (`JsValue::is_undefined`) before use.
  #[ wasm_bindgen_test ]
  fn preferred_format_returns_result_never_panics_test()
  {
    let format = gl::context::preferred_format();

    match format
    {
      Ok( _ ) | Err( gl::WebGPUError::ContexError( ContextError::WebGpuUnsupported ) ) => {},
      Err( other ) => panic!( "preferred_format must return Ok or WebGpuUnsupported, got a different error: {other}" ),
    }
  }

  #[ wasm_bindgen_test ]
  fn web_gpu_unsupported_display_message_test()
  {
    let message = ContextError::WebGpuUnsupported.to_string();
    assert_eq!( message, "WebGPU is not supported by this browser (navigator.gpu is undefined)" );
  }

  #[ wasm_bindgen_test ]
  fn web_gpu_unsupported_converts_into_web_gpu_error_test()
  {
    let error : gl::WebGPUError = ContextError::WebGpuUnsupported.into();
    assert!( matches!( error, gl::WebGPUError::ContexError( ContextError::WebGpuUnsupported ) ) );
  }
}
