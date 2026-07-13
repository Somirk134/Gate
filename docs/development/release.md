# Release

This page describes the Gate v0.9 release process.

## Version policy

Gate v0.9.2 uses `0.9.2` across:

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
   git tag v0.9.2
   git push origin v0.9.2
   ```

4. Let GitHub Actions build server binaries and desktop packages, then publish the pinned Docker Hub image.
5. Review generated artifacts and release notes.
6. Publish the GitHub Release when artifacts are verified.

## Artifact targets

| Target | Artifact |
| --- | --- |
| Windows | Server archive and desktop installer `.exe`/`.msi`. |
| macOS Intel | Server archive and desktop `.dmg`. |
| macOS Apple Silicon | Server archive and desktop `.dmg`. |
| Linux | Server archive, `.AppImage`, and `.deb`. |
| Docker | `qwe1235/gate-server:0.9.2`. |

## Updater policy

Gate uses the signed `latest.json` and updater assets from the latest GitHub Release. Every platform entry must reference a distinct uploaded asset with a non-empty signature.
