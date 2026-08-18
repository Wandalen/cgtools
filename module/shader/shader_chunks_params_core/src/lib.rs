//! Discovers tunable parameters declared in a shader chunk's `//@ param:`
//! manifest lines — a new repeatable line in the same flat `//@`-prefixed
//! header block `shader_chunks_core` already uses for `name`/`description`/
//! `tags`/`depends_on`/`export` ( see `docs/api/001_tunable_parameter_taxonomy.md` ).
//! A tunable parameter is one of 5 kinds — function argument, compile-time
//! define directive, uniform, attribute, or texture — each carrying either
//! a declared `range(min, max)` or, when absent, a range resolved by
//! [`range_infer`]'s deterministic two-stage heuristic ( see
//! `docs/algorithm/001_range_inference_heuristic.md` ): name-substring
//! pattern match first, WGSL-type fallback second.
//!
//! This crate only discovers and describes declared tunables from raw WGSL
//! text — it does not execute, bind, or animate anything, and it does not
//! modify `shader_chunks_core` or any bundled chunk file.

mod private
{

  /// The 5 tunable-parameter kinds a `//@ param:` line may declare, spelled
  /// to match the grammar's `<kind>` token verbatim.
  #[ derive( Debug, Clone, Copy, PartialEq, Eq ) ]
  pub enum ParameterKind
  {
    /// A plain WGSL function argument.
    Argument,
    /// A compile-time `override`-style define directive.
    Define,
    /// A uniform-buffer field.
    Uniform,
    /// A vertex-stage attribute.
    Attribute,
    /// A bound texture.
    Texture,
  }

  /// The WGSL type token a `//@ param:` line's `<type>` position may carry,
  /// copied verbatim from the adjacent real declaration.
  #[ derive( Debug, Clone, Copy, PartialEq, Eq ) ]
  pub enum ValueType
  {
    /// WGSL `bool`.
    Bool,
    /// WGSL `u32`.
    U32,
    /// WGSL `i32`.
    I32,
    /// WGSL `f32`.
    F32,
    /// WGSL `vec2f`.
    Vec2F,
    /// WGSL `vec3f`.
    Vec3F,
    /// WGSL `vec4f`.
    Vec4F,
    /// WGSL `vec2i`.
    Vec2I,
    /// WGSL `vec3i`.
    Vec3I,
    /// WGSL `vec4i`.
    Vec4I,
    /// WGSL `vec2u`.
    Vec2U,
    /// WGSL `vec3u`.
    Vec3U,
    /// WGSL `vec4u`.
    Vec4U,
    /// WGSL `texture_2d`.
    Texture2d,
  }

  /// Whether a [`Parameter`]'s [`Range`] came from an explicit `range(min,
  /// max)` clause or from [`range_infer`]'s heuristic.
  #[ derive( Debug, Clone, Copy, PartialEq, Eq ) ]
  pub enum RangeSource
  {
    /// Read directly from the `//@ param:` line's own `range(min, max)` clause.
    Declared,
    /// Produced by [`range_infer`] because the line declared no range.
    Inferred,
  }

  /// An inclusive numeric range, either declared or inferred.
  #[ derive( Debug, Clone, Copy, PartialEq ) ]
  pub struct Range
  {
    /// The range's lower bound.
    pub min : f64,
    /// The range's upper bound.
    pub max : f64,
  }

  /// One `//@ param:` line, fully parsed.
  #[ derive( Debug, Clone, PartialEq ) ]
  pub struct Parameter
  {
    /// The parameter's name, as declared.
    pub name : String,
    /// The parameter's kind.
    pub kind : ParameterKind,
    /// The parameter's WGSL type.
    pub value_type : ValueType,
    /// The parameter's range and where it came from — `None` when neither
    /// declared nor inferable ( e.g. a texture or a `bool` ).
    pub range : Option< ( Range, RangeSource ) >,
  }

