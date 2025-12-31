use crate::config::types::TranslatorConfig;
use reqwest::{Client, Url};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug)]
pub struct Translator {
	url: Url,
	token: Option<String>,
	client: Client,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum DetectApiResponse {
	Ok(Vec<DetectResponse>),
	Error(ApiError),
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DetectResponse {
	pub confidence: f32,
	pub language: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct LanguagesResponse {
	pub code: String,
	pub name: String,
	pub targets: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum TranslateApiResponse {
	Ok(TranslateResponse),
	Error(ApiError),
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TranslateResponse {
	pub alternatives: Option<Vec<String>>,
	pub detected_language: Option<DetectResponse>,
	pub translated_text: String,
}

#[derive(Debug, Deserialize)]
pub struct ApiError {
	#[allow(dead_code)]
	error: String,
}

impl Translator {
	pub fn new<S: Into<String>>(url: Url, token: Option<S>) -> Self {
		Self {
			url,
			token: token.map(Into::into).filter(|t| !t.trim().is_empty()),
			client: Client::new(),
		}
	}

	pub async fn languages(&self) -> Result<Vec<LanguagesResponse>, ApiError> {
		let resp = self
			.client
			.get(self.url.join("/languages").unwrap())
			.send()
			.await
			.map_err(|e| ApiError {
				error: format!("request failed: {e}"),
			})?;

		let text = resp.text().await.map_err(|e| ApiError {
			error: format!("failed to read response: {e}"),
		})?;

		let body: Vec<LanguagesResponse> = serde_json::from_str(&text).map_err(|e| ApiError {
			error: format!("invalid json: {e} | body: {text}"),
		})?;

		Ok(body)
	}

	pub async fn detect<S: Into<String>>(&self, query: S) -> Result<Vec<DetectResponse>, ApiError> {
		let query = query.into();
		let query = query.trim();

		let mut payload = serde_json::json!({ "q": query });

		if let Some(token) = &self.token {
			payload["api_key"] = token.clone().into();
		}

		let resp = self
			.client
			.post(self.url.join("/detect").unwrap())
			.json(&payload)
			.send()
			.await
			.map_err(|e| ApiError {
				error: format!("request failed: {e}"),
			})?;

		let text = resp.text().await.map_err(|e| ApiError {
			error: format!("failed to read response: {e}"),
		})?;

		let body: DetectApiResponse = serde_json::from_str(&text).map_err(|e| ApiError {
			error: format!("invalid json: {e} | body: {text}"),
		})?;

		match body {
			DetectApiResponse::Ok(data) => Ok(data),
			DetectApiResponse::Error(err) => Err(err),
		}
	}

	pub async fn translate<S: Into<String>>(
		&self,
		source: S,
		target: S,
		query: S,
	) -> Result<TranslateResponse, ApiError> {
		let query = query.into();

		let source = source.into();
		let target = target.into();

		let source = source.trim();
		let target = target.trim();

		let source = if source.is_empty() { "auto" } else { source };
		let target = if target.is_empty() { "de" } else { target };

		let mut payload = serde_json::json!({
			"source": source,
			"target": target,
			"q": query,
		});

		if let Some(token) = &self.token {
			payload["api_key"] = token.clone().into();
		}

		let resp = self
			.client
			.post(self.url.join("/translate").unwrap())
			.json(&payload)
			.send()
			.await
			.map_err(|e| ApiError {
				error: e.to_string(),
			})?;

		if !resp.status().is_success() {
			let status = resp.status();
			let err_text = resp
				.text()
				.await
				.unwrap_or_else(|_| "<failed to read body>".into());
			return Err(ApiError {
				error: format!("HTTP {}: {}", status, err_text.trim()),
			});
		}

		let body: TranslateApiResponse = resp.json().await.map_err(|e| ApiError {
			error: format!("invalid json: {}", e),
		})?;

		match body {
			TranslateApiResponse::Ok(data) => Ok(data),
			TranslateApiResponse::Error(err) => Err(err),
		}
	}
}

#[derive(Debug, Error)]
pub enum TranslatorInitError {
	#[error("translator is disabled")]
	Disabled,

	#[error("translator url is missing")]
	MissingUrl,

	#[error("translator url is malformed")]
	InvalidUrl,
}

impl TryFrom<TranslatorConfig> for Translator {
	type Error = TranslatorInitError;

	fn try_from(value: TranslatorConfig) -> Result<Self, Self::Error> {
		if !value.enabled {
			return Err(TranslatorInitError::Disabled);
		}

		let url = value.url.ok_or(TranslatorInitError::MissingUrl)?;
		let url = Url::parse(&url).map_err(|_| TranslatorInitError::InvalidUrl)?;
		if url.scheme() != "http" && url.scheme() != "https" {
			return Err(TranslatorInitError::InvalidUrl);
		}
		Ok(Self::new(url, value.token))
	}
}
