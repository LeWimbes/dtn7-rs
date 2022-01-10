use std::convert::TryFrom;
use std::net::SocketAddr;

use anyhow::Result;
use async_trait::async_trait;
use axum::{
    extract::{self, connect_info::ConnectInfo, extractor_middleware, RequestParts},
    Router,
    routing::post,
};
use http::StatusCode;
use log::{debug, info, warn};

use crate::utils::CONFIG;

struct RequireLocalhost;

#[async_trait]
impl<B> extract::FromRequest<B> for RequireLocalhost
    where
        B: Send,
{
    type Rejection = StatusCode;

    async fn from_request(conn: &mut RequestParts<B>) -> Result<Self, Self::Rejection> {
        if (*CONFIG.lock().unwrap()).unsafe_httpd {
            return Ok(Self);
        }
        if let Some(ext) = conn.extensions() {
            if let Some(ConnectInfo(addr)) = ext.get::<ConnectInfo<SocketAddr>>() {
                if addr.ip().is_loopback() {
                    return Ok(Self);
                } else if let std::net::IpAddr::V6(ipv6) = addr.ip() {
                    // workaround for bug in std when handling IPv4 in IPv6 addresses
                    if let Some(ipv4) = ipv6.to_ipv4() {
                        if ipv4.is_loopback() {
                            return Ok(Self);
                        }
                    }
                }
            }
        }

        Err(StatusCode::FORBIDDEN)
    }
}

async fn push_post(body: bytes::Bytes) -> Result<String, (StatusCode, String)> {
    let bytes = body.to_vec();

    let b_len = bytes.len();
    debug!("Received: {:?}", b_len);
    if let Ok(bndl) = bp7::Bundle::try_from(bytes.to_vec()) {
        info!("Received bundle {}", bndl.id());
        if let Err(err) = crate::core::processing::receive(bndl).await {
            warn!("Error processing bundle: {}", err);
            Err((
                StatusCode::BAD_REQUEST,
                format!("Error processing bundle: {}", err),
            ))
        } else {
            Ok(format!("Received {} bytes", b_len))
        }
    } else {
        Err((
            StatusCode::BAD_REQUEST,
            "Error decoding bundle!".to_string(),
        ))
    }
}

pub async fn spawn_httpd() -> Result<()> {
    let app = Router::new()
        .layer(extractor_middleware::<RequireLocalhost>())
        .route("/push", post(push_post));


    let port = (*CONFIG.lock().unwrap()).webport;

    let v4 = (*CONFIG.lock().unwrap()).v4;
    let v6 = (*CONFIG.lock().unwrap()).v6;
    //debug!("starting webserver");
    let server = if v4 && !v6 {
        hyper::Server::bind(&format!("0.0.0.0:{}", port).parse()?)
    } else if !v4 && v6 {
        hyper::Server::bind(&format!("[::1]:{}", port).parse()?)
    } else {
        hyper::Server::bind(&format!("[::]:{}", port).parse()?)
    }
        .serve(app.into_make_service_with_connect_info::<SocketAddr, _>());
    server.await?;
    Ok(())
}
