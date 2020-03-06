use aws_greengrass_core_rust::{log as gglog, GGResult};
use aws_greengrass_core_rust::init;
use aws_greengrass_core_rust::shadow::{ShadowClient, ShadowThing};
use log::{error, info, LevelFilter};
use std::{thread, time};

fn main() {
    gglog::init_log(LevelFilter::Debug);
    info!("Starting shadow gg lambda");

    if let Err(e) = do_stuff_with_thing() {
        error!("Error: {}", e);
        std::process::exit(1);
    }
}


fn do_stuff_with_thing() -> GGResult<()> {
    init()?;

    let (thing, response) = ShadowClient::get_thing_shadow("foo")?;

    info!("response: {:?}", response);
    info!("Shadow Thing: {:#?}", thing);

    Ok(())
}