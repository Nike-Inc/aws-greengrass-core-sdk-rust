// just a placeholder for now
#[derive(Debug)]
pub struct LambdaContext {
    pub function_arn: String,
    pub client_context: String,
    pub message: Vec<u8>,
}

impl LambdaContext {
    pub(crate) fn new(function_arn: String, client_context: String, message: Vec<u8>) -> Self {
        LambdaContext {
            function_arn,
            client_context,
            message,
        }
    }
}

pub trait Handler {
    fn handle(&self, ctx: LambdaContext);
}
