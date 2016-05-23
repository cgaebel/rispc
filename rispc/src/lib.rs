//! A library for build scripts to compile [Intel SPMD (ispc)](http://ispc.github.io) code.
//!
//! This library is intended to be used as a `build-dependencies` entry in
//! `Cargo.toml`, with a runtime component added to the `dependencies`:
//!
//! ```toml
//! [build-dependencies]
//! rispc = "0.1"
//!
//! [dependencies]
//! rispcrt = "0.1"
//! ```
//!
//! The purpose of this crate is to provide the utility functions necesasry to
//! compile ispc code into a static archive which is then linked into a Rust
//! crate. The top-level `compile_library` function serves as a convenience and
//! more advanced configuration is available through the `Config` builder.
//!
//! This crate will automatically detect situations such as cross compilation
//! or other environment variables set by Cargo and will build code appropriately.
//!
//! # Examples
//!
//! Use the default configuration:
//!
//! ```no_run
//! // build.rs
//! extern crate rispc;
//!
//! fn main() {
//!   rispc::compile_library("libmandelbrot.a", &[ "src/mandelbrot.ispc" ]);
//! }
//! ```
//!
//! Use more advanced configuration:
//!
//! ```no_run
//! // build.rs
//! extern crate rispc;
//!
//! fn main() {
//!   rispc::Config::new()
//!     .file("src/mandelbrot.ispc")
//!     .define("FOO", Some("bar"))
//!     .math_lib(ispc::Math::Fast)
//!     .enable_fast_math(true)
//!     .addressing(ispc::Addr::A64)
//!     .compile("libmandelbrot.a");
//! }
//! ```
#![allow(non_camel_case_types)]
#![deny(missing_docs)]

extern crate gcc;

use std::{cmp, fs, io};
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
struct Tool {
  path: PathBuf,
  args: Vec<OsString>,
  envs: Vec<(OsString, OsString)>,
}

impl Tool {
  fn new(path: PathBuf) -> Tool {
    Tool {
      path: path,
      args: vec![],
      envs: vec![],
    }
  }

  fn arg(&mut self, s: &str) -> &mut Self {
    self.args.push(s.into());
    self
  }

  fn to_command(&self) -> Command {
    let mut cmd = Command::new(&self.path);
    cmd.args(&self.args);
    for &(ref k, ref v) in self.envs.iter() {
      cmd.env(k, v);
    }
    cmd
  }
}

/// An addressing scheme. By default, `ispc` uses 32-bit addressing. If your
/// Arrays grow to more than `2^32` elements, this will need to be changed to
/// 64-bit.
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum Addr {
  /// 32-bit addressing
  A32,
  /// 64-bit addressing
  A64
}

/// The architecture to target. This will generally be autodetected from Cargo's
/// current target, but may be changed manually to either x86 or x86_64 systems.
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum Arch {
  /// x86
  X86,
  /// x86-64
  X86_64
}

/// The CPU families one may generate code for. This will generally be
/// autodetected based on the current set of ispc targets, but may be manually
/// overridden.
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum Cpu {
  /// generic
  Generic,
  /// atom/bonnell
  Atom,
  /// core2
  Core2,
  /// penryn
  Penryn,
  /// corei7/nehalem
  Corei7,
  /// corei7-avx/sandybridge
  Corei7_avx,
  /// core-avx-i/ivybridge
  Core_avx_i,
  /// core-avx2/haswell
  Core_avx2,
  /// broadwell
  Broadwell,
  /// slm/silvermont
  Slm,
}

impl Cpu {
  fn to_str(self) -> &'static str {
    use Cpu::*;
    match self {
      Generic    => "generic",
      Atom       => "atom",
      Core2      => "core2",
      Penryn     => "penryn",
      Corei7     => "corei7",
      Corei7_avx => "corei7-avx",
      Core_avx_i => "core-avx-i",
      Core_avx2  => "core-avx2",
      Broadwell  => "broadwell",
      Slm        => "slm",
    }
  }
}

/// Selects which math libraries to call out to.
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum Math {
  /// Use ispc's built-in math functions
  Default,
  /// Use high-performance but lower-accuracy math functions
  Fast,
  /// Use the Intel SVML math libraries. You will need to link in this
  /// dependency yourself.
  Svml,
  /// Use the system's math libraries. *WARNING: May be quite slow*.
  System,
}

impl Math {
  fn to_str(self) -> &'static str {
    use Math::*;
    match self {
      Default => "default",
      Fast    => "fast",
      Svml    => "svml",
      System  => "system",
    }
  }
}

