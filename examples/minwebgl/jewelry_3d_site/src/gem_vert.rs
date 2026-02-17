// Generated with Shader Minifier 1.5.1 (https://github.com/laurentlb/Shader_Minifier/)
#![ allow( dead_code ) ]
pub const VAR_NORMAL: &'static [u8] = b"B";
pub const VAR_NORMALMATRIX: &'static [u8] = b"normalMatrix";
pub const VAR_POSITION: &'static [u8] = b"V";
pub const VAR_PROJECTIONMATRIX: &'static [u8] = b"projectionMatrix";
pub const VAR_UV: &'static [u8] = b"z";
pub const VAR_VUVS: &'static [u8] = b"i";
pub const VAR_VVIEWPOSITION: &'static [u8] = b"t";
pub const VAR_VWORLDNORMAL: &'static [u8] = b"p";
pub const VAR_VWORLDPOSITION: &'static [u8] = b"C";
pub const VAR_VIEWMATRIX: &'static [u8] = b"viewMatrix";
pub const VAR_WORLDMATRIX: &'static [u8] = b"worldMatrix";
pub const INPUT: &'static [u8] = br#"
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
}"#;
// pub const INPUT: &'static [u8] = b"\
//  precision mediump float;\
//  layout(location=0)in vec3 V;\
//  layout(location=1)in vec3 B;\
//  layout(location=2)in vec2 z;\
//  uniform mat4x4 worldMatrix,viewMatrix,projectionMatrix;\
//  uniform mat3x3 normalMatrix;\
//  out vec2 i;\
//  out vec3 p,C,t;\
//  void main()\
//  {\
//    vec4 K=worldMatrix*vec4(V,1),J=viewMatrix*K;\
//    i=z;\
//    p=normalize(normalMatrix*B);\
//    C=K.xyz;\
//    t=J.xyz;\
//    gl_Position=projectionMatrix*J;\
//  }";
