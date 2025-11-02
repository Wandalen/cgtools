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
uniform mat4 u_light_view_projection_inv;
uniform sampler2D u_shadow_map;  // Color texture containing depth values (Chrome workaround)

out vec4 frag_color;

// Mathematical constants
#define TWO_PI 6.2831853
const float GOLDEN_ANGLE = 2.399963229728653;
/*
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
*/

#define NEAR_PLANE 1.0
#define FAR_PLANE 30.0
#define NUM_SAMPLES 126

const vec2 POISSON_DISK[ NUM_SAMPLES ] = vec2[](
    vec2(0.87343360, -0.87326241), vec2(-0.94782436, -0.84020627),
    vec2(0.99453741, -0.19828552), vec2(0.35414678, -0.99999028),
    vec2(-0.04655610, -0.89248270), vec2(0.73979825, -0.42104599),
    vec2(-0.57948315, 0.98583472), vec2(-0.33966607, -0.62310112),
    vec2(0.53637749, 0.98497677), vec2(0.18182747, 0.96373737),
    vec2(-0.97059774, 0.38048929), vec2(0.25299081, -0.52847117),
    vec2(-0.98936719, -0.32044825), vec2(0.96888447, 0.53934395),
    vec2(-0.21175319, 0.82522482), vec2(-0.64775836, -0.58933097),
    vec2(-0.03842609, 0.63841689), vec2(0.51892811, 0.44917613),
    vec2(-0.43265849, 0.04255088), vec2(0.17725945, -0.11656910),
    vec2(0.69085693, 0.00976293), vec2(0.04118512, 0.22896956),
    vec2(-0.91688049, 0.88195860), vec2(-0.25419080, 0.34440479),
    vec2(0.30132312, 0.58913958), vec2(0.90848988, -0.58197945),
    vec2(-0.51007468, -0.21855685), vec2(-0.00288251, -0.37020084),
    vec2(0.36643204, 0.16515869), vec2(0.67275685, 0.69747269),
    vec2(0.40798932, -0.17702581), vec2(-0.66952711, 0.30452684),
    vec2(-0.30906311, -0.22055620), vec2(0.05353591, -0.63102329),
    vec2(0.97816223, 0.09886479), vec2(-0.72593993, -0.06312781),
    vec2(0.24844371, 0.38815159), vec2(-0.35467464, 0.60942477),
    vec2(0.62724280, -0.19895011), vec2(-0.06821339, 0.00699195),
    vec2(0.85239571, 0.29807204), vec2(-0.02422501, 0.98506898),
    vec2(-0.43572885, -0.96347100), vec2(-0.77884811, 0.60565811),
    vec2(0.49040315, -0.68656641), vec2(-0.20785744, -0.00490710),
    vec2(0.21245645, 0.00494452), vec2(-0.62886482, -0.32367462),
    vec2(0.18721612, 0.77112049), vec2(-0.92557716, -0.01026048),
    vec2(-0.22108191, 0.99998188), vec2(0.48514190, -0.38855642),
    vec2(-0.46321285, 0.34701887), vec2(0.63720977, 0.34212971),
    vec2(-0.16327310, 0.51139450), vec2(0.09706174, -0.99427110),
    vec2(0.30606869, -0.78508103), vec2(0.91696888, 0.81745732),
    vec2(-0.39077270, -0.42080313), vec2(0.71077532, -0.69747978),
    vec2(-0.64821613, 0.02046467), vec2(0.11718012, 0.33446860),
    vec2(0.81754547, -0.01018671), vec2(-0.84039867, -0.56272310),
    vec2(-0.11261313, -0.76442111), vec2(0.44687057, 0.79374492),
    vec2(0.01140924, 0.49514785), vec2(0.78311497, 0.98822081),
    vec2(-0.79633212, 0.28711468), vec2(-0.99723387, 0.66270596),
    vec2(0.24424915, -0.34446415), vec2(-0.30452368, 0.19830869),
    vec2(0.47051394, 0.26084930), vec2(-0.53730047, -0.76997018),
    vec2(0.60193807, -0.91166663), vec2(0.10375620, -0.24133468),
    vec2(0.94270402, -0.37525204), vec2(0.07604313, 0.87116933),
    vec2(-0.07519962, -0.17704250), vec2(-0.91428578, -0.44525096),
    vec2(-0.25269786, -0.98563761), vec2(0.36442152, 0.99890810),
    vec2(0.62734187, 0.13673419), vec2(0.70624024, 0.50547791),
    vec2(0.34484196, -0.56611311), vec2(-0.50099611, 0.77128524),
    vec2(0.53597409, -0.00940428), vec2(-0.27438134, 0.73030239),
    vec2(-0.73273391, -0.86566442), vec2(0.13685513, -0.47535497),
    vec2(0.28189737, 0.88457632), vec2(-0.43236750, -0.09849200),
    vec2(-0.16515234, -0.49655610), vec2(0.99981880, 0.34327787),
    vec2(0.81432432, 0.65089357), vec2(-0.08985694, 0.33405787),
    vec2(-0.59714854, -0.48529324), vec2(0.50244677, 0.09459312),
    vec2(0.21980327, 0.20780283), vec2(0.93202513, -0.00288863),
    vec2(0.02102875, -0.05047879), vec2(0.40798363, -0.01633519),
    vec2(-0.35560951, 0.43431336), vec2(-0.16875902, 0.17056636),
    vec2(0.68910408, -0.38073578), vec2(-0.79374826, -0.27896422),
    vec2(-0.57396650, 0.51867169), vec2(0.21855269, -0.65584826),
    vec2(0.08826767, -0.79093558), vec2(-0.39578643, 0.90045953),
    vec2(0.73977506, 0.23126581), vec2(0.46820065, 0.58288515),
    vec2(0.15843478, 0.63229710), vec2(-0.78129035, 0.98592079),
    vec2(-0.21731518, -0.33538419), vec2(-0.06171720, 0.77190012),
    vec2(0.41372332, 0.06209865), vec2(-0.73708302, 0.05364802),
    vec2(-0.95719987, 0.12604818), vec2(-0.02640243, -0.25200272),
    vec2(0.69700307, 0.88788486), vec2(0.19830704, -0.89880860),
    vec2(-0.33719119, -0.81220943), vec2(-0.51351571, 0.18701968),
    vec2(0.39536104, 0.40055740), vec2(-0.99926877, -0.58189887)
);

