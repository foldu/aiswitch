use std::collections::HashMap;

use axum::{Json, response::IntoResponse};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use utoipa::{IntoResponses, ToSchema};

#[derive(Deserialize, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Runner {
    pub provides: Option<Vec<String>>,
    pub url: url::Url,
}

#[derive(Deserialize, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct RunnerResponse {
    pub runners: HashMap<String, Runner>,
    pub active: ActiveRunner,
}

#[derive(Deserialize, Serialize, ToSchema, IntoResponses)]
#[serde(rename_all = "camelCase")]
pub enum SwitchResponse {
    #[response(status = OK)]
    Ok(ActiveRunner),
    #[response(status = NOT_FOUND)]
    RunnerNotFound,
    #[response(status = INTERNAL_SERVER_ERROR)]
    SwitchingFailed { msg: String },
    #[response(status = BAD_REQUEST)]
    InvalidModel,
}

impl IntoResponse for SwitchResponse {
    fn into_response(self) -> axum::response::Response {
        // would be nice to get this from #[response]
        let status = match self {
            SwitchResponse::Ok(_) => StatusCode::OK,
            SwitchResponse::RunnerNotFound => StatusCode::NOT_FOUND,
            SwitchResponse::SwitchingFailed { .. } => StatusCode::INTERNAL_SERVER_ERROR,
            SwitchResponse::InvalidModel => StatusCode::BAD_REQUEST,
        };

        (status, Json(self)).into_response()
    }
}

#[derive(Deserialize, Serialize, ToSchema, Clone)]
pub struct ActiveRunner {
    pub name: String,
    pub model: Option<String>,
}
