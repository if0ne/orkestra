use tracing_appender::{non_blocking::WorkerGuard, rolling};
use tracing_subscriber::{
    fmt::{self, writer::MakeWriterExt},
    layer::SubscriberExt,
};

pub struct Logger {
    _guard: WorkerGuard,
}

impl Logger {
    pub fn new() -> Self {
        let (file_log, guard) = {
            let (file_log, guard) =
                tracing_appender::non_blocking(rolling::daily("./logs-sm", "info"));

            let file_log = fmt::Layer::new()
                .with_ansi(false)
                .with_writer(file_log.with_max_level(tracing::Level::TRACE))
                .pretty();

            (file_log, guard)
        };

        let console_log = fmt::Layer::new()
            .with_ansi(true)
            .with_writer(
                std::io::stdout
                    .with_min_level(tracing::Level::ERROR)
                    .with_max_level(tracing::Level::INFO),
            );

        let subscriber = tracing_subscriber::registry()
            .with(file_log)
            .with(console_log);

        let _ = tracing::subscriber::set_global_default(subscriber);

        Self { _guard: guard }
    }
}
