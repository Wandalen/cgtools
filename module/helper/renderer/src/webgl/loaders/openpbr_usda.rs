mod private
{
  /// One `Material` prim's reference to an external MaterialX asset, as
  /// authored in USD text ( e.g. the Digital Production Example Library
  /// OpenPBRShaderPlayground corpus ):
  ///
  /// ```text
  /// def Material "iceCube" (
  ///     prepend references = @./iceCube.mtlx@</MaterialX/Materials/iceCube>
  /// )
  /// ```
  ///
  /// `material` is the USD prim name; `mtlx_asset` is the referenced file path;
  /// `mtlx_target` is the optional `<…>` prim target inside that asset.
  #[ derive( Clone, Debug, PartialEq ) ]
  pub struct UsdaMtlxReference
  {
    /// Name of the USD `Material` prim carrying the reference.
    pub material : String,
    /// Referenced `.mtlx` ( or other layer ) file path from the `@…@` token.
    pub mtlx_asset : String,
    /// Optional `</…>` target prim path inside the referenced asset.
    pub mtlx_target : Option< String >,
  }

  /// Name of a `Material` prim from a `def Material "Name" ( … )` /
  /// `over Material "Name" ( … )` line.
  fn material_name( line : &str ) -> Option< String >
  {
    let keyword = line.find( "Material " )?;
    let rest = &line[ keyword + 9 .. ];
    let open = rest.find( '"' )?;
    let after = &rest[ open + 1 .. ];
    let close = after.find( '"' )?;
    Some( after[ .. close ].to_string() )
  }

  /// Extracts every `@<asset>@</target>` pair from an accumulated material
  /// block ( references can span parenthesised metadata lines ).
  fn collect_asset_tokens( block : &str ) -> Vec< ( String, Option< String > ) >
  {
    let mut tokens = Vec::new();
    let mut rest = block;
    while let Some( open ) = rest.find( '@' )
    {
      let after_open = &rest[ open + 1 .. ];
      let Some( close ) = after_open.find( '@' ) else { break };
      let asset = after_open[ .. close ].trim().to_string();
      if !asset.is_empty()
      {
        let after_close = after_open[ close + 1 .. ].trim_start();
        let target = if let Some( lt ) = after_close.find( '<' )
        {
          let inner = &after_close[ lt + 1 .. ];
          inner.find( '>' ).map( | gt | inner[ .. gt ].to_string() )
        }
        else
        {
          None
        };
        tokens.push( ( asset, target ) );
      }
      rest = &after_open[ close + 1 .. ];
    }
    tokens
  }

  /// Scans a `.usda` document for `Material` prims that reference external
  /// MaterialX ( `.mtlx` ) assets. Pure text parsing — no USD composition, no
  /// file IO, no GPU — matching the OpenPBR content pattern where the actual
  /// surface lives in the referenced `.mtlx` ( which the N2 reader parses ).
  ///
  /// Returns every `@…mtlx…@` asset token found inside each `Material`
  /// definition, in document order. Non-material references are ignored; a
  /// `.usda` whose materials embed their surface inline ( no external asset )
  /// yields nothing here — that content needs the `openusd` Stage reader (§2.3
  /// N3 full) instead.
  #[ must_use ]
  pub fn usda_mtlx_references( usda : &str ) -> Vec< UsdaMtlxReference >
  {
    let mut out = Vec::new();
    let mut material : Option< String > = None;
    let mut block = String::new();
    let mut opened = false;
    let mut depth = 0i32;

    for line in usda.lines()
    {
      if material.is_none()
      {
        if let Some( name ) = material_name( line )
        {
          material = Some( name );
          block.clear();
          opened = false;
          depth = 0;
        }
        continue;
      }

      let opens = line.matches( '{' ).count() as i32;
      let closes = line.matches( '}' ).count() as i32;
      if opens > 0
      {
        opened = true;
      }
      depth += opens - closes;

      if opened && depth <= 0
      {
        // Material scope closed on this line — flush its references.
        let name = material.take().unwrap_or_default();
        for ( asset, target ) in collect_asset_tokens( &block )
        {
          out.push( UsdaMtlxReference { material : name.clone(), mtlx_asset : asset, mtlx_target : target } );
        }
        continue;
      }

      block.push_str( line );
      block.push( '\n' );
    }

    out
  }
}

crate::mod_interface!
{
  own use
  {
    usda_mtlx_references,
    UsdaMtlxReference
  };
}
