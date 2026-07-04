# Contributing to Gate

We welcome contributions from everyone. This document outlines the process for contributing to the Gate project.

## Code of Conduct

This project adheres to a [Code of Conduct](CODE_OF_CONDUCT.md). By participating, you are expected to uphold this code.

## How to Contribute

### Reporting Bugs

1. Check if the bug has already been reported in [Issues](https://github.com/your-org/gate/issues)
2. If not, create a new issue using the **Bug Report** template
3. Include detailed steps to reproduce, expected behavior, and actual behavior

### Suggesting Features

1. Check existing issues and discussions for similar ideas
2. Create a **Feature Request** issue or start a **Discussion**
3. Describe the problem you're solving and the proposed solution

### Pull Requests

1. Fork the repository
2. Create a feature branch from `main`:
   ```bash
   git checkout -b feat/my-feature
   ```
3. Make your changes following our coding conventions
4. Write or update tests as needed
5. Ensure all checks pass:
   ```bash
   # Client checks
   cd client
   pnpm lint
   pnpm typecheck

   # Rust checks
   cargo clippy --all-targets --all-features
   cargo test
   ```
6. Commit using conventional commits format:
   ```
   feat(server): add tunnel heartbeat
   fix(client): resolve connection timeout
   docs: update deployment guide
   ```
7. Push and open a Pull Request

## Development Setup

See the [README.md](README.md#getting-started) for setup instructions.

## Project Conventions

- **Rust**: Follow Rustfmt and Clippy conventions
- **Vue/TypeScript**: Follow Prettier and ESLint config
- **Commits**: Use [Conventional Commits](https://www.conventionalcommits.org/)
- **Documentation**: Keep docs in `docs/` directory, follow existing patterns

## Review Process

- All PRs require at least one approval from a maintainer
- CI checks must pass
- Changes to protocol or core architecture require discussion first

## Questions?

Start a [Discussion](https://github.com/your-org/gate/discussions) or join our community chat.
