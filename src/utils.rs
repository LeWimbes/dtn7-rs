use std::collections::HashMap;
use std::convert::TryInto;
use std::sync::Mutex;

use anyhow::Result;
use bp7::{Bundle, CreationTimestamp, EndpointID};
use bp7::flags::BundleControlFlags;
use lazy_static::*;
use log::{debug, error, info};

use crate::cla::CLAEnum;
pub use crate::core::{DtnCore, DtnPeer};
use crate::core::bundlepack::BundlePack;
use crate::core::DtnStatistics;
use crate::core::mem_store::{BundleStore, BundleStoresEnum, InMemoryBundleStore};
use crate::core::peer::PeerAddress;
use crate::dtnconfig::DtnConfig;
use crate::routing::RoutingAgent;
pub use crate::routing::RoutingNotifcation;

lazy_static! {
    pub static ref CONFIG: Mutex<DtnConfig> = Mutex::new(DtnConfig::new());
    pub static ref DTNCORE: Mutex<DtnCore> = Mutex::new(DtnCore::new());
    pub static ref PEERS: Mutex<HashMap<String, DtnPeer>> = Mutex::new(HashMap::new());
    pub static ref STATS: Mutex<DtnStatistics> = Mutex::new(DtnStatistics::new());
    pub static ref STORE: Mutex<BundleStoresEnum> = Mutex::new(InMemoryBundleStore::new().into());
}

pub fn cla_add(cla: CLAEnum) {
    (*DTNCORE.lock().unwrap()).cl_list.push(cla);
}

/// adds a new peer to the DTN core
/// return true if peer was seen first time
/// return false if peer was already known
pub fn peers_add(peer: DtnPeer) -> bool {
    (*PEERS.lock().unwrap())
        .insert(peer.eid.node().unwrap(), peer)
        .is_none()
}

pub fn peers_get_for_node(eid: &EndpointID) -> Option<DtnPeer> {
    for (_, p) in (*PEERS.lock().unwrap()).iter() {
        if p.node_name() == eid.node().unwrap_or_default() {
            return Some(p.clone());
        }
    }
    None
}

pub fn is_local_node_id(eid: &EndpointID) -> bool {
    eid.node_id() == (*CONFIG.lock().unwrap()).host_eid.node_id()
}

pub fn peers_cla_for_node(eid: &EndpointID) -> Option<crate::cla::ClaSender> {
    if let Some(peer) = peers_get_for_node(eid) {
        return peer.first_cla();
    }
    None
}

pub fn peer_find_by_remote(addr: &PeerAddress) -> Option<String> {
    for (_, p) in (*PEERS.lock().unwrap()).iter() {
        if p.addr() == addr {
            return Some(p.node_name());
        }
    }
    None
}

pub fn store_push_bundle(bndl: &Bundle) -> Result<()> {
    (*STORE.lock().unwrap()).push(bndl)
}

pub fn store_remove(bid: &str) {
    if let Err(err) = (*STORE.lock().unwrap()).remove(bid) {
        error!("store_remove: {}", err);
    }
}

pub fn store_update_metadata(bp: &BundlePack) -> Result<()> {
    (*STORE.lock().unwrap()).update_metadata(bp)
}

pub fn store_has_item(bid: &str) -> bool {
    (*STORE.lock().unwrap()).has_item(bid)
}

pub fn store_get_bundle(bpid: &str) -> Option<Bundle> {
    (*STORE.lock().unwrap()).get_bundle(bpid)
}

pub fn store_get_metadata(bpid: &str) -> Option<BundlePack> {
    (*STORE.lock().unwrap()).get_metadata(bpid)
}

pub fn routing_notify(notification: RoutingNotifcation) {
    (*DTNCORE.lock().unwrap()).routing_agent.notify(notification);
}

pub fn broadcast(bundle: &Bundle) {
    info!("Broadcasting {:?} | {:?}", bundle.id(), bp7::dtn_time_now());
    debug!("Received raw: {:?}", bundle);

    for p in (*PEERS.lock().unwrap()).values() {
        let dst: EndpointID = p.eid.clone();
        let dst: EndpointID = format!("{}/in", dst).as_str().try_into().unwrap();
        let lifetime = std::time::Duration::from_secs(60 * 60);
        let src = (*CONFIG.lock().unwrap()).host_eid.clone();
        let flags = BundleControlFlags::BUNDLE_MUST_NOT_FRAGMENTED
            | BundleControlFlags::BUNDLE_STATUS_REQUEST_DELIVERY;
        let pblock = bp7::primary::PrimaryBlockBuilder::default()
            .bundle_control_flags(flags.bits())
            .destination(dst.clone())
            .source(src.clone())
            .report_to(src)
            .creation_timestamp(CreationTimestamp::now())
            .lifetime(lifetime)
            .build()
            .unwrap();


        let mut new_bndl = bp7::bundle::BundleBuilder::default()
            .primary(pblock)
            .canonicals(bundle.canonicals.to_vec())
            .build()
            .unwrap();
        new_bndl.set_crc(bp7::crc::CRC_NO);

        tokio::spawn(crate::core::processing::send_bundle(new_bndl));
    }
}
