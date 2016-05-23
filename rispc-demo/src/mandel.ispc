typedef unsigned int32 uint32;

static inline uint32 mandel(float c_re, float c_im, uint32 count) {
  float z_re = c_re, z_im = c_im;
  uint32 i;
  for (i = 0; i < count; ++i) {
    if (z_re * z_re + z_im * z_im > 4.)
      break;
    float new_re = z_re*z_re - z_im*z_im;
    float new_im = 2.f * z_re * z_im;
    unmasked {
      z_re = c_re + new_re;
      z_im = c_im + new_im;
    }
  }
  return i;
}

static task void mandelbrot_scanline(
    uniform float x0, uniform float dx,
    uniform float y0, uniform float dy,
    uniform uint32 width, uniform uint32 height,
    uniform uint32 xspan, uniform uint32 yspan,
    uniform uint32 maxIterations, uniform uint32 output[]) {
  const uniform uint32 xstart = taskIndex0 * xspan;
  const uniform uint32 xend   = min(xstart + xspan, width);
  const uniform uint32 ystart = taskIndex1 * yspan;
  const uniform uint32 yend   = min(ystart + yspan, height);

  foreach(yi = ystart ... yend, xi = xstart ... xend) {
    float x = x0 + xi * dx;
    float y = y0 + yi * dy;
    uint32 index = yi * width + xi;
    output[index] = mandel(x, y, maxIterations);
  }
}

export void mandelbrot_ispc(
    uniform float x0, uniform float y0,
    uniform float x1, uniform float y1,
    uniform uint32 width, uniform unsigned uint32 height,
    uniform uint32 maxIterations,
    uniform uint32 output[]) {
  uniform float dx = (x1 - x0) / width;
  uniform float dy = (y1 - y0) / height;

  const uniform uint32 xspan = max(32, programCount*2);
  const uniform uint32 yspan = 16;

  launch [width/xspan, height/yspan]
    mandelbrot_scanline(x0, dx, y0, dy, width, height, xspan, yspan, maxIterations, output);
}
