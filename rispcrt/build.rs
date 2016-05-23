extern crate gcc;

fn main() {
  gcc::Config::new()
    .cpp(true)
    .file("src/tasksys.cpp")
    .define("ISPC_USE_PTHREADS", None)
    .compile("librispcrt.a");
}
