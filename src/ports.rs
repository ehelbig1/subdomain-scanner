use crate::{common_ports, model};
use futures::{stream, StreamExt};
use tokio::net;
use std::net::ToSocketAddrs;
use std::time;

pub async fn scan_ports(mut subdomain: &mut model::Subdomain) {
    subdomain.open_ports = stream::iter(common_ports::MOST_COMMON_PORTS_100.iter())
        .map(|port| scan_port(&subdomain.domain, *port))
        .buffer_unordered(50)
        .collect()
        .await;
}

async fn scan_port(domain: &str, port: u16) -> model::Port {
    let socket_addresses: Vec<std::net::SocketAddr> = format!("{}:{}", domain, port)
        .to_socket_addrs()
        .expect("port scanner: Creating socket address")
        .collect();
        
    if socket_addresses.is_empty() {
        return model::Port {
            port,
            is_open: false,
        };
    }

    let is_open = if let Ok(_) =tokio::time::timeout(time::Duration::from_secs(2), net::TcpStream::connect(&socket_addresses[0])).await {
        true
    } else {
        true
    };

    model::Port { port, is_open }
}