// Generated with Shader Minifier 1.5.1 (https://github.com/laurentlb/Shader_Minifier/)
#![ allow( dead_code ) ]
pub const VAR_CAMERAPOSITION: &'static [u8] = b"cameraPosition\0";
pub const VAR_CUBENORMALMAP: &'static [u8] = b"m\0";
pub const VAR_DIAMONDCOLOR: &'static [u8] = b"r\0";
pub const VAR_DISTANCEATTENUATIONSPEED: &'static [u8] = b"e\0";
pub const VAR_EMISSIVE_COLOR: &'static [u8] = b"D\0";
pub const VAR_ENVMAP: &'static [u8] = b"v\0";
pub const VAR_ENVMAPINTENSITY: &'static [u8] = b"s\0";
pub const VAR_FRAG_COLOR: &'static [u8] = b"u\0";
pub const VAR_INVERSERESTMATRIX: &'static [u8] = b"l\0";
pub const VAR_INVERSEWORLDMATRIX: &'static [u8] = b"inverseWorldMatrix\0";
pub const VAR_N2: &'static [u8] = b"a\0";
pub const VAR_RADIUS: &'static [u8] = b"w\0";
pub const VAR_RAINBOWDELTA: &'static [u8] = b"c\0";
pub const VAR_RAYBOUNCES: &'static [u8] = b"n\0";
pub const VAR_RESTMATRIX: &'static [u8] = b"d\0";
pub const VAR_TRANSPARENTB: &'static [u8] = b"G\0";
pub const VAR_TRASNPARENTA: &'static [u8] = b"F\0";
pub const VAR_VUVS: &'static [u8] = b"i\0";
pub const VAR_VVIEWPOSITION: &'static [u8] = b"t\0";
pub const VAR_VWORLDNORMAL: &'static [u8] = b"p\0";
pub const VAR_VWORLDPOSITION: &'static [u8] = b"C\0";
pub const VAR_VIEWMATRIX: &'static [u8] = b"viewMatrix\0";
pub const VAR_WORLDMATRIX: &'static [u8] = b"worldMatrix\0";
pub const INPUT: &'static [u8] = b"\
 precision mediump float;\
 uniform sampler2D v;\
 uniform samplerCube m;\
 uniform mat4x4 worldMatrix,viewMatrix,inverseWorldMatrix,d,l;\
 uniform int n;\
 uniform vec3 r;\
 uniform float s,w;\
 uniform vec3 cameraPosition;\
 uniform float a,c,e;\
 in vec2 i;\
 in vec3 p,C,t;\
 layout(location=0)out vec4 u;\
 layout(location=1)out vec4 D;\
 layout(location=2)out vec4 F;\
 layout(location=3)out float G;\
 vec3 g(const float v,const vec3 z)\
 {\
   vec4 d=0.*vec4(-1,-.0275,-.572,.022)+vec4(1,.0425,1.04,-.04);\
   vec2 f=vec2(-1.04,1.04)*(min(d.x*d.x,exp2(-9.28*v))*d.x+d.y)+d.zw;\
   return z*f.x+f.y;\
 }\
 vec4 g(vec3 v)\
 {\
   vec4 d=texture(m,v);\
   d.xyz=normalize(d.xyz*2.-1.);\
   d.w*=w;\
   return d;\
 }\
 vec2 h(vec3 v)\
 {\
   float d=atan(v.z,v.x),z=asin(v.y);\
   d*=.15915;\
   z*=.3183;\
   d+=.5;\
   z+=.5;\
   return vec2(d,z);\
 }\
 vec3 A(vec3 d)\
 {\
   d=texture(v,h(mat3(viewMatrix)*d)).xyz;\
   return s*d;\
 }\
 vec3 B(vec3 v)\
 {\
   v=normalize(mat3(l)*v);\
   v.x*=-1.;\
   v.z*=-1.;\
   return A(v).xyz;\
 }\
 vec3 A(vec3 v,vec3 d)\
 {\
   float z=dot(d,d),n=2.*dot(v,d),a=n*n-4.*z*(dot(v,v)-w*w);\
   if(a>0.)\
     {\
       a=sqrt(a);\
       float m=(-n+a)/z,e=(-n-a)/z;\
       return vec3(v+d*(m>e?\
         m:\
         e));\
     }\
   return vec3(0);\
 }\
 vec3 A(vec3 v,vec3 z,vec3 d,vec3 a)\
 {\
   return z*(dot(a,d-v)/dot(a,z))+v;\
 }\
 vec3 B(vec3 v,vec3 z)\
 {\
   vec3 d=normalize(A(v,z));\
   vec4 f=g(d);\
   vec3 a=f.xyz;\
   float n=f.w;\
   d=normalize(A(v,z,d*n,-a));\
   f=g(d);\
   a=f.xyz;\
   n=f.w;\
   d*=n;\
   return A(v,z,d,-a);\
 }\
 vec3 g(vec3 v,vec3 z)\
 {\
   vec3 f=vec3(0);\
   float m=a+c,y=a-c;\
   vec3 r=vec3((a-1.)/(a+1.));\
   r*=r;\
   vec3 l=normalize(mat3x3(d)*refract(v,z,1./a)),u=(d*vec4(C,1)).xyz,i=vec3(1);\
   v=g(abs(dot(-v,z)),r);\
   i*=vec3(1)-v;\
   for(int d=0;d<n;d++)\
     {\
       vec3 z=normalize(B(u,l));\
       vec4 s=g(z);\
       vec3 c=u;\
       u=z*s.w;\
       float A=length(u-c)/w;\
       i*=exp(-A*e);\
       c=reflect(l,-s.xyz);\
       z=refract(l,-s.xyz,a);\
       if(dot(z,z)<1e-4)\
         {\
           if(d==n-1)\
             f+=B(l)*i*(vec3(1)-v);\
         }\
       else\
         {\
           vec3 v=vec3(1)-g(abs(dot(z,s.xyz)),r);\
           {\
             vec3 d=refract(l,-s.xyz,m),a=refract(l,-s.xyz,y);\
             d=vec3(B(d).x,B(z).y,B(a).z)*i*v;\
             f+=d;\
           }\
           v=g(abs(dot(c,-s.xyz)),r);\
           i*=v;\
         }\
       l=c;\
     }\
   return f;\
 }\
 vec3 E(vec3 v)\
 {\
   v=mat3x3(.59719,.076,.0284,.35458,.90834,.13383,.04823,.01566,.83777)*v;\
   return clamp(mat3x3(1.60475,-.10208,-.00327,-.53108,1.10813,-.07276,-.07367,-.00605,1.07602)*((v*(v+.0245786)-90537e-9)/(v*(.983729*v+.432951)+.238081)),vec3(0),vec3(1));\
 }\
 void main()\
 {\
   vec3 v=normalize(p),d=normalize(C-cameraPosition),z=reflect(d,v);\
   float f=(a-1.)/(a+1.);\
   f*=f;\
   vec3 n=g(dot(z,v),vec3(f));\
   d=r*(g(d,v)+n*A(z));\
   v=E(d);\
   D=vec4(v*smoothstep(.9,.91,.2126*v.x+.7152*v.y+.0722*v.z),0);\
   f=clamp(pow(min(1.,10.)+.01,3.)*1e8*pow(1.-gl_FragCoord.z*.9,3.),.01,3e3);\
   F=vec4(d*f,1);\
   G=f;\
   u=vec4(d,1);\
 }\0";