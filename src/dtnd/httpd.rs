use std::convert::TryFrom;

use anyhow::Result;
use embedded_svc::httpd::registry::Registry;
use log::{debug, info, warn};

use crate::utils::CONFIG;

async fn push_post(bytes: Vec<u8>) -> Result<String, (u16, String)> {
    let b_len = bytes.len();
    debug!("Received: {:?}", b_len);
    if let Ok(bndl) = bp7::Bundle::try_from(bytes.to_vec()) {
        info!("Received bundle {}", bndl.id());
        if let Err(err) = crate::core::processing::receive(bndl).await {
            warn!("Error processing bundle: {}", err);
            Err((
                400,
                format!("Error processing bundle: {}", err),
            ))
        } else {
            Ok(format!("Received {} bytes", b_len))
        }
    } else {
        Err((
            400,
            "Error decoding bundle!".to_string(),
        ))
    }
}

pub fn spawn_httpd() -> Result<()> {
    let port = (*CONFIG.lock().unwrap()).webport;

    let server2 = esp_idf_svc::httpd::ServerRegistry::new()
        .at("/push")
        .post(move |mut request| {
            let bytes = request.as_bytes()?;
            if let Err(_) = smol::block_on(push_post(bytes)) {
                Ok(embedded_svc::httpd::Response {
                    status: 500,
                    status_message: None,
                    headers: Default::default(),
                    body: Default::default(),
                    new_session_state: None,
                })
            } else {
                Ok(embedded_svc::httpd::Response {
                    status: 200,
                    status_message: None,
                    headers: Default::default(),
                    body: Default::default(),
                    new_session_state: None,
                })
            }
        })?;
    server2.start(&esp_idf_svc::httpd::Configuration {
        http_port: port,
        https_port: port + 10,
    })?;
    Ok(())
}
