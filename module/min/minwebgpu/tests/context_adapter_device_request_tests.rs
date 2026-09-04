//! Tests for `context::adapter_request`/`context::device_request` -- these must return
//! `Result`, never panic, on the ordinary, spec-defined "no adapter"/"device request rejected"
//! outcomes.
//!
//! `adapter_request`/`device_request` are exercised live (no live `GpuDevice` is assumed to
//! exist in the test browser -- the point of this suite is that a *missing* adapter is exactly
//! the reachable case BUG-162 fixes, not a live-hardware-dependent happy path). The new
//! `ContextError` variants' `Display`/conversion wiring is additionally exercised in isolation,
//! matching the sibling test files' "pure data-in/data-out" scope where possible.

#[ cfg( target_arch = "wasm32" ) ]
#[ cfg( test ) ]
mod tests
{
  use wasm_bindgen_test::wasm_bindgen_test;

  // Fix(BUG-110 pitfall, pre-existing in this codebase): without this line the test binary
  // defaults to Node.js, where `web_sys::window()` is always `None` -- `adapter_request` calls
  // `navigator()`, which unwraps `web_sys::window()`, so this suite needs a real browser
  // context to exercise the live-adapter path at all.
  wasm_bindgen_test::wasm_bindgen_test_configure!( run_in_browser );

  use minwebgpu as gl;
  use gl::error::ContextError;

  // test_kind: bug_reproducer(BUG-162)
  /// ## Root Cause
  /// `adapter_request`/`device_request` unconditionally `.unwrap()`ed both the `JsFuture`
  /// result and the `dyn_into` cast, panicking on two ordinary, reachable outcomes --
  /// `navigator.gpu.requestAdapter()` resolving to `null` ("no adapter available", a normal
  /// spec-defined outcome, not an exception) and `GpuAdapter.requestDevice()`'s promise being
  /// rejected -- instead of surfacing them as `Result::Err` per this crate's own written
  /// invariant (`docs/invariant/001_result_based_error_handling.md`).
  /// ## Why Not Caught
  /// No existing test called `adapter_request`/`device_request` at all; every real call site
  /// assumed a live, working WebGPU adapter would always be available.
  /// ## Fix Applied
  /// Both functions now return `Result<_, WebGPUError>`: `adapter_request` checks
  /// `.is_null()` and returns `ContextError::NoAdapterAvailable`; `device_request` maps a
  /// rejected promise to `ContextError::DeviceRequestRejected`. `setup()` and all real call
  /// sites propagate via `?`.
  /// ## Prevention
  /// This test calls `adapter_request` for real against the test browser -- a headless test
  /// environment realistically has no GPU adapter, which is exactly the case that used to
  /// panic. It asserts the call returns `Ok` or the documented `NoAdapterAvailable` error, and
  /// never anything else (never panics, never a different error variant).
  /// ## Pitfall
  /// A Promise's resolve/reject shape doesn't map 1:1 onto "success/failure" -- `request_adapter`
  /// communicates its one failure mode through a *resolved* `null`, while `request_device`
  /// communicates its failure mode through *rejection*. Each needed its own check matching its
  /// actual signature, not a uniform `.unwrap()` on the outer `Result`.
  #[ wasm_bindgen_test ]
  async fn adapter_request_returns_result_never_panics_test()
  {
    let adapter = gl::context::adapter_request().await;

    match adapter
    {
      Ok( adapter ) =>
      {
        // A real adapter was available -- device_request must likewise return a Result rather
        // than panic, whichever way the device request resolves.
        let device = gl::context::device_request( &adapter ).await;
        assert!
        (
          matches!( device, Ok( _ ) | Err( gl::WebGPUError::ContexError( ContextError::DeviceRequestRejected( _ ) ) ) ),
          "device_request must return Ok or DeviceRequestRejected, never panic or a different error variant"
        );
      },
      // BUG-164: this test browser has no WebGPU support at all, so `WebGpuUnsupported` is the
      // actual, expected outcome here -- a distinct, separately-fixed failure mode one call
      // earlier than the `NoAdapterAvailable` case this test was originally written for.
      Err( gl::WebGPUError::ContexError( ContextError::NoAdapterAvailable | ContextError::WebGpuUnsupported ) ) => {},
      Err( other ) => panic!( "adapter_request must return Ok, NoAdapterAvailable or WebGpuUnsupported, got a different error: {other}" ),
    }
  }

  #[ wasm_bindgen_test ]
  fn no_adapter_available_display_message_test()
  {
    let message = ContextError::NoAdapterAvailable.to_string();
    assert_eq!( message, "No WebGPU adapter available on this system" );
  }

  #[ wasm_bindgen_test ]
  fn device_request_rejected_display_message_test()
  {
    let message = ContextError::DeviceRequestRejected( "boom".to_string() ).to_string();
    assert_eq!( message, "WebGPU device request was rejected: boom" );
  }

  #[ wasm_bindgen_test ]
  fn context_error_converts_into_web_gpu_error_test()
  {
    let error : gl::WebGPUError = ContextError::NoAdapterAvailable.into();
    assert!( matches!( error, gl::WebGPUError::ContexError( ContextError::NoAdapterAvailable ) ) );
  }
}
