# HTTPS Tunnel Test Matrix

Gate terminates HTTPS in `HttpsTunnelRuntime`; SpringBoot, Flask, Express, Gin,
and Nginx are target HTTP services behind the tunnel. Runtime-level automation
lives in `crates/engine/tests/network/https_runtime.rs`.

## Automated Coverage

- self-signed certificate
- SNI certificate selection
- multi-domain routing
- Domain Management HostResolver adapter
- certificate hot reload without restart
- HTTP keep-alive/chunked regression through the shared HTTP runtime

## Target Service Matrix

| Target | Example path | HTTPS checks |
| --- | --- | --- |
| SpringBoot | `examples/spring-boot` | self-signed, SNI, reload |
| Flask | `examples/python-flask` | self-signed, SNI, reload |
| Express | `examples/express` | self-signed, SNI, reload |
| Gin | `examples/go-gin` | self-signed, SNI, reload |
| Nginx | `examples/nginx` | self-signed, SNI, multi-domain |

## Let's Encrypt

Let's Encrypt depends on the ACME provider network implementation. The current
runtime calls `HttpsCertificateProvider::request_missing(domain)` on SNI miss,
so a real ACME provider can request and then hot-reload the certificate. The
existing reserved provider returns `NetworkDisabled`, which keeps the interface
compatible without pretending issuance succeeded.
