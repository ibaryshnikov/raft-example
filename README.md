# Raft example

Rebuilding an example from
[openraft](https://github.com/datafuselabs/openraft)
for learning purposes.
The original example is [raft-kv-memstore](https://github.com/datafuselabs/openraft/tree/main/examples/raft-kv-memstore).

## Usage

Start a node
```bash
cargo run -- --id 1 --http-addr 127.0.0.1:8081
```

Init leader
```bash
curl http://localhost:8081/init -H "Content-Type: application/json" -d '{}'
```

Add learner
```bash
curl http://localhost:8081/add-learner -H "Content-Type: application/json" -d '[2, "127.0.0.1:8082"]'
curl http://localhost:8081/add-learner -H "Content-Type: application/json" -d '[3, "127.0.0.1:8083"]'
```

Change membership
```bash
curl http://localhost:8081/change-membership -H "Content-Type: application/json" -d '[1, 2, 3]'
```

Metrics
```bash
curl http://localhost:8081/metrics
```

Read
```bash
curl http://localhost:8081/read -H "Content-Type: application/json" -d '"foo"'
```

Write
```bash
curl http://localhost:8081/write -H "Content-Type: application/json" -d '{"Set":{"key":"foo","value":"bar"}}'
```
