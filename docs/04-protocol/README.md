# Protocol Documentation

## Purpose

Protocol documentation specifies the wire protocol used for communication
between Gate client and server.

## Contents

- **Specification.md** — Protocol specification (message format, encoding)
- **Handshake.md** — Connection handshake sequence
- **TunnelProtocol.md** — Tunnel establishment and data relay
- **ControlMessages.md** — Control message catalog
- **ErrorCodes.md** — Error code definitions

## Protocol Layers

| Layer | Description |
|-------|-------------|
| Transport | TCP or WebSocket |
| Framing | Length-prefixed JSON messages |
| Application | Tunnel control and data relay |

## Why This Design

Using a JSON-based protocol over TCP/WebSocket provides extensibility
and debuggability. Length-prefixed framing ensures message boundaries
are clear and predictable.

## Extension

Version the protocol explicitly. Add new message types without breaking
backward compatibility. Update this documentation whenever the protocol
changes.
