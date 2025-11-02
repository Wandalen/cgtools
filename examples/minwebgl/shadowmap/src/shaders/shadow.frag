#version 300 es
precision highp float;
precision highp sampler2D;

in vec3 v_world_pos;
in vec3 v_normal;
in vec4 v_light_space_pos;

uniform vec3 u_light_dir;        // Used for orthographic (directional light)
uniform vec3 u_light_position;   // Used for perspective (point light)
uniform float u_is_orthographic;
uniform float u_light_size;      // World-space light size for penumbra control
uniform sampler2D u_shadow_map;  // Color texture containing depth values (Chrome workaround)

out vec4 frag_color;

// Mathematical constants
const float TWO_PI = 6.283185307179586;
const float GOLDEN_ANGLE = 2.399963229728653;

// ═══════════════════════════════════════════════════════════════════════════
// STEP 1: SAMPLING PATTERN - Vogel Disk
// ═══════════════════════════════════════════════════════════════════════════
// This creates a spiral pattern that distributes samples evenly across a disk.
// Better than grid patterns (causes aliasing) or random (needs more samples).
//
// How it works:
//  - Samples are placed using golden angle (137.5°) for optimal distribution
//  - Radius increases with sqrt to maintain even density
//  - angle_offset rotates the pattern per-pixel to reduce banding artifacts
//
vec2 vogel_disk_sample( int sample_index, int num_samples, float angle_offset )
{
  float r = sqrt( float( sample_index ) + 0.5 ) / sqrt( float( num_samples ) );
  float theta = float( sample_index ) * GOLDEN_ANGLE + angle_offset;
  return vec2( r * cos( theta ), r * sin( theta ) );
}

// ═══════════════════════════════════════════════════════════════════════════
// STEP 2: DITHERING NOISE
// ═══════════════════════════════════════════════════════════════════════════
// Generates screen-space noise to rotate sampling patterns per-pixel.
// This breaks up banding artifacts that would otherwise be visible.
//
float interleaved_gradient_noise( vec2 position )
{
  vec3 magic = vec3( 0.06711056, 0.00583715, 52.9829189 );
  return fract( magic.z * fract( dot( position, magic.xy ) ) );
}

// ═══════════════════════════════════════════════════════════════════════════
// STEP 3: BLOCKER SEARCH (PCSS Phase 1)
// ═══════════════════════════════════════════════════════════════════════════
// Searches for objects casting shadows by sampling the shadow map.
// Returns the average depth of blockers, or -1 if fully lit.
//
// Purpose: Determine HOW FAR the shadow caster is from this surface.
//         This distance directly affects penumbra width (shadow softness).
//
// Parameters:
//  - shadow_uv: Current pixel's position in shadow map [0,1]
//  - receiver_depth: Depth of the surface receiving shadow
//  - search_radius: How far to search (in texels)
//  - num_samples: Quality vs performance tradeoff
//
float find_blocker_distance( vec2 shadow_uv, float receiver_depth, float search_radius, int num_samples )
{
  float blocker_sum = 0.0;
  float blocker_count = 0.0;
  vec2 texel_size = 1.0 / vec2( textureSize( u_shadow_map, 0 ) );

  // Rotate sampling pattern per-pixel to break up visual patterns
  float angle_offset = interleaved_gradient_noise( gl_FragCoord.xy ) * TWO_PI;

  // Sample in a disk pattern around current pixel
  for ( int i = 0; i < num_samples; i++ )
  {
    vec2 offset = vogel_disk_sample( i, num_samples, angle_offset ) * search_radius;
    vec2 sample_uv = shadow_uv + offset * texel_size;

    // Skip samples outside shadow map bounds
    if ( sample_uv.x < 0.0 || sample_uv.x > 1.0 || sample_uv.y < 0.0 || sample_uv.y > 1.0 )
    {
      continue;
    }

    // Read depth of potential blocker
    float shadow_depth = texture( u_shadow_map, sample_uv ).r;

    // Is this closer to light than receiver? Then it's a blocker!
    const float blocker_epsilon = 0.00001;
    if ( shadow_depth < receiver_depth - blocker_epsilon )
    {
      blocker_sum += shadow_depth;
      blocker_count += 1.0;
    }
  }

  // No blockers = fully lit (no shadow)
  if ( blocker_count < 0.1 )
  {
    return -1.0;
  }

  // Return average blocker depth
  return blocker_sum / blocker_count;
}

