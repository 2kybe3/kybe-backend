use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::config::types::WolframAlphaConfig;

const URL: &str = "http://api.wolframalpha.com/v2/query";

#[derive(Debug, Clone)]
pub struct WolframAlpha {
    client: Arc<reqwest::Client>,
    token: Option<String>,
}

#[derive(Debug, Deserialize)]
struct WolframAlphaResponse {
    queryresult: WolframAlphaResponseInner,
}

#[derive(Debug, Deserialize)]
struct WolframAlphaResponseInner {
    pods: Vec<Pod>,
}

#[derive(Debug, Deserialize)]
pub struct Pod {
    pub title: String,
    pub subpods: Vec<SubPod>,
}

#[derive(Debug, Deserialize)]
pub struct SubPod {
    pub plaintext: String,
}

#[derive(Debug, Serialize)]
pub struct WolframAlphaRequest {
    input: String,
    appid: String,
    format: String,
    output: String,
}

impl WolframAlphaRequest {
    pub fn new(query: String, token: String) -> Self {
        Self {
            input: query,
            appid: token,
            format: "plaintext".to_string(),
            output: "json".to_string(),
        }
    }
}

impl WolframAlpha {
    pub fn new(client: Arc<reqwest::Client>, config: WolframAlphaConfig) -> Self {
        Self {
            client,
            token: if !config.enabled { None } else { config.token },
        }
    }
}

impl WolframAlpha {
    pub async fn query(&self, query: String) -> anyhow::Result<Vec<Pod>> {
        let Some(token) = self.token.as_deref() else {
            anyhow::bail!("WolframAlpha has no token set!");
        };

        let params = serde_qs::to_string(&WolframAlphaRequest::new(query, token.to_string()))?;
        tracing::info!("{params}");

        let res = self
            .client
            .get(format!("{URL}?{params}"))
            .send()
            .await?
            .text()
            .await?;

        tracing::info!("{res}");

        let res: WolframAlphaResponse = serde_json::from_str(&res)?;
        let pods = res.queryresult.pods;

        tracing::info!("{pods:?}");

        Ok(pods)
    }
}
