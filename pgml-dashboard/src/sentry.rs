use crate::utils::{config, datadog};

pub async fn configure_reporting() -> Option<sentry::ClientInitGuard> {
    let mut log_builder = env_logger::Builder::from_default_env();
    log_builder.format_timestamp_micros();

    // TODO move sentry into a once_cell
    let sentry = match config::sentry_dsn() {
        Some(dsn) => {
            // Don't log debug or trace to sentry, regardless of environment
            let logger = log_builder.build();
            let level = logger.filter();
            let logger = sentry_log::SentryLogger::with_dest(logger);
            log::set_boxed_logger(Box::new(logger)).unwrap();
            log::set_max_level(level);

            let name = sentry::release_name!().unwrap_or_else(|| std::borrow::Cow::Borrowed("cloud2"));
            let sha = env!("GIT_SHA");
            let release = format!("{name}+{sha}");
            let result = sentry::init((
                dsn.as_str(),
                sentry::ClientOptions {
                    release: Some(std::borrow::Cow::Owned(release)),
                    debug: true,
                    ..Default::default()
                },
            ));
            info!("Configured reporting w/ Sentry");
            Some(result)
        }
        _ => {
            log_builder.try_init().unwrap();
            info!("Configured reporting w/o Sentry");
            None
        }
    };

    match datadog::client().await {
        Ok(_) => info!("Configured reporting w/ Datadog"),
        Err(err) => warn!("Configured reporting w/o Datadog: {err}"),
    };

    sentry
}
