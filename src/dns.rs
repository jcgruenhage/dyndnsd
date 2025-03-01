use std::{
    fmt::Display,
    net::{AddrParseError, Ipv4Addr, Ipv6Addr, SocketAddr},
    num::ParseIntError,
    str::FromStr,
    sync::Arc,
};

use anyhow::Context;
use hickory_client::client::{Client, ClientHandle};
use hickory_proto::{
    dnssec::{rdata::tsig::TsigAlgorithm, tsig::TSigner},
    rr::{Name, RData, Record},
    runtime::TokioRuntimeProvider,
    tcp::TcpClientStream,
    udp::UdpClientStream,
};
use serde::{Deserialize, Serialize};
use serde_with::{base64::Base64, serde_as, DisplayFromStr};
use thiserror::Error;

#[serde_as]
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Config {
    #[serde_as(as = "DisplayFromStr")]
    url: ConnectionUrl,
    #[serde_as(as = "DisplayFromStr")]
    key_name: Name,
    #[serde_as(as = "Base64")]
    key: Vec<u8>,
    algorithm: TsigAlgorithm,
}

#[derive(Clone, Debug)]
pub enum ConnectionScheme {
    Tcp,
    Udp,
}

#[derive(Clone, Debug)]
pub struct ConnectionUrl {
    scheme: ConnectionScheme,
    address: SocketAddr,
}

impl Display for ConnectionUrl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.scheme {
            ConnectionScheme::Tcp => f.write_str("tcp://")?,
            ConnectionScheme::Udp => f.write_str("udp://")?,
        };
        if self.address.is_ipv6() {
            f.write_str("[")?
        };
        f.write_str(&self.address.ip().to_string())?;
        if self.address.is_ipv6() {
            f.write_str("]")?
        };
        f.write_str(":")?;
        f.write_str(&self.address.port().to_string())?;
        Ok(())
    }
}
impl FromStr for ConnectionUrl {
    type Err = ConnectionUrlError;

    fn from_str(url: &str) -> Result<Self, Self::Err> {
        let (host, is_tcp) = if let Some(host) = url.strip_prefix("udp://") {
            (host, false)
        } else if let Some(host) = url.strip_prefix("tcp://") {
            (host, true)
        } else {
            (url, false)
        };
        let (host, port) = if let Some(host) = host.strip_prefix('[') {
            let (host, maybe_port) = host
                .rsplit_once(']')
                .ok_or(ConnectionUrlError::MalformedV6)?;

            (
                host,
                maybe_port
                    .rsplit_once(':')
                    .map(|(_, port)| port)
                    .unwrap_or("53"),
            )
        } else if let Some((host, port)) = host.rsplit_once(':') {
            (host, port)
        } else {
            (host, "53")
        };

        let address = SocketAddr::new(host.parse()?, port.parse()?);

        if is_tcp {
            Ok(ConnectionUrl {
                scheme: ConnectionScheme::Tcp,
                address,
            })
        } else {
            Ok(ConnectionUrl {
                scheme: ConnectionScheme::Udp,
                address,
            })
        }
    }
}

#[derive(Error, Debug)]
pub enum ConnectionUrlError {
    #[error("The DNS connection URL contains an opening bracket indicating an IPv6 literal, but does not contain a closing bracket.")]
    MalformedV6,
    #[error("Failure parsing IP address: {0}")]
    IpParsing(#[from] AddrParseError),
    #[error("Failure parsing port: {0}")]
    PortParsing(#[from] ParseIntError),
}

pub struct Updater {
    client: Client,
}

impl Updater {
    pub async fn new(config: Config) -> anyhow::Result<Self> {
        let signer = TSigner::new(config.key, config.algorithm, config.key_name, 60)?;
        let client = match config.url.scheme {
            ConnectionScheme::Udp => {
                let conn =
                    UdpClientStream::builder(config.url.address, TokioRuntimeProvider::default())
                        .with_signer(Some(Arc::new(signer)))
                        .build();
                let (client, bg) = Client::connect(conn).await?;
                tokio::spawn(bg);
                client
            }
            ConnectionScheme::Tcp => {
                let (stream, sender) = TcpClientStream::new(
                    config.url.address,
                    None,
                    None,
                    TokioRuntimeProvider::default(),
                );
                let (client, bg) = Client::new(stream, sender, Some(Arc::new(signer))).await?;
                tokio::spawn(bg);
                client
            }
        };
        Ok(Self { client })
    }

    async fn replace(&mut self, rdata: RData, name: Name, origin: Name) -> anyhow::Result<()> {
        self.client
            .delete_rrset(
                Record::update0(name.clone(), 0, rdata.record_type()),
                origin.clone(),
            )
            .await
            .context("Failed to delete old record")?;
        self.client
            .create(Record::from_rdata(name, 60, rdata), origin)
            .await
            .context("Failed to set new record")?;
        Ok(())
    }

    pub async fn set_ipv4(
        &mut self,
        addr: Ipv4Addr,
        name: Name,
        origin: Name,
    ) -> anyhow::Result<()> {
        self.replace(RData::A(addr.into()), name, origin)
            .await
            .context("Failed to replace A record")
    }

    pub async fn set_ipv6(
        &mut self,
        addr: Ipv6Addr,
        name: Name,
        origin: Name,
    ) -> anyhow::Result<()> {
        self.replace(RData::AAAA(addr.into()), name, origin)
            .await
            .context("Failed to replace AAAA record")
    }
}
