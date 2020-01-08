include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

use crate::context::LambdaContext;
use crate::error::GGError;

pub enum RuntimeOption {
    Async,
}

impl RuntimeOption {
    fn as_opt(&self) -> gg_runtime_opt {
        match self {
            Async => 1,
        }
    }
}

// handler is just a place hodler for the time being
pub fn start<F>(handler: Option<F>, option: RuntimeOption) -> Result<(), GGError>
where
    F: Fn(LambdaContext) -> (),
{
    unsafe {
        // todo - support handlers
        // this will probably involve creating a handler with a channel that then
        // sends info to our handler function: https://doc.rust-lang.org/nomicon/ffi.html#asynchronous-callbacks
        extern "C" fn no_op_handler(_: *const gg_lambda_context) {};
        let start_res = gg_runtime_start(Some(no_op_handler), option.as_opt());
        GGError::from_code(start_res)?;
    }
    Ok(())
}