  /// Parses every `//@ param: <name> <kind> <type> [range(min, max)]` line
  /// in `wgsl`, in file order, resolving each line's range via
  /// [`range_infer`] when no `range(min, max)` clause is present. Returns an
  /// empty `Vec` when `wgsl` declares no `//@ param:` lines.
  ///
  /// # Panics
  ///
  /// Panics with a message naming the offending line when a `//@ param:`
  /// line has the wrong argument count, an unknown kind token, an unknown
  /// WGSL type token, or a malformed `range(min, max)` clause — chunk
  /// manifests are trusted authored content, not adversarial input,
  /// mirroring `shader_chunks_core::manifest_field`'s panic-on-malformed
  /// idiom.
  #[ must_use ]
  pub fn discover( wgsl : &str ) -> Vec< Parameter >
  {
    param_lines( wgsl ).map( param_line_parse ).collect()
  }

  /// [`discover`] over a [`shader_chunks_core::ChunkDescriptor`]'s own WGSL
  /// source — this crate's only dependency on `shader_chunks_core`;
  /// [`discover`] itself has none.
  ///
  /// # Panics
  ///
  /// Same panic contract as [`discover`].
  #[ must_use ]
  pub fn chunk_discover( chunk : &shader_chunks_core::ChunkDescriptor ) -> Vec< Parameter >
  {
    discover( chunk.wgsl )
  }

  /// Resolves a range for a parameter that declared none, per the two-stage
  /// heuristic in `docs/algorithm/001_range_inference_heuristic.md`:
  /// `kind == Texture` or `value_type == Bool` never carries a numeric
  /// range; otherwise a name-substring pattern is tried first, falling back
  /// to a WGSL-type-keyed default when no pattern matches `name`.
  #[ must_use ]
  pub fn range_infer( kind : ParameterKind, value_type : ValueType, name : &str ) -> Option< Range >
  {
    if kind == ParameterKind::Texture || value_type == ValueType::Bool
    {
      return None;
    }

    range_by_name_infer( name ).or_else( || range_by_type_infer( value_type ) )
  }

  fn range_by_name_infer( name : &str ) -> Option< Range >
  {
    let patterns : &[ ( &[ &str ], Range ) ] =
    &[
      ( &[ "octaves", "count", "steps", "iterations" ], Range { min : 1.0, max : 8.0 } ),
      ( &[ "seed" ], Range { min : 0.0, max : 65535.0 } ),
      ( &[ "angle", "rotation" ], Range { min : 0.0, max : std::f64::consts::TAU } ),
      ( &[ "scale", "frequency", "freq" ], Range { min : 0.1, max : 10.0 } ),
      ( &[ "amplitude", "weight", "opacity", "alpha", "mix", "blend" ], Range { min : 0.0, max : 1.0 } ),
      ( &[ "radius", "size", "width", "height" ], Range { min : 0.0, max : 100.0 } ),
    ];
    patterns.iter()
    .find( | ( needles, _ ) | needles.iter().any( | needle | name.contains( needle ) ) )
    .map( | ( _, range ) | *range )
  }

  fn range_by_type_infer( value_type : ValueType ) -> Option< Range >
  {
    match value_type
    {
      ValueType::U32 | ValueType::Vec2U | ValueType::Vec3U | ValueType::Vec4U => Some( Range { min : 0.0, max : 16.0 } ),
      ValueType::I32 | ValueType::Vec2I | ValueType::Vec3I | ValueType::Vec4I => Some( Range { min : -16.0, max : 16.0 } ),
      ValueType::F32 | ValueType::Vec2F | ValueType::Vec3F | ValueType::Vec4F => Some( Range { min : 0.0, max : 1.0 } ),
      ValueType::Bool | ValueType::Texture2d => None,
    }
  }

  // Fix(BUG-293): recognized a `//@ param:` line even with leading whitespace via
  // `.trim_start()`, diverging from `shader_chunks_core::manifest_field`'s own convention for
  // every sibling header field ( `name`/`description`/`tags`/`depends_on`/`export`/`stage` ),
  // which requires the `//@ ` prefix at column 0 with no leniency -- despite this crate's own
  // docs explicitly claiming `//@ param:` lives in "the same" flat header block under "the same
  // trust model".
  // Root cause: `param_lines` was written independently of `manifest_field`/`manifest_field_all`
  // rather than mirroring their exact recognition rule, so it silently gained a leniency none of
  // the other 6 manifest fields have.
  // Pitfall: a manifest system built on "malformed authored content panics loudly" ( see
  // `Error Handling` in `docs/api/001_tunable_parameter_taxonomy.md` ) depends on every field
  // sharing one predictable recognition rule -- a lone lenient field can silently accept content
  // ( e.g. an indented illustrative `//@ param:` line inside a doc-comment example ) that every
  // other field would correctly ignore or panic on, undermining that guarantee for one field only.
  fn param_lines( wgsl : &str ) -> impl Iterator< Item = &str >
  {
    wgsl.lines()
    .filter_map( | line | line.strip_prefix( "//@ param:" ) )
    .map( str::trim )
  }