/// Selects which target ISA(s) and the lane width(s) to generate code for.
///
/// Only one width per ISA may be selected.
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum Target {
  /// SSE2, auto-detect lane width
  Sse2,
  /// SSE2, x4 `i32`s processed at once
  Sse2_i32x4,
  /// SSE2, x8 `i32`s processed at once
  Sse2_i32x8,

  /// SSE4, auto-detect lane width
  Sse4,
  /// SSE4, x4 `i32`s processed at once
  Sse4_i32x4,
  /// SSE4, x8 `i32`s processed at once
  Sse4_i32x8,
  /// SSE4, x8 `i16`s processed at once
  Sse4_i16x8,
  /// SSE4, x16 `i8`s processed at once
  Sse4_i8x16,

  /// AVX 1.0, auto-detect lane width
  Avx1,
  /// AVX 1.0, x4 `i32`s processed at once
  Avx1_i32x4,
  /// AVX 1.0, x8 `i32`s processed at once
  Avx1_i32x8,
  /// AVX 1.0, x16 `i32`s processed at once
  Avx1_i32x16,
  /// AVX 1.0, x4 `i64`s processed at once
  Avx1_i64x4,

  /// AVX 1.1, auto-detect lane width
  Avx1_1,
  /// AVX 1.1, x8 `i32`s processed at once
  Avx1_1_i32x8,
  /// AVX 1.1, x16 `i32`s processed at once
  Avx1_1_i32x16,
  /// AVX 1.1, x4 `i64`s processed at once
  Avx1_1_i64x4,

  /// AVX 2.0, auto-detect lane width
  Avx2,
  /// AVX 2.0, x8 `i32`s processed at once
  Avx2_i32x8,
  /// AVX 2.0, x16 `i32`s processed at once
  Avx2_i32x16,
  /// AVX 2.0, x4 `i64`s processed at once
  Avx2_i64x4,
}

impl Target {
  fn to_str(self) -> &'static str {
    use Target::*;
    match self {
      Sse2       => "sse2",
      Sse2_i32x4 => "sse2-i32x4",
      Sse2_i32x8 => "sse2-i32x8",

      Sse4       => "sse4",
      Sse4_i32x4 => "sse4-i32x4",
      Sse4_i32x8 => "sse4-i32x8",
      Sse4_i16x8 => "sse4-i16x8",
      Sse4_i8x16 => "sse4-i8x16",

      Avx1        => "avx1",
      Avx1_i32x4  => "avx1-i32x4",
      Avx1_i32x8  => "avx1-i32x8",
      Avx1_i32x16 => "avx1-i32x16",
      Avx1_i64x4  => "avx1-i64x4",

      Avx1_1        => "avx1.1",
      Avx1_1_i32x8  => "avx1.1-i32x8",
      Avx1_1_i32x16 => "avx1.1-i32x16",
      Avx1_1_i64x4  => "avx1.1-i64x4",

      Avx2        => "avx2",
      Avx2_i32x8  => "avx2-i32x8",
      Avx2_i32x16 => "avx2-i32x16",
      Avx2_i64x4  => "avx2-i64x4",
    }
  }
}

/// Extra configuration to pass to `ispc`.
#[derive(PartialEq, Eq, Hash, Debug)]
pub struct Config {
  addressing: Option<Addr>,
  architecture: Option<Arch>,
  cpu: Option<Vec<Cpu>>,
  definitions: Vec<(String, Option<String>)>,
  force_alignment: Option<u32>,
  debug: Option<bool>,
  math_lib: Math,
  files: Vec<PathBuf>,
  opt_level: Option<u32>,
  assertations: bool,
  fma: bool,
  loop_unroll: bool,
  fast_masked_vload: bool,
  fast_math: bool,
  force_aligned_memory: bool,
  pic: Option<bool>,
  targets: Option<Vec<Target>>,
  werror: bool,
  warnings: bool,
  wperf: bool,
}

impl Config {
  /// Constructs a new instance of a blank set of configuration.
  ///
  /// The builder is finished with the `compile` function.
  pub fn new() -> Config {
    Config {
      addressing: None,
      architecture: None,
      cpu: None,
      definitions: vec![],
      force_alignment: None,
      debug: None,
      math_lib: Math::Default,
      files: vec![],
      opt_level: None,
      assertations: true,
      fma: true,
      loop_unroll: true,
      fast_masked_vload: false,
      fast_math: false,
      force_aligned_memory: false,
      pic: None,
      targets: None,
      werror: true,
      warnings: true,
      wperf: true,
    }
  }

