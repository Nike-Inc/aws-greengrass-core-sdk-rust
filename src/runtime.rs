include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

use crate::error::GGError;
use std::default::Default;
use crate::handler::{Handler, LambdaContext, NoOpHandler};
use std::os::raw::c_void;
use std::ffi::{CString, CStr};
use log::error;
use std::sync::Arc;
use std::cell::RefCell;
use crossbeam_channel::{unbounded, Sender, Receiver};
use lazy_static::lazy_static;

const BUFFER_SIZE: usize = 100;

type ShareableHandler = dyn Handler + Send + Sync;

static SENDER: Arc<RefCell<Option<Sender<LambdaContext>>>> = init_sender();

const fn init_sender() -> Arc<RefCell<Option<Sender<LambdaContext>>>> {
    Arc::new(RefCell::new(None))
}


pub enum RuntimeOption {
    Async,
}

impl RuntimeOption {
    fn as_opt(&self) -> gg_runtime_opt {
        match self {
            Self::Async => 1,
        }
    }
}

pub struct Runtime {
    runtime_option: RuntimeOption,
    handler: Box<ShareableHandler>,
}

impl Default for Runtime {
    fn default() -> Self {
        Runtime {
            runtime_option: RuntimeOption::Async,
            handler: Box::new(NoOpHandler),
        }
    }
}

impl Runtime {
    pub fn start(self) -> Result<(), GGError> {
        unsafe {
            // todo - support handlers
            // this will probably involve creating a handler with a channel that then
            // sends info to our handler function: https://doc.rust-lang.org/nomicon/ffi.html#asynchronous-callbacks

            // Arc::clone(&HANDLER).replace(self.handler);        

            let (sender, receiver) = unbounded();    
            Arc::clone(&SENDER).replace(Some(sender));

            
            extern "C" fn c_handler(c_ctx: *const gg_lambda_context) {
                unsafe {
                    match build_context(c_ctx) {
                        Ok(context) => {                        
                            // Arc::clone(&HANDLER).borrow().handle(context);
                        }
                        Err(e) => error!("Handler error: {}", e)
                    }
                }
            };

            
            extern "C" fn no_op_handler(_: *const gg_lambda_context) {};
            let start_res = gg_runtime_start(Some(no_op_handler), self.runtime_option.as_opt());
            GGError::from_code(start_res)?;
        }
        Ok(())
    }

    pub fn with_runtime_option(self, runtime_option: RuntimeOption) -> Self {
        Runtime {
            runtime_option,
            ..self
        }
    }

    pub fn with_handler(self, handler: Box<ShareableHandler>) -> Self {
        Runtime {
            handler,
            ..self
        }
    }

}

pub(crate) unsafe fn build_context(c_ctx: *const gg_lambda_context) -> Result<LambdaContext, GGError> {
    let message = handler_read_message()?;
    let function_arn = CStr::from_ptr((*c_ctx).function_arn)
        .to_string_lossy()
        .to_owned()
        .to_string();
    let client_context = CStr::from_ptr((*c_ctx).client_context)
        .to_string_lossy()
        .to_owned()
        .to_string();
    Ok(LambdaContext::new(function_arn, client_context, message))
}

unsafe fn handler_read_message() -> Result<String, GGError> {
    let mut collected: Vec<u8> = Vec::new();

    loop {
        let mut buffer = [0u8; BUFFER_SIZE];
        let mut read: usize = 0;

        let raw_read = &mut read as *mut usize; 

        let pub_res = gg_lambda_handler_read(
            buffer.as_mut_ptr() as *mut c_void,
            BUFFER_SIZE,
            raw_read 
        );

        GGError::from_code(pub_res)?;

        if read > 0 {
            collected.extend_from_slice(&buffer[..read]);
        }
        else {
            break;
        }        
    }

    let c_string = CString::from_vec_unchecked(collected);
    c_string.into_string()
        .map_err(GGError::from)
    
}
