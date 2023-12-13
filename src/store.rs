use std::collections::BTreeMap;
use std::fmt::Debug;
use std::io::Cursor;
use std::ops::RangeBounds;
use std::sync::{Arc, Mutex};

use openraft::async_trait::async_trait;
use openraft::storage::{LogState, Snapshot};
use openraft::{
    BasicNode, Entry, EntryPayload, LogId, RaftLogReader, RaftSnapshotBuilder, RaftStorage,
    RaftTypeConfig, SnapshotMeta, StorageError, StorageIOError, StoredMembership, Vote,
};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use crate::{NodeId, TypeConfig};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum Request {
    Set { key: String, value: String },
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Response {
    pub value: Option<String>,
}

#[derive(Debug)]
pub struct StoredSnapshot {
    pub meta: SnapshotMeta<NodeId, BasicNode>,
    pub data: Vec<u8>,
}

#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub struct StateMachine {
    pub last_applied_log: Option<LogId<NodeId>>,
    pub last_membership: StoredMembership<NodeId, BasicNode>,
    pub data: BTreeMap<String, String>,
}

#[derive(Debug, Default)]
pub struct Store {
    last_purged_log_id: RwLock<Option<LogId<NodeId>>>,
    log: RwLock<BTreeMap<u64, Entry<TypeConfig>>>,
    pub state_machine: RwLock<StateMachine>,
    vote: RwLock<Option<Vote<NodeId>>>,
    snapshot_idx: Arc<Mutex<u64>>,
    current_snapshot: RwLock<Option<StoredSnapshot>>,
}

#[async_trait]
impl RaftLogReader<TypeConfig> for Arc<Store> {
    async fn try_get_log_entries<RB: RangeBounds<u64> + Clone + Debug + Send + Sync>(
        &mut self,
        range: RB,
    ) -> Result<Vec<Entry<TypeConfig>>, StorageError<NodeId>> {
        let log = self.log.read().await;
        let response = log
            .range(range.clone())
            .map(|_, value| value.clone())
            .collect::<Vec<_>>();
        Ok(response)
    }
}

#[async_trait]
impl RaftSnapshotBuilder<TypeConfig> for Arc<Store> {
    async fn build_snapshot(&mut self) -> Result<Snapshot<TypeConfig>, StorageError<NodeId>> {
        let data;
        let last_applied_log;
        let last_membership;
        {
            let state_machine = self.state_machine.read().await;
            data = serde_json::to_vec(&*state_machine)
                .map_err(|e| StorageIOError::read_state_machine(&e))?;

            last_applied_log = state_machine.last_applied_log;
            last_membership = state_machine.last_membership;
        }

        let snapshot_index = {
            let mut locked_index = self
                .snapshot_idx
                .lock()
                .expect("Should lock snapshot_idx in State");
            *locked_index += 1;
            *locked_index
        };

        let snapshot_id = if let Some(last) = last_applied_log {
            format!("{}-{}-{}", last.leader_id, last.index, snapshot_index)
        } else {
            format!("--{}", snapshot_index)
        };

        let meta = SnapshotMeta {
            last_log_id: last_applied_log,
            last_membership,
            snapshot_id,
        };

        let snapshot = StoredSnapshot {
            meta: meta.clone(),
            data: data.clone(),
        };

        {
            let mut current_snapshot = self.current_snapshot.write().await;
            *current_snapshot = Some(snapshot);
        }

        Ok(Snapshot {
            meta,
            snapshot: Box::new(Cursor::new(data)),
        })
    }
}

#[async_trait]
impl RaftStorage<TypeConfig> for Arc<Store> {
    type LogReader = Self;
    type SnapshotBuilder = Self;

    async fn get_log_state(&mut self) -> Result<LogState<TypeConfig>, StorageError<NodeId>> {
        let log = self.log.read().await;
        let last = log.iter().next_back().map(|(_, entry)| entry.log_id);

        let last_purged = self.last_purged_log_id.read().await;

        let last = match last {
            None => last_purged,
            Some(value) => Some(value),
        };

        Ok(LogState {
            last_purged_log_id: last_purged,
            last_log_id: last,
        })
    }

    async fn save_vote(&mut self, vote: &Vote<NodeId>) -> Result<(), StorageError<NodeId>> {
        let locked = self.vote.lock().await;
        *locked = Some(*vote);
        Ok(())
    }

    async fn read_vote(&mut self) -> Result<Vote<NodeId>, StorageError<NodeId>> {
        Ok(self.vote.read().await)
    }

    async fn append_to_log<I>(&mut self, entries: I) -> Result<(), StorageError<NodeId>>
    where
        I: IntoIterator<Item = Entry<TypeConfig>> + Send,
    {
        let mut log = self.log.write().await;
        for entry in entries {
            log.insert(entry.log_id.index, entry);
        }
        Ok(())
    }
}
