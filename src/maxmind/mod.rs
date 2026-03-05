pub mod asn;
pub mod city;

use std::net::IpAddr;

use maxminddb::Reader;
use serde::{Deserialize, Serialize};

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
		Ok(Self {
			city: config
				.city_enable
				.then(|| Reader::open_readfile(config.city))
				.transpose()?,
			asn: config
				.asn_enable
				.then(|| Reader::open_readfile(config.asn))
				.transpose()?,
		})
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
