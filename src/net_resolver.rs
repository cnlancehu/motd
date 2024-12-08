use std::net::{IpAddr, ToSocketAddrs as _};

use anyhow::Result;
use gamedig::ExtraRequestSettings;

use crate::error;

fn set_hostname_if_missing(host: &str, extra_options: &mut Option<ExtraRequestSettings>) {
    if let Some(extra_options) = extra_options {
        if extra_options.hostname.is_none() {
            // If extra_options exists but hostname is None overwrite hostname in place
            extra_options.hostname = Some(host.to_string())
        }
    } else {
        // If extra_options is None create default settings with hostname
        *extra_options = Some(ExtraRequestSettings::default().set_hostname(host.to_string()));
    }
}

pub fn resolve_ip_or_domain<T: AsRef<str>>(
    host: T,
    extra_options: &mut Option<ExtraRequestSettings>,
) -> Result<IpAddr> {
    let host_str = host.as_ref();
    if let Ok(parsed_ip) = host_str.parse() {
        Ok(parsed_ip)
    } else {
        set_hostname_if_missing(host_str, extra_options);
        resolve_domain(host_str)
    }
}

fn resolve_domain(domain: &str) -> Result<IpAddr> {
    Ok(format!("{}:0", domain)
        .to_socket_addrs()
        .map_err(|_| error::Error::InvalidHostname(domain.to_string()))?
        .next()
        .ok_or_else(|| error::Error::InvalidHostname(domain.to_string()))?
        .ip())
}