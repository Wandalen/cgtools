#version 300 es
precision highp float;

uniform vec3 u_color;
uniform float u_width;

in vec2 vUv;
in vec3 vViewPos;
in vec3 vViewA;
in vec3 vViewB;

out vec4 frag_color;

float capIntersect( in vec3 ro, in vec3 rd, in vec3 pa, in vec3 pb, in float ra )
{
  vec3  ba = pb - pa;
  vec3  oa = ro - pa;
  float baba = dot(ba,ba);
  float bard = dot(ba,rd);
  float baoa = dot(ba,oa);
  float rdoa = dot(rd,oa);
  float oaoa = dot(oa,oa);
  float a = baba      - bard*bard;
  float b = baba*rdoa - baoa*bard;
  float c = baba*oaoa - baoa*baoa - ra*ra*baba;
  float h = b*b - a*c;
  if( h >= 0.0 )
  {
      float t = (-b-sqrt(h))/a;
      float y = baoa + t*bard;
      // body
      if( y>0.0 && y<baba ) return t;
      // caps
      vec3 oc = (y <= 0.0) ? oa : ro - pb;
      b = dot(rd,oc);
      c = dot(oc,oc) - ra*ra;
      h = b*b - c;
      if( h>0.0 ) return -b - sqrt(h);
  }
  return -1.0;
}

void main()
{
  float d = capIntersect( vec3( 0.0 ), normalize( vViewPos ), vViewA, vViewB, u_width / 2.0 );
  if( d == -1.0 ) { discard; }

  vec3 col = u_color; 
  frag_color = vec4( col, 1.0 );
}
