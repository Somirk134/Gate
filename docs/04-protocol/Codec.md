# Codec

Codecs transform complete `Message` values into bytes and back. They never
serialize arbitrary business data directly.

## Interfaces

```mermaid
classDiagram
    class Codec {
        +name() str
        +version() ProtocolVersion
        +encode(Message) bytes
        +decode(bytes) Message
    }
    class Serializer {
        +name() str
        +content_type() str
        +serialize(T) bytes
        +deserialize(bytes) T
    }
    Codec --> Serializer
```

## V1

- `JsonCodec`
- `JsonSerializer`
- Serde models
- Tokio-compatible transport boundaries

## Reserved

The following codecs are named and reserved for future implementation:

- `MessagePackCodec`
- `CborCodec`
- `ProtobufCodec`

They currently return unsupported codec errors by design.
