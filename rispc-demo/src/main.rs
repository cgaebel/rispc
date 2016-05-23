extern crate time;
#[macro_use] extern crate rispcrt;

use std::cmp;
use std::fs::File;
use std::io::Write;

ispc_module!(mandel);

fn mandel(c_re: f32, c_im: f32, count: u32) -> u32 {
  let mut z_re = c_re;
  let mut z_im = c_im;

  let mut i = 0;

  while i < count {
    if z_re * z_re + z_im * z_im > 4.0f32 { break }

    let new_re = z_re*z_re - z_im*z_im;
    let new_im = 2.0f32 * z_re*z_im;
    z_re = c_re + new_re;
    z_im = c_im + new_im;

    i += 1;
  }

  i
}

#[inline(never)]
fn mandelbrot_serial(x0: f32, y0: f32, x1: f32, y1: f32, w: u32, h: u32, max_iters: u32, output: *mut u32) {
  let dx = (x1 - x0) / (w as f32);
  let dy = (y1 - y0) / (h as f32);

  for j in 0..h {
    for i in 0..w {
      let x = x0 + (i as f32) * dx;
      let y = y0 + (j as f32) * dy;

      let idx = j*w + i;

      unsafe { *output.offset(idx as isize) = mandel(x, y, max_iters); }
    }
  }
}

fn t_ns() -> u64 { time::precise_time_ns() }

fn write_ppm(buf: &[u32], w: u32, h: u32, file: &str) -> Result<(), std::io::Error> {
  {
    let mut f = try!(File::create(file));
    try!(writeln!(f, "P6"));
    try!(writeln!(f, "{} {}", w, h));
    try!(writeln!(f, "255"));
    for i in 0..w*h {
      let c = if buf[i as usize] & 0x01 > 0 { 240 } else { 20 };
      try!(f.write(&[ c, c, c ]));
    }
  }

  Ok(())
}

fn main() {
  let w = 768;
  let h = 512;

  let x0 = -2.0f32;
  let x1 = 1.0f32;
  let y0 = -1.0f32;
  let y1 = 1.0f32;

  let max_iters = 256;
  let mut buf: Vec<u32> = Vec::new();
  buf.resize((w as usize)*(h as usize), 0);

  let bp: *mut u32 = buf.as_mut_ptr();

  let mut min_ispc = 1_000_000_000_000;

  for _ in 0..3 {
    let t0 = t_ns();
    unsafe { mandel::mandelbrot_ispc(x0, y0, x1, y1, w, h, max_iters, bp) };
    let dt = t_ns() - t0;
    min_ispc = cmp::min(min_ispc, dt);
  }

  println!("[mandelbrot ispc]:\t\t[{:.3}] million cycles", min_ispc as f64 / 1000.0f64);
  write_ppm(&*buf, w, h, "mandelbrot-ispc.ppm").unwrap();

  let mut min_serial = 1_000_000_000_000;

  for _ in 0..3 {
    let t0 = t_ns();
    mandelbrot_serial(x0, y0, x1, y1, w, h, max_iters, bp);
    let dt = t_ns() - t0;
    min_serial = cmp::min(min_serial, dt);
  }

  println!("[mandelbrot serial]:\t\t[{:.3}] million cycles", min_serial as f64 / 1000.0f64);
  write_ppm(&*buf, w, h, "mandelbrot-serial.ppm").unwrap();

  println!("\t\t\t\t({:.2}x speedup from ISPC)", (min_serial as f64) / (min_ispc as f64));
}
