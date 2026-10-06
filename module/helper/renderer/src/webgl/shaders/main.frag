precision highp float;

#define PI 3.141592653589793
#define PI2 6.283185307179586
#define PI_HALF 1.5707963267948966
#define RECIPROCAL_PI 0.3183098861837907
#define RECIPROCAL_PI2 0.15915494309189535
#define EPSILON 1e-6

in vec2 vUv_0;
in vec2 vUv_1;
in vec2 vUv_2;
in vec2 vUv_3;
in vec2 vUv_4;
#ifdef USE_TANGENTS
  in vec4 vTangent;
#endif
in vec3 vWorldPos;
in vec3 vNormal;

layout( location = 0 ) out vec4 frag_color;
layout( location = 1 ) out vec4 emissive_color;
layout( location = 2 ) out vec4 trasnparentA;
layout( location = 3 ) out float transparentB;

uniform vec3 cameraPosition;
uniform float exposure;

#ifdef USE_ALPHA_CUTOFF
  uniform float alphaCutoff;
#endif

struct PhysicalMaterial
{
  vec3 diffuseColor;
  float metallness;
  float roughness;
  vec3 f0;
  vec3 f90;
  #ifdef USE_KHR_materials_clearcoat
    vec3 clearcoatNormal;
    float clearcoatFactor;
    float clearcoatRoughness;
  #endif
  #ifdef USE_KHR_materials_anisotropy
    vec3 anisotropicT;
    vec3 anisotropicB;
    float at;
    float ab;
    float anisotropyStrength;
  #endif
};

struct ReflectedLight
{
  vec3 indirectDiffuse;
  vec3 indirectSpecular;
  vec3 directDiffuse;
  vec3 directSpecular;
  #ifdef USE_KHR_materials_clearcoat
    // Direct-light coat lobe; image-based coat light is kept apart so occlusion can darken it
    // without darkening point / spot / directional highlights.
    vec3 clearcoatSpecular;
    vec3 clearcoatIndirectSpecular;
  #endif
};

struct PointLight
{
  vec3    position;
  vec3    color;
  float   strength;
  float   range;
};

struct DirectLight
{
  vec3    direction;
  vec3    color;
  float   strength;
};

struct SpotLight
{
  vec3    position;
  vec3    direction;
  vec3    color;
  float   strength;
  float   range;
  float   innerConeAngle;
  float   outerConeAngle;
  bool    useLightMap;
};

uniform PointLight pointLights[ MAX_POINT_LIGHTS ];
uniform int pointLightsCount;

uniform DirectLight directLights[ MAX_DIRECT_LIGHTS ];
uniform int directLightsCount;

uniform SpotLight spotLights[ MAX_SPOT_LIGHTS ];
uniform int spotLightsCount;

uniform float metallicFactor; // Default: 1
uniform float roughnessFactor; // Default: 1
uniform vec4 baseColorFactor; // Default: [1, 1, 1, 1]

#ifdef USE_IBL
  #ifdef GL_FRAGMENT_PRECISION_HIGH
    uniform highp samplerCube irradianceTexture;
    uniform highp samplerCube prefilterEnvMap;
    uniform highp sampler2D integrateBRDF;
  #else
    uniform mediump samplerCube irradianceTexture;
    uniform mediump samplerCube prefilterEnvMap;
    uniform mediump sampler2D integrateBRDF;
  #endif
  uniform float u_max_lod;
#endif
#ifdef USE_KHR_materials_specular
  uniform float specularFactor;
  uniform vec3 specularColorFactor;
  #ifdef USE_SPECULAR_TEXTURE
    uniform sampler2D specularTexture;
  #endif
  #ifdef USE_SPECULAR_COLOR_TEXTURE
    uniform sampler2D specularColorTexture;
  #endif
#endif
#ifdef USE_KHR_materials_clearcoat
  uniform float clearcoatFactor;
  uniform float clearcoatRoughnessFactor;
  uniform float clearcoatNormalScale;
  #ifdef USE_CLEARCOAT_TEXTURE
    uniform sampler2D clearcoatTexture;
  #endif
  #ifdef USE_CLEARCOAT_ROUGHNESS_TEXTURE
    uniform sampler2D clearcoatRoughnessTexture;
  #endif
  #ifdef USE_CLEARCOAT_NORMAL_TEXTURE
    uniform sampler2D clearcoatNormalTexture;
  #endif
#endif
#ifdef USE_KHR_materials_anisotropy
  uniform float anisotropyStrength;
  uniform float anisotropyRotation;
  #ifdef USE_ANISOTROPY_TEXTURE
    uniform sampler2D anisotropyTexture;
  #endif
#endif
#ifdef USE_MR_TEXTURE
  // Roughness is sampled from the G channel
  // Metalness is sampled from the B channel
  // vMRUv
  uniform sampler2D metallicRoughnessTexture;
#endif
#ifdef USE_BASE_COLOR_TEXTURE
  // vBaseColorUv
  uniform sampler2D baseColorTexture;
#endif


// Scales the normal in X and Y directions
// ( <sample normalTexture> * 2.0 - 1.0 ) * vec3( normalScale, normalScale, 1.0 )
uniform float normalScale; // Default: 1
#ifdef USE_NORMAL_TEXTURE
  // vNormalUv
  uniform sampler2D normalTexture;
#endif

