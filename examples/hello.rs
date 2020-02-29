use aws_greengrass_core_rust::client::IOTDataClient;
use aws_greengrass_core_rust::init;
use aws_greengrass_core_rust::log as gglog;
use log::LevelFilter;

pub fn main() -> std::io::Result<()> {
    gglog::init_log(LevelFilter::Info);
    init().map_err(|e| e.as_ioerror())?;
    let result = IOTDataClient::default().publish("mytopic", r#"{"msg": "hello greengrass!"}"#)
        .map_err(|e| e.as_ioerror());
    println!("result {:?}", result);
    Ok(())
}