  fn param_line_parse( line : &str ) -> Parameter
  {
    let mut parts = line.splitn( 4, ' ' );
    let name = parts.next().unwrap_or( "" );
    let kind_tok = parts.next()
      .unwrap_or_else( || panic!( "malformed `//@ param:` line (expected `<name> <kind> <type> [range(min, max)]`): {line:?}" ) );
    let type_tok = parts.next()
      .unwrap_or_else( || panic!( "malformed `//@ param:` line (expected `<name> <kind> <type> [range(min, max)]`): {line:?}" ) );
    let rest = parts.next().unwrap_or( "" ).trim();

    assert!( !name.is_empty(), "malformed `//@ param:` line (expected `<name> <kind> <type> [range(min, max)]`): {line:?}" );

    let kind = kind_parse( kind_tok, line );
    let value_type = value_type_parse( type_tok, line );

    let range = if rest.is_empty()
    {
      range_infer( kind, value_type, name ).map( | range | ( range, RangeSource::Inferred ) )
    }
    else
    {
      Some( ( range_clause_parse( rest, line ), RangeSource::Declared ) )
    };

    Parameter { name : name.to_string(), kind, value_type, range }
  }

  fn kind_parse( token : &str, line : &str ) -> ParameterKind
  {
    match token
    {
      "argument" => ParameterKind::Argument,
      "define" => ParameterKind::Define,
      "uniform" => ParameterKind::Uniform,
      "attribute" => ParameterKind::Attribute,
      "texture" => ParameterKind::Texture,
      _ => panic!( "malformed `//@ param:` line: unknown kind token `{token}`: {line:?}" ),
    }
  }

  fn value_type_parse( token : &str, line : &str ) -> ValueType
  {
    match token
    {
      "bool" => ValueType::Bool,
      "u32" => ValueType::U32,
      "i32" => ValueType::I32,
      "f32" => ValueType::F32,
      "vec2f" => ValueType::Vec2F,
      "vec3f" => ValueType::Vec3F,
      "vec4f" => ValueType::Vec4F,
      "vec2i" => ValueType::Vec2I,
      "vec3i" => ValueType::Vec3I,
      "vec4i" => ValueType::Vec4I,
      "vec2u" => ValueType::Vec2U,
      "vec3u" => ValueType::Vec3U,
      "vec4u" => ValueType::Vec4U,
      "texture_2d" => ValueType::Texture2d,
      _ => panic!( "malformed `//@ param:` line: unknown WGSL type token `{token}`: {line:?}" ),
    }
  }

  fn range_clause_parse( rest : &str, line : &str ) -> Range
  {
    let inner = rest.strip_prefix( "range(" )
      .and_then( | s | s.strip_suffix( ')' ) )
      .unwrap_or_else( || panic!( "malformed `//@ param:` range clause (expected `range(min, max)`): {line:?}" ) );
    let ( min_str, max_str ) = inner.split_once( ',' )
      .unwrap_or_else( || panic!( "malformed `//@ param:` range clause (expected `range(min, max)`): {line:?}" ) );
    let min = min_str.trim().parse::< f64 >()
      .unwrap_or_else( | _ | panic!( "malformed `//@ param:` range min (not a number): {min_str:?}" ) );
    let max = max_str.trim().parse::< f64 >()
      .unwrap_or_else( | _ | panic!( "malformed `//@ param:` range max (not a number): {max_str:?}" ) );
    Range { min, max }
  }

}

::mod_interface::mod_interface!
{
  own use ParameterKind;
  own use ValueType;
  own use RangeSource;
  own use Range;
  own use Parameter;
  own use discover;
  own use chunk_discover;
  own use range_infer;
}
