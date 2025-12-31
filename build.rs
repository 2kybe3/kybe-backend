use std::process::Command;

fn main() {
	if let Ok(sha) = std::env::var("GIT_SHA") {
		println!("cargo:rustc-env=KYBE_GIT_SHA={}", sha);
		println!("cargo:rerun-if-env-changed=GIT_SHA");
		return;
	}

	let output = Command::new("git").args(["rev-parse", "HEAD"]).output();

	let git_sha = match output {
		Ok(out) if out.status.success() => {
			let sha = String::from_utf8_lossy(&out.stdout).trim().to_string();
			if sha.is_empty() {
				"unknown".to_string()
			} else {
				sha
			}
		}
		_ => "unknown".to_string(),
	};
	println!("cargo:rustc-env=KYBE_GIT_SHA={git_sha}");
	println!("cargo:rerun-if-changed=.git/HEAD");
	println!("cargo:rerun-if-changed=.git/refs/heads/main");
}
