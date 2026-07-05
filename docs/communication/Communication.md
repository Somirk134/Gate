# Communication Layer

The Communication Layer provides a unified, mockable client/server messaging framework for all future Gate business features.

## Scope

Included:

- Client and server communication modules.
- Transport trait for TCP now and WebSocket/QUIC later.
- Connection state machine and metadata.
- Request manager with request id to response promise mapping.
- Response dispatcher.
- Event dispatcher with subscribe, publish, broadcast, remove, and priority ordering.
- Incoming, outgoing, and reserved priority queues.
- Command dispatcher and message router.
- Empty Project, Tunnel, Server, System, and Log handlers.
- Session manager and session context.
- Retry policy and timeout configuration.
- Logging hooks through `tracing`.
- Mock client, server, transport, dispatcher, and response helpers.

Excluded:

- Real TCP socket implementation.
- Tunnel behavior.
- Authentication workflow.
- Heartbeat loop.
- Business statistics collection.

## Rust Entry Points

- `crates/communication/src/lib.rs`
- `Transport`
- `Dispatcher`
- `Connection`
- `Session`
- `RequestHandler`
- `ResponseHandler`
- `EventHandler`

## TypeScript Entry Points

- `client/src/communication/index.ts`
- `CommunicationService`
- `ClientRequestManager`
- `ClientEventManager`
- `ClientDispatcher`
- `MockTransport`

## Client API

```ts
await service.connect(endpoint)
await service.disconnect()
await service.request(command, body)
await service.notify(command, body)
await service.subscribe(name, handler)
await service.unsubscribe(name, subscriptionId)
```

All public client calls are promise-based where asynchronous work is involved.
