mod private
{
  /// A `FnCall`/`MethodCall` node found somewhere in the AST of a script
  /// required to be pure data.
  #[ derive( Debug ) ]
  #[ non_exhaustive ]
  pub struct ImpureCall
  {
    /// Where the offending call starts.
    pub position : rhai::Position,
    /// The called function's name — an operator token (e.g. `"+"`) when the
    /// call is a desugared operator, otherwise the named function/method.
    pub name : String,
  }

  impl core::fmt::Display for ImpureCall
  {
    #[ inline ]
    fn fmt( &self, f : &mut core::fmt::Formatter< '_ > ) -> core::fmt::Result
    {
      write!
      (
        f,
        "impure call to `{}` at {} — a script-as-data document may not call the engine, \
        anywhere in the script, including via operators",
        self.name, self.position
      )
    }
  }

  impl std::error::Error for ImpureCall {}

  /// Extracts the offending call's name and position if `node` is a
  /// `FnCall`/`MethodCall` statement or expression — the two shapes a call
  /// can take depending on whether it sits in statement or expression
  /// position.
  fn call_in_node( node : rhai::ASTNode< '_ > ) -> Option< ( &str, rhai::Position ) >
  {
    match node
    {
      rhai::ASTNode::Stmt( rhai::Stmt::FnCall( call, position ) )
      | rhai::ASTNode::Expr( rhai::Expr::FnCall( call, position ) | rhai::Expr::MethodCall( call, position ) ) =>
        Some( ( call.name.as_str(), *position ) ),
      _ => None,
    }
  }

  /// Recursively checks that no statement or expression anywhere in `ast`
  /// — top-level, inside `let` initializers, array elements, object-map
  /// values, or the body of any nested block, `if`, loop, `switch`,
  /// `try`/`catch`, or script-defined function — calls a function, whether
  /// named or a desugared operator.
  ///
  /// Delegates traversal to `rhai::AST::walk`, which already descends into
  /// every statement and expression position, including script-defined
  /// function bodies that `AST::statements()` alone does not reach.
  ///
  /// This is a structural, not semantic, check limited to `FnCall`/
  /// `MethodCall` nodes — it does not catch non-call side channels (e.g. a
  /// `const` referencing an engine-registered constant).
  ///
  /// # Errors
  ///
  /// Returns `Err` naming the first `FnCall`/`MethodCall` node found,
  /// anywhere in the AST.
  #[ inline ]
  pub fn check_whole_ast_is_pure( ast : &rhai::AST ) -> Result< (), ImpureCall >
  {
    let mut found : Option< ImpureCall > = None;

    ast.walk( &mut | path |
    {
      let Some( node ) = path.last().copied() else { return true };

      match call_in_node( node )
      {
        Some( ( name, position ) ) =>
        {
          found = Some( ImpureCall { position, name : name.to_string() } );
          false
        }
        None => true,
      }
    } );

    match found
    {
      Some( impure ) => Err( impure ),
      None => Ok( () ),
    }
  }
}

crate::mod_interface!
{
  orphan use
  {
    check_whole_ast_is_pure,
    ImpureCall,
  };
}
