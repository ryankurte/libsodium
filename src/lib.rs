// Bindings may not fit rusts idea of good formatting
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]
#![no_std]

// Libc required to resolve c types
use cty;

// Include generated outputs
include!(concat!(env!("OUT_DIR"), "/libsodium.rs"));
