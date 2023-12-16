use std::collections::BTreeSet;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use openraft::error::{ForwardToLeader, NetworkError, RemoteError};
use openraft::{BasicNode, RaftMetrics, TryAsRef};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use tokio::time::timeout;

use crate::types::{self, ClientWriteError, ClientWriteResponse, RPCError, RaftError};
use crate::{NodeId, Request};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Empty {}

struct Leader {
    node_id: NodeId,
    addr: String,
}

struct ExampleClient {
    pub leader: Arc<Mutex<Leader>>,
    pub inner: reqwest::Client,
}

impl ExampleClient {
    pub fn new(leader_id: NodeId, addr: String) -> Self {
        let leader = Leader {
            node_id: leader_id,
            addr,
        };
        Self {
            leader: Arc::new(Mutex::new(leader)),
            inner: reqwest::Client::new(),
        }
    }

    pub async fn write(
        &self,
        request: &Request,
    ) -> Result<ClientWriteResponse, RPCError<ClientWriteError>> {
        self.send_rpc_to_leader_with_retry("write", Some(request))
            .await
    }

    pub async fn read(&self, request: &String) -> Result<String, RPCError> {
        self.do_send_rpc_to_leader("read", Some(request)).await
    }

    pub async fn consistent_read(
        &self,
        request: &String,
    ) -> Result<String, RPCError<types::CheckIsLeaderError>> {
        self.do_send_rpc_to_leader("consistent-read", Some(request))
            .await
    }

    pub async fn init(&self) -> Result<(), RPCError<types::InitializeError>> {
        self.do_send_rpc_to_leader("init", Some(&Empty {})).await
    }

    pub async fn add_learner(
        &self,
        request: (NodeId, String),
    ) -> Result<ClientWriteResponse, RPCError<ClientWriteError>> {
        self.send_rpc_to_leader_with_retry("add-learner", Some(&request))
            .await
    }

    pub async fn change_membership(
        &self,
        request: &BTreeSet<NodeId>,
    ) -> Result<ClientWriteResponse, RPCError<ClientWriteError>> {
        self.send_rpc_to_leader_with_retry("change_membership", Some(request))
            .await
    }

    pub async fn metrics(&self) -> Result<RaftMetrics<NodeId, BasicNode>, RPCError> {
        self.do_send_rpc_to_leader("metrics", None::<&()>).await
    }

    async fn do_send_rpc_to_leader<Req, Resp, Err>(
        &self,
        uri: &str,
        request: Option<&Req>,
    ) -> Result<Resp, RPCError<Err>>
    where
        Req: Serialize + 'static,
        Resp: Serialize + DeserializeOwned,
        Err: std::error::Error + Serialize + DeserializeOwned,
    {
        let (leader_id, url) = {
            let leader_lock = self.leader.lock().expect("Should lock the leader");
            let target_addr = format!("http://{}/{}", leader_lock.addr, uri);
            (leader_lock.node_id, target_addr)
        };

        let pending_future = if let Some(r) = request {
            tracing::debug!(
                ">>> client send POST request to {}: {}",
                url,
                serde_json::to_string_pretty(&r).expect("Should serialize request to string"),
            );
            self.inner.post(url.clone()).json(r)
        } else {
            tracing::debug!(">>> client send GET request to {}", url);
            self.inner.get(url.clone())
        }
        .send();

        let resolved = timeout(Duration::from_millis(3_000), pending_future).await;
        let response = match resolved {
            Ok(response) => response.map_err(|e| RPCError::Network(NetworkError::new(&e)))?,
            Err(timeout_error) => {
                tracing::error!("timeout {} to url: {}", timeout_error, url);
                return Err(RPCError::Network(NetworkError::new(&timeout_error)));
            }
        };

        let result: Result<Resp, RaftError<Err>> = response
            .json()
            .await
            .map_err(|e| RPCError::Network(NetworkError::new(&e)))?;
        tracing::debug!(
            "<<< client received reply from {}: {}",
            url,
            serde_json::to_string_pretty(&result).expect("Should serialize response to string"),
        );

        result.map_err(|e| RPCError::RemoteError(RemoteError::new(leader_id, e)))
    }

    async fn send_rpc_to_leader_with_retry<Req, Resp, Err>(
        &self,
        uri: &str,
        request: Option<&Req>,
    ) -> Result<Resp, RPCError<Err>>
    where
        Req: Serialize + 'static,
        Resp: Serialize + DeserializeOwned,
        Err: std::error::Error
            + Serialize
            + DeserializeOwned
            + TryAsRef<types::ForwardToLeader>
            + Clone,
    {
        let mut n_retry = 3;

        loop {
            let result: Result<Resp, RPCError<Err>> =
                self.do_send_rpc_to_leader(uri, request).await;

            let rpc_error = match result {
                Ok(r) => return Ok(r),
                Err(e) => e,
            };

            if let Some(ForwardToLeader {
                leader_id: Some(leader_id),
                leader_node: Some(leader_node),
            }) = rpc_error.forward_to_leader()
            {
                {
                    let mut leader_lock = self.leader.lock().expect("Should lock the leader");
                    *leader_lock = Leader {
                        node_id: *leader_id,
                        addr: leader_node.addr.clone(),
                    };
                }

                n_retry -= 1;
                if n_retry > 0 {
                    continue;
                }
            }

            return Err(rpc_error);
        }
    }
}
