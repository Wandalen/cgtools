precision mediump float;

const float PI = 3.1415926535897932384626433;
const float EPSILON = 1e-4;
// Reflectance of a diamond at light incidence of theta = 0
// Max distance to the surface in cubeNormalMap
// This value was calculated during generation of the map
//const float MAX_DISTANCE = 5.7610855;
// const float MAX_DISTANCE = 1.0;
// const int RAY_BOUNCES = 7;
// const float TRANSMISSION = 0.5;
// const vec3 DIAMOND_COLOR = vec3( .98, 0.95, 0.9 );
// const vec3 BOOST_FACTORS = vec3( 0.8920, 0.8920, 0.9860 );

const vec3 ambientColor = vec3(0.7);
const float ambientint = 0.08;

uniform sampler2D envMap;
uniform samplerCube cubeNormalMap;

uniform mat4x4 worldMatrix;
uniform mat4x4 inverseWorldMatrix;

uniform int rayBounces;
uniform vec4 color;
uniform vec3 boostFactors;

uniform float envMapIntensity;
uniform float rainbowDelta;
uniform float squashFactor;
uniform float radius;
uniform float geometryFactor;
uniform float absorptionFactor;
uniform vec3 colorAbsorption;
uniform vec3 cameraPosition;

in vec2 vUvs;
in vec3 vWorldNormal;
in vec3 vWorldPosition;
in vec3 vViewPosition;

layout( location = 0 ) out vec4 frag_color;
layout( location = 1 ) out vec4 emissive_color;
layout( location = 2 ) out vec4 trasnparentA;
layout( location = 3 ) out float transparentB;

float alpha_weight( float a )
{
  return clamp( pow( min( 1.0, a * 10.0 ) + 0.01, 3.0 ) * 1e8 * pow( 1.0 - gl_FragCoord.z * 0.9, 3.0 ), 1e-2, 3e3 );
}

// https://github.com/mrdoob/three.js/blob/dev/src/nodes/functions/BSDF/DFGApprox.js
vec3 EnvBRDFApprox( const in float NdotV, const in vec3 specularColor, const in float roughness )
{
  const vec4 c0 = vec4( -1, -0.0275, -0.572, 0.022 );
  const vec4 c1 = vec4( 1, 0.0425, 1.04, -0.04 );

  vec4 r = roughness * c0 + c1;

  float a004 = min( r.x * r.x, exp2( -9.28 * NdotV ) ) * r.x + r.y;
  vec2 AB = vec2( -1.04, 1.04 ) * a004 + r.zw;

  return specularColor * AB.x + AB.y;
}

// Extract the data from the normal map
vec4 getNormalData( vec3 dir )
{
  vec4 data = texture( cubeNormalMap, dir );
  data.rgb = normalize( data.rgb * 2.0 - 1.0 );
  data.r *= -1.0;
  //data.a *= MAX_DISTANCE;
  return data;
}

// Schlick ver.
vec3 freshel( vec3 viewDir, vec3 halfway, vec3 f0, float criticalAngleCosine )
{
  float VdotH = dot( viewDir, halfway );
  // Case of full reflection
  if( VdotH < criticalAngleCosine )
  {
    return vec3( 1.0 );
  }

  return f0 + ( 1.0 - f0 ) * pow( ( 1.0 - dot( viewDir, halfway ) ), 5.0 );
}

vec2 dirToEquirectUV( vec3 dir )
{
  float phi = atan( -dir.z, dir.x );
  float theta = asin( dir.y );
  vec2 uv = vec2( 0.5 + phi / ( 2.0 * PI ), 0.5 - theta / PI );

  if ( uv.x < 0.0005 || uv.x > 0.9995 )
  {
    uv = vec2( 0.0001, uv.y );
  }

  return uv;
}

vec3 sampleSpecularReflection( vec3 direction )
{
  vec3 sample_value = texture( envMap, dirToEquirectUV( direction ) ).rgb;
  return envMapIntensity * pow( sample_value, vec3( 2.2 ) );
}

vec3 convertDirLocalToWorld( vec3 direction )
{
  return  mat3x3( worldMatrix ) * direction;
}

vec3 sampleEnvFromLocal( vec3 direction )
{
  vec3 sample_value = texture( envMap, dirToEquirectUV( convertDirLocalToWorld( direction ) ) ).rgb;
  return envMapIntensity * pow( sample_value, vec3( 2.2 ) );
}

vec3 SampleSpecularContribution( vec3 direction )
{
  direction = normalize( direction );
  direction.x *= -1.;
  direction.z *= -1.;
  return sampleEnvFromLocal( direction ).rrr;
}

