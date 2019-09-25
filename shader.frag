#version 140

uniform vec2      iResolution;           // viewport resolution (in pixels)
uniform float     iTime;                 // shader playback time (in seconds)
uniform float     iTimeDelta;            // render time (in seconds)
uniform int       iFrame;                // shader playback frame
uniform float     iChannelTime[4];       // channel playback time (in seconds)
uniform vec3      iChannelResolution[4]; // channel resolution (in pixels)
uniform vec4      iMouse;                // mouse pixel coords. xy: current (if MLB down), zw: click
//uniform sampler2D iChannel0..3;          // input channel. XX = 2D/Cube
uniform vec4      iDate;                 // (year, month, day, time in seconds)
uniform float     iSampleRate;           // sound sample rate (i.e., 44100)

in vec2 fragCoord;
out vec4 fragColor;

const float PRECISION = 0.00001;
const int MAX_LOOPS = 256;
const float PI = 3.141593;


float plane(vec3 p)
{
  return p.y + length(sin(p.xz + iTime));
}

float sphere(vec3 p)
{
  return length(p) - 2.0;
}

void main() {
  float rot = iTime * 0 + PI;
  vec2 uv = vec2(
      fragCoord.x * cos(rot) + fragCoord.y * sin(rot),
      fragCoord.x * sin(rot) + fragCoord.y * -cos(rot)
      );

  // camera origin
  vec3 p = vec3(sin(iTime) * 4.0, 5.0, -10.0);
  // direction
  vec3 dir = normalize(vec3(uv, 1.0));


  // background color
  fragColor = vec4(fract(uv), 0.0, 1.0);

  for (int i=0; i<MAX_LOOPS; i++)
  {
    float d = min(plane(p), sphere(p));

    if (d < PRECISION)
    {
      fragColor = vec4(fract(p), 1.0);
      break;
    }
    p += d * dir;
  }

}

