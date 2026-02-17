// Generated with Shader Minifier 1.5.1 (https://github.com/laurentlb/Shader_Minifier/)
precision mediump float;
layout(location=0)in vec3 V;
layout(location=1)in vec3 B;
layout(location=2)in vec2 z;
uniform mat4x4 worldMatrix,viewMatrix,projectionMatrix;
uniform mat3x3 normalMatrix;
out vec2 i;
out vec3 p,C,t;
void main()
{
  vec4 K=worldMatrix*vec4(V,1),J=viewMatrix*K;
  i=z;
  p=normalize(normalMatrix*B);
  C=K.xyz;
  t=J.xyz;
  gl_Position=projectionMatrix*J;
}