  /// Sets the addressing mode of the compiled ispc code.
  ///
  /// By default, all ispc-generated code is addressed with `i32`s, so if your
  /// arrays have more than 2 billion elements, you'll need to change the
  /// addressing mode to `Addr::X86_64`.
  ///
  /// Default value: `Addr::X86`
  pub fn addressing(&mut self, a: Addr) -> &mut Self {
    self.addressing = Some(a);
    self
  }

  /// Adds an entry to the target CPU set.
  ///
  /// Code will be generated for all CPUs in the target CPU set, and the correct
  /// version to run will be autodetected at run time.
  ///
  /// By default, the target CPU set is empty, as it can be automaatically
  /// determined by the selected `target`s.
  ///
  /// Default value: `[]`
  pub fn cpu(&mut self, c: Cpu) -> &mut Self {
    if self.cpu.is_none() { self.cpu = Some(vec![]); }
    self.cpu.as_mut().map(|cs| cs.push(c));
    self
  }

  /// Specifies a `-D` variable with an optional value.
  ///
  /// Default value: `[]`
  pub fn define(&mut self, k: &str, v: Option<&str>) -> &mut Self {
    self.definitions.push((k.into(), v.map(|v| v.into())));
    self
  }

  /// Turns on or off generation of debug info.
  ///
  /// This will generally be automatically determined by the currently selected
  /// cargo profile.
  pub fn debug(&mut self, val: bool) -> &mut Self {
    self.debug = Some(val);
    self
  }

  /// Selects the math library to call out to.
  ///
  /// Default value: `Math::Default`
  pub fn math_lib(&mut self, m: Math) -> &mut Self {
    self.math_lib = m;
    self
  }

  /// Adds a file to the set of files to be compiled together.
  ///
  /// Default value: `[]`
  pub fn file<P: AsRef<Path>>(&mut self, p: P) -> &mut Self {
    self.files.push(p.as_ref().to_path_buf());
    self
  }

  /// Set the optimization level.
  ///
  /// Default value: inferred from current cargo profile
  pub fn opt_level(&mut self, level: u32) -> &mut Self {
    self.opt_level = Some(level);
    self
  }

  /// Enables or disables asssertations in the code.
  ///
  /// Default value: `true`
  pub fn enable_assertations(&mut self, val: bool) -> &mut Self {
    self.assertations = val;
    self
  }

  /// Enables or disables generation of fused multiply-add instructions.
  ///
  /// Default value: `true`
  pub fn enable_fma(&mut self, val: bool) -> &mut Self {
    self.fma = val;
    self
  }

  /// Enables or disables loop unrolling.
  ///
  /// Default value: `true`
  pub fn enable_loop_unroll(&mut self, val: bool) -> &mut Self {
    self.loop_unroll = val;
    self
  }

  /// Enables or disables faster masked vector loads on SSE. This may cause reads
  /// to go off the end of arrays.
  ///
  /// Default value: `false`
  pub fn enable_fast_masked_vload(&mut self, val: bool) -> &mut Self {
    self.fast_masked_vload = val;
    self
  }

  /// Enables or disables non-IEEE-754 compliant math operations.
  ///
  /// Enabling this option may cause code to go much faster, but have
  /// unpredictable error in the result, and corner cases such as `inf` and `NaN`
  /// may not be handled properly.
  ///
  /// Default value: `false`
  pub fn enable_fast_math(&mut self, val: bool) -> &mut Self {
    self.fast_math = val;
    self
  }

  /// Enables or disables the generation of aligned vector load and store
  /// instructions.
  ///
  /// Enabling this option may make code run faster, but it will result in
  /// undefined behavior if buffers are not aligned correctly.
  ///
  /// Default value: `false`
  pub fn force_aligned_memory(&mut self, val: bool) -> &mut Self {
    self.force_aligned_memory = val;
    self
  }

  /// Enables or disables the generation of position-independent code.
  ///
  /// This should generally be `true`, unless you have a really good reason
  /// otherwise.
  ///
  /// Default value: `true` on x86_64, `false` on x86.
  pub fn pic(&mut self, val: bool) -> &mut Self {
    self.pic = Some(val);
    self
  }

  /// Adds a target to generate code for.
  ///
  /// The first target that's added replaces the default list. Every subsequent
  /// target will be added to this new list of targets. Only one target per
  /// ISA may be selected. For example, you may have the simultaneous choises of
  /// `[ Sse2_i32x4, Sse4_i32x8 ]`, but not `[ Sse2_i32x4, Sse2_i32x8 ]`.
  ///
  /// Default value: `[ Sse2, Sse4, Avx1, Avx1_1, Avx2 ]`
  pub fn target(&mut self, t: Target) -> &mut Self {
    if self.targets.is_none() { self.targets = Some(vec![]); }
    self.targets.as_mut().map(|ts| ts.push(t));
    self
  }

