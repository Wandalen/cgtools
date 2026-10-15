#version 300 es

precision highp float;
precision highp int;

// 0 is the "nothing pickable here" sentinel: background and occluder-only meshes.
uniform uint pickId;

layout ( location = 0 ) out uint outId;

void main()
{
  outId = pickId;
}
