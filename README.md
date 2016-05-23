# ispc-rs

[![Build Status](https://travis-ci.org/cgaebel/ispc-rs.svg?branch=master)](https://travis-ci.org/cgaebel/ispc-rs)
[![Build status](https://ci.appveyor.com/api/projects/status/xxxxTODO?svg=true)](https://ci.appveyor.com/project/cgaebel/ispc-rs)
[![crates.io](https://img.shields.io/crates/v/ispc.svg)](https://crates.io/crates/ispc/)

[API Documentation](https://cgaebel.github.io/ispc-rs/)

[Intel SPMD Compiler Documentation](https://ispc.github.io/)

## Using ispc-rs

First, you'll want to both add a build script for your crate (`build.rs`) and
also add `ispc` to your `build-dependencies`, and `ispsrt` to your `dependencies`:

```toml
# Cargo.toml
[package]

# ...

build = "build.rs"

[build-dependencies]
ispc = "0.1"

[dependencies]
ispcrt = "0.1"
```

Next up, you'll want to write a build script like so:

```rust,no_run
// build.rs

extern crate ispc;

fn main() {
  ispc::compile_library("libmandelbrot.a", &[ "mandelbrot.ispc" ]);
}
```

And that's it! Running `cargo build` should take care of the rest and your Rust
application will now have the ispc file `mandelbrot.ispc` compiled into it. You
can call the functions in Rust the same way as you do C functions.

## Example

See the `ispc-demo` folder for an example application that uses an ISPC binding.

## License

The MIT License (MIT)
Copyright (c) 2016 Clark Gaebel <cg.wowus.cg@gmail.com>

Permission is hereby granted, free of charge, to any person obtaining a copy of this software and associated documentation files (the "Software"), to deal in the Software without restriction, including without limitation the rights to use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of the Software, and to permit persons to whom the Software is furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
