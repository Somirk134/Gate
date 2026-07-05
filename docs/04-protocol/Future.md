# Future

The protocol layer is intentionally built for replacement and extension.

## Planned Formats

- MessagePack
- CBOR
- Protobuf

## Planned Wire Capabilities

- Compression: Gzip, Zstd
- Encryption markers: TLS, AES
- Fragmentation
- Sticky packet buffering
- Split packet reassembly
- Plugin payload routing
- Extended error codes

## Compatibility Rules

- Business code should depend on `ProtocolManager`, `Message`, `Command`, and
  shared protocol types, not a concrete codec.
- New codecs must implement `Codec` and may reuse or replace serializers.
- New protocol versions must register through `ProtocolRegistry`.
- Business commands should remain stable dotted strings.
- Reserved enum variants and `Custom` values are extension points, not behavior.