// ═══════════════════════════════════════════════════════════════════════════
// STEP 4: PERCENTAGE CLOSER FILTERING (PCSS Phase 3)
// ═══════════════════════════════════════════════════════════════════════════
// Filters the shadow by sampling multiple times with the computed blur radius.
// This creates the actual soft shadow effect.
//
// Purpose: Convert binary shadow test into smooth gradient.
//
// How it works:
//  1. Sample shadow map multiple times in a disk
//  2. Count how many samples are in shadow
//  3. Return percentage (0 = lit, 1 = shadowed)
//
// The filter_radius determines softness (computed from blocker distance).
//
float pcss_shadow_filter( vec2 shadow_uv, float receiver_depth, float filter_radius, int num_samples, vec3 light_dir_to_use )
{
  float shadow_sum = 0.0;
  vec2 texel_size = 1.0 / vec2( textureSize( u_shadow_map, 0 ) );

  // ───────────────────────────────────────────────────────────────────────
  // Adaptive Bias: Prevents "shadow acne" artifacts
  // ───────────────────────────────────────────────────────────────────────
  // Shadow acne happens when a surface incorrectly shadows itself due to:
  //  - Limited shadow map resolution
  //  - Floating point precision
  //  - Surface angle relative to light
  //
  // Solution: Offset the depth comparison slightly based on surface angle.
  //          Steep angles need more bias than flat surfaces.
  //
  vec3 normal = normalize( v_normal );
  float cos_theta = max( dot( normal, light_dir_to_use ), 0.0 );

  // Different biases for orthographic vs perspective projections
  float base_bias = mix( 0.001, 0.0002, u_is_orthographic );
  float slope_bias = mix( 0.003, 0.0008, u_is_orthographic );
  float bias = mix( slope_bias, base_bias, cos_theta );

  bias = clamp( bias, 0.00005, 0.005 );

  // Rotate pattern per-pixel
  float angle_offset = interleaved_gradient_noise( gl_FragCoord.xy ) * TWO_PI;

  // Sample in a disk with radius = filter_radius
  for ( int i = 0; i < num_samples; ++i )
  {
    vec2 offset = vogel_disk_sample( i, num_samples, angle_offset ) * filter_radius;
    vec2 sample_uv = shadow_uv + offset * texel_size;

    // Out of bounds = treat as lit
    if ( sample_uv.x < 0.0 || sample_uv.x > 1.0 || sample_uv.y < 0.0 || sample_uv.y > 1.0 )
    {
      continue;
    }

    // Binary shadow test: is receiver deeper than shadow map?
    float shadow_depth = texture( u_shadow_map, sample_uv ).r;
    shadow_sum += ( receiver_depth - bias ) > shadow_depth ? 1.0 : 0.0;
  }

  // Return average: 0.0 = fully lit, 1.0 = fully shadowed
  return shadow_sum / float( num_samples );
}

