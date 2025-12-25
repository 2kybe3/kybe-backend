use crate::config::types::TranslatorConfig;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug)]
pub struct Translator {
    url: String,
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
    confidence: f32,
    language: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct LanguagesResponse {
    code: String,
    name: String,
    targets: Vec<String>,
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
    alternatives: Option<Vec<String>>,
    detected_language: Option<DetectResponse>,
    translated_text: String,
}

#[derive(Debug, Deserialize)]
pub struct ApiError {
    error: String,
}

impl Translator {
    pub fn new<S: Into<String>>(url: S, token: Option<S>) -> Self {
        let token = token.map(Into::into).filter(|t| !t.trim().is_empty());

        Self {
            url: url.into(),
            token,
            client: Client::new(),
        }
    }

    pub async fn languages(&self) -> Result<Vec<LanguagesResponse>, ApiError> {
        let resp = self
            .client
            .get(&(self.url.clone() + "/languages"))
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
        let mut payload = serde_json::json!({ "q": query.into() });

        if let Some(token) = &self.token {
            payload["api_key"] = token.clone().into();
        }

        let resp = self
            .client
            .post(&(self.url.clone() + "/detect"))
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
        let mut source = source.into();
        let mut target = target.into();
        if source.trim().is_empty() {
            source = "auto".into();
        }
        if target.trim().is_empty() {
            target = "de".into();
        }

        let mut payload = serde_json::json!({ "source": source.trim(), "target": target.trim(), "q": query.into() });

        if let Some(token) = &self.token {
            payload["api_key"] = token.clone().into();
        }

        let resp = self
            .client
            .post(&(self.url.clone() + "/translate"))
            .json(&payload)
            .send()
            .await
            .map_err(|e| ApiError {
                error: format!("request failed: {e}"),
            })?;

        let text = resp.text().await.map_err(|e| ApiError {
            error: format!("failed to read response: {e}"),
        })?;

        dbg!(&text);

        let body: TranslateApiResponse = serde_json::from_str(&text).map_err(|e| ApiError {
            error: format!("invalid json: {e} | body: {text}"),
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
}

impl TryFrom<TranslatorConfig> for Translator {
    type Error = TranslatorInitError;

    fn try_from(value: TranslatorConfig) -> Result<Self, Self::Error> {
        if !value.enabled {
            return Err(TranslatorInitError::Disabled);
        }

        let url = value.url.ok_or(TranslatorInitError::MissingUrl)?;
        Ok(Self::new(url, value.token))
    }
}
