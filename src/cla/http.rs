use std::net::SocketAddr;

use async_trait::async_trait;
use bp7::ByteBuffer;
use embedded_svc::http::client::{Client, Request};
use esp_idf_svc::http::client::EspHttpClient;
use log::{debug, error};

use crate::cla::ConvergenceLayerAgent;
use crate::utils::CONFIG;

#[derive(Debug, Clone, Default, Copy)]
pub struct HttpConvergenceLayer {
    local_port: u16,
}

impl HttpConvergenceLayer {
    pub fn new(port: Option<u16>) -> HttpConvergenceLayer {
        HttpConvergenceLayer {
            local_port: port.unwrap_or((*CONFIG.lock().unwrap()).webport),
        }
    }
}

#[async_trait]
impl ConvergenceLayerAgent for HttpConvergenceLayer {
    async fn setup(&mut self) {}
    fn port(&self) -> u16 {
        self.local_port
    }
    fn name(&self) -> &'static str {
        "http"
    }

    async fn scheduled_submission(&self, dest: &str, ready: &[ByteBuffer]) -> bool {
        debug!("Scheduled HTTP submission: {:?}", dest);
        if !ready.is_empty() {
            let client = EspHttpClient::new_default();
            if let Err(err) = client {
                error!("error pushing bundle to remote: {}", err);
                return false;
            }
            let mut client = client.unwrap();
            let peeraddr: SocketAddr = dest.parse().unwrap();
            debug!("forwarding to {:?}", peeraddr);
            for b in ready {
                let req_url = format!("http://{}:{}/push", peeraddr.ip(), peeraddr.port());
                let post = client.post(&req_url);
                if let Err(err) = post {
                    error!("error pushing bundle to remote: {}", err);
                    return false;
                }
                if let Err(err) = post.unwrap().send_bytes(b) {
                    error!("error pushing bundle to remote: {}", err);
                    return false;
                }
                debug!("successfully sent bundle to {}", peeraddr.ip());
            }
            debug!("successfully sent {} bundles to {}", ready.len(), dest);
        } else {
            debug!("Nothing to forward.");
        }
        true
    }
}

impl std::fmt::Display for HttpConvergenceLayer {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "http:{}", self.local_port)
    }
}
