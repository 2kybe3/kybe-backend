use lazy_static::lazy_static;
use prometheus::{
	Encoder, HistogramOpts, HistogramVec, IntCounterVec, IntGauge, Opts, Registry, TextEncoder,
};
use tokio::time::Instant;

const BASE: &str = "kybe_backend_";

lazy_static! {
	pub static ref REGISTRY: Registry = Registry::new();
	pub static ref START_TIME: Instant = Instant::now();
	pub static ref UPTIME_SECONDS: IntGauge = IntGauge::new(
		format!("{}uptime_seconds", BASE),
		"Application uptime in seconds"
	)
	.expect("Error creating Prometheus gague");
	pub static ref LASTFM_FETCH_DURATION: HistogramVec = HistogramVec::new(
		HistogramOpts::new(
			format!("{}lastfm_fetch_duration", BASE),
			"Last.fm Fetch Duration"
		)
		.buckets(vec![
			0.05, 0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0, 2.0, 5.0,
		]),
		&["duration_range"],
	)
	.expect("Error creating Prometheus histogram");
	pub static ref LASTFM_FETCH_STATUS: IntCounterVec = IntCounterVec::new(
		Opts::new(
			format!("{}lastfm_fetch_status", BASE),
			"Last.fm Fetch Status Code"
		),
		&["status"],
	)
	.expect("Error creating Prometheus counter");
}

pub fn register_custom_metrics() {
	REGISTRY
		.register(Box::new(UPTIME_SECONDS.clone()))
		.expect("collector can be registered");
	REGISTRY
		.register(Box::new(LASTFM_FETCH_DURATION.clone()))
		.expect("collector can be registered");
	REGISTRY
		.register(Box::new(LASTFM_FETCH_STATUS.clone()))
		.expect("collector can be registered");
}

pub fn update_lastfm_fetch_duration(duration_ms: u128) {
	let duration_secs = duration_ms as f64 / 1000.0;
	LASTFM_FETCH_DURATION
		.with_label_values(&["fetch_duration"])
		.observe(duration_secs);
}
pub fn update_lastfm_fetch_status(status_code: u16) {
	LASTFM_FETCH_STATUS
		.with_label_values(&[&status_code.to_string()])
		.inc();
}

pub fn export_metrics() -> anyhow::Result<String> {
	let uptime = START_TIME.elapsed().as_secs() as i64;
	UPTIME_SECONDS.set(uptime);

	let mut buffer = Vec::new();
	let encoder = TextEncoder::new();
	encoder.encode(&REGISTRY.gather(), &mut buffer)?;
	Ok(String::from_utf8(buffer)?)
}
