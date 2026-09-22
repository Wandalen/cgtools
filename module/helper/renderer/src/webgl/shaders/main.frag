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
  #ifdef USE_OPENPBR
    vec3 sheenColorFactor;
    float sheenRoughness;
  #endif
  #ifdef USE_OPENPBR_IRIDESCENCE
    // Iridescence weight after per-texel modulation by `iridescenceTexture`
    // ( the uniform alone is the scalar factor ).
    float iridescenceWeight;
  #endif
};

struct ReflectedLight
{
  vec3 indirectDiffuse;
  vec3 indirectSpecular;
  vec3 directDiffuse;
  vec3 directSpecular;
  #ifdef USE_KHR_materials_clearcoat
    vec3 clearcoatSpecular;
  #endif
  #ifdef USE_OPENPBR
    vec3 sheenSpecular;
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
#ifdef USE_ENGRAVING
  // vEngravingUv
  uniform sampler2D engravingTexture;
  // Bevel / normal-perturbation intensity at letter edges.
  uniform float engravingStrength;
  // Target roughness inside the carved groove (matte laser-etched finish).
  uniform float engravingRoughness;
  // How much the groove darkens the base albedo / specular color (0 = no change).
  uniform float engravingDarkening;
#endif
#ifdef USE_OPENPBR_IOR
  // Base dielectric IOR → F0 ( OpenPBR `specular_ior`, default 1.5 ); a
  // base-level feature, so it lives outside the full OpenPBR surface path.
  uniform float ior;
#endif
#ifdef USE_OPENPBR
  // OpenPBR Surface ( ASWF ) scalar carriers selected by `PbrMaterial`. Defaults kept in
  // sync with the Rust upload side ( ior = 1.5, fuzz/sheen disabled ), so a material that
  // enables USE_OPENPBR for one carrier always has sane values for the rest.
  uniform vec3 sheenColorFactor;     // OpenPBR `fuzz_color`, default [0, 0, 0]
  uniform float sheenRoughnessFactor; // OpenPBR `fuzz_roughness`, default 0.0
  #ifdef USE_SHEEN_COLOR_TEXTURE
    // vSheenColorUv ; RGB = per-texel multiplier for sheenColorFactor
    uniform sampler2D sheenColorTexture;
  #endif
  #ifdef USE_SHEEN_ROUGHNESS_TEXTURE
    // vSheenRoughnessUv ; A = per-texel multiplier for sheenRoughnessFactor
    uniform sampler2D sheenRoughnessTexture;
  #endif
  #ifdef USE_OPENPBR_IRIDESCENCE
    // OpenPBR `thin_film_*` layer: weight, film IOR and thickness in nanometres
    // ( single value = mean of the glTF min/max, since no thickness texture ).
    uniform float iridescenceFactor;
    uniform float iridescenceIor;
    uniform float iridescenceThickness;
    #ifdef USE_IRIDESCENCE_TEXTURE
      // vIridescenceUv ; A = per-texel multiplier for iridescenceFactor
      uniform sampler2D iridescenceTexture;
    #endif
  #endif
#endif
#ifdef USE_KHR_materials_emissive_strength
  uniform float emissiveStrength;    // KHR_materials_emissive_strength, default 1.0
#endif
#ifdef USE_KULLA_CONTY
  // Kulla–Conty multi-scatter LUT: E(μ, α) in R, row-constant E_avg(α) in G.
  uniform sampler2D kullaConty;
#endif
#ifdef USE_SHEEN_ALBEDO
  // Fuzz ( sheen ) directional-albedo LUT: E(μ, α) in R.
  uniform sampler2D sheenAlbedo;
#endif
#ifdef USE_TRANSMISSION
  // Transmission ( OpenPBR adoption plan §3.3 / §3.4 ). The renderer captures
  // the opaque pass into `transmissionSampler` ( linear color, **mip-chained**
  // so a rough surface reads a pre-blurred level ) + `transmissionDepthSampler`
  // ( nearest depth, used to reject taps landing on an occluder in front of this
  // surface ) and binds them at the material's declared units.
  uniform sampler2D transmissionSampler;
  uniform sampler2D transmissionDepthSampler;
  uniform mat4 viewMatrix;
  uniform mat4 projectionMatrix;
  uniform float transmissionFactor;    // OpenPBR `transmission_weight`
  uniform float transmissionThickness; // geometric slab depth, world units ( glTF `thicknessFactor` )
  uniform float transmissionIor;       // IOR of the transmissive medium ( 1.5 default )
  // OpenPBR `transmission_color`: the color white light becomes after travelling
  // `transmissionDepth` through the medium. Without USE_TRANSMISSION_ABSORPTION
  // ( the spec's `transmission_depth = 0`, "the interior medium is absent" ) it
  // degenerates into a constant multiplicative tint, as the spec prescribes.
  uniform vec3 transmissionColor;
  #ifdef USE_TRANSMISSION_ABSORPTION
    uniform float transmissionDepth;   // OpenPBR `transmission_depth` ( lambda ), world units
  #endif
  #ifdef USE_TRANSMISSION_DISPERSION
    // glTF `KHR_materials_dispersion.dispersion` = 20 / Abbe number, carrying
    // OpenPBR `transmission_dispersion_scale` / `..._abbe_number`.
    uniform float transmissionDispersion;
  #endif
  uniform vec2 uScreenSize;            // framebuffer pixels: screen-space tap + mip-chain top level
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
  // Upper clamp: at grazing angles gv + gl -> 0, and the unclamped reciprocal
  // spikes to ~1e6, producing bright firefly pixels along the silhouette near
  // a specular highlight ( V = G / ( 4 NoV NoL ) must be bounded by 1 ).
  return clamp( 0.5 / max( gv + gl, 1e-6 ), 0.0, 1.0 );
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

#ifdef USE_OPENPBR
// Fuzz ( microfiber / sheen ) lobe — the Charlie NDF + Ashikhmin-Premoze visibility pair
// used by the glTF KHR_materials_sheen reference renderers as the real-time stand-in for
// the OpenPBR `fuzz` microflake layer. The sheen color encodes the layer's reflectivity,
// so no separate Fresnel factor is applied.
float D_Charlie( const in float alpha, const in float dotNH )
{
  float invAlpha = 1.0 / alpha;
  float sin2h = max( 1.0 - dotNH * dotNH, 0.0078125 );
  return ( 2.0 + invAlpha ) * pow( sin2h, 0.5 * invAlpha ) * RECIPROCAL_PI2;
}

float V_Ashikhmin( const in float dotNL, const in float dotNV )
{
  return clamp( 0.25 / max( dotNL + dotNV - dotNL * dotNV, 1e-5 ), 0.0, 1.0 );
}
#endif

#ifdef USE_OPENPBR_IRIDESCENCE
// OpenPBR `thin_film_*` ( iridescence ) layer — the analytic thin-film
// interference model of Belcour ( 2017 ), as shipped by three.js. The film sits
// on the substrate ( `baseF0` = the underlying specular F0 ), with an air
// interface above. cosTheta1 is the cosine of the angle of incidence at the
// top interface ( dot( N, V ) ).
// Reference: https://belcour.github.io/blog/research/2017/05/01/brdf-thin-film.html
const mat3 XYZ_TO_REC709 = mat3
(
  3.2404542, -0.9692660, 0.0556434,
  -1.5371385, 1.8760108, -0.2040259,
  -0.4985314, 0.0415560, 1.0572252
);

vec3 fresnel0_to_ior( vec3 f0 )
{
  vec3 sqrtF0 = sqrt( f0 );
  return ( vec3( 1.0 ) + sqrtF0 ) / max( vec3( 1.0 ) - sqrtF0, vec3( 1e-6 ) );
}

vec3 ior_to_fresnel0( vec3 transmittedIor, float incidentIor )
{
  return pow2( ( transmittedIor - vec3( incidentIor ) ) / max( transmittedIor + vec3( incidentIor ), vec3( 1e-6 ) ) );
}

float ior_to_fresnel0( float transmittedIor, float incidentIor )
{
  return pow2( ( transmittedIor - incidentIor ) / max( transmittedIor + incidentIor, 1e-6 ) );
}

float fresnel_schlick_scalar( const in float f0, const in float cosTheta )
{
  return f0 + ( 1.0 - f0 ) * pow( clamp( 1.0 - cosTheta, 0.0, 1.0 ), 5.0 );
}

// Evaluation of the XYZ colour-matching sensitivity curves in Fourier space.
vec3 evalSensitivity( float OPD, vec3 shift )
{
  float phase = 2.0 * PI * OPD * 1.0e-9;
  vec3 val = vec3( 5.4856e-13, 4.4201e-13, 5.2481e-13 );
  vec3 pos = vec3( 1.6810e+06, 1.7953e+06, 2.2084e+06 );
  vec3 var = vec3( 4.3278e+09, 9.3046e+09, 6.6121e+09 );

  vec3 xyz = val * sqrt( 2.0 * PI * var ) * cos( pos * phase + shift ) * exp( -pow2( phase ) * var );
  xyz.x += 9.7470e-14 * sqrt( 2.0 * PI * 4.5282e+09 ) * cos( 2.2399e+06 * phase + shift[ 0 ] ) * exp( -4.5282e+09 * pow2( phase ) );
  xyz /= 1.0685e-7;

  return XYZ_TO_REC709 * xyz;
}

vec3 evalIridescence
(
  const in float outsideIOR,
  const in float eta2,
  const in float cosTheta1,
  const in float thinFilmThickness,
  const in vec3 baseF0
)
{
  vec3 I;

  // Force iridescenceIOR -> outsideIOR when thinFilmThickness -> 0.0.
  float iridescenceIOR = mix( outsideIOR, eta2, smoothstep( 0.0, 0.03, thinFilmThickness ) );
  // cosTheta at the base layer ( Snell ).
  float sinTheta2Sq = pow2( outsideIOR / max( iridescenceIOR, 1e-6 ) ) * ( 1.0 - pow2( cosTheta1 ) );

  // Handle total internal reflection.
  float cosTheta2Sq = 1.0 - sinTheta2Sq;
  if( cosTheta2Sq < 0.0 )
  {
    return vec3( 1.0 );
  }
  float cosTheta2 = sqrt( cosTheta2Sq );

  // First interface ( film top ).
  float R0 = ior_to_fresnel0( iridescenceIOR, outsideIOR );
  float R12 = fresnel_schlick_scalar( R0, cosTheta1 );
  float T121 = 1.0 - R12;
  float phi12 = 0.0;
  if( iridescenceIOR < outsideIOR ) { phi12 = PI; }
  float phi21 = PI - phi12;

  // Second interface ( film bottom, on the substrate ).
  vec3 baseIOR = fresnel0_to_ior( clamp( baseF0, vec3( 0.0 ), vec3( 0.9999 ) ) );
  vec3 R1 = ior_to_fresnel0( baseIOR, iridescenceIOR );
  vec3 R23 = F_Schlick( R1, vec3( 1.0 ), cosTheta2 );
  vec3 phi23 = vec3( 0.0 );
  if( baseIOR[ 0 ] < iridescenceIOR ) { phi23[ 0 ] = PI; }
  if( baseIOR[ 1 ] < iridescenceIOR ) { phi23[ 1 ] = PI; }
  if( baseIOR[ 2 ] < iridescenceIOR ) { phi23[ 2 ] = PI; }

  // Phase shift.
  float OPD = 2.0 * iridescenceIOR * thinFilmThickness * cosTheta2;
  vec3 phi = vec3( phi21 ) + phi23;

  // Compound terms.
  vec3 R123 = clamp( R12 * R23, 1e-5, 0.9999 );
  vec3 r123 = sqrt( R123 );
  vec3 Rs = pow2( T121 ) * R23 / max( vec3( 1.0 ) - R123, vec3( 1e-6 ) );

  // Reflectance for m = 0 ( DC term ).
  vec3 C0 = R12 + Rs;
  I = C0;

  // Reflectance for m > 0 ( pairs of diracs ).
  vec3 Cm = Rs - vec3( T121 );
  for( int m = 1; m <= 2; ++m )
  {
    Cm *= r123;
    vec3 Sm = 2.0 * evalSensitivity( float( m ) * OPD, float( m ) * phi );
    I += Cm * Sm;
  }

  return max( I, vec3( 0.0 ) );
}
#endif

#ifdef USE_KULLA_CONTY
// Kulla–Conty multi-scatter energy-compensation lookups ( table generated by
// loaders::kulla_conty; row = roughness, column = μ = cosθ ). See the spec's
// Microfacet-model section: single-scatter GGX loses energy on rough surfaces,
// and the missing multi-bounce term is
//   f_ms = (1 - E(μ_o)) · (1 - E(μ_i)) / (π · (1 - E_avg)).
float kulla_e( const in float mu, const in float roughness )
{
  return texture( kullaConty, vec2( clamp( mu, 0.0, 1.0 ), clamp( roughness, 0.0, 1.0 ) ) ).r;
}

float kulla_e_avg( const in float roughness )
{
  return texture( kullaConty, vec2( 0.5, clamp( roughness, 0.0, 1.0 ) ) ).g;
}

// The colored weight of the compensation term: the sum of the per-bounce
// Fresnel series, F_ms = F_avg² E_avg / ( 1 - F_avg ( 1 - E_avg ) ). Light that
// survived n bounces has been Fresnel-weighted n times, so the returned energy
// is tinted more than a single F_avg and smaller in magnitude - weighting by
// F_avg alone over-brightened the term and washed a colored metal’s tint out of
// it. Limits: a perfect mirror ( F_avg = 1 ) returns all the lost energy, a
// black substrate none. Mirrored by loaders::kulla_conty::multi_scatter_fresnel.
vec3 kulla_fms( const in vec3 Favg, const in float Eavg )
{
  return Favg * Favg * Eavg / max( vec3( 1.0 ) - Favg * ( 1.0 - Eavg ), vec3( 1e-4 ) );
}
#endif

#ifdef USE_SHEEN_ALBEDO
// Directional albedo of the fuzz lobe ( table generated by
// loaders::sheen_albedo with the *same* Charlie NDF / Ashikhmin visibility
// pair and the same 0.1 roughness floor this shader evaluates; row = fuzz
// roughness, column = μ = cosθ ). OpenPBR layers fuzz over the substrate as
// `layer( base, fuzz, weight )` - the layer takes its energy *from* the base -
// so the substrate has to be scaled by `1 - max3( fuzz_color ) * E(μ)` rather
// than having the lobe simply added on top of it.
float sheen_e( const in float mu, const in float alpha )
{
  return texture( sheenAlbedo, vec2( clamp( mu, 0.0, 1.0 ), clamp( alpha, 0.0, 1.0 ) ) ).r;
}
#endif

#ifdef USE_TRANSMISSION
// Transmission helpers. Each mirrors a pure Rust function of
// `renderer::webgl::material` ( `transmission.rs` ), which is where they are
// natively tested - this shader cannot be unit-tested, so the arithmetic lives
// in both places deliberately and the Rust side is the reference.

// Distance the refracted ray is carried inside the object - the thickness, with
// no angle dependence ( the three.js / glTF sample viewer model ). The
// angle-dependent readings, including a full two-refraction ball lens that is
// optically exact, send the outer band of the object off-screen where the
// capture has nothing; see renderer::webgl::material::traversal_length.
float transmissionTraversalLength( const in float thickness )
{
  return max( thickness, 0.0 );
}

// Mip level of the captured transmission target approximating a roughness-driven
// blur of the refracted ray at this IOR.
float transmissionBlurLod( const in float roughness, const in float ior, const in float targetSize )
{
  float maxLod = log2( max( targetSize, 1.0 ) );
  float iorFactor = clamp( 2.0 * ior - 2.0, 0.0, 1.0 );
  return clamp( maxLod * clamp( roughness, 0.0, 1.0 ) * iorFactor, 0.0, maxLod );
}

// Beer's-law transmittance of the interior medium over `pathLength`:
// `exp( -mu_t * d )` with `mu_t = -ln( transmission_color ) / transmission_depth`
// ( OpenPBR spec, transmission section ).
vec3 transmissionTransmittance( const in vec3 color, const in float depth, const in float pathLength )
{
  vec3 c = clamp( color, vec3( 1e-5 ), vec3( 1.0 ) );
  vec3 mu_t = -log( c ) / max( depth, 1e-5 );
  return exp( -mu_t * max( pathLength, 0.0 ) );
}

// View-space z ( negative in front of the camera ) of the captured opaque depth
// at `uv`. Inverts `clip.z / clip.w` for a column-major projection matrix, so it
// holds for perspective and orthographic cameras alike.
float transmissionViewDepth( const in vec2 uv )
{
  float zNdc = texture( transmissionDepthSampler, uv ).r * 2.0 - 1.0;
  return ( projectionMatrix[ 3 ][ 2 ] - zNdc * projectionMatrix[ 3 ][ 3 ] )
  / ( zNdc * projectionMatrix[ 2 ][ 3 ] - projectionMatrix[ 2 ][ 2 ] );
}

// renderer::webgl::material::occluder_dilation_texels - the radius the occluder
// test below looks over. See that function for why one is needed at all.
float transmissionOccluderDilation( const in float blurLod )
{
  return clamp( exp2( max( blurLod, 0.0 ) ), 1.0, 4.0 );
}

// View-space z of the *nearest* captured surface within `radiusTexels` of `uv`.
// View z is negative in front of the camera, so nearer is larger and the
// neighbourhood maximum is the closest thing the tap could have picked up.
float transmissionNearestViewDepth( const in vec2 uv, const in float radiusTexels )
{
  vec2 r = radiusTexels / uScreenSize;
  float z = transmissionViewDepth( uv );
  z = max( z, transmissionViewDepth( uv + vec2( r.x, 0.0 ) ) );
  z = max( z, transmissionViewDepth( uv - vec2( r.x, 0.0 ) ) );
  z = max( z, transmissionViewDepth( uv + vec2( 0.0, r.y ) ) );
  z = max( z, transmissionViewDepth( uv - vec2( 0.0, r.y ) ) );
  return z;
}

// renderer::webgl::material::dispersion_iors - the refractive indices a
// dispersive medium presents to red, green and blue. Red is refracted least and
// blue most, so the triple straddles the base index. The 0.025 is the glTF
// carrier's normalisation, 1 / ( 2 * 20 ), not a fudge factor: substituting
// dispersion = 20 / Abbe recovers the optical half-spread ( n_d - 1 ) / ( 2 Vd ).
vec3 transmissionDispersionIors( const in float ior, const in float dispersion )
{
  float base = max( ior, 1.0001 );
  float halfSpread = ( base - 1.0 ) * 0.025 * max( dispersion, 0.0 );
  return max( vec3( base - halfSpread, base, base + halfSpread ), vec3( 1.0001 ) );
}

// Screen uv the refracted ray reaches for one refractive index, or `screenUv`
// when the tap cannot be trusted. Factored out of main() because chromatic
// dispersion runs it once per colour channel, at three slightly different
// indices; without dispersion it is called once and inlines away.
//
// The depth-guided validation lives here too. The capture holds the opaque scene
// only, and this surface is not in it, so a tap is valid only when what it hits
// sits *behind* the glass; a nearer sample is an occluder between glass and
// camera, and refracting it would smear a foreground object through the surface.
// Off-screen taps have no data at all ( the target is CLAMP_TO_EDGE ). Both
// cases fall back to the un-refracted pixel, which is the correct limit as
// thickness -> 0 anyway.
//
// The test looks over a neighbourhood rather than the single tap texel, because
// the capture’s colour and depth disagree at a silhouette: colour is filtered
// ( bilinear, mip, MSAA-resolved ) while depth is one nearest texel. Testing only
// the exact texel let the half-covered edge pixels through, painting a bright
// dotted outline of every foreground object on the glass.
vec2 transmissionRefractedUv
(
  const in vec3 N,
  const in vec3 V,
  const in float ior,
  const in float thickness,
  const in vec2 screenUv,
  const in float blurLod
)
{
  // Entering the medium from air: eta = n_outside / n_inside.
  vec3 tIn = refract( -V, N, 1.0 / max( ior, 1.0001 ) );
  // Refraction into a denser medium bends towards the normal and cannot
  // total-internally-reflect, but a normal map can still hand us a degenerate
  // ray; fall back to the straight view ray there.
  tIn = any( notEqual( tIn, vec3( 0.0 ) ) ) ? normalize( tIn ) : -V;

  // The exit ray is taken to leave travelling along -V again, so only its exit
  // *position* shifts - which is all a screen-space tap needs. ( Two richer
  // models were tried and reverted : refracting a second time at the back
  // surface is optically exact for a sphere and reproduced an analytic ball
  // lens, but a real ball is a wide-angle instrument and the band it is
  // recognised by falls outside the captured frame. See
  // renderer::webgl::material::traversal_length. )
  vec3 exitPos = vWorldPos + tIn * transmissionTraversalLength( thickness );

  vec4 clip = projectionMatrix * viewMatrix * vec4( exitPos, 1.0 );
  vec2 projected = clip.xy / max( abs( clip.w ), 1e-5 ) * 0.5 + 0.5;
  // `clip.w` can go <= 0 behind the camera; pin those samples to the pixel.
  vec2 uv = clip.w > 0.0 ? projected : screenUv;

  float sampleViewZ = transmissionNearestViewDepth( uv, transmissionOccluderDilation( blurLod ) );
  float surfaceViewZ = ( viewMatrix * vec4( vWorldPos, 1.0 ) ).z;
  bool occluded = sampleViewZ > surfaceViewZ + 1e-4;
  bool offScreen = any( lessThan( uv, vec2( 0.0 ) ) ) || any( greaterThan( uv, vec2( 1.0 ) ) );
  return ( occluded || offScreen ) ? screenUv : uv;
}
#endif

#ifdef USE_KHR_materials_clearcoat
// The clearcoat layer is modeled as a fixed-IOR (1.5) dielectric coat, using the same
// isotropic GGX D/V terms as the base layer but with its own normal and roughness.
// https://github.com/KhronosGroup/glTF/blob/main/extensions/2.0/Khronos/KHR_materials_clearcoat/README.md
// The extension composes the coat as `fresnel_coat( ior = 1.5, weight = clearcoat,
// base = material, layer = specular_brdf( ccNormal, ccRoughness ) )`, i.e.
// `mix( base, layer, weight * F )` over a *Fresnel-free* `specular_brdf` = D * V.
// The Fresnel factor therefore belongs to the mix weight in `main()` and must not
// also sit inside this lobe - it used to, which multiplied the coat highlight by
// F twice ( ~0.04 head-on, so the coat read ~25x too dim there ) and made the
// direct coat term inconsistent with the coat's own IBL term, which has never
// carried a Fresnel factor. `dotVH` is kept in the signature for the callers'
// symmetry with the base lobe.
vec3 BRDF_Clearcoat( const in float dotNL, const in float dotNV, const in float dotNH, const in float dotVH, const in float roughness )
{
  float alpha = pow2( roughness );
  float D = D_GGX( alpha, dotNH );
  float V = V_GGX_SmithCorrelated( alpha, dotNL, dotNV );
  return vec3( D * V * dotNL );
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

  // Fresnel
  vec3 Fs = F_Schlick( material.f0, material.f90, dotVH );
  #ifdef USE_OPENPBR_IRIDESCENCE
    // Thin-film ( iridescence ) layer: mix in the interference reflectance of the
    // film on the substrate. The angle of incidence is the one at the *microfacet*
    // that actually reflects towards the eye - `dot( V, H )`, the same cosine the
    // Schlick Fresnel it replaces uses ( and what the Khronos reference
    // implementation of KHR_materials_iridescence feeds `evalIridescence` ).
    // Using `dot( N, V )` instead pinned the interference to the shading normal,
    // so the film's color bands stopped moving with the highlight on anything but
    // a mirror-smooth surface.
    Fs = mix( Fs, evalIridescence( 1.0, iridescenceIor, dotVH, iridescenceThickness, material.f0 ), material.iridescenceWeight );
  #endif
  // Diffuse BRDF (Burley)
  vec3 Fd = Fd_Barley( alpha, dotNV, dotNL, dotLH );

  vec3 irradiance = lightColor * lightIntensity * dotNL;
  vec3 diffuseColor = material.diffuseColor * irradiance;
  vec3 specularColor = D * V * irradiance;

  reflectedLight.directDiffuse += ( 1.0 - Fs ) * Fd * diffuseColor;
  // Fade specular at grazing angles so the silhouette rim doesn't read as a
  // hard "contour light" ( the aliased bright edge on smooth surfaces ).
  float grazingFade = smoothstep( 0.0, 0.25, dotNV );
  reflectedLight.directSpecular += Fs * specularColor * grazingFade;

  #ifdef USE_KULLA_CONTY
    // Multi-scatter energy compensation: add back the light lost between
    // microfacet bounces, weighted by the per-bounce Fresnel series.
    {
      float Eo = kulla_e( dotNV, material.roughness );
      float Ei = kulla_e( dotNL, material.roughness );
      float Eavg = kulla_e_avg( material.roughness );
      vec3 Favg = material.f0 + ( material.f90 - material.f0 ) * 0.047619;
      reflectedLight.directSpecular += kulla_fms( Favg, Eavg ) * ( ( 1.0 - Eo ) * ( 1.0 - Ei ) / ( PI * max( 1.0 - Eavg, 1e-4 ) ) ) * irradiance;
    }
  #endif

  #ifdef USE_KHR_materials_clearcoat
    float ccDotNL = clamp( dot( material.clearcoatNormal, lightDir ), 0.0, 1.0 );
    float ccDotNV = clamp( dot( material.clearcoatNormal, viewDir ), 0.0, 1.0 );
    float ccDotNH = clamp( dot( material.clearcoatNormal, halfDir ), 0.0, 1.0 );
    float ccDotVH = clamp( dot( viewDir, halfDir ), 0.0, 1.0 );
    reflectedLight.clearcoatSpecular += BRDF_Clearcoat( ccDotNL, ccDotNV, ccDotNH, ccDotVH, material.clearcoatRoughness ) * lightColor * lightIntensity;
  #endif

  #ifdef USE_OPENPBR
    // Fuzz ( OpenPBR `fuzz` ) layer on top of the coated substrate: accumulated separately
    // and added over the final color in main(). Enabled per-lobe only when its color is
    // non-zero ( the sheen/`fuzz_color` disabled default is black ).
    if( max_value( material.sheenColorFactor ) > 0.0 )
    {
      float sheenAlpha = clamp( material.sheenRoughness, 0.1, 1.0 );
      reflectedLight.sheenSpecular += material.sheenColorFactor * D_Charlie( sheenAlpha, dotNH ) * V_Ashikhmin( dotNL, dotNV ) * lightColor * lightIntensity * dotNL;
    }
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

  float dotNL = clamp( dot( normal, lightDir ), 0.0, 1.0 );

  float alpha = pow2( material.roughness );
  vec3 halfDir = normalize( lightDir + viewDir );
  float dotNV = clamp( dot( normal, viewDir ), 0.0, 1.0 );
  float dotNH = clamp( dot( normal, halfDir ), 0.0, 1.0 );
  float dotVH = clamp( dot( viewDir, halfDir ), 0.0, 1.0 );
  float dotLH = clamp( dot( lightDir, halfDir ), 0.0, 1.0 );

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

  vec3 Fs = F_Schlick( material.f0, material.f90, dotVH );
  #ifdef USE_OPENPBR_IRIDESCENCE
    // Thin-film ( iridescence ) layer: mix in the interference reflectance of the
    // film on the substrate. The angle of incidence is the one at the *microfacet*
    // that actually reflects towards the eye - `dot( V, H )`, the same cosine the
    // Schlick Fresnel it replaces uses ( and what the Khronos reference
    // implementation of KHR_materials_iridescence feeds `evalIridescence` ).
    // Using `dot( N, V )` instead pinned the interference to the shading normal,
    // so the film's color bands stopped moving with the highlight on anything but
    // a mirror-smooth surface.
    Fs = mix( Fs, evalIridescence( 1.0, iridescenceIor, dotVH, iridescenceThickness, material.f0 ), material.iridescenceWeight );
  #endif
  vec3 Fd = Fd_Barley( alpha, dotNV, dotNL, dotLH );

  vec3 irradiance = light.color * attenuation * dotNL;
  vec3 diffuseColor = material.diffuseColor * irradiance;
  vec3 specularColor = D * V * irradiance;

  reflectedLight.directDiffuse += ( 1.0 - Fs ) * Fd * diffuseColor;
  // Fade specular at grazing angles so the silhouette rim doesn't read as a
  // hard "contour light" ( the aliased bright edge on smooth surfaces ).
  float grazingFade = smoothstep( 0.0, 0.25, dotNV );
  reflectedLight.directSpecular += Fs * specularColor * grazingFade;

  #ifdef USE_KULLA_CONTY
    // Multi-scatter energy compensation: add back the light lost between
    // microfacet bounces, weighted by the per-bounce Fresnel series.
    {
      float Eo = kulla_e( dotNV, material.roughness );
      float Ei = kulla_e( dotNL, material.roughness );
      float Eavg = kulla_e_avg( material.roughness );
      vec3 Favg = material.f0 + ( material.f90 - material.f0 ) * 0.047619;
      reflectedLight.directSpecular += kulla_fms( Favg, Eavg ) * ( ( 1.0 - Eo ) * ( 1.0 - Ei ) / ( PI * max( 1.0 - Eavg, 1e-4 ) ) ) * irradiance;
    }
  #endif

  #ifdef USE_OPENPBR
    if( max_value( material.sheenColorFactor ) > 0.0 )
    {
      float sheenAlpha = clamp( material.sheenRoughness, 0.1, 1.0 );
      reflectedLight.sheenSpecular += material.sheenColorFactor * D_Charlie( sheenAlpha, dotNH ) * V_Ashikhmin( dotNL, dotNV ) * light.color * attenuation * dotNL;
    }
  #endif

  #ifdef USE_KHR_materials_clearcoat
    float ccDotNL = clamp( dot( material.clearcoatNormal, lightDir ), 0.0, 1.0 );
    float ccDotNV = clamp( dot( material.clearcoatNormal, viewDir ), 0.0, 1.0 );
    float ccDotNH = clamp( dot( material.clearcoatNormal, halfDir ), 0.0, 1.0 );
    float ccDotVH = clamp( dot( viewDir, halfDir ), 0.0, 1.0 );
    reflectedLight.clearcoatSpecular += BRDF_Clearcoat( ccDotNL, ccDotNV, ccDotNH, ccDotVH, material.clearcoatRoughness ) * light.color * attenuation;
  #endif
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
    float dotNL = clamp( dot( normal, lightDir ), 0.0, 1.0 );

    if ( dotNL > 0.0 )
    {
      computePointLight( pointLights[ i ], viewDir, normal, material, reflectedLight );
    }
  }