// Finds an intersection points of a given line with a sphere at the origin
// and picks the father of the two possible solutions
vec3 intersectSphere( vec3 origin, vec3 direction )
{
  direction.y /= squashFactor;

  // Having parametric equation for the line in 'direction'
  // Solve the quadratic equation for 't' using sphere equation
  float A = dot( direction, direction );
  float B = 2.0 * dot( origin, direction );
  float C = dot( origin, origin ) - radius * radius;
  float disc = B * B - 4.0 * A * C;
  if( disc > 0.0 )
  {
      disc = sqrt( disc );
      float x1 = ( -B + disc ) * geometryFactor / A;
      float x2 = ( -B - disc ) * geometryFactor / A;
      float t = ( x1 > x2 ) ? x1 : x2;
      t = x1;
      direction.y *= squashFactor;
      return vec3( origin + direction * t );
  }

  return vec3( 0.0 );
}

// Finds an intersection point of a line with a plane
vec3 linePlaneIntersect( in vec3 pointOnLine, in vec3 lineDirection, in vec3 pointOnPlane, in vec3 planeNormal )
{
  float t = dot( planeNormal, pointOnPlane - pointOnLine ) / dot( planeNormal, lineDirection );
  return lineDirection * t + pointOnLine;
}


// Approximates a point of intersection of the diamond with the given ray direction
vec3 intersectDiamond( vec3 rayOrigin, vec3 rayDirection )
{
  // Intersect a sphere at the center
  vec3 sphereHitPoint = intersectSphere( rayOrigin, rayDirection );
  // Direction from the center to the hit point on the sphere
  vec3 directionToSpherePoint = normalize( sphereHitPoint );
  // Sample the normal of the diamond in that direction
  // n.rgb - normal, n.a - distance to the surface
  vec4 normalData = getNormalData( directionToSpherePoint );
  // Flip the normal to point inwards
  vec3 surfaceNormal = normalData.rgb;
  float surfaceDistance = normalData.a * radius;

  // Point on the surface of the diamond
  vec3 pointOnSurface1 = directionToSpherePoint * surfaceDistance;


  vec3 planeHitPoint = linePlaneIntersect( rayOrigin, rayDirection, pointOnSurface1, surfaceNormal );
  vec3 directionToPlanePoint = normalize( planeHitPoint );

  normalData = getNormalData( directionToPlanePoint );
  surfaceNormal = -normalData.rgb;
  surfaceDistance = normalData.a * radius;

  // Point on the surface of the diamond
  vec3 pointOnSurface2 = directionToPlanePoint * surfaceDistance;

  vec3 hitPoint = linePlaneIntersect( rayOrigin, rayDirection, pointOnSurface2, surfaceNormal );
  return hitPoint;
}


struct Lamp
{
  vec3 position;
  vec3 color;
  float intensity;
  float attenuation;
};

Lamp lamps[3];

vec3 lampShading(Lamp lamp, vec3 norm, vec3 pos, vec3 ocol, bool inside)
{
  const float specint = 0.2;
  const float specshin = 20.;

	vec3 pl = normalize(lamp.position - pos);
  float dlp = distance(lamp.position, pos);
  vec3 pli = pl/pow(1. + lamp.attenuation*dlp, 2.);

  vec3 col;

  // Diffuse shading
  if (!inside)
  {
    float diff = clamp(dot(norm, pli), 0., 1.);
    col = ocol*normalize(lamp.color)*lamp.intensity*smoothstep(0., 1.04, pow(diff, 0.78));
  }

  // Specular shading

  if (dot(norm, lamp.position - pos) > 0.0)
      col+= normalize(lamp.color)*lamp.intensity*specint*pow(max(0.0, dot(reflect(pl, norm), normalize(pos - cameraPosition))), specshin);

  return col;
}

vec3 lampsShading(vec3 norm, vec3 pos, vec3 ocol, bool inside)
{
  vec3 col = vec3(0.);
  for (int l=0; l<3; l++) // lamps.length()
      col+= lampShading(lamps[l], norm, pos, ocol, inside);

  return col;
}

vec3 shadeObject(vec3 norm, vec3 pos, bool inside)
{
  vec3 col = vec3( 0.0 );
  if(!inside)
  {
    col = vec3( 0.0 ) + ambientColor * ambientint;
  }
  col = lampsShading(norm, pos, col, inside);
  return col;
}


