//! Structural and targeted pixel-correctness tests for the PMREM IBL generator
//! ( `renderer::webgl::loaders::pmrem` ).
//!
//! Most tests here exercise the real GPU pipeline in a headless WebGL2 context without
//! verifying pixel-level output ( general visual correctness on realistic environment maps
//! still relies on visual inspection of the `gltf_viewer` example ): they catch signature
//! regressions, panics, incomplete-framebuffer failures and missing output textures without a
//! human in the loop. `generate_uniform_environment_prefilters_to_uniform_color` is the one
//! exception -- it reads back actual texels and checks a specific, analytically-provable
//! invariant ( a weighted convolution of a spatially constant field must return that same
//! constant ), narrower than general pixel correctness but a real, provable property rather
//! than a crash check.

#[ cfg( target_arch = "wasm32" ) ]
#[ cfg( test ) ]
mod tests
{
  use wasm_bindgen_test::wasm_bindgen_test;

  // Browser, not Node: every test here needs a real WebGL2 context.
  wasm_bindgen_test::wasm_bindgen_test_configure!( run_in_browser );
  use minwebgl as gl;
  use gl::GL;
  use renderer::webgl::loaders::pmrem;

  /// Creates a headless WebGL2 context with the float-render-target extension PMREM needs.
  fn gl_init() -> GL
  {
    gl::browser::setup( gl::browser::Config::default() );
    let options = gl::context::ContextOptions::default().antialias( false );
    let canvas = gl::canvas::make().unwrap();
    let gl = gl::context::from_canvas_with( &canvas, options ).unwrap();

    // PMREM renders into RGBA16F attachments, which are only color-renderable with this
    // extension; without it the off-screen FBO is incomplete and `generate` returns an error.
    gl.get_extension( "EXT_color_buffer_float" )
      .expect( "get_extension call should not throw" )
      .expect( "EXT_color_buffer_float must be available in the test environment" );

    gl
  }

  /// Minimal equirectangular source. Contents are irrelevant to a structural test, so the
  /// 4x2 RGBA8 storage is left uninitialized; `RGBA8` + `LINEAR` is filterable everywhere and
  /// avoids depending on `OES_texture_float_linear`.
  fn equirect_make( gl : &GL ) -> gl::web_sys::WebGlTexture
  {
    let texture = gl.create_texture().unwrap();
    gl.bind_texture( gl::TEXTURE_2D, Some( &texture ) );
    gl.tex_storage_2d( gl::TEXTURE_2D, 1, gl::RGBA8, 4, 2 );
    gl.tex_parameteri( gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32 );
    gl.tex_parameteri( gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32 );
    gl.tex_parameteri( gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as i32 );
    gl.tex_parameteri( gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32 );
    texture
  }

  /// Same storage and filter/wrap setup as `equirect_make`, but every texel is explicitly
  /// filled with `color` instead of being left at immutable-storage's zero-initialized default
  /// -- every sampled direction then returns the exact same value, which is the fixture the
  /// solid-color PMREM invariant test below needs.
  fn equirect_make_solid( gl : &GL, color : [ u8; 4 ] ) -> gl::web_sys::WebGlTexture
  {
    let texture = equirect_make( gl );
    // equirect_make leaves the texture bound to TEXTURE_2D; 4x2 RGBA8 -> 8 texels * 4 bytes.
    let pixels : Vec< u8 > = color.repeat( 4 * 2 );
    gl.tex_sub_image_2d_with_i32_and_i32_and_u32_and_type_and_opt_u8_array
    (
      gl::TEXTURE_2D,
      0,
      0,
      0,
      4,
      2,
      gl::RGBA,
      gl::UNSIGNED_BYTE,
      Some( &pixels ),
    ).expect( "tex_sub_image_2d should upload a solid color into the equirect source" );
    texture
  }

