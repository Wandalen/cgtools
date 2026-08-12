// Full scene description for the sun/grid HUD diagram, INCLUDING the
// fragment shader that renders it — every color, opacity, radius, dynamic,
// effect, and rendering formula that isn't already keyboard-interactive
// (seed, node count, grid density stay live; see 'Params' in src/main.rs).
// Evaluated by src/scene.rs at compile time via include_str!, run through
// scene_script::engine_build(), and extracted into SceneConfig via rhai's
// serde bridge ( rhai::serde::from_dynamic ) — a rebuild is required to pick
// up edits, see readme.md.
//
// The scene is composed from six primitive families, each a list so the
// diagram can carry more than one instance of a kind: nebula_bands (fog
// layers drifting at their own speed), star_layers (background twinkle at
// two densities), orbit_rings (concentric rails), and nodes (authored
// planets/moons with their own orbit radius, angular speed, and phase —
// distinct from the keyboard-driven procedural nodes in the shader below,
// which keep working exactly as before). sun_corona and sun_disc stay
// singular — there is exactly one star — but both now carry dynamics
// (flicker_*, pulsate_*) instead of being purely static. effects is the one
// cross-cutting block: vignette, a global glow multiplier, and a faint
// scanline texture, all analytically single-pass ( no offscreen texture
// exists to sample for a true multi-tap effect like chromatic aberration ).
//
// `shader` (below, next to the data it renders) owns this scene's
// fragment-only WGSL body as a raw string. Its Uniforms layout mirrors
// every field above 1:1, and fs_main is what actually animates them against
// the live time uniform every frame. It stays a plain string as far as Rhai
// is concerned — no function or operator call ever touches it — so the
// document stays pure data per docs/pattern/004_script_as_data.md. Generic,
// reusable pieces are deliberately NOT here: the fullscreen-triangle vertex
// stage and the hash21/value_noise/fbm3 noise stack live in the shared
// shader_chunks crate instead, and get prepended ahead of this string by
// shader_source::assemble() — the same four chunks the offscreen
// minwgpu/sun_grid_lines_chunked demo composes.

let background = #{ top : [ 0.0196, 0.0549, 0.0941 ], bottom : [ 0.0549, 0.1490, 0.2392 ] };

// Three fog layers at different heights, hues, and drift speeds/directions —
// replaces the single static teal band.
let nebula_bands =
[
  #{ color : [ 0.0706, 0.2000, 0.2902 ], opacity : 0.45, center : 0.5,  thickness : 0.30, noise_scale : 1.0, drift_speed :  0.020 },
  #{ color : [ 0.2510, 0.1216, 0.3608 ], opacity : 0.28, center : 0.22, thickness : 0.16, noise_scale : 1.6, drift_speed :  0.035 },
  #{ color : [ 0.3608, 0.1490, 0.1804 ], opacity : 0.22, center : 0.80, thickness : 0.14, noise_scale : 1.3, drift_speed : -0.025 },
];

// Two star fields: a dense dim layer (as before) plus a sparser, brighter,
// slower-twinkling foreground layer.
let star_layers =
[
  #{ color : [ 0.6275, 0.8980, 1.0 ],    intensity : 0.6, density : 9.0, size : 0.06, twinkle_speed : 1.5 },
  #{ color : [ 0.8353, 0.9569, 1.0 ],    intensity : 0.9, density : 4.0, size : 0.03, twinkle_speed : 0.8 },
];

let grid = #{ color : [ 0.3137, 0.5490, 0.7451 ], opacity : 0.18, line_width : 1.0, glow : 0.15 };

// Radii were shader constants (0.08/0.15/0.25) before; now data, plus a
// slow brightness flicker.
let sun_corona =
#{
  inner : [ 1.0, 0.8941, 0.4392 ], mid : [ 1.0, 0.6824, 0.1020 ], outer : [ 1.0, 0.2314, 0.0 ],
  inner_radius : 0.08, mid_radius : 0.15, outer_radius : 0.25,
  flicker_amplitude : 0.08, flicker_speed : 2.4,
};

// Adds a gentle breathing pulsation on top of the existing granulated disc.
let sun_disc =
#{
  dark : [ 1.0, 0.4157, 0.0 ], mid : [ 1.0, 0.8941, 0.4392 ], bright : [ 1.0, 1.0, 1.0 ],
  base_radius : 0.075, pulsate_amplitude : 0.015, pulsate_speed : 1.1, granulation_scale : 1.0,
};

