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
in vec3 vViewPos;
in vec3 vNormal;

layout( location = 0 ) out vec4 frag_color;
layout( location = 1 ) out vec4 emissive_color;

uniform vec3 cameraPosition;

struct PhysicalMaterial
{
  vec3 diffuseColor;
  float metallness;
  float roughness;
  vec3 f0;
  vec3 f90;
  float occlusionFactor;
  float specularFactor;
};

#ifdef USE_PBR
  uniform float metallicFactor; // Default: 1
  uniform float roughnessFactor; // Default: 1
  uniform vec4 baseColorFactor; // Default: [1, 1, 1, 1]
  #ifdef USE_IBL
    uniform samplerCube irradianceTexture;
    uniform samplerCube prefilterEnvMap;
    uniform sampler2D integrateBRDF;
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
#endif

#ifdef USE_NORMAL_TEXTURE
  // vNormalUv
  uniform sampler2D normalTexture;
  // Scales the normal in X and Y directions
  // ( <sample normalTexture> * 2.0 - 1.0 ) * vec3( normalScale, normalScale, 1.0 )
  uniform float normalScale; // Default: 1
#endif

#ifdef USE_OCCLUSION_TEXTURE
  // vOcclusionUv
  uniform sampler2D occlusionTexture;
  // 1.0 + occlusionStrength * ( <sample occlusionTexture> - 1.0 )
  uniform float occlusionStrength; // Default: 1
#endif


#ifdef USE_EMISSION
  // vEmissionUv
  #ifdef USE_EMISSION_TEXTURE
    uniform sampler2D emissiveTexture;
  #endif
  uniform vec3 emissiveFactor;
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

float luminance( const in vec3 rgb ) 
{
  const vec3 weights = vec3( 0.2126, 0.7152, 0.0722 );
  return dot( weights, rgb );
}

// Schilck's version of Fresnel equation, with Spherical Gaussian approximation for the power
// https://blog.selfshadow.com/publications/s2013-shading-course/karis/s2013_pbs_epic_notes_v2.pdf
vec3 F_Schlick( const in vec3 f0, const in vec3 f90, const in float dotVH ) 
{
  float fresnel = exp2( ( - 5.55473 * dotVH - 6.98316 ) * dotVH );
  return f0 + ( f90 - f0 ) * fresnel;
}

// https://web.archive.org/web/20160702002225/http://www.frostbite.com/wp-content/uploads/2014/11/course_notes_moving_frostbite_to_pbr_v2.pdf
// https://inria.hal.science/hal-00942452v1/document
float V_GGX_SmithCorrelated( const in float alpha, const in float dotNL, const in float dotNV ) 
{
  float a2 = pow2( alpha );
  float gv = dotNL * sqrt( a2 + ( 1.0 - a2 ) * pow2( dotNV ) );
  float gl = dotNV * sqrt( a2 + ( 1.0 - a2 ) * pow2( dotNL ) );
  return 0.5 / max( gv + gl, 1e-6 );
}

float D_GGX( const in float alpha, const in float dotNH ) 
{
  float a2 = pow2( alpha );
  float denom = pow2( dotNH ) * ( a2 - 1.0 ) + 1.0;
  return 0.3183098861837907 * a2 / pow2( denom );
}

vec4 BRDF_GGX( const in vec3 lightDir, const in vec3 viewDir, const in vec3 normal, const in PhysicalMaterial material ) 
{
  float alpha = pow2( material.roughness );
  vec3 halfDir = normalize( lightDir + viewDir );

  float dotNL = clamp( dot( normal, lightDir ), 0.0, 1.0 );
  float dotNV = clamp( dot( normal, viewDir ), 0.0, 1.0 );
  float dotNH = clamp( dot( normal, halfDir ), 0.0, 1.0 );
  float dotVH = clamp( dot( viewDir, halfDir ), 0.0, 1.0 );

  // Fresnel
  vec3 F = F_Schlick( material.f0, material.f90, dotVH );
  // Geometry function
  float G = V_GGX_SmithCorrelated( alpha, dotNL, dotNV );
  // Normal distribution function
  float D = D_GGX( alpha, dotNH );
  return vec4( F, G * D ) ;
}

#define EXPOSURE 1.0

