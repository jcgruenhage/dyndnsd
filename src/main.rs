// Copyright (C) 2021, 2023 Jan Christian Grünhage <jan.christian@gruenhage.xyz>
//
// This file is part of cloudflare-ddns-service.
//
// cloudflare-ddns-service is non-violent software: you can use, redistribute, and/or modify it
// under the terms of the CNPLv7+ as found in the LICENSE.md file in the source code root directory
// or at <https://git.pixie.town/thufie/npl-builder>.
//
// cloudflare-ddns-service comes with ABSOLUTELY NO WARRANTY, to the extent permitted by applicable
// law. See the LICENSE.md for details.

mod network;

use anyhow::{Context, Result};
use network::{get_record, get_zone, update_record};
use serde::{Deserialize, Serialize};
use serde_yaml::{from_str, to_writer};
use std::{
    fs::{create_dir_all, read_to_string, File},
    net::{Ipv4Addr, Ipv6Addr},
    path::PathBuf,
    time::Duration,
};
use tokio::time::interval;

use cloudflare::{
    endpoints::dns::DnsContent,
    framework::{async_api::Client, auth::Credentials, Environment, HttpApiClientConfig},
};

#[derive(Serialize, Deserialize)]
struct Config {
    api_token: String,
    zone: String,
    domain: String,
    #[serde(default = "yes")]
    ipv4: bool,
    #[serde(default = "no")]
    ipv6: bool,
    #[serde(default = "default_duration")]
    interval: u64,
}

#[derive(Serialize, Deserialize, Default)]
struct Cache {
    v4: Option<Ipv4Addr>,
    v6: Option<Ipv6Addr>,
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let config_string = read_to_string("/etc/cloudflare-ddns-service/config.yaml")
        .context("couldn't read config file!")?;
    let config: Config = from_str(&config_string).context("Failed to parse config file")?;
    let cache_dir = PathBuf::from("/var/cache/cloudflare-ddns-service");
    let cache_path = cache_dir.join("cache.yaml");
    let mut cache = match read_to_string(&cache_path).map(|str| from_str(&str)) {
        Ok(Ok(cache)) => cache,
        _ => {
            create_dir_all(cache_dir)?;
            Cache::default()
        }
    };

    let mut interval = interval(Duration::new(config.interval, 0));
    let mut client = Client::new(
        Credentials::UserAuthToken {
            token: config.api_token.clone(),
        },
        HttpApiClientConfig::default(),
        Environment::Production,
    )
    .context("Failed to initiate cloudflare API client")?;
    let zone = get_zone(config.zone.clone(), &mut client)
        .await
        .context("Failed to get zone")?;
    loop {
        if let Err(error) = update(&config, &mut cache, &cache_path, &zone, &mut client).await {
            log::error!("Failed to update record: {}", error);
        }
        interval.tick().await;
    }
}

async fn update(
    config: &Config,
    cache: &mut Cache,
    cache_path: &PathBuf,
    zone: &str,
    client: &mut Client,
) -> Result<()> {
    if config.ipv4 {
        let current = public_ip::addr_v4()
            .await
            .context("Failed to query current IPv4 address")?;
        log::debug!("fetched current IP: {}", current.to_string());
        match cache.v4 {
            Some(old) if old == current => {
                log::debug!("ipv4 unchanged, continuing...");
            }
            _ => {
                log::info!("ipv4 changed, setting record");
                let rid = get_record(zone, config.domain.clone(), network::A_RECORD, client)
                    .await
                    .context("couldn't find record!")?;
                log::debug!("got record ID {}", rid);
                update_record(
                    zone,
                    &rid,
                    &config.domain,
                    DnsContent::A { content: current },
                    client,
                )
                .await
                .context("Failed to set DNS record")?;
                cache.v4 = Some(current);
                write_cache(cache, cache_path)
                    .context("Failed to write current IPv4 address to cache")?;
            }
        }
    }
    if config.ipv6 {
        let current = public_ip::addr_v6()
            .await
            .context("Failed to query current IPv4 address")?;
        log::debug!("fetched current IP: {}", current.to_string());
        match cache.v6 {
            Some(old) if old == current => {
                log::debug!("ipv6 unchanged, continuing...")
            }
            _ => {
                log::info!("ipv6 changed, setting record");
                let rid = get_record(zone, config.domain.clone(), network::AAAA_RECORD, client)
                    .await
                    .context("couldn't find record!")?;
                log::debug!("got record ID {}", rid);
                update_record(
                    zone,
                    &rid,
                    &config.domain,
                    DnsContent::AAAA { content: current },
                    client,
                )
                .await
                .context("Failed to set DNS record")?;
                cache.v6 = Some(current);
                write_cache(cache, cache_path)
                    .context("Failed to write current IPv4 address to cache")?;
            }
        }
    }
    Ok(())
}

fn write_cache(cache: &mut Cache, cache_path: &PathBuf) -> Result<()> {
    to_writer(
        File::create(cache_path).context("Failed to open cache file for writing")?,
        cache,
    )
    .context("Failed to serialize cache into file")?;
    Ok(())
}

fn yes() -> bool {
    true
}

fn no() -> bool {
    false
}

fn default_duration() -> u64 {
    60
}
