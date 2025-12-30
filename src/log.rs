use std::{fs, path::PathBuf};

pub(crate) fn init(log_file: Option<PathBuf>) {
    if !try_init(log_file) {
        tracing_subscriber::fmt()
            .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
            .pretty()
            .init()
    }
}

pub(crate) fn try_init(log_file: Option<PathBuf>) -> bool {
    use tracing_subscriber::{Layer, layer::SubscriberExt, util::SubscriberInitExt};

    let Some(log_file) = log_file.or_else(get_log_file) else {
        return false;
    };

    if log_file.exists() && !log_file.is_file() {
        return false;
    }

    let (Some(directory), Some(file_name)) = (
        log_file.parent().and_then(|p| p.canonicalize().ok()),
        log_file.file_name(),
    ) else {
        return false;
    };

    let appender = tracing_appender::rolling::never(directory, file_name);
    let (non_blocking, _guard) = tracing_appender::non_blocking(appender);
    let file_layer = tracing_subscriber::fmt::layer()
        .with_writer(non_blocking)
        .with_ansi(false)
        .with_filter(tracing_subscriber::EnvFilter::from_default_env());

    let stdout_layer = tracing_subscriber::fmt::layer()
        .with_writer(std::io::stdout)
        .pretty()
        .with_filter(tracing_subscriber::EnvFilter::from_default_env());

    tracing_subscriber::registry()
        .with(stdout_layer)
        .with(file_layer)
        .init();

    true
}

fn get_log_file() -> Option<PathBuf> {
    let mut path = match dirs::data_dir() {
        Some(dir) if dir.exists() => dir,
        _ => dirs::home_dir()?.join(".local").join("share"),
    };
    path.push("ample");

    if !path.exists() {
        fs::create_dir_all(&path).inspect_err(print_err).ok()?;
    } else if !path.is_dir() {
        return None;
    }

    path.push("log");
    Some(path)
}

fn print_err(s: &impl std::fmt::Display) {
    eprintln!("cannot initialise file logs: {}", s);
}