// 1.0 + occlusionStrength * ( <sample occlusionTexture> - 1.0 )
uniform float occlusionStrength; // Default: 1
#ifdef USE_OCCLUSION_TEXTURE
  // vOcclusionUv
  uniform sampler2D occlusionTexture;
#endif


// vEmissionUv
#ifdef USE_EMISSION_TEXTURE
  uniform sampler2D emissiveTexture;
#endif
uniform vec3 emissiveFactor;


// vLightMapUv
#ifdef USE_LIGHT_MAP
  uniform sampler2D lightMap;
#endif


float max_value( const in vec3 v )
{
  return max( v.x, max( v.y, v.z ) );
}

float pow2( const in float x )
{
  return x*x;
}

vec3 pow2( const in vec3 x )
{
  return x*x;
}

float pow3( const in float x )
{
  return x*x*x;
}

float pow4( const in float x )
{
  float x2 = x*x;
  return x2*x2;
}

float dot2( const in vec3 v )
{
  return dot( v, v );
}

vec4 SrgbToLinear( const in vec4 color )
{
  vec3 more = pow( color.rgb * 0.9478672986 + vec3( 0.0521327014 ), vec3( 2.4 ) );
  vec3 less = color.rgb * 0.0773993808;

  return vec4( mix( more, less, vec3( lessThanEqual( color.rgb, vec3( 0.04045 ) ) ) ), color.a );
}

vec4 LinearToSrgb( const in vec4 color )
{
  vec3 more = pow( color.rgb, vec3( 0.41666 ) ) * 1.055 - vec3( 0.055 );
  vec3 less = color.rgb * 12.92;

  return vec4( mix( more, less, vec3( lessThanEqual( color.rgb, vec3( 0.0031308 ) ) ) ), color.a );
}

vec3 SrgbToLinear( const in vec3 color )
{
  vec3 more = pow( color * 0.9478672986 + vec3( 0.0521327014 ), vec3( 2.4 ) );
  vec3 less = color * 0.0773993808;

  return mix( more, less, vec3( lessThanEqual( color, vec3( 0.04045 ) ) ) );
}

vec3 LinearToSrgb( const in vec3 color )
{
  vec3 more = pow( color, vec3( 0.41666 ) ) * 1.055 - vec3( 0.055 );
  vec3 less = color * 12.92;

  return mix( more, less, vec3( lessThanEqual( color, vec3( 0.0031308 ) ) ) );
}

// Schilck's version of Fresnel equation, with Spherical Gaussian approximation for the power
// https://blog.selfshadow.com/publications/s2013-shading-course/karis/s2013_pbs_epic_notes_v2.pdf
vec3 F_Schlick( const in vec3 f0, const in vec3 f90, const in float dotVH )
{
  float fresnel = exp2( ( - 5.55473 * dotVH - 6.98316 ) * dotVH );
  return f0 + ( f90 - f0 ) * fresnel;
}

vec3 Fd_Barley
(
  const in float alpha,
  const in float dotNV,
  const in float dotNL,
  const in float dotLH
)
{
  vec3 f90 = vec3( 0.5 + 2.0 * alpha * pow2( dotLH ) );
  vec3 lightScatter = F_Schlick( vec3( 1.0 ), f90, dotNL );
  vec3 viewScatter = F_Schlick( vec3( 1.0 ), f90, dotNV );
  return viewScatter * lightScatter * RECIPROCAL_PI;
}

// https://web.archive.org/web/20160702002225/http://www.frostbite.com/wp-content/uploads/2014/11/course_notes_moving_frostbite_to_pbr_v2.pdf
// https://inria.hal.science/hal-00942452v1/document
// Visibility Geometry function
// V = G / ( 4 * dotNV * dotNL )
// G = G1( L ) * G1( V )
// G1( L ) = 2dotNL / ( dotNL + sqrt( a2 + ( 1 - a2 ) * dotNL2 ) )
// The term ( 4 * dotNV * dotNL ) in BRDF cancels out
float V_GGX_SmithCorrelated( const in float alpha, const in float dotNL, const in float dotNV )
{
  float a2 = pow2( alpha );
  float gv = dotNL * sqrt( a2 + ( 1.0 - a2 ) * pow2( dotNV ) );
  float gl = dotNV * sqrt( a2 + ( 1.0 - a2 ) * pow2( dotNL ) );
  return 0.5 / max( gv + gl, 1e-6 );
}

// Normal distribution function
float D_GGX( const in float alpha, const in float dotNH )
{
  float a2 = pow2( alpha );
  float denom = pow2( dotNH ) * ( a2 - 1.0 ) + 1.0;
  return 0.3183098861837907 * a2 / pow2( denom );
}

#ifdef USE_KHR_materials_anisotropy
// Anisotropic GGX normal distribution function.
// https://github.com/KhronosGroup/glTF/blob/main/extensions/2.0/Khronos/KHR_materials_anisotropy/README.md
float D_GGX_anisotropic( const in float dotNH, const in float dotTH, const in float dotBH, const in float at, const in float ab )
{
  float a2 = at * ab;
  vec3 f = vec3( ab * dotTH, at * dotBH, a2 * dotNH );
  float w2 = a2 / dot( f, f );
  return a2 * w2 * w2 * RECIPROCAL_PI;
}

