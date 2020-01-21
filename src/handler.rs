// just a placeholder for now
#[derive(Debug)]
pub struct LambdaContext {
    pub function_arn: String,
    pub client_context: String,
    pub message: String,
}

impl LambdaContext {
    pub(crate) fn new(function_arn: String, client_context: String, message: String) -> Self {
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
