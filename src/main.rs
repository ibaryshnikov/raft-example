use axum::{routing, Router};

use store::{Request, Response, Store};

mod node;
mod store;

pub type NodeId = u64;

openraft::declare_raft_types!(
    pub TypeConfig: D: Request, R: Response, NodeId = NodeId, Node = BasicNode,
    entry = openraft::Entry<TypeConfig>, SnapshotData = Cursor<Vec<U8>>, AsyncRuntime = TokioRuntime
);

#[tokio::main]
async fn main() {
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
