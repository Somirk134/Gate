# Node.js

## Description

Expose a local Node.js service for previews, webhook callbacks, or mobile device testing.

## Configuration

```toml
[server]
address = "gate.example.com:7000"
auth_token = "replace-me"

[tunnel]
name = "node-api"
protocol = "http"
local_host = "127.0.0.1"
local_port = 3000
remote_port = 18080
```

Local app:

```bash
npm install
npm run dev
```

## Screenshot

![Tunnel screenshot](../../assets/screenshots/tunnel.svg)

## Run Steps

1. Start the Node.js service on `127.0.0.1:3000`.
2. Start Gate server.
3. Create the `node-api` tunnel.
4. Share the public URL with your tester.
5. Stop the tunnel after the test.
