#[macro_use]
extern crate lazy_static;

mod action;
mod filter;
mod handler;
mod rule;

use hudsucker::{rustls::internal::pemfile, *};
use log::*;
use std::net::SocketAddr;

async fn shutdown_signal() {
    tokio::signal::ctrl_c()
        .await
        .expect("failed to install CTRL+C signal handler");
}

#[tokio::main]
async fn main() {
    env_logger::init();
    rule::add_rule_examples();

    let mut private_key_bytes: &[u8] = include_bytes!("../assets/ca/private.key");
    let mut ca_cert_bytes: &[u8] = include_bytes!("../assets/ca/cert.crt");
    let private_key = pemfile::pkcs8_private_keys(&mut private_key_bytes)
        .expect("Failed to parse private key")
        .remove(0);
    let ca_cert = pemfile::certs(&mut ca_cert_bytes)
        .expect("Failed to parse CA certificate")
        .remove(0);

    let ca = CertificateAuthority::new(private_key, ca_cert, 1_000)
        .expect("Failed to create Certificate Authority");

    let proxy_config = ProxyConfig {
        listen_addr: SocketAddr::from(([127, 0, 0, 1], 34567)),
        shutdown_signal: shutdown_signal(),
        http_handler: handler::MitmHandler::default(),
        incoming_message_handler: handler::NoopMessageHandler {},
        outgoing_message_handler: handler::NoopMessageHandler {},
        upstream_proxy: None,
        ca,
    };

    if let Err(e) = start_proxy(proxy_config).await {
        error!("{}", e);
    }
}