  /// Force all warnings as errors.
  ///
  /// If enabled, warnings will break the build. If there are warnings, not being
  /// treated as errors, cargo will currently silently eat them. Sorry.
  ///
  /// Bikeshed about it [here](https://github.com/rust-lang/cargo/issues/1106).
  ///
  /// Default value: `true`
  pub fn werror(&mut self, val: bool) -> &mut Self {
    self.werror = val;
    self
  }

  /// Enables or disables warnings.
  ///
  /// Default value: `true`
  pub fn warn(&mut self, val: bool) -> &mut Self {
    self.warnings = val;
    self
  }

  /// Enables or disables warnings about suboptimal code performance.
  ///
  /// These will not turn into errors *even with `.werror(true)`*, so cargo will
  /// silently ignore them anyhow. This is really suboptimal. Sorry.
  ///
  /// Bikeshed about it [here](https://github.com/rust-lang/cargo/issues/1106).
  ///
  /// Default value: `true`
  pub fn warn_perf(&mut self, val: bool) -> &mut Self {
    self.wperf = val;
    self
  }

  fn get_opt_level(&self) -> u32 {
    match self.opt_level {
      Some(ol) => cmp::min(ol, 3),
      None     => self.getenv_unwrap("OPT_LEVEL").parse().unwrap()
    }
  }

  fn get_debug(&self) -> bool {
    self.debug.unwrap_or_else(|| self.getenv_unwrap("PROFILE") == "debug")
  }

  fn get_targets(&self) -> Vec<Target> {
    match self.targets.clone() {
      None    => vec![ Target::Sse2, Target::Sse4, Target::Avx1, Target::Avx1_1, Target::Avx2 ],
      Some(t) => t,
    }
  }

  fn get_x64(&self) -> bool {
    let t = self.getenv_unwrap("TARGET");
    if      t.contains("x86_64")                     { true }
    else if t.contains("i686") || t.contains("i586") { false }
    else { fail(&format!("ispc can only target x86 or x86_64. Your current target is {}", t)) }
  }

  fn get_arch(&self) -> Arch {
    if let Some(x) = self.architecture { x }
    else if self.get_x64() { Arch::X86_64 }
    else                   { Arch::X86 }
  }

  fn get_pic(&self) -> bool {
    if let Some(x) = self.pic { x }
    else { self.get_x64() }
  }

  fn get_defs(&self) -> Vec<(String, Option<String>)> {
    self.definitions.clone()
  }

  fn get_out_dir(&self) -> PathBuf {
    std::env::var_os("OUT_DIR").map(PathBuf::from).unwrap()
  }

  fn getenv(&self, v: &str) -> Option<String> {
    let r = std::env::var(v).ok();
    println!("{} = {:?}", v, r);
    r
  }

  fn getenv_unwrap(&self, v: &str) -> String {
    match self.getenv(v) {
      Some(s) => s,
      None => fail(&format!("environment variable `{}` not defined", v)),
    }
  }

  fn get_base_compiler(&self) -> Tool {
    Tool::new(PathBuf::from(self.getenv("ISPC").unwrap_or("ispc".into())))
  }

