mod private
{
  use std::f32::consts::TAU;
  use minwebgl as gl;

  /// Kulla–Conty multi-scatter energy-compensation tables, computed by
  /// [`kulla_conty_tables`]. `e_uv` is the directional-albedo `E(μ, α)` laid
  /// out `roughness_samples` rows × `uv_samples` columns ( row = roughness,
  /// column = `μ = cosθ` ), and `e_avg` is the hemisphere-averaged albedo
  /// `E_avg(α) = 2 ∫₀¹ E(μ, α) μ dμ` per roughness.
  #[ derive( Clone, Debug, PartialEq ) ]
  pub struct KullaContyTables
  {
    /// `E(μ, α)`, `[ roughness * uv_samples + uv ]`.
    pub e_uv : Vec< f32 >,
    /// `E_avg(α)`, one per roughness sample.
    pub e_avg : Vec< f32 >,
    /// Number of `μ` ( cosθ ) samples per row.
    pub uv_samples : usize,
    /// Number of roughness samples ( rows ).
    pub roughness_samples : usize,
  }

  /// Radical inverse of `bits` in base 2 ( the Hammersley low-discrepancy
  /// sequence component ).
  fn radical_inverse_base2( mut bits : u32 ) -> f32
  {
    let mut x = 0.0_f32;
    let mut inv = 0.5_f32;
    while bits != 0
    {
      if bits & 1 == 1
      {
        x += inv;
      }
      bits >>= 1;
      inv *= 0.5;
    }
    x
  }

  /// A 2D Hammersley sample in `[0, 1)²`.
  fn hammersley( index : usize, count : usize ) -> [ f32; 2 ]
  {
    [ index as f32 / count as f32, radical_inverse_base2( index as u32 ) ]
  }

  /// Samples a microfacet normal `H` from the isotropic GGX VNDF with slope
  /// roughness `alpha` ( the `roughness²` parameter used by the BRDF ), in the
  /// tangent frame where the geometric normal is `+Z`.
  fn ggx_vndf_sample( xi : [ f32; 2 ], alpha : f32 ) -> [ f32; 3 ]
  {
    let phi = TAU * xi[ 1 ];
    let cos_theta = ( ( 1.0 - xi[ 0 ] ) / ( 1.0 + ( alpha * alpha - 1.0 ) * xi[ 0 ] ) ).sqrt();
    let sin_theta = ( 1.0 - cos_theta * cos_theta ).max( 0.0 ).sqrt();
    [ sin_theta * phi.cos(), sin_theta * phi.sin(), cos_theta ]
  }

  /// Height-correlated Smith visibility `V = G / ( 4 NoV NoL )` — exactly the
  /// term the shader's `V_GGX_SmithCorrelated` uses, so the table energy is
  /// consistent with the direct-light BRDF.
  fn v_ggx( alpha : f32, no_v : f32, no_l : f32 ) -> f32
  {
    let a2 = alpha * alpha;
    let gv = no_l * ( a2 + ( 1.0 - a2 ) * no_v * no_v ).sqrt();
    let gl = no_v * ( a2 + ( 1.0 - a2 ) * no_l * no_l ).sqrt();
    0.5 / ( gv + gl ).max( 1e-6 )
  }

  /// Directional albedo `E(μ, α)` of the white GGX BRDF by importance sampling
  /// the microfacet distribution: each sample contributes
  /// `V · NoL / pdf`, with `pdf = D · NoH / ( 4 VoH )`.
  fn directional_albedo( mu : f32, roughness : f32, sample_count : usize ) -> f32
  {
    let alpha = roughness * roughness;
    // View direction in the tangent frame ( N = +Z ), at cosθ = μ.
    let v = [ ( 1.0 - mu * mu ).max( 0.0 ).sqrt(), 0.0, mu ];

    let mut sum = 0.0_f32;
    for i in 0 .. sample_count
    {
      let xi = hammersley( i, sample_count );
      let h = ggx_vndf_sample( xi, alpha );
      let voh = ( v[ 0 ] * h[ 0 ] + v[ 1 ] * h[ 1 ] + v[ 2 ] * h[ 2 ] ).max( 1e-5 );

      // L = reflect( -V, H ) = 2 ( V·H ) H - V.
      let l = [ 2.0 * voh * h[ 0 ] - v[ 0 ], 2.0 * voh * h[ 1 ] - v[ 1 ], 2.0 * voh * h[ 2 ] - v[ 2 ] ];
      let no_l = l[ 2 ].max( 0.0 );
      if no_l <= 0.0
      {
        continue;
      }
      let no_h = h[ 2 ].max( 1e-5 );

      // f · NoL / pdf = V · NoL · ( 4 VoH / NoH ).
      sum += v_ggx( alpha, mu, no_l ) * no_l * ( 4.0 * voh / no_h );
    }
    sum / sample_count as f32
  }

