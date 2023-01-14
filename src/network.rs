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

use std::net::{Ipv4Addr, Ipv6Addr};

use anyhow::{Context, Result};
use cloudflare::{
    endpoints::{
        dns::{
            DnsContent, ListDnsRecords, ListDnsRecordsParams, UpdateDnsRecord,
            UpdateDnsRecordParams,
        },
        zone::{ListZones, ListZonesParams},
    },
    framework::async_api::Client as CfClient,
};
use reqwest::Client as ReqwClient;

pub const A_RECORD: DnsContent = DnsContent::A {
    content: Ipv4Addr::UNSPECIFIED,
};
pub const AAAA_RECORD: DnsContent = DnsContent::AAAA {
    content: Ipv6Addr::UNSPECIFIED,
};

pub async fn get_current_ipv4(client: &mut ReqwClient) -> Result<Ipv4Addr> {
    Ok(client
        .get("https://ipv4.icanhazip.com")
        .send()
        .await
        .context("Failed to query current IPv4 from ipv4.icanhazip.com")?
        .text()
        .await
        .context("Failed to read text body")?
        .trim()
        .parse()
        .context("Failed to parse IPv4 address returned by ipv4.icanhazip.com")?)
}

pub async fn get_current_ipv6(client: &mut ReqwClient) -> Result<Ipv6Addr> {
    Ok(client
        .get("https://ipv6.icanhazip.com")
        .send()
        .await
        .context("Failed to query current IPv6 from ipv6.icanhazip.com")?
        .text()
        .await
        .context("Failed to read text body")?
        .trim()
        .parse()
        .context("Failed to parse IPv6 address returned by ipv6.icanhazip.com")?)
}

pub async fn get_zone(domain: String, cf_client: &mut CfClient) -> Result<String> {
    Ok(cf_client
        .request_handle(&ListZones {
            params: ListZonesParams {
                name: Some(domain),
                status: None,
                page: None,
                per_page: None,
                order: None,
                direction: None,
                search_match: None,
            },
        })
        .await
        .context("Failed to query zone from cf_client")?
        .result[0]
        .id
        .clone())
}

pub async fn get_record(
    zone_identifier: &str,
    domain: String,
    r#type: DnsContent,
    cf_client: &mut CfClient,
) -> Result<String> {
    Ok(cf_client
        .request_handle(&ListDnsRecords {
            zone_identifier,
            params: ListDnsRecordsParams {
                record_type: None,
                name: Some(domain),
                page: None,
                per_page: None,
                order: None,
                direction: None,
                search_match: None,
            },
        })
        .await
        .context("Couldn't fetch record")?
        .result
        .iter()
        .find(|record| std::mem::discriminant(&record.content) == std::mem::discriminant(&r#type))
        .context("No matching record found")?
        .id
        .clone())
}

pub async fn update_record(
    zone_identifier: &str,
    identifier: &str,
    name: &str,
    content: DnsContent,
    cf_client: &mut CfClient,
) -> Result<()> {
    cf_client
        .request_handle(&UpdateDnsRecord {
            zone_identifier,
            identifier,
            params: UpdateDnsRecordParams {
                ttl: None,
                proxied: Some(false),
                name,
                content,
            },
        })
        .await?;
    Ok(())
}
