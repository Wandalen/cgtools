precision mediump float;

#ifdef AMBIENT_COLOR
uniform vec3 ambient;
#endif
#ifdef DIFFUSE_COLOR
uniform vec3 diffuse;
#endif
#ifdef SPECULAR_COLOR
uniform vec3 specular;
#endif
//Metallic
#ifdef SHININESS
uniform float shininess;
#endif
//Alpha of the material
#ifdef DISSOLVE
uniform float dissolve;
#endif
//Index of refraction
#ifdef OPTICAL_DENSITY
uniform float optical_density;
#endif
#ifdef AMBIENT_TEXTURE
uniform sampler2D ambient_texture;
#endif
#ifdef DIFFUSE_TEXTURE
uniform sampler2D diffuse_texture;
#endif
#ifdef SPECULAR_TEXTURE
uniform sampler2D specular_texture;
#endif
#ifdef NORMAL_TEXTURE
uniform sampler2D normal_texture;
#endif
#ifdef SHININESS_TEXTURE
uniform sampler2D shininess_texture;
#endif
#ifdef DISSOLVE_TEXTURE
uniform sampler2D dissolve_texture;
#endif

const float PI = 3.1415926;

uniform vec3 cameraPosition;

in vec3 vNormal;
in vec2 vUv;
in vec3 vWorldPos;

out vec4 frag_color;

float saturate( float v )
{
  return clamp( v, 0.0, 1.0 );
}

// Schlick approximation to the Fresnel equation
vec3 fresnel( float VdotH, vec3 F0 ) 
{
  return F0 + ( 1.0 - F0 ) * pow( 1.0 - VdotH, 5.0 );
}

// Normal distribution function
float NDF( float NdotH, float roughness ) 
{
  float alpha = roughness * roughness;
  float alpha2 = alpha * alpha;
  float denom = PI * pow( NdotH * NdotH * ( alpha2 - 1.0 ) + 1.0, 2.0 );

  return alpha2 / max( denom, 0.001 );
}

// Normal dot vector(any)
// Shlick's approximation of the geometry term
float germ_schlick_ggx( float NdotV, float roughness ) 
{
  float alpha = roughness * roughness;
  float k = alpha / 2.0;
  float denom = NdotV * ( 1.0 - k ) + k;

  return NdotV / max( denom, 0.001 );
}

// Geometry term, Smith's version
float Germ( float NdotL, float NdotV, float roughness ) 
{
  return germ_schlick_ggx( NdotL, roughness ) * germ_schlick_ggx( NdotV, roughness );
}

// Simple BRDF
vec3 BRDF( vec3 albedo, vec3 lightDir, vec3 viewDir, vec3 normal, float roughness, vec3 F0 ) 
{
  vec3 halfway = normalize( lightDir + viewDir );
  float NdotL = saturate( dot( normal, lightDir ) );
  float NdotV = saturate( dot( normal, viewDir ) );
  float NdotH = saturate( dot( normal, halfway) );
  float VdotH = saturate( dot( viewDir, halfway ) );

  vec3 F = fresnel( VdotH, F0 );
  float D = NDF( NdotH, roughness );
  float G = Germ( NdotL, NdotV,  roughness );

  float denom = 4.0 * max( NdotL * NdotV, 0.00001 );

  vec3 specular = F * D * G / denom;

  vec3 kD = vec3( 1.0 ) - F; // Amount of transmitted light
  
  return (kD * albedo / PI + specular) * NdotL;
}


