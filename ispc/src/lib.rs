#![allow(non_camel_case_types)]

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

  pub fn arg(&mut self, s: &str) -> &mut Self {
    self.args.push(s.into());
    self
  }

  pub fn to_command(&self) -> Command {
    let mut cmd = Command::new(&self.path);
    cmd.args(&self.args);
    for &(ref k, ref v) in self.envs.iter() {
      cmd.env(k, v);
    }
    cmd
  }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum Addr { A32, A64 }

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum Arch { X86, X86_64 }

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum Cpu {
  Generic,    //< generic
  Atom,       //< atom/bonnell
  Core2,      //< core2
  Penryn,     //< penryn
  Corei7,     //< corei7/nehalem
  Corei7_avx, //< corei7-avx/sandybridge
  Core_avx_i, //< core-avx-i/ivybridge
  Core_avx2,  //< core-avx2/haswell
  Broadwell,  //< broadwell
  Slm         //< slm/silvermont
}

impl Cpu {
  fn to_str(self) -> &'static str {
    use Cpu::*;
    match self {
      Generic => "generic",
      Atom => "atom",
      Core2 => "core2",
      Penryn => "penryn",
      Corei7 => "corei7",
      Corei7_avx => "corei7-avx",
      Core_avx_i => "core-avx-i",
      Core_avx2 => "core-avx2",
      Broadwell => "broadwell",
      Slm => "slm",
    }
  }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum Math {
  Default, //< Use ispc's built-in math functions
  Fast,    //< Use high-performance but lower-accuracy math functions
  Svml,    //< Use the Intel SVML math libraries
  System,  //< Use the system's math libraries. *WARNING: May be quite slow*.
}

impl Math {
  fn to_str(self) -> &'static str {
    use Math::*;
    match self {
      Default => "default",
      Fast => "fast",
      Svml => "svml",
      System => "system",
    }
  }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum Target {
  Sse2,
  Sse2_i32x4,
  Sse2_i32x8,

  Sse4,
  Sse4_i32x4,
  Sse4_i32x8,
  Sse4_i16x8,
  Sse4_i8x16,

  Avx1,
  Avx1_i32x4,
  Avx1_i32x8,
  Avx1_i32x16,
  Avx1_i64x4,

  Avx1_1,
  Avx1_1_i32x8,
  Avx1_1_i32x16,
  Avx1_1_i64x4,

  Avx2,
  Avx2_i32x8,
  Avx2_i32x16,
  Avx2_i64x4,
}

impl Target {
  fn to_str(self) -> &'static str {
    use Target::*;
    match self {
      Sse2 => "sse2",
      Sse2_i32x4 => "sse2-i32x4",
      Sse2_i32x8 => "sse2-i32x8",

      Sse4 => "sse4",
      Sse4_i32x4 => "sse4-i32x4",
      Sse4_i32x8 => "sse4-i32x8",
      Sse4_i16x8 => "sse4-i16x8",
      Sse4_i8x16 => "sse4-i8x16",

      Avx1 => "avx1",
      Avx1_i32x4  => "avx1-i32x4",
      Avx1_i32x8  => "avx1-i32x8",
      Avx1_i32x16 => "avx1-i32x16",
      Avx1_i64x4  => "avx1-i64x4",

      Avx1_1 => "avx1.1",
      Avx1_1_i32x8  => "avx1.1-i32x8",
      Avx1_1_i32x16 => "avx1.1-i32x16",
      Avx1_1_i64x4  => "avx1.1-i64x4",

      Avx2 => "avx2",
      Avx2_i32x8  => "avx2-i32x8",
      Avx2_i32x16 => "avx2-i32x16",
      Avx2_i64x4  => "avx2-i64x4",
    }
  }
}

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
      werror: false,
      warnings: true,
      wperf: true,
    }
  }

  pub fn addressing(&mut self, a: Addr) -> &mut Self {
    self.addressing = Some(a);
    self
  }

  pub fn cpu(&mut self, c: Cpu) -> &mut Self {
    if self.cpu.is_none() { self.cpu = Some(vec![]); }
    self.cpu.as_mut().map(|cs| cs.push(c));
    self
  }

  pub fn define(&mut self, k: &str, v: Option<&str>) -> &mut Self {
    self.definitions.push((k.into(), v.map(|v| v.into())));
    self
  }

  pub fn force_alignment(&mut self, force: bool) -> &mut Self {
    self.force_aligned_memory = force;
    self
  }

  pub fn debug(&mut self, val: bool) -> &mut Self {
    self.debug = Some(val);
    self
  }

  pub fn math_lib(&mut self, m: Math) -> &mut Self {
    self.math_lib = m;
    self
  }

  pub fn file<P: AsRef<Path>>(&mut self, p: P) -> &mut Self {
    self.files.push(p.as_ref().to_path_buf());
    self
  }

  pub fn opt_level(&mut self, level: u32) -> &mut Self {
    self.opt_level = Some(level);
    self
  }

  pub fn enable_assertations(&mut self, val: bool) -> &mut Self {
    self.assertations = val;
    self
  }

  pub fn enable_fma(&mut self, val: bool) -> &mut Self {
    self.fma = val;
    self
  }

  pub fn enable_loop_unroll(&mut self, val: bool) -> &mut Self {
    self.loop_unroll = val;
    self
  }

  pub fn enable_fast_masked_vload(&mut self, val: bool) -> &mut Self {
    self.fast_masked_vload = val;
    self
  }

  pub fn enable_fast_math(&mut self, val: bool) -> &mut Self {
    self.fast_math = val;
    self
  }

  pub fn force_aligned_memory(&mut self, val: bool) -> &mut Self {
    self.force_aligned_memory = val;
    self
  }

  pub fn pic(&mut self, val: bool) -> &mut Self {
    self.pic = Some(val);
    self
  }

  pub fn target(&mut self, t: Target) -> &mut Self {
    if self.targets.is_none() { self.targets = Some(vec![]); }
    self.targets.as_mut().map(|ts| ts.push(t));
    self
  }

  pub fn werror(&mut self, val: bool) -> &mut Self {
    self.werror = val;
    self
  }

  pub fn warn(&mut self, val: bool) -> &mut Self {
    self.warnings = val;
    self
  }

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
  println!("\n--- stdout ---");
  println!("{}", stdout);
  println!("--- end stdout ---\n");

  if !status.success() {
    fail(&format!("command did not execute successfully, got: {}", status));
  }
}