  /// Reads back one cube face's mip level as RGBA f32 texels through an off-screen FBO -- the
  /// same `framebuffer_texture_2d` cube-face attach `pmrem.rs`'s own ( private ) `render_to_cube_face`
  /// uses to *write* a face, mirrored here with public GL calls to *read* one back. `size` must
  /// be the exact width/height of that mip level. `RGBA`/`FLOAT` is the combo
  /// `EXT_color_buffer_float` guarantees for reading back a floating-point-format framebuffer
  /// ( the same extension `gl_init` already requires for PMREM's RGBA16F attachments to be
  /// renderable in the first place ).
  fn cubemap_face_read_f32
  (
    gl : &GL,
    texture : &gl::web_sys::WebGlTexture,
    face : u32,
    mip_level : i32,
    size : u32
  ) -> Vec< f32 >
  {
    let fbo = gl.create_framebuffer().expect( "create_framebuffer should succeed" );
    gl.bind_framebuffer( gl::FRAMEBUFFER, Some( &fbo ) );
    gl.framebuffer_texture_2d
    (
      gl::FRAMEBUFFER,
      gl::COLOR_ATTACHMENT0,
      gl::TEXTURE_CUBE_MAP_POSITIVE_X + face,
      Some( texture ),
      mip_level,
    );
    assert_eq!
    (
      gl.check_framebuffer_status( gl::FRAMEBUFFER ), gl::FRAMEBUFFER_COMPLETE,
      "readback FBO must be complete for face {face} mip {mip_level}"
    );

    let out = gl::js_sys::Float32Array::new_with_length( size * size * 4 );
    gl.read_pixels_with_array_buffer_view_and_dst_offset
    (
      0,
      0,
      size as i32,
      size as i32,
      gl::RGBA,
      gl::FLOAT,
      &out,
      0,
    ).expect( "read_pixels should succeed on an RGBA16F cubemap face under EXT_color_buffer_float" );

    gl.bind_framebuffer( gl::FRAMEBUFFER, None );
    gl.delete_framebuffer( Some( &fbo ) );

    out.to_vec()
  }

  /// `generate` succeeds on a valid context and returns all three IBL textures.
  #[ wasm_bindgen_test( async ) ]
  async fn generate_returns_all_textures()
  {
    let gl = gl_init();
    let equirect = equirect_make( &gl );

    let ibl = pmrem::generate( &gl, &equirect, 64 ).expect( "PMREM generate should succeed" );

    assert!( ibl.diffuse_texture.is_some(), "diffuse irradiance texture missing" );
    assert!( ibl.specular_1_texture.is_some(), "prefiltered specular texture missing" );
    assert!( ibl.specular_2_texture.is_some(), "BRDF LUT texture missing" );
  }

  /// A single-mip cubemap ( `resolution = 1`, so `num_mips = 1` ) must not divide by zero in
  /// the prefilter roughness computation and must still produce a full IBL set.
  #[ wasm_bindgen_test( async ) ]
  async fn generate_single_mip_resolution()
  {
    let gl = gl_init();
    let equirect = equirect_make( &gl );

    let ibl = pmrem::generate( &gl, &equirect, 1 ).expect( "PMREM generate should succeed for a 1x1 cubemap" );

    assert!( ibl.diffuse_texture.is_some() );
    assert!( ibl.specular_1_texture.is_some() );
    assert!( ibl.specular_2_texture.is_some() );
  }

  /// Non-power-of-two resolution is valid in WebGL2 and must produce a full IBL set.
  #[ wasm_bindgen_test( async ) ]
  async fn generate_non_power_of_two_resolution()
  {
    let gl = gl_init();
    let equirect = equirect_make( &gl );

    let ibl = pmrem::generate( &gl, &equirect, 96 ).expect( "PMREM generate should succeed for an NPOT cubemap" );

    assert!( ibl.diffuse_texture.is_some() );
    assert!( ibl.specular_1_texture.is_some() );
    assert!( ibl.specular_2_texture.is_some() );
  }

