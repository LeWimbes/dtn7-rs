use std::collections::HashMap;

use bp7::EndpointID;
use log::{error, info};

use application_agent::ApplicationAgent;

use crate::cla::CLAEnum;
use crate::core::application_agent::ApplicationAgentEnum;
use crate::core::mem_store::BundleStore;
pub use crate::core::peer::{DtnPeer, PeerType};
use crate::routing::RoutingAgent;
use crate::routing::RoutingAgentsEnum;
use crate::utils::{PEERS, STORE, store_get_metadata};

use self::bundlepack::BundlePack;
use self::processing::forward;

pub mod application_agent;
pub mod bundlepack;
pub mod peer;
pub mod processing;
pub mod mem_store;

#[derive(Debug)]
pub struct DtnCore {
    pub endpoints: Vec<ApplicationAgentEnum>,
    pub cl_list: Vec<CLAEnum>,
    pub service_list: HashMap<u8, String>,
    pub routing_agent: RoutingAgentsEnum,
}

impl Default for DtnCore {
    fn default() -> Self {
        Self::new()
    }
}

impl DtnCore {
    pub fn new() -> DtnCore {
        DtnCore {
            endpoints: Vec::new(),
            cl_list: Vec::new(),
            service_list: HashMap::new(),
            //routing_agent: crate::routing::flooding::FloodingRoutingAgent::new().into(),
            routing_agent: crate::routing::epidemic::EpidemicRoutingAgent::new().into(),
        }
    }

    pub fn register_application_agent(&mut self, aa: ApplicationAgentEnum) {
        if self.is_in_endpoints(aa.eid()) {
            info!("Application agent already registered for EID: {}", aa.eid());
        } else {
            info!("Registered new application agent for EID: {}", aa.eid());
            self.endpoints.push(aa);
        }
    }
    pub fn is_in_endpoints(&self, eid: &EndpointID) -> bool {
        for aa in self.endpoints.iter() {
            if eid == aa.eid() {
                return true;
            }
        }
        false
    }
    pub fn get_endpoint_mut(&mut self, eid: &EndpointID) -> Option<&mut ApplicationAgentEnum> {
        for aa in self.endpoints.iter_mut() {
            if eid == aa.eid() {
                return Some(aa);
            }
        }
        None
    }
}

/// Removes peers from global peer list that haven't been seen in a while.
pub fn process_peers() {
    (*PEERS.lock().unwrap()).retain(|_k, v| {
        let val = v.still_valid();
        if !val {
            info!(
                "Have not seen {} @ {} in a while, removing it from list of known peers",
                v.eid, v.addr
            );
        }
        v.con_type == PeerType::Static || val
    });
}

/// Reprocess bundles in store
pub async fn process_bundles() {
    let forwarding_bids: Vec<String> = (*STORE.lock().unwrap()).forwarding();

    let mut forwarding_bundles: Vec<BundlePack> = forwarding_bids
        .iter()
        .filter_map(|bid| store_get_metadata(bid))
        .collect();
    forwarding_bundles.sort_unstable_by(|a, b| a.creation_time.cmp(&b.creation_time));

    for bp in forwarding_bundles {
        if let Err(err) = forward(bp).await {
            error!("Error forwarding bundle: {}", err);
        }
    }
    //forwarding_bundle_ids.iter().for_each(|bpid| {});
}
