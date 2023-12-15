use std::io::Cursor;
use std::sync::Arc;

use axum::{routing, Router};
use openraft::storage::Adaptor;
use openraft::{BasicNode, Config};
use tracing_subscriber::EnvFilter;

mod network;
mod node;
mod store;
mod types;

use network::Network;
use store::{Request, Response, Store};

pub type NodeId = u64;

openraft::declare_raft_types!(
    pub TypeConfig: D = Request, R = Response, NodeId = NodeId, Node = BasicNode,
    Entry = openraft::Entry<TypeConfig>, SnapshotData = Cursor<Vec<u8>>
);

pub type LogStore = Adaptor<TypeConfig, Arc<Store>>;
pub type StateMachineStore = Adaptor<TypeConfig, Arc<Store>>;
pub type Raft = openraft::Raft<TypeConfig, Network, LogStore, StateMachineStore>;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_target(true)
        .with_thread_ids(true)
        .with_level(true)
        .with_ansi(false)
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let app = Router::new().route("/", routing::get(hello));
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080")
        .await
        .expect("Should create a listener");
    axum::serve(listener, app)
        .await
        .expect("Error running a server");
    // node::start().await;
}

async fn hello() -> &'static str {
    "Hello from axum!"
}
