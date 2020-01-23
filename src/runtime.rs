include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

use crate::error::GGError;
use crate::handler::{Handler, LambdaContext};
use crossbeam_channel::{unbounded, Receiver, Sender};
use lazy_static::lazy_static;
use log::{error, info};
use std::default::Default;
use std::ffi::CStr;
use std::os::raw::c_void;
use std::sync::Arc;
use std::thread;

const BUFFER_SIZE: usize = 100;

/// Denotes a handler that is thread safe
pub type ShareableHandler = dyn Handler + Send + Sync;

lazy_static! {
    // This establishes a thread safe global channel that can
    // be acquired from the callback function we register with the C Api
    static ref CHANNEL: Arc<ChannelHolder> = ChannelHolder::new();
}

/// Type of runtime. Currently only one, Async exits
pub enum RuntimeOption {
    Async,
}

impl RuntimeOption {
    /// Converts to the option required by the runtime api
    fn as_opt(&self) -> gg_runtime_opt {
        match self {
            Self::Async => gg_runtime_opt_GG_RT_OPT_ASYNC,
        }
    }
}

/// Configures and instantiates the green grass core runtime
/// Runtime can only be started by the Initializer. You must pass the runtime into the Initializer::with_runtime method.
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
    /// Start the green grass core runtime
    pub(crate) fn start(self) -> Result<(), GGError> {
        unsafe {
            // If there is a handler defined, then register the
            // the c delegating handler and start a thread that
            // monitors the channel for messages from the c handler
            let c_handler = if let Some(handler) = self.handler {
                thread::spawn(move || loop {
                    match ChannelHolder::recv() {
                        Ok(context) => handler.handle(context),
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

    /// Provide a non-default runtime option
    pub fn with_runtime_option(self, runtime_option: RuntimeOption) -> Self {
        Runtime {
            runtime_option,
            ..self
        }
    }

    /// Provide a handler. If no handler is provided the runtime will register a no-op handler
    pub fn with_handler(self, handler: Option<Box<ShareableHandler>>) -> Self {
        Runtime { handler, ..self }
    }
}

/// c handler that performs a no op
extern "C" fn no_op_handler(_: *const gg_lambda_context) {
    info!("No opt handler called!");
}

/// c handler that utilizes ChannelHandler in order to pass
/// information to the Handler implementation provided
extern "C" fn delgating_handler(c_ctx: *const gg_lambda_context) {
    info!("delegating_handler called!");
    unsafe {
        let result = build_context(c_ctx).and_then(ChannelHolder::send);
        if let Err(e) = result {
            error!("{}", e);
        }
    }
}

/// Converts the c context to our rust native context
unsafe fn build_context(c_ctx: *const gg_lambda_context) -> Result<LambdaContext, GGError> {
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

/// Wraps the C gg_lambda_handler_read call
unsafe fn handler_read_message() -> Result<Vec<u8>, GGError> {
    let mut collected: Vec<u8> = Vec::new();
    loop {
        let mut buffer = [0u8; BUFFER_SIZE];
        let mut read: usize = 0;

        let raw_read = &mut read as *mut usize;

        let pub_res =
            gg_lambda_handler_read(buffer.as_mut_ptr() as *mut c_void, BUFFER_SIZE, raw_read);

        GGError::from_code(pub_res)?;

        if read > 0 {
            collected.extend_from_slice(&buffer[..read]);
        } else {
            break;
        }
    }
    Ok(collected)
}

/// Wraps a Channel.
/// This is mostly needed as there is no way to instantiate a static ref with a tuple (see CHANNEL above)
struct ChannelHolder {
    sender: Sender<LambdaContext>,
    receiver: Receiver<LambdaContext>,
}

impl ChannelHolder {
    pub fn new() -> Arc<Self> {
        let (sender, receiver) = unbounded();
        let holder = ChannelHolder { sender, receiver };
        Arc::new(holder)
    }

    /// Performs a send with CHANNEL and coerces the error type
    fn send(context: LambdaContext) -> Result<(), GGError> {
        Arc::clone(&CHANNEL)
            .sender
            .send(context)
            .map_err(GGError::from)
    }

    /// Performs a recv with CHANNEL and coerces the error type
    fn recv() -> Result<LambdaContext, GGError> {
        Arc::clone(&CHANNEL).receiver.recv().map_err(GGError::from)
    }
}
