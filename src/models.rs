use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Runner {
    pub provides: Option<Vec<String>>,
    pub url: url::Url,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RunnerResponse {
    pub runners: HashMap<String, Runner>,
    pub active: ActiveRunner,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase", tag = "status")]
pub enum ActionResponse<T> {
    Ok(T),
    Err { message: String },
}

impl<T> ActionResponse<T> {
    pub fn err(message: String) -> ActionResponse<T> {
        Self::Err { message }
    }

    pub fn ok(data: T) -> ActionResponse<T> {
        ActionResponse::Ok(data)
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ActiveRunner {
    pub name: String,
    pub model: Option<String>,
}
