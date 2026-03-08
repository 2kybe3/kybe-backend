use std::io::stdout;
use tracing::error;
use tracing::subscriber::DefaultGuard;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::filter::Directive;
use tracing_subscriber::fmt::writer::{BoxMakeWriter, MakeWriterExt};

use crate::config::types::LoggerConfig;

/// Creates a default tracing logger that is used as long as DefaultGuard is not dropped
pub fn init_logger_bootstrap() -> anyhow::Result<DefaultGuard> {
	let filter = get_logger_filter();
	let subscriber = tracing_subscriber::fmt()
		.with_max_level(tracing::Level::TRACE)
		.with_thread_ids(true)
		.with_file(true)
		.with_line_number(true)
		.with_target(true)
		.with_env_filter(filter)
		.finish();

	Ok(tracing::subscriber::set_default(subscriber))
}

/// Takes a old_logger (from init_logger_bootstrap) and creates a new logger which includes file
/// logging which location we should have from the config
///
/// Drops the old_logger before setting the new logger
pub fn init_logger(config: &LoggerConfig, old_logger: DefaultGuard) -> anyhow::Result<()> {
	let filter = get_logger_filter();

	let writer: BoxMakeWriter = if config.file_logger_enabled {
		BoxMakeWriter::new(stdout.and(tracing_appender::rolling::daily(
			"./config/log",
			"kybe_backend.log",
		)))
	} else {
		BoxMakeWriter::new(stdout)
	};

	let subscriber = tracing_subscriber::fmt()
		.with_max_level(tracing::Level::TRACE)
		.with_thread_ids(true)
		.with_file(true)
		.with_line_number(true)
		.with_target(true)
		.with_env_filter(filter)
		.with_writer(writer)
		.finish();

	drop(old_logger);
	tracing::subscriber::set_global_default(subscriber)?;
	Ok(())
}

fn get_logger_filter() -> EnvFilter {
	EnvFilter::try_from_default_env().unwrap_or_else(|_| {
		let directive = "kybe_backend=debug".parse::<Directive>();
		match directive {
			Ok(dir) => EnvFilter::new("info").add_directive(dir),
			Err(e) => {
				error!("failed to parse directive: {e:?}");
				EnvFilter::new("info")
			}
		}
	})
}
