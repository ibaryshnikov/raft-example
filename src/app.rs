use std::sync::Arc;

use openraft::Config;

use crate::{NodeId, Raft, Store};

pub struct App {
    pub id: NodeId,
    pub addr: String,
    pub raft: Raft,
    pub store: Arc<Store>,
    pub config: Arc<Config>,
}
