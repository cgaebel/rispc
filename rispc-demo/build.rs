extern crate rispc;

fn main() {
  rispc::Config::new()
    .file("src/mandel.ispc")
    .compile("libmandel.a");
}
