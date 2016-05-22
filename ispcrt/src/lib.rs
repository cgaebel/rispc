//! The task system for ispc-built code.
//!
//! If your ispc code uses tasks, you will need this library as a dependency.
//!
//! Currently, pthreads are used. In the future, I would like this library to
//! support custom, pluggable task systems. Pull requests welcome.
#![deny(missing_docs)]
