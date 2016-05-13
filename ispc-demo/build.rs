extern crate ispc;

fn main() {
  ispc::Config::new()
    .file("src/mandel.ispc")
    .compile("libmandel.a");
}
