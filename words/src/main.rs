use anyhow::{Context, Result};
use axum::{extract::State, routing::get, Json, Router};
use clap::{command, Parser, ValueEnum};
use clap_verbosity_flag::{InfoLevel, Verbosity};
use error::AppError;
use serde::Serialize;
use shadow_rs::shadow;
use strum::Display;
use tokio::{net::TcpListener, signal};
use tower_http::trace::TraceLayer;
use tracing::{info, trace};
use words::{random_noun, random_verb};

mod error;
mod words;

shadow!(build);

#[derive(ValueEnum, Debug, Display, Clone, Serialize)]
#[serde(rename_all = "kebab-case")]
enum WordsKind {
    Nouns,
    Verbs,
}

#[derive(Debug, Parser)]
#[command(version, about, long_about = build::CLAP_LONG_VERSION)]
struct Cli {
    #[command(flatten)]
    verbosity: Verbosity<InfoLevel>,

    /// Address to bind the server to
    #[arg(long, env, default_value = "127.0.0.1:3000")]
    bind_address: String,

    /// Kind of words to serve
    #[arg(value_enum, long, env)]
    kind: WordsKind,
}

#[derive(Clone)]
struct AppState {
    kind: WordsKind,
}

#[derive(Serialize)]
struct WordResponse {
    word: String,
    kind: WordsKind,
}

async fn root(State(state): State<AppState>) -> Result<Json<WordResponse>, AppError> {
    let word = match state.kind {
        WordsKind::Nouns => random_noun(),
        WordsKind::Verbs => random_verb(),
    }?;

    let response = WordResponse {
        word,
        kind: state.kind,
    };

    trace!("Responded with: {}", response.word);
    Ok(Json(response))
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    tracing_subscriber::fmt()
        .with_max_level(cli.verbosity)
        .init();

    info!(
        "Starting {} {}",
        build::PROJECT_NAME,
        build::CLAP_LONG_VERSION
    );

    let state = AppState {
        kind: cli.kind.clone(),
    };

    let app = Router::new()
        .route("/", get(root))
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let listener = TcpListener::bind(cli.bind_address.clone())
        .await
        .with_context(|| format!("Unable to bind to \"{}\"", cli.bind_address))?;

    info!("Server started on {}", cli.bind_address);
    info!("This words server will serve: {}", cli.kind);

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .with_context(|| "An error has occurred while serving with Axum")?;

    Ok(())
}
