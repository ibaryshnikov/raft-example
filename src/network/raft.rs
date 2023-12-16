use axum::extract::State;
use axum::response::IntoResponse;
use axum::Json;
use http::StatusCode;
use openraft::raft::{AppendEntriesRequest, InstallSnapshotRequest, VoteRequest};

use crate::{NodeId, SharedState, TypeConfig};

pub async fn vote(
    State(state): State<SharedState>,
    Json(request): Json<VoteRequest<NodeId>>,
) -> impl IntoResponse {
    println!("vote called");
    let app = state.lock().await;
    let response = app.raft.vote(request).await;
    Json(response)
}

pub async fn append(
    State(state): State<SharedState>,
    Json(request): Json<AppendEntriesRequest<TypeConfig>>,
) -> Result<impl IntoResponse, StatusCode> {
    println!("append called");
    let app = state.lock().await;
    let response = app.raft.append_entries(request).await;
    Ok(Json(response))
}

pub async fn snapshot(
    State(state): State<SharedState>,
    Json(request): Json<InstallSnapshotRequest<TypeConfig>>,
) -> Result<impl IntoResponse, StatusCode> {
    println!("snapshot called");
    let app = state.lock().await;
    let response = app.raft.install_snapshot(request).await;
    Ok(Json(response))
}
