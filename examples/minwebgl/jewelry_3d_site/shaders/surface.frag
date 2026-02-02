precision mediump float;

uniform vec3 surfaceColor;
uniform sampler2D surfaceTexture;

in vec2 vUvs;
in vec3 vWorldNormal;
in vec3 vWorldPosition;

layout( location = 0 ) out vec4 frag_color;
layout( location = 1 ) out vec4 emissive_color;
layout( location = 2 ) out vec4 trasnparentA;
layout( location = 3 ) out float transparentB;

float alpha_weight( float a )
{
  return clamp( pow( min( 1.0, a * 10.0 ) + 0.01, 3.0 ) * 1e8 * pow( 1.0 - gl_FragCoord.z * 0.9, 3.0 ), 1e-2, 3e3 );
}

void main()
{
  float shadow = texture( surfaceTexture, vUvs ).r;
  vec3 finalColor = ( 1.0 - shadow ) * surfaceColor;

  float alpha = 1.0;

  emissive_color = vec4( 0.0, 0.0, 0.0, 0.0 );

  float a_weight = alpha * alpha_weight( alpha );
  trasnparentA = vec4( finalColor * a_weight, alpha );
  transparentB = a_weight;
  frag_color = vec4( finalColor, alpha );
}