float linearize_depth( float depth )
{
  float z = depth * 2.0 - 1.0; // Back to NDC
  return ( 2.0 * NEAR_PLANE * FAR_PLANE ) / ( FAR_PLANE + NEAR_PLANE - z * ( FAR_PLANE - NEAR_PLANE ) );
  // return depth;
}

float rand( vec2 co )
{
  return fract( sin( dot( co, vec2( 12.9898, 78.233 ) ) ) * 43758.5453 );
}

float find_blocker( vec2 texcoords, float receiver_depth, float search_radius )
{
  vec2 texel_size = 1.0 / vec2( textureSize( u_shadow_map, 0 ) );

  float total = 0.0;
  int count = 0;

  float blocker_epsilon = 0.01;

  for ( int i = 0; i < NUM_SAMPLES; i++ )
  {
    vec2 offset = POISSON_DISK[ i ] * search_radius;

    vec2 t = texcoords + offset * texel_size;
    float depth = linearize_depth( texture( u_shadow_map, t ).r );

    bool is_inside =
    !(
      t.x < 0.0
      || t.x > 1.0
      || t.y < 0.0
      || t.y > 1.0
    );
    depth *= float( receiver_depth - blocker_epsilon > depth );
    depth *= float( is_inside );

    total += depth;
    count += int( is_inside );
  }

  return total / float( count );
}

float compute_penumbra( float receiver_depth, float blocker_depth, float light_radius )
{
  float dist = max( receiver_depth - blocker_depth, 0.0 );
  // Чем дальше блокер от ресивера, тем больше размытие
  // float penumbra = ( dist / max( blocker_depth, 1e-4 ) ) * light_radius;
  float penumbra = pow( dist, 2.0 ) * light_radius;// * 5.0; // масштаб подбирается эмпирически
  return penumbra;
}

