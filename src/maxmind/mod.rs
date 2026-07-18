pub mod asn;
pub mod city;

use std::net::IpAddr;

use maxminddb::Reader;
use serde::{Deserialize, Serialize};
use tracing::info;

use crate::{
    config::types::MaxMindConfig,
    maxmind::{asn::AsnMin, city::CityMin},
};

#[derive(Debug)]
pub struct MaxMind {
    pub city: Option<Reader<Vec<u8>>>,
    pub asn: Option<Reader<Vec<u8>>>,
}

#[derive(Deserialize, Serialize, Clone, Debug, Default)]
pub struct LookupResponse {
    pub city: Option<CityMin>,
    pub asn: Option<AsnMin>,
}

impl MaxMind {
    pub fn new(config: MaxMindConfig) -> anyhow::Result<Self> {
        let city = config
            .city_enable
            .then(|| Reader::open_readfile(config.city))
            .transpose()?;
        let asn = config
            .asn_enable
            .then(|| Reader::open_readfile(config.asn))
            .transpose()?;

        if let Some(ref city) = city {
            if config.city_db_check {
                info!("verifying maxmind City");
                city.verify()?;
                info!(metadata = ?city.metadata(), "maxmind City DB verified and loaded");
            }
            info!(metadata = ?city.metadata(), "maxmind City DB loaded");
        }

        if let Some(ref asn) = asn {
            if config.asn_db_check {
                info!("verifying maxmind ASN");
                asn.verify()?;
                info!(metadata = ?asn.metadata(), "maxmind ASN DB verified and loaded");
            }
            info!(metadata = ?asn.metadata(), "maxmind ASN DB loaded");
        }

        Ok(Self { city, asn })
    }

    pub fn lookup(&self, ip: IpAddr) -> anyhow::Result<LookupResponse> {
        Ok(LookupResponse {
            city: self
                .city
                .as_ref()
                .map(|c| c.lookup(ip)?.decode::<CityMin>())
                .transpose()?
                .flatten(),
            asn: self
                .asn
                .as_ref()
                .map(|a| a.lookup(ip)?.decode::<AsnMin>())
                .transpose()?
                .flatten(),
        })
    }
}
