# Oxidauth

Self-hosted authentication service — a project wrapping the oxidauth stack.

## Architecture

Each stack follows an 8-layer architecture:
1. **kernel** - Pure domain logic
2. **repository** - Query traits
3. **postgres** - SQL implementation
4. **services** - Use cases
5. **http** - HTTP DTOs
6. **api** - HTTP handlers
7. **rs** - Frontend API client
8. **web** - Leptos components

## Getting Started

```bash
# Configure environment
cp example.env .env

# Start backing services
docker compose up -d

# Build the workspace
cargo build

# Run a helper script
bin/build-server.sh
```

> The per-stack compose layout arrives with migration plan 10.

## Available Stacks

- **oxidauth** — authentication and identity service (kernel, repository,
  postgres, http, cli, import-export, permission crates).

## Development

```bash
# Format
cargo fmt

# Lint
cargo clippy
```