// Anisotropic visibility (masking-shadowing) function.
float V_GGX_anisotropic
(
  const in float dotNL, const in float dotNV,
  const in float dotBV, const in float dotTV,
  const in float dotTL, const in float dotBL,
  const in float at, const in float ab
)
{
  float GGXV = dotNL * length( vec3( at * dotTV, ab * dotBV, dotNV ) );
  float GGXL = dotNV * length( vec3( at * dotTL, ab * dotBL, dotNL ) );
  return clamp( 0.5 / max( GGXV + GGXL, 1e-6 ), 0.0, 1.0 );
}
#endif

#ifdef USE_KHR_materials_clearcoat
// The clearcoat layer is modeled as a fixed-IOR (1.5) dielectric coat, using the same
// isotropic GGX D/V terms as the base layer but with its own normal and roughness.
// This is the extension's `clearcoat_brdf`: the microfacet lobe WITHOUT Fresnel. The coat
// Fresnel is applied exactly once, by the fresnel_mix in main(), to direct and image-based
// coat light alike; weighting it here as well would square it (~0.04^2 head-on).
// https://github.com/KhronosGroup/glTF/blob/main/extensions/2.0/Khronos/KHR_materials_clearcoat/README.md
float BRDF_Clearcoat( const in float dotNL, const in float dotNV, const in float dotNH, const in float roughness )
{
  float alpha = pow2( roughness );
  float D = D_GGX( alpha, dotNH );
  float V = V_GGX_SmithCorrelated( alpha, dotNL, dotNV );
  return D * V * dotNL;
}
#endif

void applyLightContribution
(
  const in vec3 lightDir,
  const in vec3 viewDir,
  const in vec3 normal,
  const in PhysicalMaterial material,
  const in vec3 lightColor,
  const in float lightIntensity,
  inout ReflectedLight reflectedLight
)
{
  float alpha = pow2( material.roughness );
  vec3 halfDir = normalize( lightDir + viewDir );

  float dotNL = clamp( dot( normal, lightDir ), 0.0, 1.0 );
  float dotNV = clamp( dot( normal, viewDir ), 0.0, 1.0 );
  float dotNH = clamp( dot( normal, halfDir ), 0.0, 1.0 );
  float dotVH = clamp( dot( viewDir, halfDir ), 0.0, 1.0 );
  float dotLH = clamp( dot( lightDir, halfDir ), 0.0, 1.0 );

  // Fresnel
  vec3 Fs = F_Schlick( material.f0, material.f90, dotVH );
  // Diffuse BRDF (Burley)
  vec3 Fd = Fd_Barley( alpha, dotNV, dotNL, dotLH );
  // Visibility Geometry function and Normal distribution function
  #ifdef USE_KHR_materials_anisotropy
    float dotTL = dot( material.anisotropicT, lightDir );
    float dotBL = dot( material.anisotropicB, lightDir );
    float dotTV = dot( material.anisotropicT, viewDir );
    float dotBV = dot( material.anisotropicB, viewDir );
    float dotTH = dot( material.anisotropicT, halfDir );
    float dotBH = dot( material.anisotropicB, halfDir );
    float V = V_GGX_anisotropic( dotNL, dotNV, dotBV, dotTV, dotTL, dotBL, material.at, material.ab );
    float D = D_GGX_anisotropic( dotNH, dotTH, dotBH, material.at, material.ab );
  #else
    float V = V_GGX_SmithCorrelated( alpha, dotNL, dotNV );
    float D = D_GGX( alpha, dotNH );
  #endif

  vec3 irradiance = lightColor * lightIntensity * dotNL;
  vec3 diffuseColor = material.diffuseColor * irradiance;
  vec3 specularColor = D * V * irradiance;

  reflectedLight.directDiffuse += ( 1.0 - Fs ) * Fd * diffuseColor;
  reflectedLight.directSpecular += Fs * specularColor;

  #ifdef USE_KHR_materials_clearcoat
    float ccDotNL = clamp( dot( material.clearcoatNormal, lightDir ), 0.0, 1.0 );
    float ccDotNV = clamp( dot( material.clearcoatNormal, viewDir ), 0.0, 1.0 );
    float ccDotNH = clamp( dot( material.clearcoatNormal, halfDir ), 0.0, 1.0 );
    reflectedLight.clearcoatSpecular += BRDF_Clearcoat( ccDotNL, ccDotNV, ccDotNH, material.clearcoatRoughness ) * lightColor * lightIntensity;
  #endif
}

void computeDirectLight
(
  DirectLight light,
  const in vec3 viewDir,
  const in vec3 normal,
  const in PhysicalMaterial material,
  inout ReflectedLight reflectedLight
)
{
  applyLightContribution( light.direction, viewDir, normal, material, light.color, light.strength, reflectedLight );
}

void computePointLight
(
  PointLight light,
  const in vec3 viewDir,
  const in vec3 normal,
  const in PhysicalMaterial material,
  inout ReflectedLight reflectedLight
)
{
  vec3 lightDir = light.position - vWorldPos;
  float distance_ = length( lightDir );
  lightDir = normalize( lightDir );

  // float attenuation = 1.0 / ( 1.0 + pow( distance_ / light.range, 4.0 ) );
  float attenuation = pow( clamp( 1.0 - distance_ / light.range, 0.0, 1.0 ), 2.0 ) / ( distance_ * distance_ + 1.0 );
  attenuation *= light.strength;

  applyLightContribution( lightDir, viewDir, normal, material, light.color, attenuation, reflectedLight );
}

