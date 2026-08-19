//! Verifies every example `.rhai` script follows the project's
//! top-level-bindings convention: only `let`/`const` bindings and, as
//! the final statement, a single entry-point call may sit at top level —
//! all imperative code (loops, branches, mutation) must live inside a
//! function. See `scene_script::check_top_level_is_declarative`.

use scene_script::{ engine_build, check_top_level_is_declarative };
use std::{ fs, path::PathBuf };

fn example_scripts() -> Vec< PathBuf >
{
  let examples_root = PathBuf::from( concat!( env!( "CARGO_MANIFEST_DIR" ), "/../../../examples/scene_script" ) );

  let mut scripts = Vec::new();

  for category_entry in fs::read_dir( &examples_root ).unwrap()
  {
    let src_dir = category_entry.unwrap().path().join( "src" );
    if !src_dir.is_dir()
    {
      continue;
    }

    for file_entry in fs::read_dir( &src_dir ).unwrap()
    {
      let path = file_entry.unwrap().path();
      if path.extension().is_some_and( | ext | ext == "rhai" )
      {
        scripts.push( path );
      }
    }
  }

  scripts.sort();
  scripts
}

#[ test ]
fn example_scripts_follow_declarative_top_level_convention()
{
  let engine = engine_build();
  let scripts = example_scripts();

  assert!
  (
    !scripts.is_empty(),
    "expected to find at least one example .rhai script under examples/scene_script/*/src/"
  );

  for path in &scripts
  {
    let source = fs::read_to_string( path ).unwrap();
    let ast = engine.compile( &source ).unwrap_or_else( | err | panic!( "{} failed to compile: {err}", path.display() ) );

    if let Err( violation ) = check_top_level_is_declarative( &ast )
    {
      panic!( "{}: {violation}", path.display() );
    }
  }
}

#[ test ]
fn checker_rejects_a_top_level_loop()
{
  let engine = engine_build();
  let ast = engine.compile( "let x = 1; for i in 0..3 { x += 1; }" ).unwrap();

  let violation = check_top_level_is_declarative( &ast ).expect_err( "a top-level `for` loop must be rejected" );
  assert_eq!( violation.kind, "for" );
}

#[ test ]
fn checker_rejects_a_premature_top_level_call()
{
  let engine = engine_build();
  let ast = engine.compile( "fn main() {} main(); let x = 1;" ).unwrap();

  let violation = check_top_level_is_declarative( &ast ).expect_err( "a call that isn't the final statement must be rejected" );
  assert_eq!( violation.kind, "function call" );
}

#[ test ]
fn checker_rejects_a_trailing_call_to_something_other_than_main()
{
  let engine = engine_build();
  let ast = engine.compile( "fn not_main() {} not_main();" ).unwrap();

  let violation = check_top_level_is_declarative( &ast ).expect_err( "a trailing call that isn't `main` must be rejected" );
  assert_eq!( violation.kind, "function call" );
}

#[ test ]
fn checker_accepts_bindings_plus_trailing_main_call()
{
  let engine = engine_build();
  let ast = engine.compile( "let x = 1; fn main( x ) { for i in 0..x {} } main( x )" ).unwrap();

  check_top_level_is_declarative( &ast ).unwrap();
}

#[ test ]
fn checker_rejects_a_trailing_non_main_call_without_semicolon()
{
  // A bare trailing call keeps the same `Stmt::FnCall` shape whether or not
  // it ends in a semicolon — Rhai does not route the semicolon-less
  // (implicit-return) form through `Stmt::Expr` the way it does for other
  // expression kinds. This locks that fact down: the reject path was
  // previously exercised only with a trailing `;` (see
  // `checker_rejects_a_trailing_call_to_something_other_than_main`); this
  // closes the matching semicolon-less gap.
  let engine = engine_build();
  let ast = engine.compile( "fn not_main() {} not_main()" ).unwrap();

  let violation = check_top_level_is_declarative( &ast ).expect_err( "a trailing non-main call must be rejected even without a semicolon" );
  assert_eq!( violation.kind, "function call" );
}

#[ test ]
fn checker_rejects_a_trailing_non_main_method_call()
{
  // `t.update( .. )` parses as `Expr::Dot( BinaryExpr { rhs: Expr::MethodCall( .. ), .. } )`
  // — the call sits one level behind the dot, not at the statement's own
  // expression top level like the bare-function-name shapes covered by the
  // two tests above. `call_expr()` must unwrap the `Dot` to reach it and
  // classify this as `Role::Call` (rejected, being non-`main`); without
  // that unwrap this silently fell through to `Role::PlainExpression` and
  // was never checked at all.
  let engine = engine_build();
  let ast = engine.compile( "let t = tween( f32x2(0.0, 0.0), f32x2(1.0, 0.0), 1.0 ); t.update(0.5)" ).unwrap();

  let violation = check_top_level_is_declarative( &ast ).expect_err( "a trailing method call other than `main` must be rejected" );
  assert_eq!( violation.kind, "expression" );
}

