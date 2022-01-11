#![recursion_limit = "256"]

use std::collections::HashMap;
use std::convert::TryInto;
use std::sync::Arc;

use anyhow::bail;
use anyhow::Result;
use embedded_svc::ipv4;
use embedded_svc::ping::Ping;
use embedded_svc::wifi::{AccessPointConfiguration, ApIpStatus, ApStatus, ClientConfiguration, ClientConnectionStatus, ClientIpStatus, ClientStatus, Configuration, Status, Wifi};
use esp_idf_svc::netif::EspNetifStack;
use esp_idf_svc::nvs::EspDefaultNvs;
use esp_idf_svc::sysloop::EspSysLoopStack;
use esp_idf_svc::wifi::EspWifi;
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


const SSID: &str = env!("DTN7_WIFI_SSID");
const PASS: &str = env!("DTN7_WIFI_PASS");

fn main() -> Result<()> {
    esp_idf_sys::link_patches();
    esp_idf_sys::esp!(unsafe {
        esp_idf_sys::esp_vfs_eventfd_register(&esp_idf_sys::esp_vfs_eventfd_config_t {
            max_fds: 5,
            ..Default::default()
        })
    })?;

    let netif_stack = Arc::new(EspNetifStack::new()?);
    let sys_loop_stack = Arc::new(EspSysLoopStack::new()?);
    let default_nvs = Arc::new(EspDefaultNvs::new()?);
    let wifi = wifi(
        netif_stack.clone(),
        sys_loop_stack.clone(),
        default_nvs.clone(),
    )?;

    smol::block_on(run())?;

    drop(wifi);
    info!("Wifi stopped");
    Ok(())
}

// taken from https://github.com/ivmarkov/rust-esp32-std-demo (Apache License 2.0)
fn wifi(
    netif_stack: Arc<EspNetifStack>,
    sys_loop_stack: Arc<EspSysLoopStack>,
    default_nvs: Arc<EspDefaultNvs>,
) -> Result<Box<EspWifi>> {
    let mut wifi = Box::new(EspWifi::new(netif_stack, sys_loop_stack, default_nvs)?);

    info!("Wifi created, about to scan");

    let ap_infos = wifi.scan()?;

    let ours = ap_infos.into_iter().find(|a| a.ssid == SSID);

    let channel = if let Some(ours) = ours {
        info!(
            "Found configured access point {} on channel {}",
            SSID, ours.channel
        );
        Some(ours.channel)
    } else {
        info!(
            "Configured access point {} not found during scanning, will go with unknown channel",
            SSID
        );
        None
    };

    wifi.set_configuration(&Configuration::Mixed(
        ClientConfiguration {
            ssid: SSID.into(),
            password: PASS.into(),
            channel,
            ..Default::default()
        },
        AccessPointConfiguration {
            ssid: "aptest".into(),
            channel: channel.unwrap_or(1),
            ..Default::default()
        },
    ))?;

    info!("Wifi configuration set, about to get status");

    let status = wifi.get_status();

    if let Status(
        ClientStatus::Started(ClientConnectionStatus::Connected(ClientIpStatus::Done(ip_settings))),
        ApStatus::Started(ApIpStatus::Done),
    ) = status
    {
        info!("Wifi connected");

        // ping(&ip_settings)?;
    } else {
        bail!("Unexpected Wifi status: {:?}", status);
    }

    Ok(wifi)
}

// taken from https://github.com/ivmarkov/rust-esp32-std-demo (Apache License 2.0)
fn ping(ip_settings: &ipv4::ClientSettings) -> Result<()> {
    info!("About to do some pings for {:?}", ip_settings);

    let ping_summary =
        esp_idf_svc::ping::EspPing::default().ping(ip_settings.subnet.gateway, &Default::default())?;
    if ping_summary.transmitted != ping_summary.received {
        bail!(
            "Pinging gateway {} resulted in timeouts",
            ip_settings.subnet.gateway
        );
    }

    info!("Pinging done");

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