void computeSpotLight
(
  SpotLight light,
  const in vec3 viewDir,
  const in vec3 normal,
  const in PhysicalMaterial material,
  inout ReflectedLight reflectedLight
)
{
  vec3 lightDir = light.position - vWorldPos;
  float distance_ = length( lightDir );
  lightDir = normalize( lightDir );

  // Distance attenuation
  float attenuation = pow( clamp( 1.0 - distance_ / light.range, 0.0, 1.0 ), 2.0 ) / ( distance_ * distance_ + 1.0 );

  // Angular attenuation (spotlight cone)
  float angle = acos( dot( -lightDir, light.direction ) ); // light.direction assumed to be normalized on cpu side
  float innerAngle = light.innerConeAngle;
  float outerAngle = light.outerConeAngle;
  float angularAttenuation = smoothstep( outerAngle, innerAngle, angle );

  attenuation *= angularAttenuation * light.strength;

  // Apply lightmap if enabled
  #ifdef USE_LIGHT_MAP
    if( light.useLightMap )
    {
      float shadowFactor = 1.0 - texture( lightMap, vLightMapUv ).r;
      attenuation *= shadowFactor;
      // specularColor *= shadowFactor;
    }
  #endif

  applyLightContribution( lightDir, viewDir, normal, material, light.color, attenuation, reflectedLight );
}

// Whether light from `lightDir` reaches a layer: the base through `normal` or, with a clearcoat,
// the coat through its own normal. Only an early-out, since every BRDF term clamps its own N.L,
// but it must not skip the coat where a base normal map turns the base away from the light.
bool lightFacing( const in vec3 lightDir, const in vec3 normal, const in PhysicalMaterial material )
{
  float dotNL = dot( normal, lightDir );
  #ifdef USE_KHR_materials_clearcoat
    dotNL = max( dotNL, dot( material.clearcoatNormal, lightDir ) );
  #endif
  return dotNL > 0.0;
}

void computeLights
(
  const in vec3 viewDir,
  const in vec3 normal,
  const in PhysicalMaterial material,
  inout ReflectedLight reflectedLight
)
{
  for( int i = 0; i < min( pointLightsCount, MAX_POINT_LIGHTS ); i++ )
  {
    vec3 lightDir = pointLights[ i ].position - vWorldPos;

    if ( lightFacing( lightDir, normal, material ) )
    {
      computePointLight( pointLights[ i ], viewDir, normal, material, reflectedLight );
    }
  }

  for( int i = 0; i < min( directLightsCount, MAX_DIRECT_LIGHTS ); i++ )
  {
    if ( lightFacing( directLights[ i ].direction, normal, material ) )
    {
      computeDirectLight( directLights[ i ], viewDir, normal, material, reflectedLight );
    }
  }

  for( int i = 0; i < min( spotLightsCount, MAX_SPOT_LIGHTS ); i++ )
  {
    vec3 lightDir = spotLights[ i ].position - vWorldPos;

    if ( lightFacing( lightDir, normal, material ) )
    {
      computeSpotLight( spotLights[ i ], viewDir, normal, material, reflectedLight );
    }
  }
}

// Screen-space dither noise (Interleaved Gradient Noise, Jimenez 2014).
// Returns a value in [0, 1) that is well-distributed across pixels and
// has low visible pattern — ideal for breaking up color banding from
// limited-precision HDR textures (RGBE / RGB16F).
float ditherNoise( vec2 fragCoord )
{
  return fract( 52.9829189 * fract( 0.06711056 * fragCoord.x + 0.00583715 * fragCoord.y ) );
}