  /// PMREM's specular prefiltering ( `prefilter_specular.frag` ) is a GGX-weighted convolution
  /// of the source cubemap: `prefilteredColor / totalWeight`, where every sample's contribution
  /// is `envMap` sampled along some direction, weighted by `NdotL`. If `envMap` is a spatially
  /// constant color `C` at every direction and every LOD -- true here because
  /// `equirect_to_cube.frag` is a direct, unweighted `texture()` sample of a uniform source, and
  /// mipmap generation of a uniform field stays uniform -- then every weighted sum collapses to
  /// `C * totalWeight / totalWeight == C`, regardless of which roughness/mip or direction is
  /// sampled. A blur of a constant field is that same constant field; this checks that provable
  /// property against real rendered and read-back pixels rather than only "did it crash".
  #[ wasm_bindgen_test( async ) ]
  async fn generate_uniform_environment_prefilters_to_uniform_color()
  {
    let gl = gl_init();

    // Mid-range, non-power-of-two-friendly color: far from the 0/255 saturation edges ( where
    // clamping could silently mask a real bug ) and well above RGBA8's ~0.4% quantization step.
    let color : [ u8; 4 ] = [ 180, 90, 45, 255 ];
    let expected : [ f32; 3 ] =
    [
      f32::from( color[ 0 ] ) / 255.0,
      f32::from( color[ 1 ] ) / 255.0,
      f32::from( color[ 2 ] ) / 255.0,
    ];
    let equirect = equirect_make_solid( &gl, color );

    let resolution = 32u32;
    let ibl = pmrem::generate( &gl, &equirect, resolution ).expect( "PMREM generate should succeed" );
    // Fix(BUG-440): `.clone()` before `.expect()` -- `IBL` now implements `Drop` ( to free its
    // own textures ), so Rust forbids moving a field out of it by value; `ibl` is still used
    // below ( `ibl.num_mips` ), so the whole struct can't be consumed here either.
    // Root cause: adding `impl Drop for IBL` retroactively made every pre-existing move-out
    // field access ( `ibl.field.expect(..)`, `.unwrap()`, `.take()` ) a compile error --
    // Rust's partial-move-out-of-a-`Drop`-type restriction is unconditional.
    // Pitfall: adding `Drop` to a previously-`Drop`-free struct is not purely additive -- audit
    // every existing call site that moves a field out of the type ( `grep` for `.expect(`/
    // `.unwrap(`/`.take(` on each field name across the workspace ) before landing the change.
    let specular = ibl.specular_1_texture.clone().expect( "prefiltered specular texture missing" );

    // A few percent of the [0,1] dynamic range: covers RGBA8 source quantization ( ~1/255 ),
    // RGBA16F storage rounding at each of the two writes ( equirect->cube, then prefilter ), and
    // 1024-sample summation error in `prefilter_specular.frag` -- all far smaller in practice
    // ( expected well under 1% ), but this leaves headroom rather than chasing float-exactness.
    let tolerance = 0.03f32;

    // Mip 0 ( full resolution, roughness 0, sharpest GGX lobe ) and the last mip ( 1x1,
    // roughness 1, widest GGX lobe ) are opposite ends of the prefiltered chain.
    let last_mip = ibl.num_mips as i32 - 1;
    let last_mip_size = resolution >> ( ibl.num_mips - 1 );

    for &( mip, mip_size ) in &[ ( 0i32, resolution ), ( last_mip, last_mip_size ) ]
    {
      for face in 0..6u32
      {
        let pixels = cubemap_face_read_f32( &gl, &specular, face, mip, mip_size );

        // A couple of sample texels per face is enough -- the invariant is per-texel exact
        // ( see the analysis above ), so more samples add cost without adding evidence.
        for &( x, y ) in &[ ( 0u32, 0u32 ), ( mip_size - 1, mip_size - 1 ) ]
        {
          let idx = ( ( y * mip_size + x ) * 4 ) as usize;
          let rgb = [ pixels[ idx ], pixels[ idx + 1 ], pixels[ idx + 2 ] ];
          for channel in 0..3
          {
            assert!
            (
              ( rgb[ channel ] - expected[ channel ] ).abs() < tolerance,
              "face {face} mip {mip} ( size {mip_size} ) texel ({x},{y}) channel {channel}: \
               got {}, expected {} within {tolerance}", rgb[ channel ], expected[ channel ]
            );
          }
        }
      }
    }
  }

  /// Reads one texel of a `TEXTURE_2D` as RGBA f32 through an off-screen FBO.
  fn texture_texel_read_f32( gl : &GL, texture : &gl::web_sys::WebGlTexture, x : i32, y : i32 ) -> [ f32; 4 ]
  {
    let fbo = gl.create_framebuffer().expect( "create_framebuffer should succeed" );
    gl.bind_framebuffer( gl::FRAMEBUFFER, Some( &fbo ) );
    gl.framebuffer_texture_2d( gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT0, gl::TEXTURE_2D, Some( texture ), 0 );
    assert_eq!( gl.check_framebuffer_status( gl::FRAMEBUFFER ), gl::FRAMEBUFFER_COMPLETE, "readback FBO must be complete" );
    let out = gl::js_sys::Float32Array::new_with_length( 4 );
    gl.read_pixels_with_array_buffer_view_and_dst_offset( x, y, 1, 1, gl::RGBA, gl::FLOAT, &out, 0 )
    .expect( "read_pixels should succeed on an RGBA16F texture under EXT_color_buffer_float" );
    gl.bind_framebuffer( gl::FRAMEBUFFER, None );
    gl.delete_framebuffer( Some( &fbo ) );
    let texel = out.to_vec();
    [ texel[ 0 ], texel[ 1 ], texel[ 2 ], texel[ 3 ] ]
  }