  for( int i = 0; i < min( directLightsCount, MAX_DIRECT_LIGHTS ); i++ )
  {
    float dotNL = clamp( dot( normal, directLights[ i ].direction ), 0.0, 1.0 );

    if ( dotNL > 0.0 )
    {
      computeDirectLight( directLights[ i ], viewDir, normal, material, reflectedLight );
    }
  }

  for( int i = 0; i < min( spotLightsCount, MAX_SPOT_LIGHTS ); i++ )
  {
    vec3 lightDir = normalize( spotLights[ i ].position - vWorldPos );
    float dotNL = clamp( dot( normal, lightDir ), 0.0, 1.0 );

    if ( dotNL > 0.0 )
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

  void sampleEnvIrradiance( const in vec3 N, const in vec3 V, const in PhysicalMaterial material, inout ReflectedLight reflectedLight )
  {
    float dotNV = clamp( dot( N, V ), 0.01, 1.0 );

    vec3 R = reflect( -V, N );

    // Anisotropic IBL: bend the reflection vector towards the anisotropic tangent frame
    // (bent-normal approximation from the glTF-Sample-Renderer reference implementation).
    // The LOD / envBRDF / multi-scatter terms below stay driven by the original roughness —
    // only the sampled direction changes.
    #ifdef USE_KHR_materials_anisotropy
      vec3 anisotropicTangent = cross( material.anisotropicB, V );
      vec3 anisotropicNormal = cross( anisotropicTangent, material.anisotropicB );
      float bendFactor = 1.0 - material.anisotropyStrength * ( 1.0 - material.roughness );
      vec3 bentNormal = normalize( mix( anisotropicNormal, N, pow4( bendFactor ) ) );
      R = reflect( -V, bentNormal );
    #endif

    // Base LOD from roughness. No fixed floor (e.g. `max( .., 1.0 )`) is applied: with the
    // PMREM prefilter, mip 0 of prefilterEnvMap is the sharp environment, which is the correct,
    // physically expected result for a mirror-smooth surface. Specular aliasing comes from the
    // reflection vector being under-sampled across a pixel (curved geometry, silhouettes,
    // grazing angles) — exactly the case where the screen-space variance term below is large
    // and raises the LOD. On flat smooth surfaces reflVariance ~ 0, there is no sub-pixel
    // variation to alias, so sampling mip 0 there is safe; a fixed floor would only blur
    // legitimate mirror reflections.
    float lod = material.roughness * u_max_lod;

    // GSAA-style specular antialiasing: widen the filter where the reflected direction changes
    // rapidly in screen space.
    vec3 dRdx = dFdx( R );
    vec3 dRdy = dFdy( R );
    float reflVariance = dot( dRdx, dRdx ) + dot( dRdy, dRdy );
    lod = max( lod, 0.5 * log2( max( reflVariance, 1e-6 ) ) + 4.0 );
    lod = min( lod, u_max_lod );

    float dither = ( ditherNoise( gl_FragCoord.xy ) - 0.5 ) / 512.0;

    vec3 irradiance = texture( irradianceTexture, N ).xyz + dither;
    vec3 radiance = textureLod( prefilterEnvMap, R, lod ).xyz + dither;

    vec2 envBrdf = texture( integrateBRDF, vec2( dotNV, material.roughness ) ).xy;

    // Thin-film ( iridescence ) under the environment. The film replaces the
    // substrate’s F0 with its own interference reflectance, exactly as it does
    // for a direct light - the only difference is the angle it is evaluated at.
    // The direct paths use dot( V, H ) because the microfacet that reflects
    // towards the eye is H; the split-sum gathers its prefiltered reflection
    // about R = reflect( -V, N ), for which the reflecting microfacet *is* N, so
    // dot( V, H ) = dot( N, V ) here. Same choice, not a different one.
    //
    // What this cannot do: a prefiltered environment is one RGB radiance sample
    // per pixel and carries no spectral detail, so the film can only tint that
    // sample per channel. Interference against the environment’s actual spectrum
    // would need the spectrum, which the prefiltered map has already integrated
    // away. `evalIridescence` returns an RGB reflectance, which is precisely a
    // per-channel tint, so substituting it for F0 is the whole implementation.
    vec3 envF0 = material.f0;
    #ifdef USE_OPENPBR_IRIDESCENCE
      envF0 = mix
      (
        envF0,
        evalIridescence( 1.0, iridescenceIor, dotNV, iridescenceThickness, material.f0 ),
        material.iridescenceWeight
      );
    #endif

    // Split-sum with multiple-scattering energy compensation, matching three.js
    // computeMultiscattering(). The single-scatter term is the usual prefiltered
    // reflection; the multi-scatter term feeds the energy lost between microfacet
    // bounces back as a soft, irradiance-weighted lobe — without it rough metals /
    // plastics read as pure mirrors and the overall specular is too dim.
    //
    // `f90` stays the substrate’s: at grazing incidence the interference washes
    // out towards total reflection, which a white `f90` already says. `Favg` is
    // derived from the *substituted* F0 so the multi-scatter term carries the
    // same tint as the single-scatter one it compensates for - deriving it from
    // the bare substrate would put an untinted lobe under a tinted reflection.
    vec3 FssEss = envF0 * envBrdf.x + material.f90 * envBrdf.y;
    float Ess = envBrdf.x + envBrdf.y;
    float Ems = 1.0 - Ess;
    vec3 Favg = envF0 + ( 1.0 - envF0 ) * 0.047619; // 1.0 / 21.0
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
      float lodc = min( material.clearcoatRoughness * u_max_lod, u_max_lod );
      reflectedLight.clearcoatSpecular += textureLod( prefilterEnvMap, Rc, lodc ).xyz;
    #endif

    // Fuzz ( OpenPBR `fuzz` ) environment term: the microflake lobe is wide, so the diffuse
    // irradiance sample along N is a reasonable stand-in for its environment response.
    // Its directional albedo scales that response, so the energy this layer adds is the
    // same fraction the albedo scaling in main() takes away from the substrate; without
    // the factor a fuzzy surface under an environment reads as a uniformly brighter one.
    #ifdef USE_OPENPBR
      if( max_value( material.sheenColorFactor ) > 0.0 )
      {
        float fuzzEnvE = 1.0;
        #ifdef USE_SHEEN_ALBEDO
          fuzzEnvE = sheen_e( dotNV, clamp( material.sheenRoughness, 0.1, 1.0 ) );
        #endif
        reflectedLight.sheenSpecular += material.sheenColorFactor * fuzzEnvE * irradiance;
      }
    #endif
  }

#endif

float alpha_weight( float a )
{
  return clamp( pow( min( 1.0, a * 10.0 ) + 0.01, 3.0 ) * 1e8 * pow( 1.0 - gl_FragCoord.z * 0.9, 3.0 ), 1e-2, 3e3 );
}

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
  #endif
  #ifdef USE_OPENPBR
    reflectedLight.sheenSpecular = vec3( 0.0 );
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
  // The dielectric F0 comes from the IOR ( OpenPBR `specular_ior`, default 1.5
  // → F0 = 0.04 ), the glTF/three.js default otherwise.
  #ifdef USE_OPENPBR_IOR
    material.f0 = vec3( pow2( ior - 1.0 ) / pow2( ior + 1.0 ) );
  #else
    material.f0 = vec3( 0.04 );
  #endif
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

  // faceDirection is applied to the geometric normal up front (before TBN / normal-map /
  // clearcoat / anisotropy all consume it), so every one of those is consistently oriented
  // on double-sided back faces.
  float faceDirection = gl_FrontFacing ? 1.0 : -1.0;
  vec3 geometricNormal = normalize( vNormal ) * faceDirection;

  #ifdef USE_TBN
    mat3 TBN;
    #ifdef USE_TANGENTS
      vec3 bitangent = cross( geometricNormal, vTangent.xyz ) * vTangent.w;
      TBN = mat3( vTangent.xyz, bitangent, geometricNormal );
    #else
      // No per-texture UV is threaded through here (unlike the normal texture's own vNormalUv
      // below) — vUv_0 is used as a simplification for the clearcoat-normal/anisotropy-only case.
      TBN = getTBN( geometricNormal, vWorldPos, vUv_0 );
    #endif
  #endif

  vec3 normal = geometricNormal;
  #ifdef USE_NORMAL_TEXTURE
    vec3 normalSample = texture( normalTexture, vNormalUv ).xyz * 2.0 - 1.0;
    //material.roughness = adjustRoughnessNormalMap( material.roughness, normalSample );
    normalSample.xy *= vec2( normalScale );
    normal = normalize( TBN * normalSample );
  #endif

  // Engraving: a laser-etched text mask sampled from a dedicated UV channel (vEngravingUv,
  // typically TEXCOORD_1), separate from the base material's UVs. Three things happen here:
  //
  //  1. Bounds check — vEngravingUv is expected to only be meaningful inside [0, 1]; outside
  //     that range (e.g. the rest of the mesh sharing the same UV set with degenerate/unused
  //     coordinates there) the mask is forced to 0 so nothing bleeds in from CLAMP_TO_EDGE
  //     sampling at the strip's border.
  //
  //  2. Relief LOD estimate — `engravingLod` approximates which mip level a filtered lookup at
  //     vEngravingUv would resolve to, from the screen-space footprint of the UV itself
  //     (texels-per-pixel, isotropic/worst-axis, same rule hardware trilinear filtering uses).
  //     This only looks at how fast the UV moves across the screen, never at the mask value's
  //     own derivative, so the estimate stays stable even right at a glyph edge; step 3 uses it
  //     to pick both the explicit LOD and the UV offset for its central-difference samples.
  //
  //  3. Relief via a surface gradient (Mikkelsen), sampled explicitly in UV space —
  //     engravingTexture is a binary-ish mask (0 outside glyphs, 1 inside, antialiased in
  //     between). Treating it as a height field h(u, v), the bevel at a stroke edge perturbs the
  //     normal by the standard first-order bump-mapping approximation (Blinn 1978):
  //
  //       N' = normalize( N - engravingStrength * ( dh/du * T + dh/dv * B ) )
  //
  //     — for a surface *raised* along +N by h (bas-relief/emboss). A laser-engraved groove is
  //     the opposite: it's carved *into* the metal, i.e. displaced along -N by h, so entering a
  //     glyph (mask 0 -> 1) must tilt the normal inward toward the channel instead of outward
  //     away from it. Flipping the sign of the whole perturbation term (+ instead of -) accounts
  //     for that -h displacement and is what actually reads as a carved/concave groove rather
  //     than a raised/convex bump — see step 3's code below.
  //
  //     A prior version of this code computed dh/du, dh/dv as dFdx(h)/dFdy(h) — the change in
  //     mask value per *screen* pixel — and fed that directly into the T/B (UV-axis tangent)
  //     projection above. That's a unit mismatch: screen X/Y only line up with UV U/V when the
  //     camera is exactly front-on with no in-plane rotation, so at any other angle a screen-space
  //     step mixes U and V in the wrong proportions and the bump rotates away from the true groove
  //     wall as the camera turns. Worse, dFdx/dFdy of a *filtered texture fetch* differentiates
  //     across a 2x2 fragment quad, and each fragment in that quad can resolve a slightly
  //     different mip/bilinear sample — so even the magnitude was jittery, which reads as
  //     sparkling specular noise (worst at low roughness, where the lobe is narrow enough to make
  //     per-pixel normal jitter visible as flickering).
  //
  //     Both problems go away by computing dh/du, dh/dv as explicit central differences of
  //     `textureLod` samples taken along the actual U and V axes, at a fixed LOD derived from
  //     step 2: every fragment now samples the exact same, deliberately-chosen texels (instead of
  //     an implicit, quad-dependent derivative), and the offset is expressed purely in UV space,
  //     so it composes correctly with T = dP/du and B = dP/dv regardless of camera orientation.
  //     The finite difference is left undivided by the UV step (rather than forming a literal
  //     dh/du = Δh / (2Δu)) so the gradient stays O(1) — matching the previous tuning range of
  //     engravingStrength — and bounded at any LOD/distance instead of blowing up as the sample
  //     step shrinks at grazing angles.
  //
  //  4. PBR response — inside the groove (mask towards 1) roughness is pushed towards
  //     engravingRoughness (a matte, laser-scattered finish) and the albedo / specular color is
  //     darkened by engravingDarkening, approximating the micro-occlusion of a recessed groove
  //     without desaturating the metal's own hue (both diffuseColor and f0 are scaled by the
  //     same factor, so hue/chroma survive, only brightness drops).
  #ifdef USE_ENGRAVING
    bool engravingInBounds = all( greaterThanEqual( vEngravingUv, vec2( 0.0 ) ) ) && all( lessThanEqual( vEngravingUv, vec2( 1.0 ) ) );
    float engravingMask = engravingInBounds ? texture( engravingTexture, vEngravingUv ).r : 0.0;

    ivec2 engravingTexDims = textureSize( engravingTexture, 0 );
    vec2 engravingInvTexelSize = 1.0 / vec2( engravingTexDims );
    vec2 engravingUvDx = dFdx( vEngravingUv ) * vec2( engravingTexDims );
    vec2 engravingUvDy = dFdy( vEngravingUv ) * vec2( engravingTexDims );
    float engravingTexelsPerPixel = max( length( engravingUvDx ), length( engravingUvDy ) );
    float engravingLod = log2( max( engravingTexelsPerPixel, 1.0 ) );

    // Clamp before exp2: engravingLod is only bounded below (max(..,1.0) inside the log2), so at
    // a near-grazing/sliver footprint it can grow large enough that exp2() overflows to inf and
    // poisons the sample offsets with NaN. Past the texture's own top mip there is nothing left
    // to resolve anyway, so clamping to it costs nothing.
    float engravingMaxLod = log2( float( max( engravingTexDims.x, engravingTexDims.y ) ) );
    float engravingSampleLod = clamp( engravingLod, 0.0, engravingMaxLod );

    // One texel *at the resolved LOD* in each axis — matches the support size of the sample
    // being differentiated, avoiding both sub-texel noise (step too small) and over-blurring
    // (step too large).
    vec2 engravingStep = max( engravingInvTexelSize * exp2( engravingSampleLod ), engravingInvTexelSize );
    float engravingHR = textureLod( engravingTexture, vEngravingUv + vec2( engravingStep.x, 0.0 ), engravingSampleLod ).r;
    float engravingHL = textureLod( engravingTexture, vEngravingUv - vec2( engravingStep.x, 0.0 ), engravingSampleLod ).r;
    float engravingHU = textureLod( engravingTexture, vEngravingUv + vec2( 0.0, engravingStep.y ), engravingSampleLod ).r;
    float engravingHD = textureLod( engravingTexture, vEngravingUv - vec2( 0.0, engravingStep.y ), engravingSampleLod ).r;
    vec2 engravingGrad = vec2( engravingHR - engravingHL, engravingHU - engravingHD );

    // '+' (not '-'): the mask is carved *into* the surface (height -h along N, not +h), so the
    // sign of the standard bump-mapping perturbation is flipped to match — see step 3 above.
    normal = normalize( normal + engravingStrength * ( engravingGrad.x * TBN[ 0 ] + engravingGrad.y * TBN[ 1 ] ) );

    material.roughness = mix( material.roughness, engravingRoughness, engravingMask );

    float engravingAo = 1.0 - engravingDarkening * engravingMask;
    material.diffuseColor *= engravingAo;
    material.f0 *= engravingAo;
  #endif

  // KHR_materials_clearcoat: an additional dielectric (IOR 1.5) coat layer, using its own
  // normal (starting from the *unperturbed* geometric normal, not the base normal map result)
  // and roughness. See https://github.com/KhronosGroup/glTF/blob/main/extensions/2.0/Khronos/KHR_materials_clearcoat/README.md
  #ifdef USE_KHR_materials_clearcoat
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
  #endif

  // KHR_materials_anisotropy: tangent/bitangent frame construction. The roughness split
  // (material.at / material.ab) is computed further below, once the final (GSAA-adjusted)
  // roughness is known.
  // See https://github.com/KhronosGroup/glTF/blob/main/extensions/2.0/Khronos/KHR_materials_anisotropy/README.md
  #ifdef USE_KHR_materials_anisotropy
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
    material.anisotropyStrength = anisotropyMagnitude;
  #endif

  #ifdef USE_OPENPBR
    // Fuzz ( OpenPBR `fuzz` ) layer inputs; the uniforms carry the disabled default
    // ( black color ) when the KHR_materials_sheen extension is absent. Textures,
    // when present, are per-texel multipliers of the factors ( glTF semantics ).
    material.sheenColorFactor = sheenColorFactor;
    material.sheenRoughness = sheenRoughnessFactor;
    #ifdef USE_SHEEN_COLOR_TEXTURE
      material.sheenColorFactor *= texture( sheenColorTexture, vSheenColorUv ).rgb;
    #endif
    #ifdef USE_SHEEN_ROUGHNESS_TEXTURE
      material.sheenRoughness *= texture( sheenRoughnessTexture, vSheenRoughnessUv ).a;
    #endif
    #ifdef USE_OPENPBR_IRIDESCENCE
      material.iridescenceWeight = iridescenceFactor;
      #ifdef USE_IRIDESCENCE_TEXTURE
        material.iridescenceWeight *= texture( iridescenceTexture, vIridescenceUv ).a;
      #endif
    #endif
  #endif

  // -------------------------------------------------------------------------
  // Specular aliasing at grazing angles — why it happens and the layered fix.
  //
  // A near-mirror surface ( tiny roughness ) lit by a small analytic light
  // produces a specular band that, near the silhouette, is foreshortened to
  // *less than a pixel*. A sub-pixel GGX lobe cannot be resolved per pixel, so
  // its intensity jumps between a huge spike and ~0 across pixel boundaries,
  // aliasing into isolated bright dots along the contour. Fresnel reflectance
  // -> 1 at grazing makes the rim the brightest part, which concentrates the
  // sparkle there. Mitigations, in order:
  //   1. V_GGX_SmithCorrelated is clamped to [0,1] ( kills 1/0 fireflies ).
  //   2. a minimum-roughness floor keeps lobes at least a few pixels wide.
  //   3. Tokuyoshi & Kaplanyan GSAA ( below ) widens roughness from the
  //      screen-space normal variance.
  //   4. direct specular fades out at grazing ( smoothstep on dotNV, in
  //      applyLightContribution / computeSpotLight ) so the rim no longer
  //      reads as a hard "contour light".
  // Residual single-pixel sparkle is accepted; a soft area/environment light
  // would remove sub-pixel highlights at the source rather than post-fading.
  // -------------------------------------------------------------------------
  // Geometric Specular Anti-Aliasing (Tokuyoshi & Kaplanyan 2019)
  // Increases roughness where screen-space normal derivatives are large
  // ( silhouette / grazing angles ), widening the lobe so it no longer aliases
  // into single bright pixels. The per-axis *maximum* variance ( rather than
  // the sum ) with a 2x weight matches the reference; capped so distant edges
  // don't over-blur.
  vec3 dNdx = dFdx( normal );
  vec3 dNdy = dFdy( normal );
  float geometricVariance = max( dot( dNdx, dNdx ), dot( dNdy, dNdy ) );
  float kernelRoughnessSq = clamp( 2.0 * geometricVariance, 0.0, 0.5 );
  material.roughness = sqrt( clamp( material.roughness * material.roughness + kernelRoughnessSq, 0.0, 1.0 ) );
  material.roughness = max( material.roughness, 0.089 );

  #ifdef USE_KHR_materials_anisotropy
    #ifdef USE_OPENPBR
      // OpenPBR energy-preserving per-axis alpha mapping ( spec § Microfacet model ):
      //   alpha_t = r^2 * sqrt( 2 / ( 1 + (1-a)^2 ) ),  alpha_b = (1-a) * alpha_t
      float openpbr_r = max( material.roughness, 0.001 );
      float openpbr_a = min( material.anisotropyStrength, 0.999 );
      material.at = openpbr_r * openpbr_r * sqrt( 2.0 / ( 1.0 + pow2( 1.0 - openpbr_a ) ) );
      material.ab = ( 1.0 - openpbr_a ) * material.at;
    #else
      float anisotropyBaseAlpha = pow2( material.roughness );
      material.at = mix( anisotropyBaseAlpha, 1.0, pow2( material.anisotropyStrength ) );
      material.ab = clamp( anisotropyBaseAlpha, 0.001, 1.0 );
    #endif
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
    float specOcclusion = clamp( pow( dotNV + ao, exp2( -16.0 * material.roughness - 1.0 ) ) - 1.0 + ao, 0.0, 1.0 );
    reflectedLight.indirectSpecular *= specOcclusion;
  #endif

  emissive_color = vec4( emissiveFactor, 1.0 );
  #ifdef USE_EMISSION_TEXTURE
    emissive_color.xyz *= SrgbToLinear( texture( emissiveTexture, vEmissionUv ).rgb );
  #endif
  #ifdef USE_KHR_materials_emissive_strength
    emissive_color.xyz *= emissiveStrength;
  #endif


  color = reflectedLight.indirectDiffuse +
  reflectedLight.indirectSpecular +
  reflectedLight.directDiffuse +
  reflectedLight.directSpecular;

  // Transmission ( OpenPBR adoption plan §3.3 + §3.4 ). The diffuse response of a
  // transmissive surface is replaced by a sample of the captured opaque scene,
  // taken where the view ray refracted into the medium leaves the slab, blurred
  // by surface roughness and absorbed along the way by the interior medium.
  //
  // Screen-space refraction, so its limits are structural: only what the opaque
  // pass drew is available ( no caustics, no transmissive-through-transmissive,
  // nothing outside the frame ), and the slab is treated as parallel-sided -
  // backfaces are not traced ( §3.3c ).
  #ifdef USE_TRANSMISSION
  {
    vec2 screenUv = gl_FragCoord.xy / uScreenSize;
    vec3 N = normal;
    vec3 V = viewDir;
    // Named apart from main()'s own occlusion-path dotNV: nested-scope
    // shadowing is legal GLSL, but two different cosines under one name are not
    // worth the reader's double-take.
    float transmissionDotNV = clamp( dot( N, V ), 0.0, 1.0 );

    // Thin-walled ( OpenPBR `geometry_thin_walled` ) : an infinitely thin shell
    // has no interior to refract through, so the ray passes straight on and no
    // volumetric path length accumulates.
    float pathLength = 0.0;

    // Roughness blur: the transmission target carries a mip chain ( built right
    // after the capture ), so a rough surface reads a pre-filtered level instead
    // of needing many taps. Computed here because the level also sets how far the
    // occluder test has to look - a blurrier tap gathers colour from further
    // away, so it can be contaminated from further away.
    float transmissionLod = transmissionBlurLod( material.roughness, transmissionIor, max( uScreenSize.x, uScreenSize.y ) );

    vec3 transmitted = textureLod( transmissionSampler, screenUv, transmissionLod ).rgb;

    #ifndef USE_TRANSMISSION_THIN_WALLED
      pathLength = transmissionTraversalLength( transmissionThickness );

      #ifdef USE_TRANSMISSION_DISPERSION
        // Chromatic dispersion ( §3.6 ). A real medium has a different index for
        // every wavelength, so the three colour channels leave the body along
        // slightly different rays and land on different parts of the scene -
        // which is the coloured fringing seen through a thick edge. Three taps,
        // one per channel, at the indices material::dispersion_iors spreads
        // around the base one ( red refracted least, blue most ).
        vec3 iors = transmissionDispersionIors( transmissionIor, transmissionDispersion );
        vec2 uvR = transmissionRefractedUv( N, V, iors.r, transmissionThickness, screenUv, transmissionLod );
        vec2 uvG = transmissionRefractedUv( N, V, iors.g, transmissionThickness, screenUv, transmissionLod );
        vec2 uvB = transmissionRefractedUv( N, V, iors.b, transmissionThickness, screenUv, transmissionLod );
        transmitted = vec3
        (
          textureLod( transmissionSampler, uvR, transmissionLod ).r,
          textureLod( transmissionSampler, uvG, transmissionLod ).g,
          textureLod( transmissionSampler, uvB, transmissionLod ).b
        );
      #else
        vec2 refrUv = transmissionRefractedUv( N, V, transmissionIor, transmissionThickness, screenUv, transmissionLod );
        transmitted = textureLod( transmissionSampler, refrUv, transmissionLod ).rgb;
      #endif
    #endif

    #ifdef USE_TRANSMISSION_ABSORPTION
      // Beer-Lambert absorption over the traversed slab ( §3.4 ).
      transmitted *= transmissionTransmittance( transmissionColor, transmissionDepth, pathLength );
    #else
      // Spec's `transmission_depth = 0`: no interior medium, `transmission_color`
      // tints the refraction by a constant amount instead.
      transmitted *= transmissionColor;
    #endif

    // Energy split at the interface: only the non-reflected share is
    // transmitted. The reflected share is already in `color` as the specular /
    // IBL response, so without this factor a grazing rim would carry both at
    // full strength.
    transmitted *= 1.0 - F_Schlick( material.f0, material.f90, transmissionDotNV );

    // OpenPBR-style: the transmissive slab replaces the diffuse response of the
    // base ( specular / IBL terms stay on top, carrying the interface reflection ).
    vec3 diffuseTerm = reflectedLight.indirectDiffuse + reflectedLight.directDiffuse;
    color = ( color - diffuseTerm ) + mix( diffuseTerm, transmitted, transmissionFactor );
  }
  #endif

  // KHR_materials_clearcoat: fresnel_mix the base result with the coat lobe. The coat's own
  // Fresnel term is applied once here (not per light / per IBL term), and the emissive
  // output is dampened by the same weight, matching the extension's "coated_emission" note.
  #ifdef USE_KHR_materials_clearcoat
    vec3 clearcoatFresnel = F_Schlick( vec3( 0.04 ), vec3( 1.0 ), clamp( dot( material.clearcoatNormal, viewDir ), 0.0, 1.0 ) );
    vec3 clearcoatWeight = clamp( material.clearcoatFactor * clearcoatFresnel, 0.0, 1.0 );
    color = mix( color, reflectedLight.clearcoatSpecular, clearcoatWeight );
    emissive_color.rgb *= ( 1.0 - clearcoatWeight );
  #endif

  // OpenPBR `fuzz` sits on top of the ( coated ) substrate: add its accumulated lobe.
  #ifdef USE_OPENPBR
    #ifdef USE_SHEEN_ALBEDO
      // Energy ( albedo ) scaling. The fuzz layer reflects a fraction
      // `max3( fuzz_color ) * E( NoV, alpha )` of what reaches it, and that
      // fraction never reaches the substrate - so the substrate is dimmed by
      // it instead of the lobe being pure gain. Same alpha clamp as the lobe
      // itself, and the same one the table was generated with.
      {
        float fuzzAlpha = clamp( material.sheenRoughness, 0.1, 1.0 );
        float fuzzNoV = clamp( dot( normal, viewDir ), 0.0, 1.0 );
        color *= clamp( 1.0 - max_value( material.sheenColorFactor ) * sheen_e( fuzzNoV, fuzzAlpha ), 0.0, 1.0 );
      }
    #endif
    color += reflectedLight.sheenSpecular;
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

