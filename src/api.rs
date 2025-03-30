use std::sync::Arc;

use axum::{
    Json,
    extract::State,
    response::sse::{Event, Sse},
};
use futures::TryStreamExt;
use tokio_stream::{
    Stream,
    wrappers::{BroadcastStream, errors::BroadcastStreamRecvError},
};

use crate::{
    Ctx,
    models::{ActiveRunner, Runner, RunnerResponse, SwitchResponse},
};

#[utoipa::path(get, path = "/api/runner", responses((status = OK, body = RunnerResponse)))]
pub async fn get_programs(ctx: State<Arc<Ctx>>) -> Json<RunnerResponse> {
    let runners = ctx
        .config
        .runners
        .iter()
        .map(|(name, runner)| {
            (
                name.clone(),
                Runner {
                    provides: runner.provides.clone(),
                    url: runner.url.clone(),
                },
            )
        })
        .collect();

    let active = ctx.currently_running.read().await.clone();

    Json(RunnerResponse { active, runners })
}

#[utoipa::path(put, path = "/api/runner", request_body(content = ActiveRunner), responses(SwitchResponse))]
pub async fn switch_program(ctx: State<Arc<Ctx>>, new: Json<ActiveRunner>) -> SwitchResponse {
    let Some(new_runner) = ctx.config.runners.get(&new.name) else {
        return SwitchResponse::RunnerNotFound;
    };

    let mut active = ctx.currently_running.write().await;

    let current_runner = ctx
        .config
        .runners
        .get(&active.name)
        .expect("Invalid state, current_runner doesn't exist");

    println!("Current runner: {:?}", current_runner);

    if let Err(e) = current_runner.stop(&active).await {
        return SwitchResponse::SwitchingFailed { msg: e.to_string() };
    }

    if let Err(e) = new_runner.start(&new).await {
        return SwitchResponse::SwitchingFailed { msg: e.to_string() };
    }

    tracing::info!(runner = new.name, "Switched to");

    *active = new.0.clone();
    if ctx.updates.send(new.0.clone()).is_err() {
        tracing::debug!("All receiver handles dropped");
    }

    tracing::info!(runner = new.0.name, model = ?new.0.model, "Changed runner");

    SwitchResponse::Ok(new.0)
}

#[utoipa::path(get, path = "/api/update",
    responses((status = OK, content_type = "text/event-stream", body = ActiveRunner)))]
pub async fn stream_updates(
    ctx: State<Arc<Ctx>>,
) -> Sse<impl Stream<Item = Result<Event, BroadcastStreamRecvError>>> {
    let rx = ctx.updates.subscribe();

    Sse::new(BroadcastStream::new(rx).map_ok(|data| Event::default().json_data(data).unwrap()))
}
