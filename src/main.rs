mod api;
mod models;
mod runner;
mod serve_config;
mod spa;

use std::{
    net::SocketAddr,
    path::{Path, PathBuf},
    sync::Arc,
};

use axum::routing::get;
use clap::Parser;
use eyre::Context as _;
use serve_config::Config;
use tokio::{signal, sync::broadcast};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use utoipa::OpenApi;
use utoipa_axum::router::OpenApiRouter;
use utoipa_redoc::Servable;

use crate::models::ActiveRunner;

#[derive(clap::Parser)]
enum Opt {
    Serve {
        /// Path to the configuration file
        config_path: PathBuf,
    },
}

#[derive(utoipa::OpenApi)]
#[openapi(tags())]
struct ApiDoc;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), eyre::Error> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| format!("{}=info", env!("CARGO_CRATE_NAME")).into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let opt = Opt::parse();

    match opt {
        Opt::Serve { config_path } => {
            serve(&config_path).await?;
        }
    }
    Ok(())
}

async fn serve(config_path: &Path) -> Result<(), eyre::Error> {
    let config = serve_config::load(&config_path).await?;

    let mut running = None;
    for (name, runner) in config.runners.iter() {
        // FIXME: this sux
        let def = ActiveRunner {
            name: name.to_string(),
            model: None,
        };
        if runner.check_active(&def).await {
            match running {
                Some(_) => {
                    eyre::bail!("Multiple runners active, please fix it");
                }
                None => {
                    running = Some(def);
                }
            }
        }
    }

    let active = match running {
        Some(active) => active,
        None => {
            let active = ActiveRunner {
                name: config.default.clone(),
                model: config.default_model.clone(),
            };

            let Some(runner) = config.runners.get(&active.name) else {
                eyre::bail!(
                    "Default runner {} specified in config not defined",
                    active.name
                );
            };

            runner
                .start(&active)
                .await
                .context("Failed starting default runner")?;

            active
        }
    };

    let (tx, _rx) = broadcast::channel(1);
    let ctx = Ctx {
        currently_running: tokio::sync::RwLock::new(active),
        updates: tx,
        config,
    };

    let addr = ctx.config.addr;

    let (app, api) = OpenApiRouter::with_openapi(ApiDoc::openapi())
        .routes(utoipa_axum::routes!(api::get_programs, api::switch_program))
        .routes(utoipa_axum::routes!(api::stream_updates))
        .fallback(spa::static_handler)
        .with_state(Arc::new(ctx))
        .split_for_parts();

    let api_json = api
        .to_json()
        .context("Failed converting api definition to json")?;

    let doc = utoipa_redoc::Redoc::with_url("/doc", api);
    let app = app
        .route(
            "/spec",
            get(move || async move {
                (
                    [(axum::http::header::CONTENT_TYPE, "application/json")],
                    api_json,
                )
            }),
        )
        .merge(doc);

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .with_context(|| format!("Failed binding to address {addr}"))?;
    tracing::info!("Listening on {}", listener.local_addr().unwrap());
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .with_graceful_shutdown(shutdown_signal())
    .await?;

    Ok(())
}

pub struct Ctx {
    pub currently_running: tokio::sync::RwLock<ActiveRunner>,
    pub updates: broadcast::Sender<models::ActiveRunner>,
    pub config: Config,
}

#[derive(rust_embed::Embed)]
#[folder = "frontend/dist/"]
struct Assets;

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
