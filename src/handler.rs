// just a placeholder for now
#[derive(Debug)]
pub struct LambdaContext {
    function_arn: String,
    client_context: String,
    message: String,
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
