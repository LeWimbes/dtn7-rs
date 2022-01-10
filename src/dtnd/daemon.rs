use std::convert::TryFrom;

use bp7::EndpointID;
use log::{error, info};

use crate::cla::ConvergenceLayerAgent;
use crate::core::application_agent::SimpleApplicationAgent;
use crate::dtnconfig::DtnConfig;
use crate::ipnd::neighbour_discovery;
use crate::utils::{cla_add, CONFIG, DTNCORE, peers_add, STORE};

use super::{httpd, janitor};

/*
use crate::core::core::DtnCore;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};

#[derive(Debug)]
pub enum DtnCmd {
    DtnCore(Sender<DtnCmdResult>),
}

#[derive(Debug)]
pub enum DtnCmdResult {
    DtnCore(Sender<DtnCore>, DtnCore),
}

pub fn access_core<F>(tx: Sender<DtnCmd>, mut f: F)
where
    F: FnMut(&mut DtnCore),
{
    let (tx2, rx2) = mpsc::channel();
    tx.send(DtnCmd::DtnCore(tx2)).unwrap();
    let DtnCmdResult::DtnCore(tx3, mut core) = rx2.recv().expect("Couldn't access dtn core!");
    f(&mut core);
    tx3.send(core).expect("IPC Error");
}

fn spawn_core_daemon(rx: Receiver<DtnCmd>) {
    for received in rx {
        //println!("Got: {:?}", received);
        match received {
            DtnCmd::DtnCore(tx) => {
                let (tx2, rx2) = mpsc::channel();
                tx.send(DtnCmdResult::DtnCore(tx2, core))
                    .expect("IPC Error");
                core = rx2.recv().unwrap();
            }
        };
    }
}*/

async fn start_convergencylayers() {
    info!("Starting convergency layers");

    for cl in &mut (*DTNCORE.lock().unwrap()).cl_list {
        info!("Setup {}", cl);
        cl.setup().await;
    }
}

pub async fn start_dtnd(cfg: DtnConfig) -> anyhow::Result<()> {
    {
        (*CONFIG.lock().unwrap()).set(cfg);
    }
    info!("Local Node ID: {}", (*CONFIG.lock().unwrap()).host_eid);

    (*STORE.lock().unwrap()) = crate::core::mem_store::InMemoryBundleStore::new().into();

    info!(
        "Announcement Interval: {}",
        humantime::format_duration((*CONFIG.lock().unwrap()).announcement_interval)
    );

    info!(
        "Janitor Interval: {}",
        humantime::format_duration((*CONFIG.lock().unwrap()).janitor_interval)
    );

    info!(
        "Peer Timeout: {}",
        humantime::format_duration((*CONFIG.lock().unwrap()).peer_timeout)
    );

    info!("Web Port: {}", (*CONFIG.lock().unwrap()).webport);
    info!("IPv4: {}", (*CONFIG.lock().unwrap()).v4);
    info!("IPv6: {}", (*CONFIG.lock().unwrap()).v6);

    info!(
        "Generate Status Reports: {}",
        (*CONFIG.lock().unwrap()).generate_status_reports
    );

    let routing = (*CONFIG.lock().unwrap()).routing.clone();
    (*DTNCORE.lock().unwrap()).routing_agent = crate::routing::new(&routing);

    info!("RoutingAgent: {}", (*DTNCORE.lock().unwrap()).routing_agent);

    info!("Adding CLA: \"http\"");
    cla_add(crate::cla::http::HttpConvergenceLayer::new(None).into());

    for s in &(*CONFIG.lock().unwrap()).statics {
        let port_str = if s.cla_list[0].1.is_some() {
            format!(":{}", s.cla_list[0].1.unwrap())
        } else {
            "".into()
        };
        info!(
            "Adding static peer: {}://{}{}/{}",
            s.cla_list[0].0,
            s.addr,
            port_str,
            s.eid.node().unwrap()
        );
        peers_add(s.clone());
    }

    let local_host_id = (*CONFIG.lock().unwrap()).host_eid.clone();
    (*DTNCORE.lock().unwrap())
        .register_application_agent(SimpleApplicationAgent::with(local_host_id.clone()).into());
    for e in &(*CONFIG.lock().unwrap()).endpoints {
        let eid = if let Ok(eid) = EndpointID::try_from(e.clone()) {
            // TODO: add check if non-local ID that service name is non-singleton ('~') for naming scheme dtn
            eid
        } else {
            local_host_id
                .new_endpoint(e)
                .expect("Error constructing new endpoint")
        };

        (*DTNCORE.lock().unwrap()).register_application_agent(SimpleApplicationAgent::with(eid).into());
    }
    start_convergencylayers().await;
    if CONFIG.lock().unwrap().janitor_interval.as_micros() != 0 {
        janitor::spawn_janitor();
    }
    if CONFIG.lock().unwrap().announcement_interval.as_micros() != 0 {
        if let Err(errmsg) = neighbour_discovery::spawn_neighbour_discovery().await {
            error!("Error spawning service discovery: {:?}", errmsg);
        }
    }
    httpd::spawn_httpd()?;
    println!("Done!");
    Ok(())
}
