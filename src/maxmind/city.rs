use maxminddb::geoip2::city;
use serde::{Deserialize, Serialize};

/// Taken from maxminddb but slighly reduced
#[derive(Deserialize, Serialize, Clone, Debug, Default)]
pub struct CityMin {
	#[serde(default, skip_serializing_if = "City::is_empty")]
	pub city: City,
	#[serde(default, skip_serializing_if = "Continent::is_empty")]
	pub continent: Continent,
	#[serde(default, skip_serializing_if = "Country::is_empty")]
	pub country: Country,
	#[serde(default, skip_serializing_if = "Location::is_empty")]
	pub location: Location,
	#[serde(default, skip_serializing_if = "Postal::is_empty")]
	pub postal: Postal,
	#[serde(default, skip_serializing_if = "Country::is_empty")]
	pub registered_country: Country,
	#[serde(default, skip_serializing_if = "RepresentedCountry::is_empty")]
	pub represented_country: RepresentedCountry,
	#[serde(default, skip_serializing_if = "Vec::is_empty")]
	pub subdivisions: Vec<Subdivision>,
	#[serde(default, skip_serializing_if = "city::Traits::is_empty")]
	pub traits: city::Traits,
}

#[derive(Deserialize, Serialize, Clone, Debug, Default, PartialEq, Eq)]
pub struct Names {
	#[serde(rename = "de", default, skip_serializing_if = "Option::is_none")]
	pub german: Option<String>,
	#[serde(rename = "en", default, skip_serializing_if = "Option::is_none")]
	pub english: Option<String>,
}

impl Names {
	#[must_use]
	pub fn is_empty(&self) -> bool {
		*self == Self::default()
	}
}

#[derive(Deserialize, Serialize, Clone, Debug, Default, PartialEq)]
pub struct City {
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub geoname_id: Option<u32>,
	#[serde(default, skip_serializing_if = "Names::is_empty")]
	pub names: Names,
}

impl City {
	#[must_use]
	pub fn is_empty(&self) -> bool {
		*self == Self::default()
	}
}

#[derive(Deserialize, Serialize, Clone, Debug, Default, PartialEq)]
pub struct Continent {
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub code: Option<String>,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub geoname_id: Option<u32>,
}

impl Continent {
	#[must_use]
	pub fn is_empty(&self) -> bool {
		*self == Self::default()
	}
}

#[derive(Deserialize, Serialize, Clone, Debug, Default, PartialEq)]
pub struct Country {
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub geoname_id: Option<u32>,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub is_in_european_union: Option<bool>,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub iso_code: Option<String>,
}

impl Country {
	#[must_use]
	pub fn is_empty(&self) -> bool {
		*self == Self::default()
	}
}

#[derive(Deserialize, Serialize, Clone, Debug, Default, PartialEq)]
pub struct Location {
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub accuracy_radius: Option<u16>,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub latitude: Option<f64>,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub longitude: Option<f64>,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub metro_code: Option<u16>,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub time_zone: Option<String>,
}

impl Location {
	#[must_use]
	pub fn is_empty(&self) -> bool {
		*self == Self::default()
	}
}

#[derive(Deserialize, Serialize, Clone, Debug, Default, PartialEq)]
pub struct Postal {
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub code: Option<String>,
}

impl Postal {
	#[must_use]
	pub fn is_empty(&self) -> bool {
		*self == Self::default()
	}
}

#[derive(Deserialize, Serialize, Clone, Debug, Default, PartialEq)]
pub struct RepresentedCountry {
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub geoname_id: Option<u32>,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub is_in_european_union: Option<bool>,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub iso_code: Option<String>,
	#[serde(rename = "type", default, skip_serializing_if = "Option::is_none")]
	pub representation_type: Option<String>,
}

impl RepresentedCountry {
	#[must_use]
	pub fn is_empty(&self) -> bool {
		*self == Self::default()
	}
}

#[derive(Deserialize, Serialize, Clone, Debug, Default, PartialEq)]
pub struct Subdivision {
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub geoname_id: Option<u32>,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub iso_code: Option<String>,
	#[serde(default, skip_serializing_if = "Names::is_empty")]
	pub names: Names,
}

impl Subdivision {
	#[must_use]
	pub fn is_empty(&self) -> bool {
		*self == Self::default()
	}
}
