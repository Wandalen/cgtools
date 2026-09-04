#version 300 es
precision mediump float;
#pragma vscode_glsllint_stage : frag

const float PI = 3.1415926;

const int GRID_COLS = 5;
const int GRID_ROWS = 2;

uniform vec2 u_resolution;
uniform vec3 u_base_color;
uniform float u_light_intensity;
uniform float u_ambient_intensity;
uniform float u_exposure;
uniform float u_time;

in vec2 vUv;
out vec4 frag_color;

float circleSDF( vec2 pos, float r )
{
  return length( pos ) - r;
}

// Schlick ver.
vec3 freshel( vec3 viewDir, vec3 halfway, float metallic, float reflectance )
{
  vec3 f0 = vec3( 0.16 * reflectance * reflectance );
  f0 = mix( f0, u_base_color, metallic );

  return f0 + ( 1.0 - f0 ) * pow( ( 1.0 - dot( viewDir, halfway ) ), 5.0 );
}

// Normal distribution function
float NDF( vec3 normal, vec3 halfway, float roughness )
{
  float alpha = roughness * roughness;
  float alpha2 = alpha * alpha;
  float denom = PI * pow( pow( dot( normal, halfway ), 2.0 ) * ( alpha2 - 1.0 ) + 1.0, 2.0 );

  return alpha2 / denom;
}

float germ_schlick_ggx( vec3 normal, vec3 v, float roughness )
{
  float alpha = roughness * roughness;
  float k = alpha / 2.0;
  float NoV = dot( normal, v );
  float denom = NoV * ( 1.0 - k ) + k;

  return max( NoV, 0.001 ) / denom;
}

// Geometry term, Smith ver.
float Germ( vec3 lightDir, vec3 viewDir, vec3 normal, float roughness )
{
  return germ_schlick_ggx( normal, lightDir, roughness ) * germ_schlick_ggx( normal, viewDir, roughness );
}

vec3 BRDF( vec3 lightDir, vec3 viewDir, vec3 normal, float metallic, float roughness, float reflectance )
{
  vec3 halfway = normalize( lightDir + viewDir );

  vec3 F = freshel( viewDir, halfway, metallic, reflectance );
  float D = NDF( normal, halfway, roughness );
  float G = Germ( lightDir, viewDir, normal, roughness );

  float denom = 4.0 * max( dot( normal, lightDir ) * dot( normal, viewDir ), 0.001 );

  vec3 specular = F * D * G / denom;

  vec3 diffuse = u_base_color;
  diffuse *= vec3( 1.0 ) - F; // Amount of transmitted light
  diffuse *= 1.0 - metallic; // Metals do not have diffuse light
  diffuse /= PI;

  return diffuse + specular;
}

mat2x2 rot( float angle )
{
  float s = sin( angle );
  float c = cos( angle );
  return mat2x2( c, s, -s, c );
}

// Two-tone hemisphere ambient: sky tint above the horizon, ground tint below, so no side of a
// sphere is ever pure black just because no key/fill/rim light happens to reach it.
vec3 ambient( vec3 normal, vec3 albedo )
{
  vec3 sky = vec3( 0.55, 0.65, 0.85 );
  vec3 ground = vec3( 0.25, 0.22, 0.18 );
  vec3 hemi = mix( ground, sky, normal.y * 0.5 + 0.5 );

  return hemi * albedo * u_ambient_intensity;
}

// Narkowicz ACES filmic tonemap approximation.
vec3 tonemapACES( vec3 color )
{
  float a = 2.51;
  float b = 0.03;
  float c = 2.43;
  float d = 0.59;
  float e = 0.14;

  return clamp( ( color * ( a * color + b ) ) / ( color * ( c * color + d ) + e ), 0.0, 1.0 );
}

