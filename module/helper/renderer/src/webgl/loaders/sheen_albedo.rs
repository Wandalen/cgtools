mod private
{
  use std::f32::consts::TAU;
  use minwebgl as gl;

  /// Lower bound the shader clamps the fuzz roughness to before it reaches
  /// `D_Charlie` ( `clamp( material.sheenRoughness, 0.1, 1.0 )` in `main.frag` ).
  /// The Charlie NDF has a `1 / alpha` exponent, so it degenerates into a
  /// zero-width spike as `alpha -> 0`; the shader's floor keeps the grazing rim
  /// from aliasing, and the table has to be built with the *same* floor or the
  /// albedo it reports would not be the albedo of the lobe actually evaluated.
  pub const FUZZ_ALPHA_MIN : f32 = 0.1;

  /// Directional albedo `E( mu, alpha )` of the fuzz ( sheen ) lobe, computed by
  /// [`sheen_albedo_table`]. Laid out `roughness_samples` rows x `uv_samples`
  /// columns ( row = fuzz roughness, column = `mu = cos(theta_v)` ), matching the
  /// Kulla-Conty table's layout so both LUTs sample with the same `vec2( mu, r )`
  /// convention.
  #[ derive( Clone, Debug, PartialEq ) ]
  pub struct SheenAlbedoTable
  {
    /// `E( mu, alpha )`, `[ roughness * uv_samples + uv ]`.
    pub e : Vec< f32 >,
    /// Number of `mu` ( cos theta ) samples per row.
    pub uv_samples : usize,
    /// Number of fuzz-roughness samples ( rows ).
    pub roughness_samples : usize,
  }

  /// The Charlie ( Estevez-Kulla ) sheen NDF, mirroring `D_Charlie` in
  /// `main.frag` exactly - including the `sin^2 h` floor that keeps the
  /// `pow` finite at `dotNH = 1`.
  fn d_charlie( alpha : f32, dot_nh : f32 ) -> f32
  {
    let inv_alpha = 1.0 / alpha;
    let sin2h = ( 1.0 - dot_nh * dot_nh ).max( 0.007_812_5 );
    ( 2.0 + inv_alpha ) * sin2h.powf( 0.5 * inv_alpha ) / TAU
  }

  /// The Ashikhmin-Premoze visibility term, mirroring `V_Ashikhmin` in
  /// `main.frag`.
  fn v_ashikhmin( no_l : f32, no_v : f32 ) -> f32
  {
    ( 0.25 / ( no_l + no_v - no_l * no_v ).max( 1e-5 ) ).clamp( 0.0, 1.0 )
  }

  /// Directional albedo of the white fuzz lobe seen at `mu = cos(theta_v)`:
  /// `E( mu, alpha ) = integral over the hemisphere of D_Charlie * V_Ashikhmin * NoL`.
  ///
  /// The result is clamped to `[ 0, 1 ]`: it is an albedo, but the
  /// Ashikhmin-Premoze visibility term is a fit rather than a real
  /// `G / ( 4 NoL NoV )`, and at exact grazing ( `mu = 0` ) with a narrow lobe it
  /// integrates to marginally above one. Letting that through would make the
  /// substrate scaling `1 - max3( fuzz_color ) * E` go negative.
  ///
  /// The lobe is smooth and low-frequency, so a deterministic midpoint rule over
  /// `( cos theta_l, phi )` converges faster here than importance sampling would,
  /// and unlike a stochastic estimate it makes the table byte-for-byte
  /// reproducible, which is what the regression tests lean on. `alpha` is
  /// clamped to [`FUZZ_ALPHA_MIN`] to match the shader.
  ///
  /// # Panics
  ///
  /// Panics if `cos_samples` or `phi_samples` is `0`.
  #[ must_use ]
  pub fn sheen_directional_albedo( mu : f32, alpha : f32, cos_samples : usize, phi_samples : usize ) -> f32
  {
    assert!( cos_samples > 0, "cos_samples must be > 0" );
    assert!( phi_samples > 0, "phi_samples must be > 0" );

    let alpha = alpha.clamp( FUZZ_ALPHA_MIN, 1.0 );
    let mu = mu.clamp( 0.0, 1.0 );
    // View direction in the tangent frame ( N = +Z ), at cos theta = mu.
    let v = [ ( 1.0 - mu * mu ).max( 0.0 ).sqrt(), 0.0, mu ];

    let mut sum = 0.0_f32;
    for j in 0 .. cos_samples
    {
      let cos_l = ( j as f32 + 0.5 ) / cos_samples as f32;
      let sin_l = ( 1.0 - cos_l * cos_l ).max( 0.0 ).sqrt();
      for i in 0 .. phi_samples
      {
        let phi = TAU * ( i as f32 + 0.5 ) / phi_samples as f32;
        let l = [ sin_l * phi.cos(), sin_l * phi.sin(), cos_l ];

        let h = [ v[ 0 ] + l[ 0 ], v[ 1 ] + l[ 1 ], v[ 2 ] + l[ 2 ] ];
        let h_len = ( h[ 0 ] * h[ 0 ] + h[ 1 ] * h[ 1 ] + h[ 2 ] * h[ 2 ] ).sqrt().max( 1e-6 );
        let dot_nh = ( h[ 2 ] / h_len ).clamp( 0.0, 1.0 );

        sum += d_charlie( alpha, dot_nh ) * v_ashikhmin( cos_l, mu ) * cos_l;
      }
    }

    // d omega = d( cos theta ) d phi, midpoint weights over [ 0, 1 ] x [ 0, 2pi ).
    ( sum * TAU / ( cos_samples * phi_samples ) as f32 ).clamp( 0.0, 1.0 )
  }

  /// Computes the fuzz directional-albedo grid `E( mu, alpha )` used for the
  /// energy ( albedo ) scaling of the substrate underneath the fuzz layer.
  ///
  /// OpenPBR layers fuzz over the base as `layer( base, fuzz, weight )`, i.e. the
  /// fuzz takes its energy *from* the base rather than adding to it; the glTF
  /// formulation of the same thing is
  /// `f = f_fuzz + f_base * ( 1 - max3( fuzz_color ) * E( mu ) )`. That needs the
  /// lobe's directional albedo, which has no closed form for the Charlie NDF -
  /// hence this table.
  ///
  /// Pure CPU work, so it is natively unit-testable and can be precomputed once
  /// at startup and uploaded as a 2D texture ( see [`sheen_albedo_lut_upload`] ).
  ///
  /// # Panics
  ///
  /// Panics if `uv_samples` or `roughness_samples` is `< 2`.
  #[ must_use ]
  pub fn sheen_albedo_table
  (
    uv_samples : usize,
    roughness_samples : usize,
    cos_samples : usize,
    phi_samples : usize
  ) -> SheenAlbedoTable
  {
    assert!( uv_samples >= 2, "uv_samples must be >= 2" );
    assert!( roughness_samples >= 2, "roughness_samples must be >= 2" );

    let mut e = vec![ 0.0_f32; uv_samples * roughness_samples ];
    for r in 0 .. roughness_samples
    {
      let alpha = r as f32 / ( roughness_samples - 1 ) as f32;
      for u in 0 .. uv_samples
      {
        let mu = u as f32 / ( uv_samples - 1 ) as f32;
        e[ r * uv_samples + u ] = sheen_directional_albedo( mu, alpha, cos_samples, phi_samples );
      }
    }

    SheenAlbedoTable { e, uv_samples, roughness_samples }
  }

  /// Uploads the fuzz directional-albedo table as an `RGBA32F` 2D texture with
  /// `E( mu, alpha )` in the red channel, laid out like the Kulla-Conty LUT
  /// ( column = `mu`, row = fuzz roughness ) so `main.frag` samples both the
  /// same way.
  ///
  /// # Errors
  ///
  /// Returns [`gl::WebglError::FailedToAllocateResource`] if the texture cannot
  /// be created or uploaded.
  pub fn sheen_albedo_lut_upload
  (
    gl : &gl::WebGl2RenderingContext,
    uv_samples : usize,
    roughness_samples : usize,
    cos_samples : usize,
    phi_samples : usize
  ) -> Result< gl::web_sys::WebGlTexture, gl::WebglError >
  {
    let table = sheen_albedo_table( uv_samples, roughness_samples, cos_samples, phi_samples );

    let mut data = Vec::with_capacity( uv_samples * roughness_samples * 4 );
    for value in &table.e
    {
      data.extend_from_slice( &[ *value, 0.0, 0.0, 0.0 ] );
    }

    let texture = gl.create_texture().ok_or( gl::WebglError::FailedToAllocateResource( "sheen albedo LUT" ) )?;
    gl.bind_texture( gl::TEXTURE_2D, Some( &texture ) );

    let image_data : gl::js_sys::Object = gl::js_sys::Float32Array::from( data.as_slice() ).into();
    gl.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_array_buffer_view_and_src_offset
    (
      gl::TEXTURE_2D,
      0,
      gl::RGBA32F as i32,
      uv_samples as i32,
      roughness_samples as i32,
      0,
      gl::RGBA,
      gl::FLOAT,
      &image_data,
      0
    )
    .map_err( | _ | gl::WebglError::FailedToAllocateResource( "sheen albedo LUT upload" ) )?;

    gl::texture::d2::filter_linear( gl );
    gl::texture::d2::wrap_clamp( gl );
    gl.bind_texture( gl::TEXTURE_2D, None );

    Ok( texture )
  }
}

crate::mod_interface!
{
  own use
  {
    FUZZ_ALPHA_MIN,
    SheenAlbedoTable,
    sheen_directional_albedo,
    sheen_albedo_table,
    sheen_albedo_lut_upload
  };
}