// Three concentric rails — replaces the single orbit ring. Radii and the
// innermost color/glow match the original ring exactly.
let orbit_rings =
[
  #{ color : [ 0.3922, 0.8235, 1.0 ], radius : 0.425, glow : 0.35, stroke_width : 1.00, pulse_speed : 0.60 },
  #{ color : [ 0.6000, 0.7000, 1.0 ], radius : 0.600, glow : 0.28, stroke_width : 0.85, pulse_speed : 0.45 },
  #{ color : [ 0.8500, 0.6000, 1.0 ], radius : 0.780, glow : 0.20, stroke_width : 0.70, pulse_speed : 0.30 },
];

// Six authored planets/moons, two per ring, each with its own orbit radius,
// angular speed (sign = direction), starting phase, size, and color — a
// small designed solar system layered on top of the ring-only diagram.
let nodes =
[
  #{ color : [ 1.00, 0.75, 0.45 ], size : 0.014, radius : 0.425, speed :  0.50, phase : 0.0 },
  #{ color : [ 0.55, 0.85, 1.00 ], size : 0.011, radius : 0.425, speed : -0.35, phase : 3.4 },
  #{ color : [ 0.60, 0.90, 0.60 ], size : 0.016, radius : 0.600, speed :  0.32, phase : 1.1 },
  #{ color : [ 0.90, 0.55, 0.90 ], size : 0.010, radius : 0.600, speed : -0.22, phase : 4.6 },
  #{ color : [ 1.00, 0.95, 0.60 ], size : 0.019, radius : 0.780, speed :  0.20, phase : 2.0 },
  #{ color : [ 0.75, 0.80, 1.00 ], size : 0.012, radius : 0.780, speed : -0.15, phase : 5.2 },
];

let effects = #{ vignette_strength : 0.35, vignette_radius : 0.55, glow_intensity : 1.1, scanline_intensity : 0.05 };