// ═══════════════════════════════════════════════════════════════════════════
// MAIN PCSS CALCULATION (Puts it all together)
// ═══════════════════════════════════════════════════════════════════════════
// This is the complete PCSS algorithm with contact hardening.
//
// PCSS Algorithm Overview:
//   1. Search for blockers → Find average occluder depth
//   2. Estimate penumbra  → Calculate blur radius from geometry
//   3. Filter shadow map  → Sample with variable-size kernel
//   4. Modulate intensity → Optional: lighten distant soft shadows
//
// Result: Physically-based soft shadows that are:
//   - Sharp near contact points (contact hardening)
//   - Increasingly soft with distance from shadow caster
//   - Realistic penumbra that mimics area light sources
//
float calculate_shadow_pcss( vec4 light_space_pos )
{
  // ───────────────────────────────────────────────────────────────────────
  // Setup: Determine light direction and validate surface
  // ───────────────────────────────────────────────────────────────────────
  vec3 light_dir_to_use;
  if ( u_is_orthographic > 0.0 )
  {
    light_dir_to_use = u_light_dir;
  }
  else
  {
    light_dir_to_use = normalize( u_light_position - v_world_pos );
  }

  // Back-facing surfaces can't be in shadow (facing away from light)
  vec3 normal = normalize( v_normal );
  float n_dot_l = dot( normal, light_dir_to_use );
  if ( n_dot_l < 0.0 )
  {
    return 0.0;
  }

  // Transform to shadow map space
  vec3 proj_coords = light_space_pos.xyz / light_space_pos.w;
  proj_coords = proj_coords * 0.5 + 0.5;  // [-1,1] → [0,1]

  // Outside shadow map frustum = no shadow
  if ( proj_coords.z > 1.0 || proj_coords.z < 0.0 )
  {
    return 0.0;
  }

  float receiver_depth = proj_coords.z;

  // ╔══════════════════════════════════════════════════════════════════════╗
  // ║ PHASE 1: BLOCKER SEARCH                                             ║
  // ╠══════════════════════════════════════════════════════════════════════╣
  // ║ Goal: Find average depth of objects casting shadows                 ║
  // ║ Why:  Blocker distance determines penumbra width                    ║
  // ╚══════════════════════════════════════════════════════════════════════╝

  float light_world_size = u_light_size;  // Artistic control parameter

  // Calculate search radius
  float search_radius;
  if ( u_is_orthographic > 0.0 )
  {
    search_radius = light_world_size * 15.0;
  }
  else
  {
    float depth_scale = mix( 1.0, 2.5, clamp( receiver_depth, 0.0, 1.0 ) );
    search_radius = light_world_size * 30.0 * depth_scale;
  }

  // QUALITY SETTING: More samples = smoother blocker search
  // For baking: use high quality (64-128 samples)
  const int blocker_search_samples = 64;  // Increased from 32

  float avg_blocker_depth = find_blocker_distance
  (
    proj_coords.xy,
    receiver_depth,
    search_radius,
    blocker_search_samples
  );

  // No blockers = fully lit (early exit)
  if ( avg_blocker_depth < 0.0 )
  {
    return 0.0;
  }

  // ╔══════════════════════════════════════════════════════════════════════╗
  // ║ PHASE 2: PENUMBRA ESTIMATION                                        ║
  // ╠══════════════════════════════════════════════════════════════════════╣
  // ║ Goal: Calculate blur radius based on geometry                       ║
  // ║ Formula: penumbra ∝ (receiver - blocker) distance                   ║
  // ╚══════════════════════════════════════════════════════════════════════╝

  float blocker_distance = receiver_depth - avg_blocker_depth;
  float penumbra_width;

  if ( u_is_orthographic > 0.0 )
  {
    // Directional light: penumbra proportional to blocker distance
    penumbra_width = blocker_distance * light_world_size * 25.0;
  }
  else
  {
    // Point/spot light: standard PCSS formula
    // penumbra = (receiver - blocker) * lightSize / blocker
    penumbra_width = blocker_distance * search_radius / max( avg_blocker_depth, 0.01 );
  }

  // Clamp to reasonable range
  penumbra_width = clamp( penumbra_width, 2.0, 200.0 );  // Increased max from 150

  // ╔══════════════════════════════════════════════════════════════════════╗
  // ║ PHASE 3: PERCENTAGE CLOSER FILTERING                                ║
  // ╠══════════════════════════════════════════════════════════════════════╣
  // ║ Goal: Sample shadow map with computed blur radius                   ║
  // ║ Result: Smooth shadow gradient instead of binary                    ║
  // ╚══════════════════════════════════════════════════════════════════════╝

  // QUALITY SETTING: More samples = smoother shadows
  // For baking: use very high quality (128+ samples)
  const int pcf_samples = 128;  // Increased from 64

  float shadow = pcss_shadow_filter
  (
    proj_coords.xy,
    receiver_depth,
    penumbra_width,
    pcf_samples,
    light_dir_to_use
  );

  // ╔══════════════════════════════════════════════════════════════════════╗
  // ║ OPTIONAL: INTENSITY MODULATION                                      ║
  // ╠══════════════════════════════════════════════════════════════════════╣
  // ║ Makes very soft/distant shadows lighter for artistic reasons        ║
  // ║ Physically: soft = more light scattering around edges               ║
  // ╚══════════════════════════════════════════════════════════════════════╝

  // Normalize penumbra width to 0-1
  float penumbra_norm = clamp( ( penumbra_width - 2.0 ) / ( 200.0 - 2.0 ), 0.0, 1.0 );

  // Smooth curve: keep shadows dark longer, then fade
  penumbra_norm = pow( penumbra_norm, 0.6 );

  // Sharp shadows = dark (1.0), Soft shadows = lighter (0.3)
  float intensity_factor = mix( 1.0, 0.3, penumbra_norm );
  shadow *= intensity_factor;

  return shadow;
}

