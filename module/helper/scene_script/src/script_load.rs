mod private
{
  use crate::{ check_top_level_is_declarative, check_whole_ast_is_pure, ImperativeTopLevelStatement, ImpureCall };

  /// Why a script was rejected: it never reached a valid AST at all
  /// ( `Parse` ), or it parsed but violates the structural convention its
  /// declared script form requires ( `Lint` ) — shared by both
  /// [`script_as_glue_load`] and [`script_as_data_load`], with `L` fixed to
  /// each form's own lint-violation type ( [`ImperativeTopLevelStatement`]
  /// for glue, [`ImpureCall`] for data ), so the two forms are never forced
  /// through a shared, lossier error shape.
  #[ derive( Debug ) ]
  #[ non_exhaustive ]
  pub enum ScriptLoadError< L >
  {
    /// `source` is not valid Rhai syntax.
    Parse( rhai::ParseError ),
    /// `source` parsed but violates its script form's structural convention.
    Lint( L ),
  }

  impl< L : core::fmt::Display > core::fmt::Display for ScriptLoadError< L >
  {
    #[ inline ]
    fn fmt( &self, f : &mut core::fmt::Formatter< '_ > ) -> core::fmt::Result
    {
      match self
      {
        Self::Parse( err ) => write!( f, "{err}" ),
        Self::Lint( err ) => write!( f, "{err}" ),
      }
    }
  }

  impl< L : core::fmt::Debug + core::fmt::Display > std::error::Error for ScriptLoadError< L > {}

  /// Compiles `source` against `engine` and rejects it unless every
  /// top-level statement is a declarative binding or the single trailing
  /// `main( ... )` entry-point call — the script-as-glue convention
  /// `check_top_level_is_declarative` enforces. The returned `AST` is ready
  /// to evaluate ( e.g. via `Engine::eval_ast` ) — no separate compile step
  /// is needed or should be performed again.
  ///
  /// # Errors
  ///
  /// Returns [`ScriptLoadError::Parse`] if `source` is not valid Rhai, or
  /// [`ScriptLoadError::Lint`] if it parses but violates the convention.
  #[ inline ]
  pub fn script_as_glue_load
  (
    engine : &rhai::Engine,
    source : &str
  ) -> Result< rhai::AST, ScriptLoadError< ImperativeTopLevelStatement > >
  {
    let ast = engine.compile( source ).map_err( ScriptLoadError::Parse )?;
    check_top_level_is_declarative( &ast ).map_err( ScriptLoadError::Lint )?;
    Ok( ast )
  }

  /// Compiles `source` against `engine` and rejects it unless the whole AST
  /// is free of engine calls, anywhere — the script-as-data convention
  /// `check_whole_ast_is_pure` enforces. The returned `AST` is ready to
  /// evaluate ( e.g. via `Engine::eval_ast` ) — no separate compile step is
  /// needed or should be performed again.
  ///
  /// # Errors
  ///
  /// Returns [`ScriptLoadError::Parse`] if `source` is not valid Rhai, or
  /// [`ScriptLoadError::Lint`] if it parses but calls the engine anywhere.
  #[ inline ]
  pub fn script_as_data_load
  (
    engine : &rhai::Engine,
    source : &str
  ) -> Result< rhai::AST, ScriptLoadError< ImpureCall > >
  {
    let ast = engine.compile( source ).map_err( ScriptLoadError::Parse )?;
    check_whole_ast_is_pure( &ast ).map_err( ScriptLoadError::Lint )?;
    Ok( ast )
  }
}

crate::mod_interface!
{
  orphan use
  {
    ScriptLoadError,
    script_as_glue_load,
    script_as_data_load,
  };
}
