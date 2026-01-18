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
		let city = if config.city_enable {
			Some(Reader::open_readfile(config.city)?)
		} else {
			None
		};

		let asn = if config.asn_enable {
			Some(Reader::open_readfile(config.asn)?)
		} else {
			None
		};

		Ok(Self { city, asn })
	}

	pub fn lookup(&self, ip: IpAddr) -> anyhow::Result<LookupResponse> {
		let city = match &self.city {
			Some(city) => city.lookup(ip)?.decode::<CityMin>()?,
			None => None,
		};

		let asn = match &self.asn {
			Some(asn) => asn.lookup(ip)?.decode::<AsnMin>()?,
			None => None,
		};

		Ok(LookupResponse { city, asn })
	}
}
