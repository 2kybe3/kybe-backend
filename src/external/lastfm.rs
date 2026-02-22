use std::{
	sync::Arc,
	time::{Duration, Instant},
};

use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use tracing::{error, info, warn};

use crate::config::types::LastFMConfig;

const BASE_URL: &str = "http://ws.audioscrobbler.com/2.0/";

#[derive(Debug, Clone)]
pub struct LastFM {
	client: reqwest::Client,
	interval_secs: u8,
	username: String,
	token: String,

	cache: Arc<Mutex<Cache>>,
}

#[derive(Debug)]
struct Cache {
	last_time: Instant,
	last_result: Option<Response>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Response {
	pub artist: String,
	pub name: String,
	pub url: String,
}

const INTERVAL_DEFAULT: u8 = 10;

impl LastFM {
	pub fn new(lastfm: &LastFMConfig) -> Option<Self> {
		if lastfm.username.as_deref().unwrap_or("").is_empty() {
			warn!("lastfm enabled but username empty disabling");
			return None;
		}

		if lastfm.token.as_deref().unwrap_or("").is_empty() {
			warn!("lastfm enabled but token empty disabling");
			return None;
		}

		if lastfm.interval_secs.is_none() {
			info!(
				"{}",
				format!(
					"lastfm interval_secs not set using default ({} seconds)",
					INTERVAL_DEFAULT
				)
			)
		}

		if let (Some(username), Some(token)) = (lastfm.username.to_owned(), lastfm.token.to_owned())
		{
			let client = reqwest::Client::builder()
				.timeout(Duration::from_secs(2))
				.build();
			if let Err(ref e) = client {
				error!("Error building client {:?} using default client", e);
			}

			return Some(Self {
				client: client.unwrap_or_default(),
				interval_secs: lastfm.interval_secs.unwrap_or(INTERVAL_DEFAULT),
				username,
				token,
				cache: Arc::new(Mutex::new(Cache {
					last_time: Instant::now() - Duration::from_secs(INTERVAL_DEFAULT.into()),
					last_result: None,
				})),
			});
		}

		unreachable!()
	}

	pub async fn run_cacher(self: Arc<Self>) {
		tokio::spawn(async move {
			loop {
				if let Err(e) = self.refresh_cache().await {
					error!("Failed to refresh Last.fm cache: {:?}", e);
				}
				tokio::time::sleep(Duration::from_secs(self.interval_secs.into())).await;
			}
		});
	}

	async fn refresh_cache(&self) -> anyhow::Result<()> {
		let url = format!(
			"{}?method=user.getrecenttracks&user={}&api_key={}&format=json",
			BASE_URL, self.username, self.token
		);

		let resp = self.client.get(url).send().await?;
		let raw_text = resp.text().await?;
		let parsed: UserGetRecentTracksResponse = serde_json::from_str(&raw_text)?;

		let track = parsed
			.recenttracks
			.track
			.into_iter()
			.find(|t| t.attr.as_ref().and_then(|a| a.nowplaying).unwrap_or(false));

		let result = track.map(|t| Response {
			artist: t.artist.text,
			name: t.name,
			url: t.url,
		});

		let mut cache = self.cache.lock().await;
		cache.last_result = result;
		cache.last_time = Instant::now();

		Ok(())
	}

	pub async fn get_playing(&self) -> Option<Response> {
		let cache = self.cache.lock().await;
		cache.last_result.clone()
	}
}

fn string_to_bool<'de, D>(deserializer: D) -> Result<Option<bool>, D::Error>
where
	D: serde::Deserializer<'de>,
{
	let s: Option<String> = Deserialize::deserialize(deserializer)?;
	Ok(s.map(|s| s == "true"))
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct UserGetRecentTracksResponse {
	recenttracks: RecentTracks,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct RecentTracks {
	track: Vec<Track>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct Track {
	artist: Artist,
	name: String,
	url: String,
	#[serde(rename = "@attr")]
	attr: Option<Attr>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct Artist {
	#[serde(rename = "#text")]
	text: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct Attr {
	#[serde(deserialize_with = "string_to_bool")]
	nowplaying: Option<bool>,
}
