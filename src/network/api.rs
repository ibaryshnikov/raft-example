use axum::extract::State;
use axum::response::IntoResponse;
use axum::Json;
use http::StatusCode;
use openraft::error::{CheckIsLeaderError, Infallible, RaftError};
use openraft::BasicNode;

use crate::store::Request;
use crate::{NodeId, SharedState};

pub async fn write(
    State(state): State<SharedState>,
    Json(request): Json<Request>,
) -> Result<impl IntoResponse, StatusCode> {
    println!("write called");
    let app = state.lock().await;
    let response = app.raft.client_write(request).await;
    Ok(Json(response))
}

pub async fn read(
    State(state): State<SharedState>,
    Json(key): Json<String>,
) -> Result<impl IntoResponse, StatusCode> {
    println!("read called");
    let app = state.lock().await;
    let state_machine = app.store.state_machine.read().await;
    let value = state_machine.data.get(&key).cloned();

    let response: Result<String, Infallible> = Ok(value.unwrap_or_default());
    Ok(Json(response))
}

pub async fn consistent_read(
    State(state): State<SharedState>,
    Json(key): Json<String>,
) -> impl IntoResponse {
    println!("consistent_read called");
    let app = state.lock().await;
    let is_leader = app.raft.is_leader().await;

    match is_leader {
        Ok(_) => {
            let state_machine = app.store.state_machine.read().await;
            let value = state_machine.data.get(&key).cloned();

            let response: Result<String, RaftError<NodeId, CheckIsLeaderError<NodeId, BasicNode>>> =
                Ok(value.unwrap_or_default());
            Json(response)
        }
        Err(e) => Json(Err(e)),
    }
}
