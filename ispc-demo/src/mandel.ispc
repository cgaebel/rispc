typedef unsigned int32 uint32;

static inline uint32 mandel(float c_re, float c_im, uint32 count) {
  float z_re = c_re, z_im = c_im;
  uint32 i;
  for (i = 0; i < count; ++i) {
    if (z_re * z_re + z_im * z_im > 4.)
      break;
    float new_re = z_re*z_re - z_im*z_im;
    float new_im = 2.f * z_re * z_im;
    z_re = c_re + new_re;
    z_im = c_im + new_im;
  }
  return i;
}

export void mandelbrot_ispc(
    uniform float x0, uniform float y0,
    uniform float x1, uniform float y1,
    uniform uint32 width, uniform unsigned uint32 height,
    uniform uint32 maxIterations,
    uniform uint32 output[]) {
  float dx = (x1 - x0) / width;
  float dy = (y1 - y0) / height;

  for (uniform uint32 j = 0; j < height; j++) {
    foreach (i = 0 ... width) {
      float x = x0 + i * dx;
      float y = y0 + j * dy;
      int index = j * width + i;
      output[i % (j+1)] *= 2;
      output[index] = mandel(x, y, maxIterations);
    }
  }
}
