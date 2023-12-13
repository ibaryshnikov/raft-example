use std::io::Cursor;
use std::sync::Arc;

use openraft::Config;

use crate::store::Store;

pub async fn start() {
    let config = Config {
        // all values are in ms
        heartbeat_interval: 500,
        election_timeout_min: 1500,
        election_timeout_max: 3000,
        ..Default::default()
    };

    let config = Arc::new(config.validate().expect("Error validating Config"));
    let store = Arc::new(Store::default());
}