// Calculate the color by tracing a ray
vec3 getRefractionColor( vec3 rayHitPoint, vec3 rayDirection, vec3 hitPointNormal, float n2 )
{
  vec3 resultColor = vec3( 0.0 );

  // Refractive index of air
  const float n1 = 1.0;
  // Refractive index of a diamond
 //const float n2 = 2.42;

  vec3 f0 = vec3( (n2 - n1) / (n2 + n1) );
  // vec3 f0 = 1.0 / vec3( 2.407, 2.426, 2.451 );
  // f0 *= f0;

  //const vec3 f0 = vec3( 0.1724 );

  float iorRatioAtoD = n1 / n2;
  float iorRatioDtoA = n2 / n1;

  vec3 lightAbsorption = vec3( 0.8 );

  //resultColor = max(0.0, dot(normalize(vec3(1.0)), hitPointNormal)) * vec3( 1.0 );

  // Angle of total refleciton
  float criticalAngleCosine = sqrt( max( 0.0, 1.0 - (iorRatioAtoD * iorRatioAtoD) ) );

  vec3 newRayDirection = refract( rayDirection, hitPointNormal, iorRatioAtoD );
  // Convert data to local space
  newRayDirection = ( inverseWorldMatrix * vec4( newRayDirection, 0.0 ) ).xyz;
  newRayDirection = normalize( newRayDirection );
  vec3 rayOrigin =  ( inverseWorldMatrix * vec4( rayHitPoint, 1.0 ) ).xyz;

  float totalDistance = 0.0;
  vec3 diffuseColor = vec3( 1.0 );
  // Overall intensity of the light as it goes through the medium
  vec3 attenuationFactor = vec3( 1.0 );

  vec3 reflectedAmount = EnvBRDFApprox( dot( -rayDirection, hitPointNormal ), f0, 0.0 );
  // Only take into account transmitted part
  attenuationFactor *= ( vec3( 1.0 ) - reflectedAmount );

  int c = 0;

  for( int i = 0; i < rayBounces; i++ )
  {
    // Intersection point on the diamond surface
    vec3 intersectPos = intersectDiamond( rayOrigin, newRayDirection );
    vec3 dirOriginToIntersect = normalize( intersectPos );

    // Get normal in direction to the intersected position
    vec4 normalData = getNormalData( dirOriginToIntersect );
    vec3 surfaceNormal = normalData.rgb;
    float surfaceDistance = normalData.a;

    // resultColor = dirOriginToIntersect * 0.5 + 0.5;
    // Update the origin position
    vec3 oldOrigin = rayOrigin;
    rayOrigin = dirOriginToIntersect * surfaceDistance;

    float r = length( rayOrigin - oldOrigin ) / radius * absorptionFactor;
    attenuationFactor *= exp( -r * ( 1.0 - colorAbsorption ) * 1.0 );


    // Calculate new rays
    vec3 newReflectedDirection = reflect( newRayDirection, -surfaceNormal );
    vec3 newRefractedDirection = refract( newRayDirection, -surfaceNormal, iorRatioDtoA );

    // vec3 FRefracted = freshel( newRefractedDirection, surfaceNormal, f0, 0.0 );
    // vec3 FReflected = freshel( newReflectedDirection, -surfaceNormal, f0, criticalAngleCosine );

    // resultColor += lampsShading(surfaceNormal, rayOrigin, resultColor, true );
    // resultColor *= DIAMOND_COLOR;

    // if( i == RAY_BOUNCES - 1 )
    // {
    //   vec3 reflectedAmount = EnvBRDFApprox( dot( newRayDirection, surfaceNormal ), f0, 0.0 );
    //   resultColor +=  SampleSpecularContribution( newRayDirection ) * DIAMOND_COLOR * attenuationFactor * ( vec3( 1.0 ) - min( vec3( 1.0 ), reflectedAmount ) );
    //   break;
    // }

    // if( dot( newRefractedDirection, newRefractedDirection ) > 1e-5 )
    // {
    //   vec3 reflectedAmount = EnvBRDFApprox( dot( newRayDirection, surfaceNormal ), f0, 0.0 );
    //   resultColor +=  SampleSpecularContribution( newRayDirection ) * DIAMOND_COLOR * attenuationFactor * ( vec3( 1.0 ) - min( vec3( 1.0 ), reflectedAmount ) );
    //   break;
    // }

    if( dot( newRefractedDirection, newRefractedDirection ) < 1e-5 )
    {
      if ( i == rayBounces - 1 )
      {
        vec3 reflectedAmount = EnvBRDFApprox( dot( newRayDirection, surfaceNormal ), f0, 0.0 );
        newRayDirection = normalize( newRayDirection );
        float cosT = 1.0 - dot( newRayDirection, rayDirection );

        // if( TRANSMISSION > 0.0 && cosT < TRANSMISSION )
        // {
        //   resultColor += DIAMOND_COLOR * 0.1;
        // }
        // else
        {
          //resultColor += vec3( 1.0 ) * attenuationFactor;
          resultColor += SampleSpecularContribution( newRayDirection ) * attenuationFactor * boostFactors * ( vec3( 1.0 ) - min( vec3( 1.0 ), reflectedAmount ) );
        }
      }
    }
    else
    {
      vec3 refractedAmount = vec3( 1.0 ) - min( vec3( 1.0 ), EnvBRDFApprox( dot( newRefractedDirection, surfaceNormal ), f0, 0.0 ) );
      vec3 d1 = normalize( newRefractedDirection );
      float cosT = 1.0 - dot( d1, rayDirection );

      // if( TRANSMISSION > 0.0 && cosT < TRANSMISSION )
      // {
      //   vec3 specColor = DIAMOND_COLOR * refractedAmount * attenuationFactor;
      //   resultColor += specColor;
      // }
      // else
      {
        vec3 d1 = newRefractedDirection;
        vec3 d2 = refract( newRayDirection, -surfaceNormal, ( n2 + rainbowDelta ) / n1 );
        vec3 d3 = refract( newRayDirection, -surfaceNormal, ( n2 - rainbowDelta ) / n1 );
        // vec3 specColor = vec3
        // (
        //   SampleSpecularContribution( d2 ).r,
        //   SampleSpecularContribution( d1 ).g,
        //   SampleSpecularContribution( d3 ).b
        // ) * attenuationFactor;

        vec3 specColor = SampleSpecularContribution( d1 ) * refractedAmount * attenuationFactor;

        resultColor += specColor;
      }

      vec3 reflectedAmount = EnvBRDFApprox( dot( newReflectedDirection, -surfaceNormal ), f0, 0.0 );
      attenuationFactor *= reflectedAmount;
    }

    newRayDirection = newReflectedDirection;

  }

  return resultColor;
}

