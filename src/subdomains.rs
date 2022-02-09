// use crate::{
//     model::{CrtShEntry, Subdomain},
//     Error,
// };

use std::{collections::HashSet, time::Duration};

use reqwest::blocking::Client;
use trust_dns_resolver::{
    config::{ResolverConfig, ResolverOpts},
    Resolver,
};

use crate::{
    model::{CrtShEntry, Subdomain},
    Error,
};
// use std::{collections::HashSet, time::Duration};
// use trust_dns_resolver::{
//     config::{ResolverConfig, ResolverOpts},
//     Resolver,
// };

pub fn enumerate(http_client: &Client, target: &str) -> Result<Vec<Subdomain>, Error> {
    let entries: Vec<CrtShEntry> = http_client
        .get(&format!("https://crt.sh/?q=%25.{}&output=json", target))
        .send()?
        .json()?;

    let mut subdomains: HashSet<String> = entries
        .into_iter()
        .map(|entry| {
            entry
                .name_value
                .split("\n")
                .map(|subdomain| subdomain.trim().to_string())
                .collect::<Vec<String>>()
        })
        .flatten()
        .filter(|subdomain| subdomain != target)
        .filter(|subdomain| !subdomain.contains("*"))
        .collect();

    subdomains.insert(target.to_string());

    let subdomains: Vec<Subdomain> = subdomains
        .into_iter()
        .map(|domain| Subdomain {
            domain,
            open_ports: Vec::new(),
        })
        .filter(resolves)
        .collect();

    Ok(subdomains)
}

pub fn resolves(subdomain: &Subdomain) -> bool {
    let dns_resolver = Resolver::new(
        ResolverConfig::default(),
        ResolverOpts {
            timeout: Duration::from_secs(4),
            ..ResolverOpts::default()
        },
    )
    .expect("subdomain resolver: building DNS client");

    let is_ok = dns_resolver.lookup_ip(subdomain.domain.as_str()).is_ok();

    println!("{} is ok {}", subdomain.domain, is_ok);

    is_ok
}