  /// Computes the Kulla–Conty `E(μ, α)` grid and `E_avg(α)` table for the GGX
  /// BRDF used by this crate. Pure CPU work — no GPU — so it is natively
  /// unit-testable and can be precomputed once at startup and uploaded as a
  /// 2D texture ( `E` in one channel, row-constant `E_avg` in another ).
  ///
  /// `uv_samples`/`roughness_samples` must be `>= 2` ( `E_avg` integration uses
  /// the trapezoid rule over `μ` ).
  ///
  /// # Panics
  ///
  /// Panics if `uv_samples` or `roughness_samples` is `< 2`.
  #[ must_use ]
  pub fn kulla_conty_tables
  (
    uv_samples : usize,
    roughness_samples : usize,
    sample_count : usize
  ) -> KullaContyTables
  {
    assert!( uv_samples >= 2, "uv_samples must be >= 2" );
    assert!( roughness_samples >= 2, "roughness_samples must be >= 2" );

    let mut e_uv = vec![ 0.0_f32; uv_samples * roughness_samples ];

    for r in 0 .. roughness_samples
    {
      let roughness = r as f32 / ( roughness_samples - 1 ) as f32;
      for u in 0 .. uv_samples
      {
        let mu = u as f32 / ( uv_samples - 1 ) as f32;
        e_uv[ r * uv_samples + u ] = directional_albedo( mu, roughness, sample_count );
      }
    }

    // E_avg(α) = 2 ∫₀¹ E(μ, α) μ dμ ( trapezoid over μ ).
    let mut e_avg = vec![ 0.0_f32; roughness_samples ];
    for r in 0 .. roughness_samples
    {
      let mut integral = 0.0_f32;
      for u in 0 .. uv_samples
      {
        let mu = u as f32 / ( uv_samples - 1 ) as f32;
        let weight = if u == 0 || u == uv_samples - 1 { 1.0 } else { 2.0 };
        integral += e_uv[ r * uv_samples + u ] * mu * weight;
      }
      integral *= 0.5 / ( uv_samples - 1 ) as f32;
      e_avg[ r ] = 2.0 * integral;
    }

    KullaContyTables { e_uv, e_avg, uv_samples, roughness_samples }
  }

  /// Uploads the Kulla–Conty tables as an `RGBA32F` 2D texture: `E(μ, α)` in
  /// the red channel and the row-constant `E_avg(α)` in the green channel, for
  /// the shader's multi-scatter energy compensation.
  ///
  /// # Errors
  ///
  /// Returns [`gl::WebglError::FailedToAllocateResource`] if the texture cannot
  /// be created or uploaded.
  pub fn kulla_conty_lut_upload
  (
    gl : &gl::WebGl2RenderingContext,
    uv_samples : usize,
    roughness_samples : usize,
    sample_count : usize
  ) -> Result< gl::web_sys::WebGlTexture, gl::WebglError >
  {
    let tables = kulla_conty_tables( uv_samples, roughness_samples, sample_count );

    let mut data = Vec::with_capacity( uv_samples * roughness_samples * 4 );
    for r in 0 .. roughness_samples
    {
      let e_avg = tables.e_avg[ r ];
      for u in 0 .. uv_samples
      {
        let e = tables.e_uv[ r * uv_samples + u ];
        data.extend_from_slice( &[ e, e_avg, 0.0, 0.0 ] );
      }
    }

    let texture = gl.create_texture().ok_or( gl::WebglError::FailedToAllocateResource( "Kulla-Conty LUT" ) )?;
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
    .map_err( | _ | gl::WebglError::FailedToAllocateResource( "Kulla-Conty LUT upload" ) )?;

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
    KullaContyTables,
    kulla_conty_tables,
    kulla_conty_lut_upload
  };
}
