#version 300 es
precision highp float;

uniform sampler2D transparentA;
uniform sampler2D transparentB;

in vec2 vUv;
out vec4 frag_color;

void main()
{
  vec4 accum = texelFetch( transparentA, ivec2( gl_FragCoord.xy ), 0 );
  float r = 1.0 - accum.a;
  accum.a = texelFetch( transparentB, ivec2( gl_FragCoord.xy ), 0 ).r;
  frag_color = vec4( r * accum.rgb / clamp( accum.a, 1e-4, 5e4 ), r );
}