use std::{convert::TryInto, time::Duration};
use std::collections::HashMap;

use bp7::EndpointID;
use rand::{Rng, thread_rng};
use rand::distributions::Alphanumeric;
use serde::Serialize;

use crate::core::DtnPeer;

#[derive(Debug, Default, Clone, Serialize)]
pub struct DtnConfig {
    pub debug: bool,
    pub unsafe_httpd: bool,
    pub v4: bool,
    pub v6: bool,
    pub custom_timeout: bool,
    pub enable_period: bool,
    pub nodeid: String,
    pub host_eid: EndpointID,
    pub webport: u16,
    pub announcement_interval: Duration,
    pub discovery_destinations: HashMap<String, u32>,
    pub janitor_interval: Duration,
    pub endpoints: Vec<String>,
    pub services: HashMap<u8, String>,
    pub routing: String,
    pub peer_timeout: Duration,
    pub statics: Vec<DtnPeer>,
    pub generate_status_reports: bool,
}

pub fn rnd_node_name() -> String {
    thread_rng()
        .sample_iter(&Alphanumeric)
        .take(10)
        .map(char::from)
        .collect()
}

impl DtnConfig {
    pub fn new() -> DtnConfig {
        let node_rnd: String = rnd_node_name();
        let local_node_id: EndpointID = format!("dtn://{}", node_rnd).try_into().unwrap();
        DtnConfig {
            debug: false,
            unsafe_httpd: false,
            v4: true,
            v6: false,
            custom_timeout: false,
            enable_period: false,
            nodeid: local_node_id.to_string(),
            host_eid: local_node_id,
            announcement_interval: "2s".parse::<humantime::Duration>().unwrap().into(),
            discovery_destinations: HashMap::new(),
            webport: 3000,
            janitor_interval: "10s".parse::<humantime::Duration>().unwrap().into(),
            endpoints: Vec::new(),
            services: HashMap::new(),
            routing: "epidemic".into(),
            peer_timeout: "20s".parse::<humantime::Duration>().unwrap().into(),
            statics: Vec::new(),
            generate_status_reports: false,
        }
    }
    pub fn set(&mut self, cfg: DtnConfig) {
        self.debug = cfg.debug;
        self.unsafe_httpd = cfg.unsafe_httpd;
        self.v4 = cfg.v4;
        self.v6 = cfg.v6;
        self.custom_timeout = cfg.custom_timeout;
        self.enable_period = cfg.enable_period;
        self.nodeid = cfg.host_eid.to_string();
        self.host_eid = cfg.host_eid;
        self.webport = cfg.webport;
        self.announcement_interval = cfg.announcement_interval;
        self.discovery_destinations = cfg.discovery_destinations;
        self.janitor_interval = cfg.janitor_interval;
        self.endpoints = cfg.endpoints;
        self.services = cfg.services;
        self.routing = cfg.routing;
        self.peer_timeout = cfg.peer_timeout;
        self.statics = cfg.statics;
        self.generate_status_reports = cfg.generate_status_reports;
    }

    // If no discovery destination is specified via CLI or config use the default discovery destinations
    // depending on whether to use ipv4 or ipv6
    pub fn check_destinations(&mut self) -> std::io::Result<()> {
        if self.discovery_destinations.is_empty() {
            match (self.v4, self.v6) {
                (true, true) => {
                    self.discovery_destinations
                        .insert("224.0.0.26:3003".to_string(), 0);
                    self.discovery_destinations
                        .insert("[FF02::1]:3003".to_string(), 0);
                }
                (true, false) => {
                    self.discovery_destinations
                        .insert("224.0.0.26:3003".to_string(), 0);
                }
                (false, true) => {
                    self.discovery_destinations
                        .insert("[FF02::1]:3003".to_string(), 0);
                }
                (false, false) => {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        String::from("Only IP destinations supported at the moment"),
                    ));
                }
            }
        }
        Ok(())
    }

    /// Updates the beacon sequence number everytime a beacon is sent to a specific IP address
    pub fn update_beacon_sequence_number(&mut self, destination: &str) {
        if let Some(sequence) = self.discovery_destinations.get_mut(destination) {
            if *sequence == u32::MAX {
                *sequence = 0;
            } else {
                *sequence += 1;
            }
        }
    }
}
