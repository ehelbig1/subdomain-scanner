use anyhow::Error;
use std::time;
use futures::{stream, StreamExt};

mod common_ports;
mod error;
mod model;
mod subdomain;
mod ports;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let http_client = reqwest::ClientBuilder::new()
        .timeout(time::Duration::from_secs(6))
        .build()
        .unwrap();

    let mut subdomains = subdomain::enumerate(&http_client, "evanhelbig.com").await?;
    stream::iter(subdomains.iter_mut())
        .for_each_concurrent(50, ports::scan_ports)
        .await;

    println!("{:#?}", subdomains);

    Ok(())
}