float pcss_shadow( vec3 proj_coords, float receiver_depth, float light_radius )
{
  // 5.1 — поиск среднего блокера
  float avg_blocker_depth = find_blocker( proj_coords.xy, receiver_depth, light_radius );
  float blocker_depth = avg_blocker_depth;

  // 5.2 — вычисление penumbra (размытие) с учётом размера света
  float penumbra = compute_penumbra( receiver_depth, blocker_depth, light_radius );
  // penumbra = 10.0;
  // 5.3 — рандомизация угла вращения сэмплов, чтобы убрать зерно
  float angle = rand( gl_FragCoord.xy ) * TWO_PI;
  mat2 rot = mat2
  (
    cos( angle ), -sin( angle ),
    sin( angle ),  cos( angle )
  );

  // 5.4 — фильтрация с depth-aware весами
  float shadow = 0.0;
  float total_weight = 0.0;

  vec2 texel_size = 1.0 / vec2( textureSize( u_shadow_map, 0 ) );
  float bias = 0.01;

  for ( int i = 0; i < NUM_SAMPLES; i++ )
  {
    vec2 offset = rot * POISSON_DISK[ i ] * penumbra * texel_size;
    float depth = linearize_depth( texture( u_shadow_map, proj_coords.xy + offset ).r );
    float w = exp( -abs( depth - receiver_depth ) * 50.0 ); // глубинный вес

    if ( receiver_depth - bias > depth )
    {
      shadow += 1.0;
    }
    // shadow += ( step( receiver_depth - bias, depth ) );// * w;
    total_weight += w;
  }

  // shadow /= total_weight;

  // 5.5 — добавляем эффект light falloff (тень светлеет на краях)
  // float light_falloff = exp( -penumbra );
  // shadow = mix( shadow, 0.0, 1.0 - light_falloff );

  // float dist = max( receiver_depth - blocker_depth, 0.0 );
  // penumbra = ( dist / max( blocker_depth, 1e-4 ) ) * u_light_size;
  // float softness = clamp( penumbra, 0.0, 1.0 );
  // float visibility = mix( 1.0, 0.0, softness );

  return shadow / float( NUM_SAMPLES );
}

// Simple hard shadow (kept for comparison/debugging)
float calculate_shadow( vec4 light_space_pos, vec3 light_dir, vec3 normal )
{
  // Perspective divide to get NDC coordinates [-1, 1]
  vec3 proj_coords = light_space_pos.xyz / light_space_pos.w;
  proj_coords = proj_coords * 0.5 + 0.5;
  bool is_inside =
  !(
    proj_coords.x < 0.0
    || proj_coords.x > 1.0
    || proj_coords.y < 0.0
    || proj_coords.y > 1.0
  );
  float n_dot_l = max( dot( normal, light_dir ), 0.0 );

  float receiver_depth = linearize_depth( proj_coords.z );
  float blocker_depth = linearize_depth( texture( u_shadow_map, proj_coords.xy ).r );

  float shadow = float( receiver_depth > blocker_depth );
  shadow *= float( proj_coords.z < 1.0 );
  shadow *= float( proj_coords.z > 0.0 );
  shadow *= float( n_dot_l > 0.0 );
  shadow *= float( is_inside );

  bool is_shadowed = shadow == 1.0;

  float bias = max( 0.03 * ( 1.0 - n_dot_l ), 0.005 );

  shadow = pcss_shadow( proj_coords, receiver_depth - bias, u_light_size );
  // float dist = max( receiver_depth - blocker_depth, 0.0 );

  // float penumbra = ( dist / max( blocker_depth, 1e-4 ) ) * u_light_size;
  // float softness = clamp( penumbra, 0.0, 1.0 );
  // float visibility = mix( 1.0, 0.0, softness );

  // shadow = visibility;

  // vec2 texel_size = 1.0 / vec2( textureSize( u_shadow_map, 0 ) );

  return shadow * float( is_shadowed );
  // return float( is_shadowed );
  // return 0.0;
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
  float shadow = calculate_shadow( v_light_space_pos, light_dir, normal );

  // Output: R channel = shadow (0 = lit, 1 = shadowed)
  frag_color = vec4( shadow, 0.0, 0.0, 1.0 );
}