// float avg_blocker_depth()
// {

// }

// Simple hard shadow (kept for comparison/debugging)
float calculate_shadow_hard( vec4 light_space_pos, vec3 light_dir, vec3 normal )
{
  // Perspective divide to get NDC coordinates [-1, 1]
  vec3 proj_coords = light_space_pos.xyz / light_space_pos.w;
  proj_coords = proj_coords * 0.5 + 0.5;
  float n_dot_l = max( dot( normal, light_dir ), 0.0 );
  float receiver_depth = proj_coords.z;

  float blocker_depth = texture( u_shadow_map, proj_coords.xy ).r;

  float is_outside = float
  (
    proj_coords.x < 0.0
    || proj_coords.x > 1.0
    || proj_coords.y < 0.0
    || proj_coords.y > 1.0
  );

  float shadow = float( receiver_depth > blocker_depth );
  shadow *= float( proj_coords.z < 1.0 );
  shadow *= float( proj_coords.z > 0.0 );
  shadow *= float( n_dot_l > 0.0 );
  shadow = mix( shadow, 0.0, is_outside );

  // vec2 texel_size = 1.0 / vec2( textureSize( u_shadow_map, 0 ) );

  // float shadow = read_depth( proj_coords, receiver_depth, normal, light_dir );

  return shadow;
}

void main()
{
  // ═══════════════════════════════════════════════════════════════════════════
  // ENTRY POINT: Calculate shadow for this fragment
  // ═══════════════════════════════════════════════════════════════════════════
  //
  // Switch between PCSS (soft, realistic) and hard shadows:
  //   - calculate_shadow_pcss(): Soft shadows with contact hardening
  //   - calculate_shadow_hard(): Simple binary hard shadows
  //

  // Use PCSS for realistic soft shadows
  // float shadow = calculate_shadow_pcss( v_light_space_pos );

  // For debugging/comparison, you can switch to hard shadows:
  vec3 light_dir = bool( u_is_orthographic ) ? -u_light_dir : normalize( u_light_position - v_world_pos );
  vec3 normal = normalize( v_normal );
  float shadow = calculate_shadow_hard( v_light_space_pos, light_dir, normal );

  // Output: R channel = shadow (0 = lit, 1 = shadowed)
  frag_color = vec4( shadow, 0.0, 0.0, 1.0 );
}
