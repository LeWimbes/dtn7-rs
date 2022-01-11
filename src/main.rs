#![recursion_limit = "256"]

use std::collections::HashMap;
use std::convert::TryInto;

use log::info;

use crate::dtnconfig::DtnConfig;
use crate::dtnd::daemon::start_dtnd;

mod utils;
mod dtnconfig;
mod routing;
mod cla;
mod core;
mod dtnd;
mod ipnd;

fn main() -> Result<(), String> {
    esp_idf_sys::link_patches();
    if let Err(err) = esp_idf_sys::esp!(unsafe {
        esp_idf_sys::esp_vfs_eventfd_register(&esp_idf_sys::esp_vfs_eventfd_config_t {
            max_fds: 5,
            ..Default::default()
        })
    }) {
        return Err(format!("Error while registering eventfd: {}", err).to_string());
    }

    if let Err(_err) = smol::block_on(run()) {
        return Err("Error running run!".to_string());
    }
    Ok(())
}

async fn run() -> Result<(), std::io::Error> {
    let mut cfg = DtnConfig::new();

    cfg.debug = false;
    if cfg.debug {
        std::env::set_var(
            "RUST_LOG",
            "dtn7=debug,dtnd=debug,actix_server=debug,actix_web=debug",
        );
    } else {
        std::env::set_var(
            "RUST_LOG",
            "dtn7=info,dtnd=info,actix_server=info,actix_web=info",
        );
    }
    pretty_env_logger::init_timed();

    cfg.host_eid = "dtn://n1".try_into().unwrap();
    cfg.endpoints.push("in".to_string());
    cfg.routing = "epidemic".into();

    cfg.v6 = false;
    cfg.v4 = true;
    cfg.unsafe_httpd = false;
    cfg.enable_period = false;
    cfg.announcement_interval = "2s".parse::<humantime::Duration>().unwrap().into();
    cfg.webport = 3000;
    cfg.janitor_interval = "10s".parse::<humantime::Duration>().unwrap().into();
    cfg.peer_timeout = "20s".parse::<humantime::Duration>().unwrap().into();

    cfg.services = HashMap::new();
    cfg.discovery_destinations = HashMap::new();
    cfg.check_destinations()
        .expect("Encountered an error while checking for the existence of discovery addresses");
    cfg.statics = Vec::new();

    info!("starting dtnd");
    start_dtnd(cfg).await.unwrap();

    Ok(())
}
