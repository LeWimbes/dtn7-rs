use std::fmt::Debug;
use std::fmt::Display;

use bp7::Bundle;
use bp7::EndpointID;
use derive_more::*;
use enum_dispatch::enum_dispatch;

use epidemic::EpidemicRoutingAgent;
use flooding::FloodingRoutingAgent;
use sink::SinkRoutingAgent;

use crate::cla::ClaSender;
use crate::core::bundlepack::BundlePack;

pub mod epidemic;
pub mod flooding;
pub mod sink;

pub enum RoutingNotifcation<'a> {
    SendingFailed(&'a str, &'a str),
    IncomingBundle(&'a Bundle),
    EncounteredPeer(&'a EndpointID),
}

#[enum_dispatch]
#[derive(Debug, Display)]
pub enum RoutingAgentsEnum {
    EpidemicRoutingAgent,
    FloodingRoutingAgent,
    SinkRoutingAgent,
}

#[enum_dispatch(RoutingAgentsEnum)]
pub trait RoutingAgent: Debug + Display {
    fn notify(&mut self, _notification: RoutingNotifcation) {}
    fn sender_for_bundle(&mut self, _bp: &BundlePack) -> (Vec<ClaSender>, bool) {
        unimplemented!();
    }
}

pub fn new(routingagent: &str) -> RoutingAgentsEnum {
    match routingagent {
        "flooding" => FloodingRoutingAgent::new().into(),
        "epidemic" => EpidemicRoutingAgent::new().into(),
        "sink" => sink::SinkRoutingAgent::new().into(),
        _ => panic!("Unknown routing agent {}", routingagent),
    }
}
