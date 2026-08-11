//! Tests for `BindGroupLayoutEntry`'s fallible conversion to `web_sys::GpuBindGroupLayoutEntry`.
//!
//! `BindingType::Other` is the entry's default `ty` (see `BindGroupLayoutEntry::new`). Converting
//! an entry that never had `.ty(..)` called must return `Err`, not panic — see
//! `descriptor/bind_group_layout_entry.rs`'s `TryFrom` impl.

#[ cfg( target_arch = "wasm32" ) ]
#[ cfg( test ) ]
mod tests
{
  use wasm_bindgen_test::wasm_bindgen_test;
  use minwebgpu as gl;

  // test_kind: bug_reproducer(BUG-051)
  /// ## Root Cause
  /// `BindGroupLayoutEntry`'s conversion to `web_sys::GpuBindGroupLayoutEntry` was an infallible
  /// `From` impl that panicked with "The type of the binding entry was not set" whenever `ty`
  /// was still `BindingType::Other` — yet `Other` is `BindGroupLayoutEntry::new()`'s own default
  /// and is documented as "a placeholder for other or unhandled binding types", i.e. expected,
  /// reachable, recoverable input, not an invariant violation.
  /// ## Why Not Caught
  /// No existing test ever converted a `BindGroupLayoutEntry` without first calling `.ty(..)` —
  /// every prior caller set a concrete type before converting, so the default-`Other` panic
  /// branch was never exercised.
  /// ## Fix Applied
  /// The conversion is now `TryFrom< BindGroupLayoutEntry >`, returning
  /// `Err( WebGPUError::BindGroupError( BindGroupError::TypeNotSet( binding ) ) )` on
  /// `BindingType::Other` instead of panicking.
  /// ## Prevention
  /// A placeholder/default enum variant reachable via ordinary construction is foreseeable
  /// caller input, not an invariant violation — the conversion touching it must be fallible
  /// (`TryFrom`), never `From` plus a panic on the unhandled arm.
  /// ## Pitfall
  /// An infallible `From` impl is a contract that every input converts successfully — silently
  /// panicking on one variant breaks that contract for any caller who reaches the type's own
  /// documented default.
  #[ wasm_bindgen_test ]
  fn entry_without_ty_yields_type_not_set_err_test()
  {
    let entry = gl::BindGroupLayoutEntry::new();

    let result : Result< gl::web_sys::GpuBindGroupLayoutEntry, _ > = entry.try_into();

    assert!
    (
      result.is_err(),
      "converting a BindGroupLayoutEntry whose `.ty(..)` was never called must return Err, not panic or succeed"
    );
  }

  #[ wasm_bindgen_test ]
  fn entry_with_ty_converts_ok_test()
  {
    let entry = gl::BindGroupLayoutEntry::new().ty( gl::binding_type::buffer_type() );

    let result : Result< gl::web_sys::GpuBindGroupLayoutEntry, _ > = entry.try_into();

    assert!( result.is_ok(), "converting a BindGroupLayoutEntry with `.ty(..)` set must succeed" );
  }

  #[ wasm_bindgen_test ]
  fn descriptor_entry_without_ty_propagates_err_test()
  {
    let result = gl::BindGroupLayoutDescriptor::new().entry( gl::BindGroupLayoutEntry::new() );

    assert!
    (
      result.is_err(),
      "BindGroupLayoutDescriptor::entry must propagate the TypeNotSet error, not panic"
    );
  }

  #[ wasm_bindgen_test ]
  fn descriptor_entry_with_ty_succeeds_test()
  {
    let result = gl::BindGroupLayoutDescriptor::new()
    .entry( gl::BindGroupLayoutEntry::new().ty( gl::binding_type::buffer_type() ) );

    assert!( result.is_ok(), "BindGroupLayoutDescriptor::entry must succeed once `.ty(..)` was called" );
  }

  #[ wasm_bindgen_test ]
  fn descriptor_entry_from_ty_always_succeeds_test()
  {
    // `entry_from_ty` always supplies a concrete `BindingType` itself, so it can never hit the
    // `TypeNotSet` path — this documents that guarantee explicitly.
    let result = gl::BindGroupLayoutDescriptor::new()
    .entry_from_ty( gl::binding_type::buffer_type() );

    assert!( result.is_ok(), "BindGroupLayoutDescriptor::entry_from_ty always supplies a concrete type" );
  }
}
