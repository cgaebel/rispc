//! The task system for rispc-built code.
//!
//! If your ispc code uses tasks, you will need this library as a dependency.
//!
//! Currently, pthreads are used. In the future, I would like this library to
//! support custom, pluggable task systems. Pull requests welcome.
//!
//! For more documentation, see the `rispc` crate.
#![deny(missing_docs)]

/// Convenience macro for generating the module to hold the raw/unsafe ISPC bindings.
///
/// In addition to building the library with ISPC we use rust-bindgen to generate
/// a rust module containing bindings to the functions exported from ISPC. These
/// can be imported by passing the name of your library to the `ispc_module` macro.
///
/// # Example
///
/// ```rust,no_run
/// #[macro_use] extern crate rispcrt;
///
/// // ispc code must have been generated into `libfoo.a`, and the rust bindings
/// // will be available under `foo::*`.
/// ispc_module!(foo);
/// ```
#[macro_export]
macro_rules! ispc_module {
    ($lib:ident) => (
        #[allow(dead_code, non_camel_case_types, non_snake_case)]
        mod $lib {
            include!(concat!(env!("OUT_DIR"), "/", stringify!($lib), ".rs"));
        }
    );
    (pub $lib:ident) => (
        #[allow(dead_code, non_camel_case_types, non_snake_case)]
        pub mod $lib {
            include!(concat!(env!("OUT_DIR"), "/", stringify!($lib), ".rs"));
        }
    )
}