// This scene's fragment-only WGSL body — everything above this point is
// what its `uniforms` binding reads. Not valid standalone WGSL on its own:
// `VertexOutput`, `hash21`, and `fbm3` come from the shared `shader_chunks`
// crate, prepended ahead of this string by `shader_source::assemble()`
// before the combined source reaches the shader module. A raw string, not
// a function/operator call, so the document stays pure data.
let shader =
`
// Procedural sci-fi HUD diagram: animated star, three orbit rings, six
// authored planets/moons, drifting multi-band nebula, twinkling multi-layer
// star field, and a Cartesian grid, rendered by a fullscreen fragment
// shader. WGSL port of the WebGL2 minwebgl_sun_grid_lines example's
// scene.frag; glow uses the same analytic radial-falloff / exp terms as
// that example's original single-pass version ( no multi-pass Gaussian
// bloom infrastructure exists for WebGPU in this workspace ).
//
// Every color/opacity/radius/dynamic below reads from uniforms, sourced
// from the scene data above in this same file ( see ../src/scene.rs for the
// loader ) — nothing here is a hardcoded visual constant except generation
// internals ( noise/hash magic numbers, AA epsilons, node jitter ) that
// aren't meant to be author-facing content. List-shaped scene content
// ( nebula bands, star layers, orbit rings, nodes ) uses fixed-size arrays,
// matching how scene.rs requires scene.rhai to declare exactly
// NEBULA_BAND_COUNT/STAR_LAYER_COUNT/ORBIT_RING_COUNT/NODE_COUNT entries —
// a WGSL uniform binding's arrays must be a compile-time fixed size, so
// there is no runtime element count to loop against; the counts below are
// this shader's own copy of those same numbers, kept in sync by
// tests/shader_source_test.rs's wgsl_scene_constants_match_scene_rs.
//
// Fragment-only: VertexOutput/vs_main come from the
// shader_chunks::FULLSCREEN_TRIANGLE chunk, and hash21/value_noise/fbm3
// come from the HASH21/VALUE_NOISE/FBM3 chunks — all prepended ahead of
// this string by shader_source::assemble() before the combined source
// reaches the shader module. This string alone is not valid standalone
// WGSL.

struct Uniforms
{
  time : f32,
  seed : f32,
  node_count : i32,
  grid_density : f32,

  bg_top : vec4f,
  bg_bottom : vec4f,

  // .xyz = color, .w = opacity
  nebula_colors : array< vec4f, 3 >,
  // .x = vertical center, .y = thickness, .z = noise scale, .w = drift speed
  nebula_params : array< vec4f, 3 >,

  // .xyz = color, .w = intensity
  star_colors : array< vec4f, 2 >,
  // .x = density (cells across), .y = point size, .z = twinkle speed, .w = unused
  star_params : array< vec4f, 2 >,

  grid_color : vec4f,
  // .x = opacity, .y = line width, .z = glow, .w = unused
  grid_params : vec4f,

  corona_inner : vec4f,
  corona_mid : vec4f,
  corona_outer : vec4f,
  // .x = inner radius, .y = mid radius, .z = outer radius, .w = unused
  corona_radii : vec4f,
  // .x = flicker amplitude, .y = flicker speed, .zw = unused
  corona_flicker : vec4f,

  disc_dark : vec4f,
  disc_mid : vec4f,
  disc_bright : vec4f,
  // .x = base radius, .y = pulsate amplitude, .z = pulsate speed, .w = granulation scale
  disc_params : vec4f,

  // .xyz = color, .w = glow amount
  ring_colors : array< vec4f, 3 >,
  // .x = radius, .y = stroke width, .z = pulse speed, .w = unused
  ring_params : array< vec4f, 3 >,

  // .xyz = color, .w = size
  node_colors : array< vec4f, 6 >,
  // .x = orbit radius, .y = angular speed, .z = phase, .w = unused
  node_params : array< vec4f, 6 >,

  // .x = vignette strength, .y = vignette radius, .z = glow intensity, .w = scanline intensity
  effects : vec4f,
}

@group( 0 ) @binding( 0 ) var< uniform > uniforms : Uniforms;

const MAX_NODES : i32 = 8; // keyboard-controlled procedural nodes (unchanged demo)
const NEBULA_BAND_COUNT : u32 = 3u;
const STAR_LAYER_COUNT : u32 = 2u;
const ORBIT_RING_COUNT : u32 = 3u;
const NODE_COUNT : u32 = 6u;

@fragment
fn fs_main( in : VertexOutput ) -> @location( 0 ) vec4f
{
  let uv = in.uv; // y = 0 at canvas bottom, y = 1 at canvas top
  let center = vec2f( 0.5, 0.5 );
  let d = distance( uv, center );
  let effects = uniforms.effects;

  // 1. Background: vertical gradient, lighter toward vertical center.
  let navy = uniforms.bg_top.xyz;
  let slate = uniforms.bg_bottom.xyz;
  let vgrad = 1.0 - abs( uv.y - 0.5 ) * 2.0;
  var color = mix( navy, slate, vgrad );

  // 2. Nebula: up to NEBULA_BAND_COUNT drifting fog bands, each its own
  // height, thickness, hue, noise scale, and drift direction/speed.
  for ( var i : u32 = 0u; i < NEBULA_BAND_COUNT; i++ )
  {
    let band_color = uniforms.nebula_colors[ i ].xyz;
    let opacity = uniforms.nebula_colors[ i ].w;
    let band_center = uniforms.nebula_params[ i ].x;
    let thickness = uniforms.nebula_params[ i ].y;
    let noise_scale = uniforms.nebula_params[ i ].z;
    let drift_speed = uniforms.nebula_params[ i ].w;

    let half_thickness = thickness * 0.5;
    let falloff = thickness / 3.0;
    let band = smoothstep( band_center - half_thickness, band_center - half_thickness + falloff, uv.y )
      * ( 1.0 - smoothstep( band_center + half_thickness - falloff, band_center + half_thickness, uv.y ) );
    let fog_n = fbm3( vec2f( uv.x * 3.0 * noise_scale, uv.y * 8.0 * noise_scale ) + uniforms.seed * 0.37 + uniforms.time * drift_speed );
    color = mix( color, band_color, band * fog_n * opacity );
  }

  // 3. Background stars: up to STAR_LAYER_COUNT hashed point fields at
  // different densities, sizes, and twinkle speeds.
  for ( var i : u32 = 0u; i < STAR_LAYER_COUNT; i++ )
  {
    let layer_color = uniforms.star_colors[ i ].xyz;
    let intensity = uniforms.star_colors[ i ].w;
    let density = uniforms.star_params[ i ].x;
    let size = uniforms.star_params[ i ].y;
    let twinkle_speed = uniforms.star_params[ i ].z;
    let layer_seed = uniforms.seed + f32( i ) * 91.7;

    let cell = floor( uv * density );
    let cell_uv = fract( uv * density );
    let has_star = step( 0.86, hash21( cell + layer_seed ) );
    let star_pos = vec2f( hash21( cell + 0.17 + layer_seed ), hash21( cell + 4.31 + layer_seed ) );
    let star_d = distance( cell_uv, star_pos );
    let twinkle = 0.5 + 0.5 * sin( uniforms.time * ( twinkle_speed + hash21( cell + layer_seed ) * 2.0 ) + hash21( cell + layer_seed ) * 6.283 );
    let star = has_star * ( 1.0 - smoothstep( 0.0, size, star_d ) ) * ( 0.4 + 0.6 * twinkle );
    color += layer_color * star * intensity;
  }

  // 4. Grid overlay: density stays keyboard-live; line width and glow are
  // scene-authored, constant screen-space line width via fwidth.
  {
    let grid_color = uniforms.grid_color.xyz;
    let grid_params = uniforms.grid_params;
    let g = uv * uniforms.grid_density;
    let grid_d = abs( fract( g - 0.5 ) - 0.5 ) / fwidth( g );
    let min_grid_d = min( grid_d.x, grid_d.y );
    let line = 1.0 - min( min_grid_d / grid_params.y, 1.0 );
    let grid_glow = exp( -min_grid_d * 3.0 ) * grid_params.z * effects.z;
    color = mix( color, grid_color, line * grid_params.x );
    color += grid_color * grid_glow;
  }

  // 5. Central star corona: three-stop radial falloff, back to front, radii
  // and a slow brightness flicker are scene-authored.
  {
    let c0 = uniforms.corona_inner.xyz; // inner-most, warm yellow
    let c1 = uniforms.corona_mid.xyz; // mid corona, amber
    let c2 = uniforms.corona_outer.xyz; // outer corona, red-orange fading out
    let r0 = uniforms.corona_radii.x;
    let r1 = uniforms.corona_radii.y;
    let r2 = uniforms.corona_radii.z;
    let flicker = 1.0 + uniforms.corona_flicker.x * sin( uniforms.time * uniforms.corona_flicker.y );
    let a0 = ( 1.0 - smoothstep( 0.0, r0, d ) ) * flicker;
    let a1 = ( 1.0 - smoothstep( r0, r1, d ) ) * 0.8 * flicker;
    let a2 = ( 1.0 - smoothstep( r1, r2, d ) ) * 0.3 * flicker;
    let corona = c0 * a0 + c1 * a1 * ( 1.0 - a0 ) + c2 * a2 * ( 1.0 - a0 ) * ( 1.0 - a1 );
    let corona_a = clamp( a0 + a1 + a2, 0.0, 1.0 );
    color = mix( color, corona, corona_a );
  }

  // 6. Star disk: fbm surface granulation inside a noise-jagged rim, plus a
  // gentle authored breathing pulsation.
  {
    let pulsate = 1.0 + uniforms.disc_params.y * sin( uniforms.time * uniforms.disc_params.z );
    let base_radius = uniforms.disc_params.x * pulsate;
    let angle = atan2( uv.y - 0.5, uv.x - 0.5 );
    let rim_noise = fbm3( vec2f( cos( angle ), sin( angle ) ) * 4.0 ) - 0.4375;
    let radius = base_radius + rim_noise * 0.015;
    let disk = 1.0 - smoothstep( radius - 0.004, radius, d );

    let gran_n = fbm3( uv * 40.0 * uniforms.disc_params.w + 3.0 );
    let dark = uniforms.disc_dark.xyz;
    let mid = uniforms.disc_mid.xyz;
    let bright = uniforms.disc_bright.xyz;
    var surface = mix( dark, mid, smoothstep( 0.3, 0.6, gran_n ) );
    surface = mix( surface, bright, smoothstep( 0.75, 0.95, gran_n ) );

    color = mix( color, surface, disk );
  }

  // 7. Orbit rings: up to ORBIT_RING_COUNT concentric rails, each a soft
  // wide glow plus a crisp stroke core, with a slow authored brightness
  // pulse (phase-offset per ring so they don't pulse in lockstep).
  for ( var i : u32 = 0u; i < ORBIT_RING_COUNT; i++ )
  {
    let ring_color = uniforms.ring_colors[ i ].xyz;
    let ring_glow_amt = uniforms.ring_colors[ i ].w;
    let ring_r = uniforms.ring_params[ i ].x;
    let stroke_width = uniforms.ring_params[ i ].y;
    let pulse_speed = uniforms.ring_params[ i ].z;

    let pulse = 1.0 + 0.15 * sin( uniforms.time * pulse_speed + f32( i ) * 2.1 );
    let ring_d = abs( d - ring_r );
    let glow = exp( -ring_d * 220.0 ) * ring_glow_amt * pulse * effects.z;
    let core = 1.0 - smoothstep( 0.0, 0.0022 * stroke_width, ring_d );
    color += ring_color * glow;
    color = mix( color, ring_color, core );
  }

  // 8. Authored planets/moons: up to NODE_COUNT bodies, each with its own
  // orbit radius, angular speed (sign = direction), phase, size, and color
  // — independent of the keyboard-controlled procedural nodes below.
  for ( var i : u32 = 0u; i < NODE_COUNT; i++ )
  {
    let node_color = uniforms.node_colors[ i ].xyz;
    let size = uniforms.node_colors[ i ].w;
    let radius = uniforms.node_params[ i ].x;
    let speed = uniforms.node_params[ i ].y;
    let phase = uniforms.node_params[ i ].z;

    let theta = uniforms.time * speed + phase;
    let pos = vec2f( 0.5 + radius * cos( theta ), 0.5 - radius * sin( theta ) );
    let pd = distance( uv, pos );

    let halo = ( 1.0 - smoothstep( 0.0, size * 1.6, pd ) ) * 0.85 * effects.z;
    color += node_color * halo;

    let core = 1.0 - smoothstep( size * 0.2, size * 0.4, pd );
    color = mix( color, node_color, core );
  }

  // 9. Keyboard-controlled procedural nodes, count controlled by
  // uniforms.node_count. Each node's phase and orbit radius are perturbed
  // by a hash of uniforms.seed so re-seeding visibly reshuffles the layout,
  // not just the star field. Unchanged demo behavior — orbits at the
  // innermost authored ring's radius/color.
  {
    let node_count = clamp( uniforms.node_count, 1, MAX_NODES );
    for ( var i : i32 = 0; i < MAX_NODES; i++ )
    {
      if ( i >= node_count )
      {
        break;
      }

      let fi = f32( i );
      let node_seed = vec2f( uniforms.seed + fi * 12.9898, uniforms.seed + fi * 78.233 );
      let phase_jitter = ( hash21( node_seed ) - 0.5 ) * 1.2;
      let radius_jitter = 0.85 + 0.3 * hash21( node_seed + 5.17 );

      // Reference composition specifies its angle ( 325 deg ) in image space
      // ( y grows downward ); negating the sine term converts it into this
      // shader's y-up uv space. Time adds a slow orbital drift on top, with
      // a small per-node speed offset so nodes don't move in lockstep.
      let theta = radians( 325.0 ) + uniforms.time * ( 0.15 - fi * 0.015 )
        + fi * ( 6.28318 / f32( node_count ) ) + phase_jitter;
      let orbit_r = uniforms.ring_params[ 0 ].x * radius_jitter;
      let planet_pos = vec2f( 0.5 + orbit_r * cos( theta ), 0.5 - orbit_r * sin( theta ) );
      let pd = distance( uv, planet_pos );

      let halo_color = uniforms.ring_colors[ 0 ].xyz;
      let halo = ( 1.0 - smoothstep( 0.0, 0.018, pd ) ) * 0.85;
      color += halo_color * halo * 0.85;

      let core = 1.0 - smoothstep( 0.003, 0.006, pd );
      color = mix( color, vec3f( 1.0, 1.0, 1.0 ), core );
    }
  }

  // 10. Effects: vignette darkens toward the frame edge; scanline adds a
  // faint sci-fi HUD texture. Both are single-pass analytic — no offscreen
  // texture exists to sample at an offset for a true multi-tap effect like
  // chromatic aberration.
  {
    let vignette_d = d * 1.4142136; // normalize so the farthest corner (uv (0,0)/(1,1)) reaches ~1.0
    let vignette = 1.0 - effects.x * smoothstep( effects.y, 1.0, vignette_d );
    let scanline = 1.0 - effects.w * ( 0.5 + 0.5 * sin( uv.y * 800.0 ) );
    color *= vignette * scanline;
  }

  return vec4f( color, 1.0 );
}
`;

#{
  background : background,
  nebula_bands : nebula_bands,
  star_layers : star_layers,
  grid : grid,
  sun_corona : sun_corona,
  sun_disc : sun_disc,
  orbit_rings : orbit_rings,
  nodes : nodes,
  effects : effects,
  shader : shader,
}
