# Nginx

## Description

Use Nginx as a public reverse proxy for TLS termination, HTTP routing, access logs, and rate limiting.

## Configuration

```nginx
server {
    listen 443 ssl http2;
    server_name gate.example.com;

    ssl_certificate /etc/letsencrypt/live/gate.example.com/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/gate.example.com/privkey.pem;

    location / {
        proxy_pass http://127.0.0.1:18080;
        proxy_set_header Host $host;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto https;
    }
}
```

## Screenshot

![Dashboard screenshot](../../assets/screenshots/dashboard.svg)

## Run Steps

1. Start Gate server.
2. Start a tunnel with remote port `18080`.
3. Configure Nginx to proxy to `127.0.0.1:18080`.
4. Add TLS certificates.
5. Reload Nginx.
6. Test the public HTTPS URL.
