#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

pub mod context;
pub mod error;
pub mod runtime;

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

use crate::error::GGError;
use std::ffi::CString;
use std::os::raw::c_void;
use std::ptr;

pub fn init() -> Result<(), GGError> {
    unsafe {
        let init_res = gg_global_init(0);
        GGError::from_code(init_res)?;
    }
    Ok(())
}