#ifdef USE_IBL

  // Mip of prefilterEnvMap to sample along reflection `R` for `roughness`. No fixed floor (e.g.
  // `max( .., 1.0 )`) is applied: with the PMREM prefilter, mip 0 of prefilterEnvMap is the sharp
  // environment, which is the correct, physically expected result for a mirror-smooth surface.
  // Specular aliasing comes from the reflection vector being under-sampled across a pixel
  // (curved geometry, silhouettes, grazing angles) — exactly the case where the screen-space
  // variance term below is large and raises the LOD. On flat smooth surfaces reflVariance ~ 0,
  // there is no sub-pixel variation to alias, so sampling mip 0 there is safe; a fixed floor
  // would only blur legitimate mirror reflections.
  float envLod( const in vec3 R, const in float roughness )
  {
    float lod = roughness * u_max_lod;

    // GSAA-style specular antialiasing: widen the filter where the reflected direction changes
    // rapidly in screen space.
    vec3 dRdx = dFdx( R );
    vec3 dRdy = dFdy( R );
    float reflVariance = dot( dRdx, dRdx ) + dot( dRdy, dRdy );
    lod = max( lod, 0.5 * log2( max( reflVariance, 1e-6 ) ) + 4.0 );
    return min( lod, u_max_lod );
  }

  #ifdef USE_KHR_materials_anisotropy
    // Anisotropic IBL: the normal to reflect about, bent towards the anisotropic tangent frame
    // (bent-normal approximation from the glTF-Sample-Renderer reference implementation).
    // The LOD / envBRDF / multi-scatter terms stay driven by the original roughness — only the
    // sampled direction changes.
    vec3 anisotropicBentNormal( const in vec3 N, const in vec3 V, const in PhysicalMaterial material )
    {
      vec3 anisotropicTangent = cross( material.anisotropicB, V );
      vec3 anisotropicNormal = cross( anisotropicTangent, material.anisotropicB );
      float bendFactor = 1.0 - material.anisotropyStrength * ( 1.0 - material.roughness );
      return normalize( mix( anisotropicNormal, N, pow4( bendFactor ) ) );
    }
  #endif

  void sampleEnvIrradiance( const in vec3 N, const in vec3 V, const in PhysicalMaterial material, inout ReflectedLight reflectedLight )
  {
    float dotNV = clamp( dot( N, V ), 0.01, 1.0 );

    vec3 R = reflect( -V, N );
    #ifdef USE_KHR_materials_anisotropy
      R = reflect( -V, anisotropicBentNormal( N, V, material ) );
    #endif

    float lod = envLod( R, material.roughness );

    float dither = ( ditherNoise( gl_FragCoord.xy ) - 0.5 ) / 512.0;

    vec3 irradiance = texture( irradianceTexture, N ).xyz + dither;
    vec3 radiance = textureLod( prefilterEnvMap, R, lod ).xyz + dither;

    vec2 envBrdf = texture( integrateBRDF, vec2( dotNV, material.roughness ) ).xy;

    // Split-sum with multiple-scattering energy compensation, matching three.js
    // computeMultiscattering(). The single-scatter term is the usual prefiltered
    // reflection; the multi-scatter term feeds the energy lost between microfacet
    // bounces back as a soft, irradiance-weighted lobe — without it rough metals /
    // plastics read as pure mirrors and the overall specular is too dim.
    vec3 FssEss = material.f0 * envBrdf.x + material.f90 * envBrdf.y;
    float Ess = envBrdf.x + envBrdf.y;
    float Ems = 1.0 - Ess;
    vec3 Favg = material.f0 + ( 1.0 - material.f0 ) * 0.047619; // 1.0 / 21.0
    vec3 Fms = FssEss * Favg / ( 1.0 - Ems * Favg );

    vec3 singleScatter = FssEss;
    vec3 multiScatter = Fms * Ems;
    vec3 totalScatter = singleScatter + multiScatter;
    vec3 diffuse = material.diffuseColor * ( 1.0 - max_value( totalScatter ) );

    reflectedLight.indirectSpecular += radiance * singleScatter;
    reflectedLight.indirectSpecular += multiScatter * irradiance;
    reflectedLight.indirectDiffuse += diffuse * irradiance;

    // Clearcoat IBL: raw prefiltered radiance sampled along the clearcoat normal, with no
    // split-sum Fresnel weighting here — the coat's Fresnel is applied once, at the final
    // mix with the base result (see KHR_materials_clearcoat's fresnel_mix in main()).
    #ifdef USE_KHR_materials_clearcoat
      vec3 Rc = reflect( -V, material.clearcoatNormal );
      float lodc = envLod( Rc, material.clearcoatRoughness );
      reflectedLight.clearcoatIndirectSpecular += textureLod( prefilterEnvMap, Rc, lodc ).xyz;
    #endif
  }

#endif

float alpha_weight( float a )
{
  return clamp( pow( min( 1.0, a * 10.0 ) + 0.01, 3.0 ) * 1e8 * pow( 1.0 - gl_FragCoord.z * 0.9, 3.0 ), 1e-2, 3e3 );
}

#ifndef USE_TANGENTS
  // The per-pixel counterpart of the MikkTSpace frame glTF specifies for meshes without
  // tangents. T is the surface direction in which u increases, made orthogonal to the normal.
  // B is perpendicular to both, on the side up the image: glTF's tangent space has +Y up, the
  // UV origin is the image's upper-left corner, and images are uploaded unflipped, so up is the
  // direction in which v decreases. Mirrored UVs are followed. The frame is orthonormal, so an
  // anisotropy direction keeps its angle where u and v have different texel density or are
  // sheared, which a frame built from the UV gradients does not.
  mat3 getTBN( vec3 surf_normal, vec3 pos, vec2 uv )
  {
    vec3 dE1 = dFdx( pos );
    vec3 dE2 = dFdy( pos );
    vec2 dUv1 = dFdx( uv );
    vec2 dUv2 = dFdy( uv );

    // Surface directions in which u and v increase: the inverse of the UV Jacobian, scaled by
    // its determinant, whose sign restores their orientation.
    float det = dUv1.x * dUv2.y - dUv2.x * dUv1.y;
    vec3 dPdu = ( dUv2.y * dE1 - dUv1.y * dE2 ) * sign( det );
    vec3 dPdv = ( dUv1.x * dE2 - dUv2.x * dE1 ) * sign( det );

    vec3 T = dPdu - surf_normal * dot( surf_normal, dPdu );
    // No usable UV gradient ( constant UVs, e.g. a mesh without TEXCOORD_0, or UVs collapsed onto
    // a line ): there is no tangent direction, and normalizing the zero vector gives NaN, which
    // would reach the anisotropic lobe and the bent environment normal even at strength 0. Any
    // tangent around the normal keeps the frame finite.
    if ( dot( T, T ) < 1e-30 )
    {
      vec3 axis = abs( surf_normal.x ) < 0.9 ? vec3( 1.0, 0.0, 0.0 ) : vec3( 0.0, 1.0, 0.0 );
      T = axis - surf_normal * dot( surf_normal, axis );
    }
    T = normalize( T );
    vec3 B = cross( surf_normal, T );
    B *= dot( B, dPdv ) > 0.0 ? -1.0 : 1.0;
    // On a back face the whole frame flips, as in the vertex-tangent branch of tangentFrame.
    // surf_normal is already negated there, but T and B come out the same for N and -N ( the
    // projection ignores N's sign and B is oriented by dPdv ), so they are negated here: a
    // normal-map sample then resolves to the reversed front-face normal, as glTF and the Khronos
    // sample renderer give, instead of keeping its x and y and lighting the relief from the
    // wrong side.
    float faceDirection = gl_FrontFacing ? 1.0 : -1.0;
    return mat3( T * faceDirection, B * faceDirection, surf_normal );
  }
