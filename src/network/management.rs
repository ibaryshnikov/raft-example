use axum::extract::State;
use axum::response::IntoResponse;
use axum::Json;
use std::collections::{BTreeMap, BTreeSet};

use http::StatusCode;
use openraft::error::Infallible;
use openraft::{BasicNode, RaftMetrics};

use crate::{NodeId, SharedState};

pub async fn add_learner(
    State(state): State<SharedState>,
    Json((node_id, addr)): Json<(NodeId, String)>,
) -> Result<impl IntoResponse, StatusCode> {
    println!("add_learner called");
    let app = state.lock().await;
    let node = BasicNode { addr };
    let response = app.raft.add_learner(node_id, node, true).await;
    Ok(Json(response))
}

pub async fn change_membership(
    State(state): State<SharedState>,
    Json(request): Json<BTreeSet<NodeId>>,
) -> Result<impl IntoResponse, StatusCode> {
    println!("change_membership called");
    let app = state.lock().await;
    let response = app.raft.change_membership(request, false).await;
    Ok(Json(response))
}

pub async fn init(State(state): State<SharedState>) -> Result<impl IntoResponse, StatusCode> {
    println!("init called");
    let app = state.lock().await;
    let mut nodes = BTreeMap::new();
    nodes.insert(
        app.id,
        BasicNode {
            addr: app.addr.clone(),
        },
    );
    let response = app.raft.initialize(nodes).await;
    Ok(Json(response))
}

pub async fn metrics(State(state): State<SharedState>) -> Result<impl IntoResponse, StatusCode> {
    println!("metrics called");
    let app = state.lock().await;
    let metrics = app.raft.metrics().borrow().clone();
    let response: Result<RaftMetrics<NodeId, BasicNode>, Infallible> = Ok(metrics);
    Ok(Json(response))
}
