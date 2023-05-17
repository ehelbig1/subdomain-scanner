use crate::error::Error;
use crate::model;
use futures::{stream, StreamExt};
use std::collections;
use std::sync::Arc;
use std::time;
use trust_dns_resolver;

pub async fn enumerate(
    http_client: &reqwest::Client,
    target: &str,
) -> Result<Vec<model::Subdomain>, Error> {
    let response = http_client
        .get(format!("https://crt.sh/?q=%25.{}&output=json", target))
        .send()
        .await?;

    if response.status() != 200 {
        return Err(Error::Reqwest(format!("Request to crt.sh failed: {}", response.status())))
    }

    let entries: Vec<model::CrtShEntry> = response
        .json()
        .await?;

    let subdomains: collections::HashSet<String> = entries
        .into_iter()
        .flat_map(|entry| {
            entry
                .name_value
                .split('\n')
                .map(|subdomain| subdomain.trim().to_string())
                .collect::<Vec<String>>()
        })
        .filter(|subdomain| !subdomain.contains('*'))
        .collect();

    let mut dns_opts = trust_dns_resolver::config::ResolverOpts::default();
    dns_opts.timeout = time::Duration::from_secs(4);

    let dns_resolver = Arc::new(
        trust_dns_resolver::TokioAsyncResolver::tokio(
            trust_dns_resolver::config::ResolverConfig::default(),
            dns_opts,
        )
        .unwrap(),
    );

    let subdomains: Vec<model::Subdomain> = stream::iter(subdomains.into_iter())
        .map(|subdomain| model::Subdomain {
            domain: subdomain,
            open_ports: Vec::new(),
        })
        .filter_map(|subdomain| {
            let dns_resolver = dns_resolver.clone();
            async move {
                if resolves(dns_resolver, &subdomain).await {
                    Some(subdomain)
                } else {
                    None
                }
            }
        })
        .collect()
        .await;

    Ok(subdomains)
}

async fn resolves(
    dns_resolver: Arc<
        trust_dns_resolver::AsyncResolver<
            trust_dns_resolver::name_server::GenericConnection,
            trust_dns_resolver::name_server::GenericConnectionProvider<
                trust_dns_resolver::name_server::TokioRuntime,
            >,
        >,
    >,
    subdomain: &model::Subdomain,
) -> bool {
    dns_resolver.lookup_ip(&subdomain.domain).await.is_ok()
}
