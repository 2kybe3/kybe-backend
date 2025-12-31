use std::process::Command;

fn main() {
	println!("cargo:rerun-if-changed=build.rs");
	println!("cargo:rerun-if-env-changed=GIT_SHA");

	let git_sha = std::env::var("GIT_SHA").unwrap_or_else(|_| {
		if std::env::var_os("CI").is_none() {
			Command::new("git")
				.args(["rev-parse", "HEAD"])
				.output()
				.ok()
				.filter(|o| o.status.success())
				.and_then(|o| String::from_utf8(o.stdout).ok())
				.map(|s| s.trim().to_string())
				.unwrap_or_else(|| "unknown".into())
		} else {
			"unknown".into()
		}
	});

	println!("cargo:rustc-env=KYBE_GIT_SHA={git_sha}");
}