#ifdef USE_IBL
  vec3 sampleEnvIrradiance( const in vec3 N, const in vec3 V, const in PhysicalMaterial material )
  {
    vec3 color = vec3( 0.0 );
    float dotNV = clamp( dot( N, V ), 0.0, 1.0 );

    const float MAX_LOD = 9.0;
    if( dotNV > 0.0 )
    {
      vec3 Fs = F_Schlick( material.f0, material.f90, dotNV );
      float Fd = 1.0 - max_value( Fs );
      Fd *= 1.0 - material.metallness;
      vec3 R = reflect( -V, N );

      vec3 diffuse = texture( irradianceTexture, N ).xyz * pow( 2.0, EXPOSURE );
      vec3 prefilter = texture( prefilterEnvMap, R, material.roughness * MAX_LOD ).xyz * pow( 2.0, EXPOSURE );
      vec2 envBrdf = texture( integrateBRDF, vec2( dotNV, material.roughness ) ).xy;

      vec3 diffuseBRDF = diffuse * material.diffuseColor;
      vec3 specularBRDF = prefilter * ( material.f0 * envBrdf.x + envBrdf.y );
      color = Fd * diffuseBRDF + specularBRDF;
    }

    return color;
  }
#endif

#ifndef USE_TANGENTS
  // http://www.thetenthplanet.de/archives/1180
  mat3 getTBN( vec3 surf_normal, vec3 pos, vec2 uv )
  {
    vec3 dE1 = dFdx( pos );
    vec3 dE2 = dFdy( pos );
    vec2 dUv1 = dFdx( uv );
    vec2 dUv2 = dFdy( uv );

    vec3 q1perp = cross( dE2, surf_normal );
		vec3 q0perp = cross( surf_normal, dE1 );
    
    vec3 T = q1perp * dUv1.x + q0perp * dUv2.x;
		vec3 B = q1perp * dUv1.y + q0perp * dUv2.y;

    float det = max( dot( T, T ), dot( B, B ) );
		float scale = ( det == 0.0 ) ? 0.0 : inversesqrt( det );

		return mat3( T * scale, B * scale, surf_normal );
  }
#endif

#ifdef RENDER_TO_SCREEN
  vec3 aces_tone_map( vec3 hdr )
  {
    mat3x3 m1 = mat3x3
    (
      0.59719, 0.07600, 0.02840,
      0.35458, 0.90834, 0.13383,
      0.04823, 0.01566, 0.83777
    );
    mat3x3 m2 = mat3x3
    (
      1.60475, -0.10208, -0.00327,
      -0.53108,  1.10813, -0.07276,
      -0.07367, -0.00605,  1.07602
    );

    vec3 v = m1 * hdr;
    vec3 a = v * ( v + 0.0245786 ) - 0.000090537;
    vec3 b = v * ( 0.983729 * v + 0.4329510 ) + 0.238081;

    return clamp( m2 * ( a / b ), vec3( 0.0 ), vec3( 1.0 ) );
  }
#endif

