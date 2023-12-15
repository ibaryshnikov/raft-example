use openraft::error::{self, Infallible};
use openraft::{raft, BasicNode};

use crate::{NodeId, TypeConfig};

pub type RaftError<E = Infallible> = error::RaftError<NodeId, E>;
pub type RPCError<E = Infallible> = error::RPCError<NodeId, BasicNode, RaftError<E>>;

pub type ClientWriteError = error::ClientWriteError<NodeId, BasicNode>;
pub type CheckIsLeaderError = error::CheckIsLeaderError<NodeId, BasicNode>;
pub type ForwardToLeader = error::ForwardToLeader<NodeId, BasicNode>;
pub type InitializeError = error::InitializeError<NodeId, BasicNode>;

pub type ClientWriteResponse = raft::ClientWriteResponse<TypeConfig>;
