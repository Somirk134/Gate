# Nginx HTTP Tunnel Example

Use Nginx as a local HTTP service behind Gate. This example does not configure HTTPS, TLS, or certificates.

## Run

```bash
nginx -c /absolute/path/to/examples/nginx/nginx.conf
```

The service listens on `127.0.0.1:8080`.

## Gate tunnel

```toml
[tunnel]
name = "nginx-static"
protocol = "http"
local_host = "127.0.0.1"
local_port = 8080
remote_port = 18080
host = "static.example.com"
path = "/"
```

## Test

```bash
curl -H "Host: static.example.com" http://127.0.0.1:18080/
```
