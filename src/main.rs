use std::io::Cursor;
use std::sync::Arc;

use clap::Parser;
use openraft::storage::Adaptor;
use openraft::BasicNode;
use tokio::sync::Mutex;
use tracing_subscriber::EnvFilter;

mod app;
mod client;
mod network;
mod node;
mod store;
mod types;

use app::App;
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
pub type SharedState = Arc<Mutex<App>>;

#[derive(Parser, Clone, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Options {
    #[clap(long)]
    pub id: u64,

    #[clap(long)]
    pub http_addr: String,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_target(true)
        .with_thread_ids(true)
        .with_level(true)
        .with_ansi(false)
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let options = Options::parse();

    node::start(options.id, options.http_addr).await;
}