#endif

#ifdef USE_TBN
  // The tangent frame shared by every tangent-space texture (base normal, clearcoat normal,
  // anisotropy direction), around `geometricNormal`, which already faces the viewer.
  mat3 tangentFrame( const in vec3 geometricNormal, const in float faceDirection )
  {
    #ifdef USE_TANGENTS
      // On a back face the whole frame flips, not just the normal: geometricNormal is already
      // negated, the bitangent derived from it follows, and the tangent must be negated too.
      // Flipping only N and B would mirror tangent-space X relative to master and to the Khronos
      // sample renderer (which negates t, b and ng together when !gl_FrontFacing).
      vec3 tangent = vTangent.xyz * faceDirection;
      vec3 bitangent = cross( geometricNormal, vTangent.xyz ) * vTangent.w;
      return mat3( tangent, bitangent, geometricNormal );
    #else
      // Without vertex tangents the frame is reconstructed from the screen-space derivatives of
      // a UV set. glTF derives a mesh's tangent frame from the texcoords of its normal texture,
      // so use the base normal texture's UV set. Only when the material has no base normal
      // texture does the next tangent-space texture's UV set stand in, and UV set 0 only when
      // there is none.
      #if defined( USE_NORMAL_TEXTURE )
        return getTBN( geometricNormal, vWorldPos, vNormalUv );
      #elif defined( USE_CLEARCOAT_NORMAL_TEXTURE )
        return getTBN( geometricNormal, vWorldPos, vClearcoatNormalUv );
      #elif defined( USE_ANISOTROPY_TEXTURE )
        return getTBN( geometricNormal, vWorldPos, vAnisotropyUv );
      #else
        return getTBN( geometricNormal, vWorldPos, vUv_0 );
      #endif
    #endif
  }
#endif

float adjustRoughnessNormalMap ( const in float roughness, const in vec3 normal )
{
  float nlen2 = dot (normal, normal );
  if( nlen2 < 1.0 )
  {
    float nlen = sqrt( nlen2 );
    float kappa = (3.0 * nlen -  nlen2 * nlen) / (1.0 - nlen2);
    return min(1.0, sqrt(roughness * roughness + 1.0 / kappa));
  }
  return roughness;
}

// Geometric Specular Anti-Aliasing (Tokuyoshi & Kaplanyan 2019): widens `roughness` where the
// screen-space derivatives of the shading normal `n` are large (geometry edges), which selects
// blurrier environment map mip levels and prevents specular aliasing. The 0.0525 floor keeps
// D_GGX finite: at roughness 0 it is zero off the exact mirror direction and 0/0 on it, so a
// smooth surface would show no highlight or a single flickering pixel.
float gsaaRoughness( const in vec3 n, const in float roughness )
{
  vec3 dNdx = dFdx( n );
  vec3 dNdy = dFdy( n );
  float variance = dot( dNdx, dNdx ) + dot( dNdy, dNdy );
  return max( sqrt( clamp( pow2( roughness ) + 0.5 * variance, 0.0, 1.0 ) ), 0.0525 );
}

// Specular occlusion for ambient occlusion `ao`, from the view angle and roughness of the lobe
// it darkens (Lagarde & de Rousiers 2014).
float specularOcclusion( const in float dotNV, const in float ao, const in float roughness )
{
  return clamp( pow( dotNV + ao, exp2( -16.0 * roughness - 1.0 ) ) - 1.0 + ao, 0.0, 1.0 );
}