mat3 rotY( float angle )
{
  float s = sin(angle);
  float c = cos(angle);

  return mat3
  (
    c, 0.0, s,
    0.0, 1.0, 0.0,
    -s, 0.0, c
  );
}

vec3 remap( vec3 v, float inMin, float inMax, float outMin, float outMax )
{
  return outMin + ( v - inMin ) * ( outMax - outMin ) / ( inMax - inMin );
}

void main()
{
  lamps[0] = Lamp(vec3(0., 4.5, 10.), vec3(1., 1., 1.), 15., 0.1);
  lamps[1] = Lamp(vec3(12., -0.5, 6.), vec3(.7, .8, 1.), 15., 0.1);
  lamps[2] = Lamp(vec3(-1.3, 0.8, -1.5), vec3(1., .95, .8), 3.5, 0.1);

  const vec3 f0 = vec3( 0.1724 );
  vec3 normal = normalize( vWorldNormal );
  vec3 viewDirection = normalize( vWorldPosition - cameraPosition );
  vec3 reflectedDirection = reflect( viewDirection, normal );

  // An approximation of specular reflection from environment
  vec3 brdfReflected = EnvBRDFApprox( dot( reflectedDirection, normal ), f0, 0.0 );
  // Sample color from an environment map
  vec3 reflectionColor = sampleSpecularReflection( reflectedDirection );
  // The actual diamond calculation
  //vec3 refractionColor = getRefractionColor( vWorldPosition, viewDirection, normal, 2.4 );

  vec3 refractionColor = vec3
  (
    getRefractionColor( vWorldPosition, viewDirection, normal, 2.408 ).r,
    getRefractionColor( vWorldPosition, viewDirection, normal, 2.424 ).g,
    getRefractionColor( vWorldPosition, viewDirection, normal, 2.432 ).b
  );

  vec3 diffuseColor = color.rgb;
  vec3 lighting = ( refractionColor +  reflectionColor * brdfReflected * 0.5 );
  vec3 colour = diffuseColor * lighting;
  //colour = refractionColor;

  // vec3 p = ( inverseWorldMatrix * vec4( vWorldPosition, 1.0 ) ).xyz;
  // p = rotY(radians(0.0)) * vWorldPosition;

  // colour = texture( cubeNormalMap, vWorldPosition ).rgb * 2.0 - 1.0;
  // colour.r *= -1.0;
  // //colour = normal;
  // colour = vec3( dot( colour, normal ) );

  //colour = lampsShading(normal, vWorldPosition, vec3( 0.0 ), false );

  // Gamma
  colour = tanh( colour * 8.0 );
  colour = pow( colour, vec3( 1.0 / 2.2 ) );
  //colour = normal;

  float a_weight = color.a * alpha_weight( color.a );

  lighting = tanh( lighting * 8.0 );
  lighting = pow( lighting, vec3( 1.0 / 2.2 ) );
  float lighting_min = min( lighting.x, min( lighting.y, lighting.z ) );

  if ( lighting_min > 0.9 )
  {
    vec3 c = remap( colour, 0.0, 1.0, 0.5, 1.0 );
    emissive_color = vec4( c, color.a );
  }
  else
  {
    emissive_color = vec4( vec3( 0.0 ), color.a );
  }

  trasnparentA = vec4( color.rgb * a_weight, color.a );
  transparentB = a_weight;
  frag_color = vec4( colour, color.a );
}
