# Express HTTP Tunnel Example

Expose a local Express service through an HTTP tunnel.

## Run

```bash
npm install
npm start
```

The service listens on `127.0.0.1:3000`.

## Gate tunnel

```toml
[tunnel]
name = "express-api"
protocol = "http"
local_host = "127.0.0.1"
local_port = 3000
remote_port = 18080
host = "example.com"
path = "/api"
```

## Test

```bash
curl -H "Host: example.com" http://127.0.0.1:18080/api/health
curl -H "Host: example.com" http://127.0.0.1:18080/api/stream
```
