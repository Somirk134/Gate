# Protocol Documentation

Gate Protocol is now defined as an independent protocol layer. It provides the
client-server message contract, codec boundaries, packet/frame boundaries,
version negotiation model, error model, and future extension points.

## Documents

- [Protocol](Protocol.md)
- [Packet](Packet.md)
- [Codec](Codec.md)
- [Version](Version.md)
- [Future](Future.md)

## Current Phase Constraints

- Do not implement Tunnel.
- Do not implement forwarding.
- Do not implement authentication behavior.
- Do not bind TCP, HTTP, WebSocket, or IPC behavior to business flows.
- Keep business payloads inside `Message { Header, Body, Metadata }`.
