use anyhow::anyhow;
use prometheus_client::{
    encoding::{EncodeLabelSet, text::encode},
    metrics::{counter::Counter, family::Family, gauge::Gauge, histogram::Histogram},
    registry::Registry,
};
use std::sync::{LazyLock, Mutex};
use tokio::time::Instant;

const BASE: &str = "kybe_backend_";

#[derive(Clone, Hash, PartialEq, Eq, EncodeLabelSet, Debug)]
pub struct StatusLabel {
    pub status: u16,
}

pub static REGISTRY: LazyLock<Mutex<Registry>> =
    LazyLock::new(|| Mutex::new(<Registry>::default()));
pub static START_TIME: LazyLock<Instant> = LazyLock::new(Instant::now);
pub static UPTIME_SECONDS: LazyLock<Gauge> = LazyLock::new(Gauge::default);
pub static LASTFM_FETCH_DURATION: LazyLock<Histogram> = LazyLock::new(|| {
    Histogram::new(vec![
        0.05, 0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0, 2.0, 5.0,
    ])
});
pub static LASTFM_FETCH_STATUS: LazyLock<Family<StatusLabel, Counter>> =
    LazyLock::new(Family::default);
pub static LASTFM_LISTENING_STATE: LazyLock<Gauge> = LazyLock::new(Gauge::default);
pub static LASTFM_FETCH_TIMESTAMP: LazyLock<Gauge> = LazyLock::new(Gauge::default);

pub fn register_custom_metrics() {
    let mut reg = REGISTRY.lock().expect("Mutex lock shouldn't fail");
    reg.register(
        format!("{BASE}uptime_seconds"),
        "Application uptime in seconds",
        UPTIME_SECONDS.clone(),
    );
    reg.register(
        format!("{BASE}lastfm_fetch_duration"),
        "Last.fm Fetch Duration",
        LASTFM_FETCH_DURATION.clone(),
    );
    reg.register(
        format!("{BASE}lastfm_fetch_status"),
        "Last.fm Fetch Status Code",
        LASTFM_FETCH_STATUS.clone(),
    );
    reg.register(
        format!("{BASE}lastfm_listening_state"),
        "Last.fm Listening State: 1 = listening, 0 = not",
        LASTFM_LISTENING_STATE.clone(),
    );
    reg.register(
        format!("{BASE}lastfm_fetch_timestamp"),
        "Unix timestamp of the last Last.fm sync event (seconds since epoch)",
        LASTFM_FETCH_TIMESTAMP.clone(),
    );
}

pub fn update_lastfm_fetch_duration(duration_ms: u128) {
    let duration_secs = duration_ms as f64 / 1000.0;
    LASTFM_FETCH_DURATION.observe(duration_secs);
}

pub fn update_lastfm_fetch_status(status: u16) {
    LASTFM_FETCH_STATUS
        .get_or_create(&StatusLabel { status })
        .inc();
}

pub fn update_lastfm_sync_timestamp(unix_timestamp: i64) {
    LASTFM_FETCH_TIMESTAMP.set(unix_timestamp);
}

pub fn set_listening_state(is_listening: bool) {
    LASTFM_LISTENING_STATE.set(if is_listening { 1 } else { 0 });
}

pub fn export_metrics() -> anyhow::Result<String> {
    let uptime = START_TIME.elapsed().as_secs() as i64;
    UPTIME_SECONDS.set(uptime);

    let mut buffer = String::new();
    encode(
        &mut buffer,
        &*REGISTRY.lock().map_err(|_| anyhow!("Matrix lock failed"))?,
    )?;
    Ok(buffer)
}
