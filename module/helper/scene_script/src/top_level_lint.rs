mod private
{
  /// A top-level statement that isn't a declarative binding or the
  /// script's single trailing entry-point call.
  #[ derive( Debug ) ]
  #[ non_exhaustive ]
  pub struct ImperativeTopLevelStatement
  {
    /// Where the offending statement starts.
    pub position : rhai::Position,
    /// Which kind of `rhai::Stmt` triggered the rejection (e.g. `"for"`).
    pub kind : &'static str,
  }

  impl core::fmt::Display for ImperativeTopLevelStatement
  {
    #[ inline ]
    fn fmt( &self, f : &mut core::fmt::Formatter< '_ > ) -> core::fmt::Result
    {
      write!
      (
        f,
        "top-level `{}` statement at {} — imperative code must live inside a function \
        (e.g. `main`); only declarative bindings and a single trailing entry-point call \
        are allowed at top level",
        self.kind, self.position
      )
    }
  }

  impl std::error::Error for ImperativeTopLevelStatement {}

  fn stmt_kind( stmt : &rhai::Stmt ) -> &'static str
  {
    match stmt
    {
      rhai::Stmt::Var( .. ) => "let/const",
      rhai::Stmt::Noop( .. ) => "noop",
      rhai::Stmt::Expr( .. ) => "expression",
      rhai::Stmt::FnCall( .. ) => "function call",
      rhai::Stmt::If( .. ) => "if",
      rhai::Stmt::Switch( .. ) => "switch",
      rhai::Stmt::While( .. ) => "while/loop",
      rhai::Stmt::Do( .. ) => "do..while",
      rhai::Stmt::For( .. ) => "for",
      rhai::Stmt::Assignment( .. ) => "assignment",
      rhai::Stmt::Block( .. ) => "block",
      rhai::Stmt::TryCatch( .. ) => "try/catch",
      rhai::Stmt::BreakLoop( .. ) => "break/continue",
      rhai::Stmt::Return( .. ) => "return/throw",
      _ => "statement",
    }
  }

  /// The one function name allowed to appear as a bare top-level call —
  /// matches the `fn main( ... ) { ... }` convention this checker enforces.
  const ENTRY_POINT_NAME : &str = "main";

  /// What role a top-level statement plays, for convention-checking
  /// purposes.
  enum Role< 'a >
  {
    /// A declarative binding (`let`/`const`) or a no-op.
    Binding,
    /// A value-producing expression with no side effect: a literal, a
    /// variable reference, an operator expression (`a + b`, `-x`, ...), or
    /// any other non-call expression.
    ///
    /// Rhai represents operator expressions as `FnCallExpr` too — `+`/`*`
    /// on a custom type like `F32x2` desugar to a function call under the
    /// hood (registered via `Engine::register_fn`) — but they're
    /// declarative arithmetic, not an imperative action, so they're
    /// classified here rather than as a `Call`.
    PlainExpression,
    /// A call to a named function — `main( ... )`, `foo()`, etc.
    Call( &'a str ),
    /// Anything else: a loop, branch-as-statement, assignment, and so on.
    Imperative,
  }

  /// Extracts the `FnCallExpr` if `stmt` is a bare call statement or a
  /// trailing (implicit-return) call expression — either shape can carry
  /// a call, since Rhai collapses "a single call expression forming the
  /// whole statement" into `Stmt::FnCall` directly, whether the call is a
  /// named one or an operator.
  ///
  /// A dotted method call (`receiver.method( .. )`) does not appear as a
  /// bare `Expr::MethodCall` at the statement's own top level — Rhai wraps
  /// it in `Expr::Dot( BinaryExpr { rhs, .. }, .. )` with the call sitting
  /// in `rhs`, one level behind the dot (chained dots nest the same way,
  /// `rhs`-first, all the way down to the terminal call), so the wrapper
  /// must be unwound to reach it.
  fn call_expr( stmt : &rhai::Stmt ) -> Option< &rhai::FnCallExpr >
  {
    fn call_in_expr( expr : &rhai::Expr ) -> Option< &rhai::FnCallExpr >
    {
      match expr
      {
        rhai::Expr::FnCall( call, .. ) | rhai::Expr::MethodCall( call, .. ) => Some( call ),
        rhai::Expr::Dot( binary, .. ) => call_in_expr( &binary.rhs ),
        _ => None,
      }
    }

    match stmt
    {
      rhai::Stmt::FnCall( call, .. ) => Some( call ),
      rhai::Stmt::Expr( inner ) => call_in_expr( inner ),
      _ => None,
    }
  }

  fn role( stmt : &rhai::Stmt ) -> Role< '_ >
  {
    match ( stmt, call_expr( stmt ) )
    {
      ( rhai::Stmt::Var( .. ) | rhai::Stmt::Noop( .. ), _ ) => Role::Binding,
      ( _, Some( call ) ) if call.is_operator_call() => Role::PlainExpression,
      ( _, Some( call ) ) => Role::Call( call.name.as_str() ),
      ( rhai::Stmt::Expr( .. ), None ) => Role::PlainExpression,
      _ => Role::Imperative,
    }
  }

  /// Checks that every top-level statement in `ast` is either a
  /// declarative binding (`let`/`const`), a value-producing expression
  /// with no side effect (e.g. a script's trailing return value), or —
  /// only as the very last statement, and only calling `main` — the
  /// single call that kicks off execution.
  ///
  /// `rhai::AST::statements()` returns only the top-level list; it never
  /// descends into `fn` bodies, so imperative code (loops, branches,
  /// mutation) properly nested inside `fn main() { ... }` is exactly what
  /// this permits — it only rejects imperative constructs sitting bare at
  /// top level, outside of any function.
  ///
  /// This is a structural check, not a semantic one: a `let` binding whose
  /// initializer calls a side-effecting host function is indistinguishable
  /// from a pure one at the AST level, so it is not caught here.
  ///
  /// # Errors
  ///
  /// Returns `Err` naming the first top-level statement that is neither a
  /// declarative binding nor the final `main( ... )` entry-point call.
  #[ inline ]
  pub fn check_top_level_is_declarative( ast : &rhai::AST ) -> Result< (), ImperativeTopLevelStatement >
  {
    let statements = ast.statements();
    let last_index = statements.len().saturating_sub( 1 );

    for ( index, stmt ) in statements.iter().enumerate()
    {
      let allowed = match role( stmt )
      {
        Role::Binding | Role::PlainExpression => true,
        Role::Call( ENTRY_POINT_NAME ) => index == last_index,
        Role::Call( _ ) | Role::Imperative => false,
      };

      if !allowed
      {
        return Err( ImperativeTopLevelStatement { position : stmt.position(), kind : stmt_kind( stmt ) } );
      }
    }

    Ok( () )
  }
}

crate::mod_interface!
{
  orphan use
  {
    check_top_level_is_declarative,
    ImperativeTopLevelStatement,
  };
}
