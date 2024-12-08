use anyhow::{Context, Result};
use axum::{extract::State, routing::get, Json, Router};
use clap::{command, Parser};
use clap_verbosity_flag::{InfoLevel, Verbosity};
use error::AppError;
use serde::{Deserialize, Serialize};
use shadow_rs::shadow;
use tokio::{net::TcpListener, signal};
use tower_http::trace::TraceLayer;
use tracing::{info, trace};

mod error;

shadow!(build);

#[derive(Debug, Parser)]
#[command(version, about, long_about = build::CLAP_LONG_VERSION)]
struct Cli {
    #[command(flatten)]
    verbosity: Verbosity<InfoLevel>,

    /// Address to bind the server to
    #[arg(long, env, default_value = "127.0.0.1:3010")]
    bind_address: String,

    /// Address of the nouns microservice
    #[arg(long, env)]
    nouns_address: String,

    /// Address of the verbs microservice
    #[arg(long, env)]
    verbs_address: String,
}

#[derive(Clone)]
struct AppState {
    nouns_address: String,
    verbs_address: String,
}

#[derive(Deserialize)]
struct WordResponse {
    word: String,
}

#[derive(Serialize)]
struct SentenceResponse {
    sentence: String,
}

async fn root(State(state): State<AppState>) -> Result<Json<SentenceResponse>, AppError> {
    let noun = reqwest::get(state.nouns_address)
        .await
        .with_context(|| "While fetching a noun")?
        .json::<WordResponse>()
        .await
        .with_context(|| "While deserializing the nouns response")?;

    let verb = reqwest::get(state.verbs_address)
        .await
        .with_context(|| "While fetching a verb")?
        .json::<WordResponse>()
        .await
        .with_context(|| "While deserializing the verb response")?;

    let response = SentenceResponse {
        sentence: format!("{} {}", verb.word, noun.word),
    };

    trace!("Responded with: {}", response.sentence);
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
        nouns_address: cli.nouns_address.clone(),
        verbs_address: cli.verbs_address.clone(),
    };

    let app = Router::new()
        .route("/", get(root))
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let listener = TcpListener::bind(cli.bind_address.clone())
        .await
        .with_context(|| format!("Unable to bind to \"{}\"", cli.bind_address))?;

    info!("Server started on {}", cli.bind_address);
    info!("Nouns will be fetched from: {}", cli.nouns_address);
    info!("Verbs will be fetched from: {}", cli.verbs_address);

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .with_context(|| "An error has occurred while serving with Axum")?;

    Ok(())
}
