# Protocol Internals

The protocol crate defines package boundaries for Gate command, packet, and codec types.

## Release boundary

The v0.9 release cleanup does not modify protocol behavior or wire compatibility.

## Maintainer guidance

- Treat protocol changes as compatibility-sensitive.
- Keep codec tests close to protocol code.
- Document any future wire-format change in release notes before publishing a tag.
- Prefer additive changes when compatibility matters.