void main()
{
  // Pixel coordinates in the full canvas, then figure out which grid cell this pixel is in.
  vec2 pixelCoords = vUv * u_resolution;
  vec2 cellSize = u_resolution / vec2( float( GRID_COLS ), float( GRID_ROWS ) );

  vec2 cellF = clamp( floor( pixelCoords / cellSize ), vec2( 0.0 ), vec2( float( GRID_COLS - 1 ), float( GRID_ROWS - 1 ) ) );

  // Columns sweep roughness smooth -> rough; rows split dielectric (bottom) / metal (top).
  float roughness = mix( 0.05, 0.95, cellF.x / float( GRID_COLS - 1 ) );
  float metallic = cellF.y / float( GRID_ROWS - 1 );
  float reflectance = 0.5;

  // Recenter pixel coordinates on the cell this pixel belongs to.
  vec2 cellCenter = ( cellF + 0.5 ) * cellSize;
  vec2 local = pixelCoords - cellCenter;

  float circleRadius = min( cellSize.x, cellSize.y ) * 0.38;
  float circle = circleSDF( local, circleRadius );

  vec2 xy = local;

  // This is needed to smoothout the edges of the sphere
  if( circle > 0.0 )
  {
    xy = circleRadius * normalize( xy );
  }

  float r = length( xy );
  float z = sqrt( max( circleRadius * circleRadius - r * r, 0.0 ) );

  vec3 position = vec3( xy, z );
  vec3 normal = normalize( position );
  vec3 viewDir = normalize( vec3( 0.0, 0.0, 1.0 ) );

  float time = u_time / 1000.0;

  // Key: warm, slowly orbiting overhead-front light -- the dominant light.
  vec2 keyXY = rot( time * 0.3 ) * vec2( 0.6, 0.5 );
  vec3 keyDir = normalize( vec3( keyXY, 0.9 ) );
  vec3 keyColor = vec3( 1.0, 0.98, 0.92 ) * u_light_intensity;

  // Fill: soft, cool, static light from camera-left, keeping the shadow side readable.
  vec3 fillDir = normalize( vec3( -0.7, 0.1, 0.4 ) );
  vec3 fillColor = vec3( 0.55, 0.65, 0.85 ) * u_light_intensity * 0.35;

  // Rim: near-grazing highlight from above -- the impostor sphere has no back hemisphere, so
  // this is the closest thing to backlighting it can render (small z, mostly tangent).
  vec3 rimDir = normalize( vec3( 0.2, 0.9, 0.15 ) );
  vec3 rimColor = vec3( 1.0, 0.95, 0.85 ) * u_light_intensity * 0.5;

  vec3 color = ambient( normal, u_base_color );

  float keyIrradiance = max( dot( normal, keyDir ), 0.0 );
  color += BRDF( keyDir, viewDir, normal, metallic, roughness, reflectance ) * keyColor * keyIrradiance;

  float fillIrradiance = max( dot( normal, fillDir ), 0.0 );
  color += BRDF( fillDir, viewDir, normal, metallic, roughness, reflectance ) * fillColor * fillIrradiance;

  float rimIrradiance = max( dot( normal, rimDir ), 0.0 );
  color += BRDF( rimDir, viewDir, normal, metallic, roughness, reflectance ) * rimColor * rimIrradiance;

  vec3 background = mix( vec3( 0.02, 0.02, 0.035 ), vec3( 0.09, 0.10, 0.14 ), vUv.y );

  //Smooth out the edges of the sphere
  color = mix( color, background, smoothstep( 0.0, 2.0, circle ) );

  color *= u_exposure;
  color = tonemapACES( color );

  // Gamma correction. Exponent is a precomputed literal (1.0 / 2.2) rather than a division
  // expression -- this Mesa software-GL driver silently miscompiles `pow(color, vec3(1.0/2.2))`
  // to all-zero output, while the identical precomputed constant works correctly.
  color = pow( color, vec3( 0.4545455 ) );

  frag_color = vec4( color, 1.0 );
}
