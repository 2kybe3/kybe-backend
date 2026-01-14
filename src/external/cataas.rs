use axum::body::Bytes;
use axum::http::StatusCode;
use chrono::{DateTime, Utc};
use reqwest::Url;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::OnceCell;
use tracing::error;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct CATAASCatResponse {
	#[allow(unused)]
	pub id: String,
	#[allow(unused)]
	pub tags: Vec<String>,
	#[allow(unused)]
	pub created_at: DateTime<Utc>,
	pub url: String,
	pub mimetype: String,
}

#[derive(Debug, Serialize, poise::ChoiceParameter)]
#[serde(rename_all = "lowercase")]
pub enum Type {
	Square,
	Medium,
	Small,
	XSmall,
}

#[derive(Debug, Serialize, poise::ChoiceParameter)]
#[serde(rename_all = "lowercase")]
pub enum Filter {
	Mono,
	Negate,
	Custom,
}

#[derive(Debug, Serialize, poise::ChoiceParameter)]
#[serde(rename_all = "lowercase")]
pub enum Fit {
	Cover,
	Contain,
	Fill,
	Inside,
	Outside,
}

#[derive(Debug, Serialize, Default, poise::ChoiceParameter)]
#[serde(rename_all = "lowercase")]
pub enum Position {
	Top,
	#[serde(rename = "right top")]
	RightTop,
	Right,
	#[serde(rename = "right bottom")]
	RightBottom,
	Bottom,
	#[serde(rename = "left bottom")]
	LeftBottom,
	Left,
	#[serde(rename = "left top")]
	LeftTop,
	#[default]
	Center,
}

#[derive(Debug, Serialize)]
pub struct CATAASCatRequest {
	#[serde(rename = "type")]
	pub cat_type: Option<Type>,
	pub filter: Option<Filter>,
	pub fit: Option<Fit>,
	pub position: Option<Position>,
	pub width: Option<i32>,
	pub height: Option<i32>,
	pub blur: Option<i32>,
	pub r: Option<i32>,
	pub g: Option<i32>,
	pub b: Option<i32>,
	pub brightness: Option<i32>,
	pub saturation: Option<i32>,
	pub hue: Option<i32>,
	pub lightness: Option<i32>,
	pub json: Option<bool>,
}

#[derive(Debug, Clone)]
pub struct CATAAS {
	client: Arc<reqwest::Client>,
	tags: OnceCell<Vec<String>>,
}

impl CATAAS {
	pub fn new(client: Arc<reqwest::Client>) -> Self {
		Self {
			client,
			tags: OnceCell::new(),
		}
	}

	pub async fn tags(&self) -> &Vec<String> {
		self.tags
			.get_or_init(async || {
				return self.fetch_tags().await.unwrap_or_else(|e| {
					error!("Error fething cat tags: {:?}", e);
					vec![]
				});
			})
			.await
	}

	async fn fetch_tags(&self) -> anyhow::Result<Vec<String>> {
		let tags: Vec<String> = self
			.client
			.get("https://cataas.com/api/tags")
			.send()
			.await?
			.json::<Vec<String>>()
			.await?
			.into_iter()
			.filter(|tag| !tag.trim().is_empty())
			.collect();
		Ok(tags)
	}

	pub async fn get_image(&self, url: &str) -> anyhow::Result<Bytes> {
		let res = self.client.get(url).send().await?.bytes().await?;
		Ok(res)
	}

	pub async fn get_cat_url(
		&self,
		req: &CATAASCatRequest,
		tag: Option<&str>,
		says: Option<&str>,
	) -> anyhow::Result<Option<CATAASCatResponse>> {
		let mut url = "https://cataas.com/cat".to_string();

		if let Some(tag) = tag {
			url.push('/');
			url.push_str(tag);
		}
		if let Some(says) = says {
			url.push_str("/says/");
			url.push_str(&urlencoding::encode(says));
		}

		let mut url = Url::parse(&url)?;
		let query = serde_qs::to_string(req)?;
		url.set_query(Some(&query));

		let resp = self.client.get(url).send().await?;

		if resp.status() == StatusCode::NOT_FOUND {
			Ok(None)
		} else {
			Ok(Some(resp.json::<CATAASCatResponse>().await?))
		}
	}
}
