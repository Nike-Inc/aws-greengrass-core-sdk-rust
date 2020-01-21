include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

use crate::error::GGError;
use std::default::Default;
use crate::handler::{Handler, LambdaContext};
use std::os::raw::c_void;
use std::ffi::{CString, CStr};
use log::error;
use std::sync::Arc;
use crossbeam_channel::{unbounded, Sender, Receiver};
use lazy_static::lazy_static;
use std::thread;

const BUFFER_SIZE: usize = 100;

type ShareableHandler = dyn Handler + Send + Sync;

lazy_static! {
    static ref CHANNEL: Arc<ChannelHolder> = ChannelHolder::new();
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
    handler: Option<Box<ShareableHandler>>,
}

impl Default for Runtime {
    fn default() -> Self {
        Runtime {
            runtime_option: RuntimeOption::Async,
            handler: None,
        }
    }
}

impl Runtime {
    pub fn start(self) -> Result<(), GGError> {
        unsafe {
            
            let c_handler = if let Some(handler) = self.handler {
                thread::spawn(move || {
                    match ChannelHolder::recv() {
                        Ok(context) =>  handler.handle(context),
                        Err(e) => error!("{}", e),
                    }
                });

                delgating_handler
            } else {
                no_op_handler
            };
            
            let start_res = gg_runtime_start(Some(c_handler), self.runtime_option.as_opt());
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

    pub fn with_handler(self, handler: Option<Box<ShareableHandler>>) -> Self {
        Runtime {
            handler,
            ..self
        }
    }

}

extern "C" fn no_op_handler(_: *const gg_lambda_context) {}

extern "C" fn delgating_handler(c_ctx: *const gg_lambda_context) {
    unsafe {
        let result = build_context(c_ctx) 
            .and_then(ChannelHolder::send);

        if let Err(e) = result {
            error!("{}", e);
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

struct ChannelHolder {
    sender: Sender<LambdaContext>,
    receiver: Receiver<LambdaContext>,
}

impl ChannelHolder {
    pub fn new() -> Arc<Self> {
        let (sender, receiver) = unbounded();
        let holder = ChannelHolder {
            sender,
            receiver,
        };
        Arc::new(holder)
    }

    fn send(context: LambdaContext) -> Result<(), GGError> {
        Arc::clone(&CHANNEL).sender.send(context)
            .map_err(GGError::from)
    }
    
    fn recv() -> Result<LambdaContext, GGError> {
        Arc::clone(&CHANNEL).receiver.recv()
            .map_err(GGError::from)
    }
}

