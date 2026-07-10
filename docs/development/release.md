# Release

This page describes the Gate v0.9 release process.

## Version policy

Gate v0.9.1 uses `0.9.1` across:

- Rust workspace package version.
- Root npm workspace package version.
- Client npm package version.
- Tauri desktop version.

## Release checklist

1. Ensure the working tree contains no local caches, logs, or secrets.
2. Run the release gate locally:

   ```bash
   cargo check --workspace
   cargo test --workspace
   npm run typecheck
   npm run build
   ```

3. Create and push a tag:

   ```bash
   git tag v0.9.1
   git push origin v0.9.1
   ```

4. Let GitHub Actions build server binaries, desktop packages, and Docker image.
5. Review generated artifacts and release notes.
6. Publish the GitHub Release when artifacts are verified.

## Artifact targets

| Target | Artifact |
| --- | --- |
| Windows | Server archive and desktop installer `.exe`/`.msi`. |
| macOS Intel | Server archive and desktop `.dmg`. |
| macOS Apple Silicon | Server archive and desktop `.dmg`. |
| Linux | Server archive, `.AppImage`, and `.deb`. |
| Docker | `ghcr.io/<owner>/gate-server:v0.9.1`. |

## Updater policy

Gate v0.9 does not configure a dedicated Tauri updater endpoint. Users install updates from GitHub Releases until a signed update server is available.
