# Release Note

Every release should use the same structure so users can decide whether to upgrade quickly.

## Template

```markdown
# Gate vX.Y.Z

Release date: YYYY-MM-DD

## Highlights

- Short summary of why this release matters.

## Added

- New behavior, docs, examples, packages, or workflows.

## Improved

- Existing behavior that became clearer, faster, safer, or easier to use.

## Fixed

- Bugs fixed in this release.

## Breaking Changes

- Required user action, config changes, removed APIs, or incompatible data changes.

## Known Issues

- Important limitations users should know before upgrading.

## Upgrade Guide

1. Back up configuration.
2. Read breaking changes.
3. Install the new binary or image.
4. Restart the server.
5. Verify authentication, heartbeat, tunnels, dashboard, and logs.

## Checksums

| Artifact | SHA256 |
| --- | --- |
| gate-server-linux-amd64 | TBD |
```

## Rules

- Never hide breaking changes in a general summary.
- Include rollback notes when state or configuration changes.
- Link to issues and pull requests where helpful.
- Keep marketing language out of operational sections.
