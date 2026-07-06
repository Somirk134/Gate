# Authentication

Gate validates sessions before accepting tunnel registration or forwarding traffic.

## Production checklist

- Store tokens outside the repository.
- Do not bake secrets into Docker images.
- Use TLS at the edge or runtime layer.
- Rotate credentials after incidents or operator changes.