  fn basic_tool(&self) -> Tool {
    let mut t = self.get_base_compiler();

    match self.addressing {
      Some(Addr::A32) => { t.arg("--addressing=32"); },
      Some(Addr::A64) => { t.arg("--addressing=64"); },
      None => {},
    }

    match self.get_arch() {
      Arch::X86 => t.arg("--arch=x86"),
      Arch::X86_64 => t.arg("--arch=x86_64"),
    };

    t.arg("--colored-output");

    match self.cpu.clone() {
      None => {},
      Some(cpus) => {
        let mut cpu_s = String::new();

        for (i, c) in cpus.into_iter().enumerate() {
          cpu_s.push_str(if i == 0 { "--cpu=" } else { "," });
          cpu_s.push_str(c.to_str());
        }

        t.arg(&*cpu_s);
      }
    }

    for (k, ov) in self.get_defs() {
      match ov {
        None    => { t.arg(&*format!("-D{}", k)); },
        Some(v) => { t.arg(&*format!("-D{}={}", k, v)); },
      }
    }

    t.arg("--emit-obj");

    if let Some(align) = self.force_alignment {
      t.arg(&*format!("--force-alignment={}", align));
    }

    if self.get_debug() {
      t.arg("-g");
    }

    t.arg(&*format!("--math-lib={}", self.math_lib.to_str()));

    t.arg(&*format!("-O{:?}", self.get_opt_level()));

    if !self.assertations { t.arg("--opt=disable-assertations"); }
    if !self.fma { t.arg("--opt=disable-fma"); }
    if !self.loop_unroll { t.arg("--opt=disable-loop-unroll"); }
    if self.fast_masked_vload { t.arg("--opt=fast-masked-vload"); }
    if self.fast_math { t.arg("--opt=fast-math"); }
    if self.force_aligned_memory { t.arg("--arg=fast-aligned-memory"); }

    if self.get_pic() { t.arg("--pic"); }

    let mut target_s = String::new();

    for (i, t) in self.get_targets().into_iter().enumerate() {
      target_s.push_str(if i == 0 { "--target=" } else { "," });
      target_s.push_str(t.to_str());
    }

    t.arg(&*target_s);

    if self.werror { t.arg("--werror"); }
    if !self.warnings { t.arg("--woff"); }
    if !self.wperf { t.arg("--wno-perf"); }

    t
  }

  fn compile_object(&self, file: &Path, dst: &Path, mut t: Tool) {
    fs::create_dir_all(&dst.parent().unwrap()).unwrap();

    t.arg(&*file.to_string_lossy());
    t.arg("-o");
    t.arg(&*dst.to_string_lossy());

    run(&mut t.to_command());
  }

  /// Runs the compiler, generating the `output`.
  ///
  /// The name  `output` must begin with `lib` and end with `.a`.
  pub fn compile(&self, output: &str) {
    assert!(output.starts_with("lib"));
    assert!(output.ends_with(".a"));

    let dst = self.get_out_dir();

    let base = self.basic_tool();

    let mut objects = Vec::new();

    for file in self.files.iter() {
      let lfile = file.file_stem().unwrap().to_string_lossy();
      println!("cargo:rerun-if-changed={}", file.to_string_lossy());
      let obj: PathBuf = dst.join(file).with_extension("o");
      self.compile_object(file, &obj, base.clone());
      let candidates : Vec<PathBuf> =
        vec![ obj.clone(),
              obj.clone().with_file_name(format!("{}_sse2",  lfile)).with_extension("o"),
              obj.clone().with_file_name(format!("{}_sse4",  lfile)).with_extension("o"),
              obj.clone().with_file_name(format!("{}_avx",   lfile)).with_extension("o"),
              obj.clone().with_file_name(format!("{}_avx11", lfile)).with_extension("o"),
              obj.clone().with_file_name(format!("{}_avx2",  lfile)).with_extension("o") ];

      for c in candidates {
        let exists = c.exists();
        println!("candidate {:?} exists={}", &c, exists);
        if exists {
          objects.push(c);
        }
      }
    }

    let mut c = gcc::Config::new();
    for o in &objects { c.object(&*o); }
    c.compile(output);
  }
}

/// Compile a library from the given set of input `.ispc` files.
///
/// This will simply compile all files into object files and then assemble them
/// into the output. This will read the standard environment variables to detect
/// cross compilation and such.
///
/// This function will also print all metadata on standard output for Cargo.
pub fn compile_library(output: &str, files: &[&str]) {
  let mut c = Config::new();
  for f in files { c.file(f); }
  c.compile(output)
}

fn fail(s: &str) -> ! {
  println!("\n\n{}\n\n", s);
  panic!()
}

fn run(cmd: &mut Command) {
  println!("running: {:?}", cmd);
  let output =
    match cmd.output() {
      Ok(output) => output,
      Err(ref e) if e.kind() == io::ErrorKind::NotFound => {
        fail(&format!("failed to execute command: {}\nIs `ispc` \
                       not installed?", e));
      },
      Err(e) => fail(&format!("failed to execute command: {}", e)),
    };
  let status = output.status;
  println!("{:?}", status);
  let stdout = String::from_utf8_lossy(&output.stdout);

  if !stdout.is_empty() {
    println!("\n--- stdout ---");
    println!("{}", stdout);
    println!("--- end stdout ---\n");
  }

  let stderr = String::from_utf8_lossy(&output.stderr);

  if !stderr.is_empty() {
    println!("\n--- stderr ---");
    println!("{}", stderr);
    println!("--- end stderr ---\n");
  }

  if !status.success() {
    fail(&format!("command did not execute successfully, got: {}", status));
  }
}
