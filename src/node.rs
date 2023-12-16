use std::sync::Arc;

use axum::{routing, Router};
use openraft::storage::Adaptor;
use openraft::Config;
use tokio::sync::Mutex;

use crate::app::App;
use crate::network::{api, management, raft, Network};
use crate::store::Store;
use crate::NodeId;

pub async fn start(node_id: NodeId, http_addr: String) {
    let config = Config {
        // all values are in ms
        heartbeat_interval: 500,
        election_timeout_min: 1500,
        election_timeout_max: 3000,
        ..Default::default()
    };

    let config = Arc::new(config.validate().expect("Error validating Config"));
    let store = Arc::new(Store::default());

    let (log_store, state_machine) = Adaptor::new(store.clone());

    let network = Network {};

    let raft = openraft::Raft::new(node_id, config.clone(), network, log_store, state_machine)
        .await
        .expect("Should create a new Raft");

    let app_data = Arc::new(Mutex::new(App {
        id: node_id,
        addr: http_addr.clone(),
        raft,
        store,
        config,
    }));

    let app = Router::new()
        // raft internal RPC
        .route("/raft-append", routing::post(raft::append))
        .route("/raft-snapshot", routing::post(raft::snapshot))
        .route("/raft-vote", routing::post(raft::vote))
        // admin API
        .route("/init", routing::post(management::init))
        .route("/add-learner", routing::post(management::add_learner))
        .route(
            "/change-membership",
            routing::post(management::change_membership),
        )
        .route("/metrics", routing::get(management::metrics))
        // application API
        .route("/write", routing::post(api::write))
        .route("/read", routing::post(api::read))
        .route("/consistent-read", routing::post(api::consistent_read))
        .with_state(app_data);

    let listener = tokio::net::TcpListener::bind(http_addr)
        .await
        .expect("Should create a listener");

    axum::serve(listener, app)
        .await
        .expect("Error running a server");
}