#ifdef USE_KHR_materials_clearcoat
  // KHR_materials_clearcoat: an additional dielectric (IOR 1.5) coat layer with its own factor,
  // roughness and normal. The normal starts from the *unperturbed* geometric normal, not the
  // base normal map result; a coat normal map is read in the shared tangent frame `TBN`. Called
  // after the base layer's GSAA, which it does not read.
  // See https://github.com/KhronosGroup/glTF/blob/main/extensions/2.0/Khronos/KHR_materials_clearcoat/README.md
  void clearcoatSetup( inout PhysicalMaterial material, const in mat3 TBN, const in vec3 geometricNormal )
  {
    material.clearcoatFactor = clearcoatFactor;
    #ifdef USE_CLEARCOAT_TEXTURE
      material.clearcoatFactor *= texture( clearcoatTexture, vClearcoatUv ).r;
    #endif
    material.clearcoatRoughness = clearcoatRoughnessFactor;
    #ifdef USE_CLEARCOAT_ROUGHNESS_TEXTURE
      material.clearcoatRoughness *= texture( clearcoatRoughnessTexture, vClearcoatRoughnessUv ).g;
    #endif
    material.clearcoatRoughness = clamp( material.clearcoatRoughness, 0.0, 1.0 );
    material.clearcoatNormal = geometricNormal;
    #ifdef USE_CLEARCOAT_NORMAL_TEXTURE
      vec3 ccNormalSample = texture( clearcoatNormalTexture, vClearcoatNormalUv ).xyz * 2.0 - 1.0;
      ccNormalSample.xy *= vec2( clearcoatNormalScale );
      material.clearcoatNormal = normalize( TBN * ccNormalSample );
    #endif

    // The coat gets the same safeguards as the base layer, measured on the coat's own normal; the
    // floor matters most here, since the glTF default clearcoatRoughness is 0.
    material.clearcoatRoughness = gsaaRoughness( material.clearcoatNormal, material.clearcoatRoughness );
  }
#endif

#ifdef USE_KHR_materials_clearcoat
  // KHR_materials_clearcoat: fresnel_mix the base result `color` with the coat lobe. The coat's
  // own Fresnel term is applied once here (not per light / per IBL term), and the emissive
  // output is dampened by the same weight, matching the extension's "coated_emission" note.
  void clearcoatMix
  (
    const in PhysicalMaterial material,
    const in vec3 viewDir,
    const in ReflectedLight reflectedLight,
    inout vec3 color,
    inout vec4 emissive
  )
  {
    vec3 clearcoatFresnel = F_Schlick( vec3( 0.04 ), vec3( 1.0 ), clamp( dot( material.clearcoatNormal, viewDir ), 0.0, 1.0 ) );
    vec3 clearcoatWeight = clamp( material.clearcoatFactor * clearcoatFresnel, 0.0, 1.0 );
    vec3 clearcoatColor = reflectedLight.clearcoatSpecular + reflectedLight.clearcoatIndirectSpecular;
    color = mix( color, clearcoatColor, clearcoatWeight );
    emissive.rgb *= ( 1.0 - clearcoatWeight );
  }
#endif

#ifdef USE_KHR_materials_anisotropy
  // KHR_materials_anisotropy: the anisotropy direction in the shared tangent frame `TBN`, its
  // strength, and the roughness split ( at / ab ) of the GSAA-adjusted base roughness, so this
  // runs after the base layer's GSAA.
  // See https://github.com/KhronosGroup/glTF/blob/main/extensions/2.0/Khronos/KHR_materials_anisotropy/README.md
  void anisotropySetup( inout PhysicalMaterial material, const in mat3 TBN, const in vec3 geometricNormal )
  {
    vec2 anisotropyDirection = vec2( 1.0, 0.0 );
    float anisotropyMagnitude = anisotropyStrength;
    #ifdef USE_ANISOTROPY_TEXTURE
      vec3 anisotropySample = texture( anisotropyTexture, vAnisotropyUv ).rgb;
      anisotropyDirection = anisotropySample.rg * 2.0 - 1.0;
      anisotropyMagnitude *= anisotropySample.b;
    #endif
    float anisoRotCos = cos( anisotropyRotation );
    float anisoRotSin = sin( anisotropyRotation );
    anisotropyDirection = mat2( anisoRotCos, anisoRotSin, -anisoRotSin, anisoRotCos ) * normalize( anisotropyDirection );

    material.anisotropicT = normalize( TBN * vec3( anisotropyDirection, 0.0 ) );
    material.anisotropicB = cross( geometricNormal, material.anisotropicT );
    // The extension defines the strength on [ 0, 1 ]; clamping here covers loaded assets and
    // `anisotropy_strength_set` callers alike, as the coat's inputs are clamped.
    material.anisotropyStrength = clamp( anisotropyMagnitude, 0.0, 1.0 );

    float anisotropyBaseAlpha = pow2( material.roughness );
    material.at = mix( anisotropyBaseAlpha, 1.0, pow2( material.anisotropyStrength ) );
    material.ab = clamp( anisotropyBaseAlpha, 0.001, 1.0 );
  }
#endif

