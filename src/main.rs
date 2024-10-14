// Copyright (C) 2021-2024 Jan Christian Grünhage <jan.christian@gruenhage.xyz>
//
// This file is part of dyndnsd.
//
// dyndnsd is non-violent software: you can use, redistribute, and/or modify it
// under the terms of the CNPLv7+ as found in the LICENSE.md file in the source code root directory
// or at <https://git.pixie.town/thufie/npl-builder>.
//
// dyndnsd comes with ABSOLUTELY NO WARRANTY, to the extent permitted by applicable
// law. See the LICENSE.md for details.

use anyhow::{bail, Context, Result};
use dns_update::{DnsRecord, DnsUpdater, DnsUpdaterConfig};
use serde::{Deserialize, Serialize};
use tokio::time::interval;
use toml::{from_str, to_string};

use std::{
    fs::{create_dir_all, read_to_string, File},
    io::Write,
    net::{Ipv4Addr, Ipv6Addr},
    path::PathBuf,
    time::Duration,
};

#[derive(Serialize, Deserialize)]
struct Config {
    dns_provider_config: DnsUpdaterConfig,
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

    let config_string =
        read_to_string("/etc/dyndnsd/config.toml").context("couldn't read config file!")?;
    let config: Config = from_str(&config_string).context("Failed to parse config file")?;
    let cache_dir = PathBuf::from("/var/cache/dyndnsd");
    let cache_path = cache_dir.join("cache.toml");
    let mut cache = match read_to_string(&cache_path).map(|str| from_str(&str)) {
        Ok(Ok(cache)) => cache,
        _ => {
            create_dir_all(cache_dir)?;
            Cache::default()
        }
    };

    let mut interval = interval(Duration::new(config.interval, 0));
    let mut dns_updater: DnsUpdater = config
        .dns_provider_config
        .clone()
        .try_into()
        .context("Failed to initiate DNS updater")?;
    loop {
        if let Err(error) = update(&config, &mut cache, &cache_path, &mut dns_updater).await {
            log::error!("Failed to update record: {}", error);
        }
        interval.tick().await;
    }
}

async fn update(
    config: &Config,
    cache: &mut Cache,
    cache_path: &PathBuf,
    dns_updater: &mut DnsUpdater,
) -> Result<()> {
    let mut records = Vec::new();
    let mut update_required = false;

    if config.ipv4 {
        let current = public_ip::addr_v4()
            .await
            .context("Failed to query current IPv4 address")?;
        log::debug!("fetched current IP: {}", current.to_string());
        records.push((
            &config.domain,
            DnsRecord::A { content: current },
            300,
            &config.zone,
        ));
        match cache.v4 {
            Some(old) if old == current => {
                log::debug!("ipv4 unchanged, continuing...");
            }
            _ => {
                log::info!("ipv4 changed, setting record");
                update_required = true;
            }
        }
    }
    if config.ipv6 {
        let current = public_ip::addr_v6()
            .await
            .context("Failed to query current IPv4 address")?;
        log::debug!("fetched current IP: {}", current.to_string());
        records.push((
            &config.domain,
            DnsRecord::AAAA { content: current },
            300,
            &config.zone,
        ));
        match cache.v6 {
            Some(old) if old == current => {
                log::debug!("ipv6 unchanged, continuing...")
            }
            _ => {
                log::info!("ipv6 changed, setting record");
                update_required = true;
            }
        }

        if update_required {
            dns_updater.delete(&config.domain, &config.zone).await?;
            for record in records {
                let cloned_record = match record.1 {
                    DnsRecord::A { content } => DnsRecord::A { content },
                    DnsRecord::AAAA { content } => DnsRecord::AAAA { content },
                    _ => bail!("This code should be unreachable"),
                };
                dns_updater
                    .create(record.0, record.1, record.2, record.3)
                    .await?;
                match cloned_record {
                    DnsRecord::A { content } => {
                        cache.v4 = Some(content);
                        write_cache(cache, cache_path)
                            .context("Failed to write current IPv4 address to cache")?;
                    }
                    DnsRecord::AAAA { content } => {
                        cache.v6 = Some(content);
                        write_cache(cache, cache_path)
                            .context("Failed to write current IPv4 address to cache")?;
                    }
                    _ => {}
                };
            }
        }
    }
    Ok(())
}

fn write_cache(cache: &mut Cache, cache_path: &PathBuf) -> Result<()> {
    let cache_str = to_string(cache).context("Failed to serialize cache file")?;
    let mut cache_file =
        File::create(cache_path).context("Failed to open cache file for writing")?;
    cache_file
        .write_all(cache_str.as_bytes())
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