// This shader uses PBR model for lighting calculations
// https://learnopengl.com/PBR/Theory
void main()
{
  // Ambient
  #ifdef AMBIENT_COLOR
  vec3 ambient_color = ambient;
  #else
  vec3 ambient_color = vec3( 0.1 );
  #endif
  #ifdef AMBIENT_TEXTURE
  ambient_color *= texture( ambient_texture, vUv ).rgb;
  #endif
  // Gamma correct to linear space
  ambient_color = pow( ambient_color, vec3( 2.2 ) );
  ///////////////////////

  // Diffuse
  #ifdef DIFFUSE_COLOR
  vec3 albedo = diffuse;
  #else
  vec3 albedo = vec3( 1.0 );
  #endif
  #ifdef DIFFUSE_TEXTURE
  albedo *= texture( diffuse_texture, vUv ).rgb;
  #endif
  // Gamma correct to linear space
  albedo = pow( albedo, vec3( 2.2 ) );
  ///////////////////////


  // Specular
  #ifdef SPECULAR_COLOR
  vec3 F0 = specular;
  #else
  vec3 F0 = vec3( 0.04 ); // Glass
  #endif
  #ifdef SPECULAR_TEXTURE
  F0 *= texture( specular_texture, vUv ).rgb;
  #endif
  ///////////////////////

  // Shineness
  // In obj format, shininess is a value from 0.0 to 1000.0
  // It is used as an exponent in Phong model
  // Since I use pbr, I just approximate roughness by normalizing the value
  #ifdef SHININESS
  float shiny = shininess;
  #else
  float shiny = 500.0;
  #endif
  #ifdef SHININESS_TEXTURE
  shiny *= texture( shininess_texture, vUv ).r;
  #endif
  float roughness = max( 1.0 - shiny / 1000.0, 0.0 );
  ///////////////////////

  // Dissolve
  #ifdef DISSOLVE
  float alpha = dissolve;
  #else
  float alpha = 1.0;
  #endif
  #ifdef DISSOLVE_TEXTURE
  alpha *= texture( dissolve_texture, vUv ).r;
  #endif

  vec3 normal = normalize( vNormal );
  // Normal mapping
  // https://learnopengl.com/Advanced-Lighting/Normal-Mapping
  #ifdef NORMAL_TEXTURE
  vec3 n = normalize( texture( normal_texture, vUv ).rgb * 2.0 - 1.0 );
  vec3 dE1 = dFdx( vWorldPos );
  vec3 dE2 = dFdy( vWorldPos );
  vec2 dUv1 = dFdx( vUv );
  vec2 dUv2 = dFdy( vUv );
  float denom = 1.0 / ( dUv1.x * dUv2.y - dUv1.y * dUv2.x );
  vec3 T;
  T.x = dUv2.y * dE1.x - dUv1.y * dE2.x;
  T.y = dUv2.y * dE1.y - dUv1.y * dE2.y;
  T.z = dUv2.y * dE1.z - dUv1.y * dE2.z;
  T = normalize( T * denom );
  vec3 B;
  B.x = -dUv2.x * dE1.x + dUv1.x * dE2.x;
  B.y = -dUv2.x * dE1.y + dUv1.x * dE2.y;
  B.z = -dUv2.x * dE1.z + dUv1.x * dE2.z;
  B = normalize( B * denom );
  mat3x3 TBN = mat3x3(T, B, normal);
  normal = normalize( TBN * n );
  #endif

  float lightIntensity = 2.0;
  vec3 viewDir = normalize( cameraPosition - vWorldPos );
  vec3 lightDirs[] = vec3[]
  (
    vec3( 1.0, 0.0, 0.0 ),
    vec3( 0.0, 1.0, 0.0 ),
    vec3( 0.0, 0.0, 1.0 ),
    vec3( -1.0, 0.0, 0.0 ),
    vec3( 0.0, -1.0, 0.0 ),
    vec3( 0.0, 0.0, -1.0 )
  );

  vec3 color = vec3( 0.0 );

  // If normal were not provided with the model, then their value is going to be 0
  if( length( normal ) > 0.0 )
  {
    vec3 Lo = vec3( 0.0 );
    for( int i = 0; i < 6; i++ )
    {
      Lo += BRDF( albedo, lightDirs[ i ], viewDir, normal, roughness, F0 ) * lightIntensity;
    }
    color += ambient_color * albedo + Lo;
  }
  else
  {
    color = ambient_color * albedo + albedo;
  }

  // Gamma correciton
  color = pow( color, vec3( 1.0 / 2.2 ) );
  
  frag_color = vec4( color, alpha );
}
