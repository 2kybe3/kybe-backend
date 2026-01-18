use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct AsnMin {
	#[serde(skip_serializing_if = "Option::is_none")]
	pub autonomous_system_number: Option<u32>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub autonomous_system_organization: Option<String>,
}
