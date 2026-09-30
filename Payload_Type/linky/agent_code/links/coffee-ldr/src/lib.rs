#![cfg_attr(windows, feature(c_variadic))]
#![cfg_attr(windows, feature(core_intrinsics))]
#![allow(internal_features)]

#[cfg(windows)]
pub mod loader;