  /// `brdf_integration.frag`'s split-sum integral, evaluated on the CPU in f64 with the same
  /// 1024-point Hammersley set, sample frame and clamps. `collapsed` replaces the radical
  /// inverse with 0 — what a 16-bit `mediump int` does to it — to show what a broken
  /// sequence would put in the LUT.
  fn brdf_lut_reference( n_dot_v : f64, roughness : f64, collapsed : bool ) -> [ f64; 2 ]
  {
    use std::f64::consts::PI;
    const SAMPLE_COUNT : u32 = 1024;
    let g1 = | n_dot_x : f64 | { let k = roughness * roughness / 2.0; n_dot_x / ( n_dot_x * ( 1.0 - k ) + k ) };
    let v = [ ( 1.0 - n_dot_v * n_dot_v ).sqrt(), 0.0, n_dot_v ];
    let a = roughness * roughness;
    let ( mut scale, mut bias ) = ( 0.0, 0.0 );
    for i in 0..SAMPLE_COUNT
    {
      let xi_x = f64::from( i ) / f64::from( SAMPLE_COUNT );
      let xi_y = if collapsed { 0.0 } else { f64::from( i.reverse_bits() ) * 2.328_306_436_538_696_3e-10 };
      let phi = 2.0 * PI * xi_x;
      let cos_theta = ( ( 1.0 - xi_y ) / ( 1.0 + ( a * a - 1.0 ) * xi_y ) ).sqrt();
      let sin_theta = ( 1.0 - cos_theta * cos_theta ).sqrt();
      // The shader's tangent frame for N = +Z: tangent ( 0, -1, 0 ), bitangent ( 1, 0, 0 ).
      let h = [ phi.sin() * sin_theta, -phi.cos() * sin_theta, cos_theta ];
      let v_dot_h_raw = v[ 0 ] * h[ 0 ] + v[ 1 ] * h[ 1 ] + v[ 2 ] * h[ 2 ];
      let l_z = 2.0 * v_dot_h_raw * h[ 2 ] - v[ 2 ];
      let n_dot_l = l_z.max( 0.0 );
      let n_dot_h = h[ 2 ].max( 0.0 );
      let v_dot_h = v_dot_h_raw.clamp( 0.0, 1.0 );
      if n_dot_l > 0.0
      {
        let g = g1( n_dot_v.max( 0.0 ) ) * g1( n_dot_l );
        let g_vis = g * v_dot_h / ( n_dot_h.max( 0.001 ) * n_dot_v.max( 0.001 ) );
        let fc = ( 1.0 - v_dot_h ).powi( 5 );
        scale += ( 1.0 - fc ) * g_vis;
        bias += fc * g_vis;
      }
    }
    [ scale / f64::from( SAMPLE_COUNT ), bias / f64::from( SAMPLE_COUNT ) ]
  }