#[ test ]
fn checker_rejects_a_top_level_if()
{
  // `for` is the only `Role::Imperative` kind the existing tests exercise;
  // `if` is a structurally distinct `Stmt` variant that falls through the
  // same catch-all. The condition and body can't be trivial, though:
  // `engine_build()` runs Rhai's default `OptimizationLevel::Simple`, which
  // folds an empty-bodied, else-less `if` into `Stmt::Block` regardless of
  // its condition (taking the branch or not is observationally identical)
  // — and a body holding only an unused `let` is dead-code-eliminated down
  // to empty first, so `if cond { let y = 1; }` collapses the same way
  // even though it isn't textually empty. A condition the optimizer can't
  // evaluate (a script-`fn` call — `Simple` never evaluates calls, only
  // `Full` does) plus a body that mutates an outer binding read afterward
  // (so the mutation can't be proven dead) together survive as `Stmt::If`.
  let engine = engine_build();
  let ast = engine.compile( "let x = 1; fn condition() { true } if condition() { x += 1; } x" ).unwrap();

  let violation = check_top_level_is_declarative( &ast ).expect_err( "a top-level `if` must be rejected" );
  assert_eq!( violation.kind, "if" );
}

#[ test ]
fn checker_accepts_a_const_binding()
{
  // The convention's own prose (and this checker's doc comment) promises
  // `let`/`const` bindings alike are allowed at top level, but every
  // existing accept-case here uses `let` only — `const` had no dedicated
  // coverage prior to this test.
  let engine = engine_build();
  let ast = engine.compile( "const X = 1; X" ).unwrap();

  check_top_level_is_declarative( &ast ).unwrap();
}

#[ test ]
fn checker_accepts_bindings_plus_bare_trailing_expression_with_no_call()
{
  // Mirrors `f32x2_vector_arithmetic.rhai`'s actual shape: the declarative
  // pattern needs no `main()` (or any call) at all — a bare value-producing
  // expression as the trailing statement is sufficient on its own.
  let engine = engine_build();
  let ast = engine.compile( "let a = f32x2( 1.0, 2.0 ); let b = f32x2( 3.0, 4.0 ); a + b" ).unwrap();

  check_top_level_is_declarative( &ast ).unwrap();
}

#[ test ]
fn checker_accepts_multiple_top_level_function_definitions()
{
  // Confirms `fn` definitions never appear in `ast.statements()` regardless
  // of how many are declared — only `helper()`'s *call* would count as a
  // statement, and nothing here calls it.
  let engine = engine_build();
  let ast = engine.compile( "fn helper( n ) { n + 1 } fn main( x ) { helper( x ) } main( 1 )" ).unwrap();

  check_top_level_is_declarative( &ast ).unwrap();
}

/// `bug_reproducer(BUG-351)`
///
/// Root Cause: `call_in_expr()`'s `Expr::Dot` arm only recursed into the chain's `.rhs` — when
/// the chain's own tail is a plain property read (e.g. `.x`) rather than another call, and the
/// real call sits in the receiver (`.lhs`) instead, no call is ever found; `role()` then falls
/// through to `Role::PlainExpression`, which is allowed at any top-level position.
///
/// Why Not Caught: the only existing dotted-call test
/// (`checker_rejects_a_trailing_non_main_method_call`) puts the call in the chain's own tail
/// (`t.update(0.5)`) — the one shape `.rhs`-only recursion already handled; no test exercised a
/// call sitting in the chain's receiver instead.
///
/// Fix Applied: `call_in_expr()`'s `Dot` arm now falls back to `.lhs` whenever `.rhs` yields no
/// call, so a call on either side of a dot — at any nesting depth — is found.
///
/// Prevention: this test pins the exact reported shape (`trigger().x`) as a permanent
/// regression guard.
///
/// Pitfall: a Rhai dot chain can carry its one real call on EITHER side of the dot, not only
/// the tail — code that unwinds only one side silently misses the other.
#[ test ]
fn checker_rejects_a_trailing_call_disguised_as_a_dot_chain_property_read()
{
  // `trigger()` is a real, non-operator function call sitting in the RECEIVER position of the
  // dot chain; `.x` (the chain's own tail) is a plain property read, not a call. Before the
  // fix, `call_in_expr()` only ever inspected the tail, so this whole statement was silently
  // misclassified as `Role::PlainExpression` (allowed anywhere) instead of `Role::Call(
  // "trigger" )` (rejected, since `trigger` isn't `main`).
  let engine = engine_build();
  let ast = engine.compile( "fn trigger() { #{ x: 1 } } trigger().x" ).unwrap();

  let violation = check_top_level_is_declarative( &ast )
    .expect_err( "a call sitting in a dot chain's receiver must be rejected even when the chain's own tail is a plain property read" );
  assert_eq!( violation.kind, "expression" );
}