void main()
{
  PhysicalMaterial material;
  ReflectedLight reflectedLight;
  reflectedLight.indirectDiffuse = vec3( 0.0 );
  reflectedLight.indirectSpecular = vec3( 0.0 );
  reflectedLight.directDiffuse = vec3( 0.0 );
  reflectedLight.directSpecular = vec3( 0.0 );
  #ifdef USE_KHR_materials_clearcoat
    reflectedLight.clearcoatSpecular = vec3( 0.0 );
    reflectedLight.clearcoatIndirectSpecular = vec3( 0.0 );
  #endif

  float alpha = 1.0;

  material.metallness = metallicFactor;
  material.roughness = roughnessFactor;
  material.diffuseColor = baseColorFactor.rgb;
  alpha *= baseColorFactor.a;
  #ifdef USE_BASE_COLOR_TEXTURE
    vec4 baseColor = texture( baseColorTexture, vBaseColorUv );
    baseColor.rgb = SrgbToLinear( baseColor.rgb );
    material.diffuseColor *= baseColor.rgb;
    alpha *= baseColor.a;
  #endif

  #ifdef USE_MR_TEXTURE
    vec4 mr_sample = texture( metallicRoughnessTexture, vMRUv );
    material.metallness *= mr_sample.b;
    material.roughness *= mr_sample.g;
  #endif

  #ifdef USE_ALPHA_CUTOFF
    if( alpha < alphaCutoff )
    {
      discard;
    }
    alpha = 1.0;
  #endif

  //Specular part
  // https://github.com/KhronosGroup/glTF/blob/main/extensions/2.0/Khronos/KHR_materials_specular/README.md
  // 0.04 - reflectance of the Glass
  material.f0 = vec3( 0.04 );
  material.f90 = vec3( 1.0 );
  #ifdef USE_KHR_materials_specular
    float sf = specularFactor;
    material.f0 *= specularColorFactor;
    #ifdef USE_SPECULAR_COLOR_TEXTURE
      material.f0 *= SrgbToLinear( texture( specularColorTexture, vSpecularColorUv ).rgb );
    #endif
    #ifdef USE_SPECULAR_TEXTURE
      sf *= texture( specularTexture, vSpecularUv ).a;
    #endif
    material.f0 = min( material.f0 * sf, vec3( 1.0 ) );
  #endif
  material.f0 = mix( material.f0, material.diffuseColor, material.metallness );
  material.diffuseColor *= 1.0 - material.metallness;

  // faceDirection is applied to the geometric normal up front, before the tangent frame,
  // normal maps, clearcoat and anisotropy consume it. tangentFrame flips its tangent and
  // bitangent on back faces as well ( both branches ), so a tangent-space sample resolves to the
  // reversed front-face normal on double-sided back faces.
  float faceDirection = gl_FrontFacing ? 1.0 : -1.0;
  vec3 geometricNormal = normalize( vNormal ) * faceDirection;

  // The shared tangent frame, built only under USE_TBN, where a tangent-space texture or
  // anisotropy reads it.
  mat3 TBN = mat3( 1.0 );
  #ifdef USE_TBN
    TBN = tangentFrame( geometricNormal, faceDirection );
  #endif

  vec3 normal = geometricNormal;
  #ifdef USE_NORMAL_TEXTURE
    vec3 normalSample = texture( normalTexture, vNormalUv ).xyz * 2.0 - 1.0;
    //material.roughness = adjustRoughnessNormalMap( material.roughness, normalSample );
    normalSample.xy *= vec2( normalScale );
    normal = normalize( TBN * normalSample );
  #endif

  material.roughness = gsaaRoughness( normal, material.roughness );
  #ifdef USE_KHR_materials_clearcoat
    clearcoatSetup( material, TBN, geometricNormal );
  #endif
  #ifdef USE_KHR_materials_anisotropy
    anisotropySetup( material, TBN, geometricNormal );
  #endif

  vec3 color = vec3( 0.0 );
  vec3 viewDir = normalize( cameraPosition - vWorldPos );

  computeLights( viewDir, normal, material, reflectedLight );

  #if defined( USE_IBL )
    sampleEnvIrradiance( normal, viewDir, material, reflectedLight );
  #else
    reflectedLight.indirectDiffuse += 0.1 * material.diffuseColor;
  #endif

  #ifdef USE_OCCLUSION_TEXTURE
    float occlusion = texture( occlusionTexture, vOcclusionUv ).r;
    float ao = 1.0 + occlusionStrength * ( occlusion - 1.0 );
    reflectedLight.indirectDiffuse *= ao;
    float dotNV = clamp( dot( normal, viewDir ), 0.0, 1.0 );
    reflectedLight.indirectSpecular *= specularOcclusion( dotNV, ao, material.roughness );
    #ifdef USE_KHR_materials_clearcoat
      // The coat's environment reflection is occluded like the base one, but with the coat's
      // own normal and roughness.
      float ccDotNV = clamp( dot( material.clearcoatNormal, viewDir ), 0.0, 1.0 );
      reflectedLight.clearcoatIndirectSpecular *= specularOcclusion( ccDotNV, ao, material.clearcoatRoughness );
    #endif
  #endif

  emissive_color = vec4( emissiveFactor, 1.0 );
  #ifdef USE_EMISSION_TEXTURE
    emissive_color.xyz *= SrgbToLinear( texture( emissiveTexture, vEmissionUv ).rgb );
  #endif


  color = reflectedLight.indirectDiffuse +
  reflectedLight.indirectSpecular +
  reflectedLight.directDiffuse +
  reflectedLight.directSpecular;

  #ifdef USE_KHR_materials_clearcoat
    clearcoatMix( material, viewDir, reflectedLight, color, emissive_color );
  #endif

  // Exposure is applied uniformly to the whole lit result here ( the tone mapping
  // pass operates in display-referred space ). The clear-color background is not
  // drawn by this shader, so it stays exposure-independent.
  color *= exp2( exposure );

  float a_weight = alpha * alpha_weight( alpha );
  trasnparentA = vec4( color * a_weight, alpha );
  transparentB = a_weight;
  // Opaque pass writes alpha = 1 to mark covered pixels; background stays at the
  // cleared alpha = 0 so the tone mapping pass can leave it untouched. The transparent
  // pass renders to locations 2/3 only, so this alpha never reaches the WBOIT buffers.
  frag_color = vec4( color, 1.0 );
}