  /// The BRDF LUT must hold the split-sum integral itself, not just exist. Texels are compared
  /// with the CPU evaluation of the same integral at the texel centres. Where a Hammersley
  /// sequence collapsed onto H = N ( the 16-bit `mediump int` fault `precision highp int`
  /// fixes ) would give a different value, the check could not pass with a broken sequence.
  #[ wasm_bindgen_test( async ) ]
  async fn brdf_lut_matches_the_split_sum_integral()
  {
    const LUT_SIZE : f64 = 512.0;
    let gl = gl_init();
    let equirect = equirect_make( &gl );
    let ibl = pmrem::generate( &gl, &equirect, 8 ).expect( "PMREM generate should succeed" );
    let lut = ibl.specular_2_texture.clone().expect( "BRDF LUT texture missing" );

    // Texel ( x, y ): N.V along x, roughness along y, both sampled at the texel centre.
    for &( x, y ) in &[ ( 51_i32, 460_i32 ), ( 256, 256 ), ( 460, 153 ), ( 153, 358 ) ]
    {
      let n_dot_v = ( f64::from( x ) + 0.5 ) / LUT_SIZE;
      let roughness = ( f64::from( y ) + 0.5 ) / LUT_SIZE;
      let expected = brdf_lut_reference( n_dot_v, roughness, false );
      let texel = texture_texel_read_f32( &gl, &lut, x, y );
      for ( channel, name ) in [ ( 0, "scale" ), ( 1, "bias" ) ]
      {
        assert!
        (
          ( f64::from( texel[ channel ] ) - expected[ channel ] ).abs() < 0.01,
          "N.V {n_dot_v:.3}, roughness {roughness:.3}: {name} is {}, the integral gives {:.4}",
          texel[ channel ], expected[ channel ]
        );
      }
    }

    // The grazing, rough texel tells a working sequence from a collapsed one.
    let ( n_dot_v, roughness ) = ( 51.5 / LUT_SIZE, 460.5 / LUT_SIZE );
    let working = brdf_lut_reference( n_dot_v, roughness, false );
    let collapsed = brdf_lut_reference( n_dot_v, roughness, true );
    assert!
    (
      ( working[ 0 ] + working[ 1 ] ) - ( collapsed[ 0 ] + collapsed[ 1 ] ) > 0.1,
      "precondition: a collapsed sequence changes the grazing texel, working {working:?}, collapsed {collapsed:?}"
    );
  }

  /// Prefiltering must actually blur. With a uniform environment ( the test above ) every
  /// sample returns the same colour, so a collapsed Hammersley sequence — every sample on the
  /// texel's own direction — would pass too. Here the environment is bright at one pole and
  /// dark at the other: at roughness 0 the two pole faces keep that contrast, at roughness 1
  /// the wide GGX lobe averages in the latitudes between, so the contrast must drop. A
  /// collapsed sequence samples only the pole itself and keeps the full contrast.
  #[ wasm_bindgen_test( async ) ]
  async fn prefilter_blurs_a_non_uniform_environment()
  {
    let gl = gl_init();
    let equirect = equirect_make( &gl );
    // equirect_make leaves the texture bound; row 0 bright, row 1 dark, 4 texels each.
    let pixels : Vec< u8 > = [ [ 230_u8, 230, 230, 255 ].repeat( 4 ), [ 10_u8, 10, 10, 255 ].repeat( 4 ) ].concat();
    gl.tex_sub_image_2d_with_i32_and_i32_and_u32_and_type_and_opt_u8_array
    (
      gl::TEXTURE_2D, 0, 0, 0, 4, 2, gl::RGBA, gl::UNSIGNED_BYTE, Some( &pixels ),
    ).expect( "tex_sub_image_2d should upload the two-band equirect source" );

    let resolution = 32_u32;
    let ibl = pmrem::generate( &gl, &equirect, resolution ).expect( "PMREM generate should succeed" );
    let specular = ibl.specular_1_texture.clone().expect( "prefiltered specular texture missing" );
    let last_mip = ibl.num_mips as i32 - 1;
    let last_mip_size = resolution >> ( ibl.num_mips - 1 );

    // Mean red of a face's mip; the +Y and -Y faces ( 2 and 3 ) look at the two poles.
    let face_mean = | face : u32, mip : i32, size : u32 |
    {
      let pixels = cubemap_face_read_f32( &gl, &specular, face, mip, size );
      let red : f32 = pixels.iter().step_by( 4 ).sum();
      red / ( size * size ) as f32
    };
    let contrast = | mip : i32, size : u32 | ( face_mean( 2, mip, size ) - face_mean( 3, mip, size ) ).abs();

    let sharp = contrast( 0, resolution );
    let blurred = contrast( last_mip, last_mip_size );
    assert!( sharp > 0.4, "precondition: the pole faces differ at roughness 0, contrast {sharp}" );
    assert!
    (
      blurred < 0.85 * sharp,
      "roughness 1 must blur the poles together: contrast {blurred} at the last mip, {sharp} at mip 0"
    );
    assert!( blurred > 0.05, "roughness 1 must still see which pole is bright, contrast {blurred}" );
  }
}
