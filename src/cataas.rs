use axum::body::Bytes;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
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

#[derive(Debug, Clone)]
pub struct CatAss {
	client: Arc<reqwest::Client>,
	tags: Option<Vec<String>>,
}

impl CatAss {
	pub fn new(client: Arc<reqwest::Client>) -> Self {
		Self { client, tags: None }
	}

	pub async fn tags(&mut self) -> Vec<String> {
		match self.tags.clone() {
			Some(tags) => tags,
			None => match self.fetch_tags().await {
				Ok(tags) => tags,
				Err(e) => {
					error!("Error fetching tags: {:?}", e);
					vec![]
				}
			},
		}
	}

	async fn fetch_tags(&mut self) -> anyhow::Result<Vec<String>> {
		let tags: Vec<String> = self
			.client
			.get("https://cataas.com/api/tags")
			.send()
			.await?
			.json::<Vec<String>>()
			.await?
			.into_iter()
			.filter(|tag| !tag.is_empty())
			.collect();
		self.tags = Some(tags.clone());
		Ok(tags)
	}

	pub async fn get_image(&self, url: &str) -> anyhow::Result<Bytes> {
		let res = self.client.get(url).send().await?.bytes().await?;
		Ok(res)
	}

	pub async fn get_cat_url(
		&self,
		tag: Option<&str>,
		says: Option<&str>,
	) -> anyhow::Result<CATAASCatResponse> {
		let mut url = "https://cataas.com/cat".to_string();

		if let Some(tag) = tag {
			url.push('/');
			url.push_str(tag);
		}
		if let Some(says) = says {
			url.push_str("/says/");
			url.push_str(&urlencoding::encode(says));
		}
		url.push_str("?json=true");

		let resp = self.client.get(url).send().await?;

		Ok(resp.json::<CATAASCatResponse>().await?)
	}
}
