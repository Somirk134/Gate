# API Documentation

## Purpose

API documentation provides the reference for all public REST API endpoints
and WebSocket protocols.

## Contents

- **REST.md** — REST API reference
- **WebSocket.md** — WebSocket message reference
- **Authentication.md** — Auth flow and token management
- **RateLimiting.md** — Rate limit policies
- **Errors.md** — API error response format
- **Changelog.md** — API version history

## Base URL

Production: `https://api.gate-project.dev/v1`
Development: `http://localhost:5800/api/v1`

## Authentication

All API requests (except `/health` and `/auth/login`) require a Bearer token
in the `Authorization` header.

## Why This Design

Versioned API (`/api/v1/`) allows breaking changes without disrupting
existing clients. Standardized error responses make client-side error
handling predictable.

## Extension

Document all new endpoints with request/response schemas. Use OpenAPI/Swagger
for machine-readable API documentation. Version the API when making
breaking changes.