void main()
{
  PhysicalMaterial material;

  float alpha = 1.0;
  #ifdef USE_PBR
    material.metallness = metallicFactor;
    material.roughness = roughnessFactor;
    #ifdef USE_BASE_COLOR_TEXTURE
      vec4 baseColor = baseColorFactor * SrgbToLinear( texture( baseColorTexture, vBaseColorUv ) );
      material.diffuseColor =  baseColor.rgb;
      alpha = baseColor.a;
    #else
      material.diffuseColor = baseColorFactor.xyz;
      alpha = baseColorFactor.w;
    #endif
    #ifdef USE_MR_TEXTURE
      vec4 mr_sample = texture( metallicRoughnessTexture, vMRUv );
      material.metallness *= mr_sample.b;
      material.roughness *= mr_sample.g;
    #endif
  #else
    material.metallness = 0.0;
    material.roughness = 1.0;
  #endif

  //Specular part
  // https://github.com/KhronosGroup/glTF/blob/main/extensions/2.0/Khronos/KHR_materials_specular/README.md
  // 0.04 - reflectance of the Glass
  material.f0 = vec3( 0.04 );
  material.f90 = vec3( 1.0 );
  material.specularFactor = 1.0;
  #ifdef USE_KHR_materials_specular
    material.specularFactor *= specularFactor;
    material.f0 *= specularColorFactor;
    #ifdef USE_SPECULAR_COLOR_TEXTURE
      material.f0 *= SrgbToLinear( texture( specularColorTexture, vSpecularColorUv ).rgb );
    #endif
    #ifdef USE_SPECULAR_TEXTURE
      material.specularFactor *= texture( specularTexture, vSpecularUv ).a;
    #endif
    material.f0 = min( material.f0 * material.specularFactor, vec3( 1.0 ) );
  #endif
  material.f0 = mix( material.f0, material.diffuseColor, material.metallness );
 
  vec3 normal = normalize( vNormal );
  #ifdef USE_NORMAL_TEXTURE
    vec3 normalSample = texture( normalTexture, vNormalUv ).xyz * 2.0 - 1.0;
    normalSample.xy *= vec2( normalScale );

    #ifdef USE_TANGENTS
    {
      vec3 bitangent = cross( normal, vTangent.xyz ) * vTangent.w;
      mat3x3 TBN = mat3x3( vTangent.xyz, bitangent, normal );
      normal = TBN * normalSample;
    }
    #else
      normal = getTBN( normal, vWorldPos, vNormalUv ) * normalSample;
    #endif
    normal = normalize( normal );
  #endif
  if( !gl_FrontFacing )
  {
    normal *= -1.0;
  }

  // Works only with indirect light
  #ifdef USE_OCCLUSION_TEXTURE
    float occlusion = texture( occlusionTexture, vOcclusionUv ).r;
    material.occlusionFactor = 1.0 + occlusionStrength * ( occlusion - 1.0 );
  #else
    material.occlusionFactor = 1.0;
  #endif


  vec3 color = vec3( 0.0 );
  vec3 viewDir = normalize( cameraPosition - vWorldPos );
  // vec3 lightDirs[] = vec3[]
  // (
  //   vec3( 1.0, 0.0, 0.0 ),
  //   vec3( 0.0, 1.0, 0.0 ),
  //   vec3( 0.0, 0.0, 1.0 ),
  //   vec3( -1.0, 0.0, 0.0 ),
  //   vec3( 0.0, -1.0, 0.0 ),
  //   vec3( 0.0, 0.0, -1.0 )
  // );

  #if defined( USE_PBR ) && !defined( USE_IBL )
    vec3 lightDirs[] = vec3[]
    (
      vec3( 1.0, 1.0, 1.0 ),
      vec3( -1.0, 1.0, 1.0 ),
      vec3( 1.0, -1.0, 1.0 ),
      vec3( -1.0, -1.0, 1.0 ),
      vec3( 1.0, 1.0, -1.0 ),
      vec3( 1.0, -1.0, -1.0 ),
      vec3( -1.0, 1.0, -1.0 ),
      vec3( -1.0, -1.0, -1.0 )
    );

    const float lightIntensity = 3.0;
    const vec3 lightColor = vec3( 1.0 );
    float dotVN = clamp( dot( viewDir, normal ), 0.0, 1.0 );

    for( int i = 0; i < 8; i++ )
    {
      vec3 lightDir = normalize( lightDirs[ i ] );
      float dotNL = clamp( dot( normal, lightDir ), 0.0, 1.0 );

      if( dotNL > 0.0 )
      {
        vec4 brdf = BRDF_GGX( lightDir, viewDir, normal, material );
        vec3 Fs = brdf.xyz;
        float Fd = 1.0 - max_value( Fs );
        Fd *= 1.0 - material.metallness;
        float DG = brdf.w;

        vec3 light_diffuse = Fd * material.diffuseColor * RECIPROCAL_PI;
        vec3 light_specular = Fs * DG;// / max( 4.0 * dotVN * dotNL, 0.0001 );

        color += ( light_diffuse + light_specular ) * lightColor * lightIntensity * dotNL;
      }
    }
  #endif

  // Ambient color
  #if defined( USE_PBR ) && defined( USE_IBL )
    color += sampleEnvIrradiance( normal, viewDir, material ) * material.occlusionFactor;
  #elif defined( USE_PBR )
    color += 0.1 * material.diffuseColor * material.occlusionFactor;
  #endif

  emissive_color = vec4( vec3( 0.0 ), 1.0 );
  #ifdef USE_EMISSION 
    emissive_color.xyz = emissiveFactor;
    #ifdef USE_EMISSION_TEXTURE
      emissive_color.xyz *= SrgbToLinear( texture( emissiveTexture, vEmissionUv ).rgb );
    #endif
  #endif
  
  // float v = luminance( color );
  // float lum_alpha = smoothstep( 2.0, 5.0, v );
  // emissive_color = vec4( mix( vec3( 0.0 ), color, lum_alpha ), 1.0 );
  // emissive_color = vec4( 1.0, 0.0, 0.0, 1.0 );
  //emissive_color = vec4( color * 0.1, 1.0 );


  frag_color = vec4( color * alpha, alpha );
}
