use std::time::{Duration, SystemTime, UNIX_EPOCH};

use reqwest::multipart::{Form, Part};

const URL_0X0: &str = "https://0x0.st";

pub async fn upload_to_0x0(text: &str, duration: Option<Duration>) -> Option<String> {
	let part = Part::bytes(text.as_bytes().to_vec())
		.file_name("log.log")
		.mime_str("text/plain")
		.ok()?;

	let mut form = Form::new().part("file", part);

	if let Some(dur) = duration {
		form = form.text(
			"expires",
			SystemTime::now()
				.checked_add(dur)?
				.duration_since(UNIX_EPOCH)
				.ok()?
				.as_millis()
				.to_string(),
		);
	}

	let client = reqwest::Client::builder()
		.timeout(Duration::from_secs(10))
		.build()
		.ok()?;

	let res = match client
		.post(URL_0X0)
		.header("User-Agent", "kyeb-backend/v0.0.0")
		.multipart(form)
		.send()
		.await
	{
		Ok(res) => res,
		Err(e) => {
			eprintln!("{e}");
			return None;
		}
	};

	res.text().await.ok()
}
