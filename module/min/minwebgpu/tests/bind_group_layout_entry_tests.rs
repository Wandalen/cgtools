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

  // Test-integrity correction (audit finding, not a BUG-NNN -- `entry_from_ty`'s own `src/`
  // behavior was already correct both before and after this edit; only this comment's claim was
  // wrong, so there is no behavioral state to revert-and-rerun against).
  //
  // This comment previously read: "`entry_from_ty` always supplies a concrete `BindingType`
  // itself, so it can never hit the `TypeNotSet` path -- this documents that guarantee
  // explicitly." That is false as a claim about the *function*: `entry_from_ty`'s parameter is
  // `ty : impl Into< BindingType >`, and `BindingType::Other` -- a fieldless unit variant of the
  // `#[ non_exhaustive ]` `BindingType` enum -- is freely constructible from this external
  // `tests/` crate (confirmed empirically: `#[ non_exhaustive ]` on an enum blocks only
  // exhaustive `match` and struct-literal variant construction from other crates, never plain
  // unit/tuple variant construction -- the same reason external code can write
  // `std::io::ErrorKind::NotFound`). Passing `BindingType::Other` through `entry_from_ty` reaches
  // exactly the `TypeNotSet` path this comment called unreachable, matching `entry_from_ty`'s own
  // `# Errors` doc comment in `descriptor/bind_group_layout.rs`, which already documented this
  // case correctly -- only this test's adjacent comment overclaimed. See
  // `descriptor_entry_from_ty_with_other_still_errors_test` below for the previously-uncovered
  // counter-case, added as a direct result of this audit.
  #[ wasm_bindgen_test ]
  fn descriptor_entry_from_ty_always_succeeds_test()
  {
    // Scoped claim: `entry_from_ty` succeeds for this specific input (`buffer_type()`, which
    // converts to `BindingType::Buffer`, never `Other`) -- not a guarantee about every possible
    // `impl Into<BindingType>` argument. See `descriptor_entry_from_ty_with_other_still_errors_test`
    // for the documented error path this function's own `# Errors` section describes.
    let result = gl::BindGroupLayoutDescriptor::new()
    .entry_from_ty( gl::binding_type::buffer_type() );

    assert!( result.is_ok(), "BindGroupLayoutDescriptor::entry_from_ty must succeed for a concrete BindingType input" );
  }

  /// Test-integrity correction (audit finding, not a `bug_reproducer`): closes the coverage gap
  /// the corrected comment above documents. `entry_from_ty`'s own `# Errors` doc comment in
  /// `descriptor/bind_group_layout.rs` already states this function returns
  /// `BindGroupError::TypeNotSet` when `ty`'s conversion yields `BindingType::Other` -- no
  /// existing test exercised that path *through `entry_from_ty` specifically* (as opposed to
  /// through `.entry()` directly, which the two `descriptor_entry_*_without_ty_*` tests above
  /// already cover). `BindingType::Other` is reachable here despite `BindingType` being
  /// `#[ non_exhaustive ]`: that attribute blocks external exhaustive matching and struct-literal
  /// variant construction, not plain unit-variant construction -- verified via `cargo check
  /// --target wasm32-unknown-unknown --tests` and a live `wasm_bindgen_test` run during this
  /// audit, both green.
  #[ wasm_bindgen_test ]
  fn descriptor_entry_from_ty_with_other_still_errors_test()
  {
    let result = gl::BindGroupLayoutDescriptor::new().entry_from_ty( gl::BindingType::Other );

    assert!
    (
      result.is_err(),
      "BindGroupLayoutDescriptor::entry_from_ty( BindingType::Other ) must propagate TypeNotSet, matching its own documented `# Errors` contract"
    );
  }
}
