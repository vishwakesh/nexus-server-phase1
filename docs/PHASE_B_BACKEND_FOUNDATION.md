# NEXUS Phase B backend foundation

## Included in this patch

- Authenticated bot metadata APIs implemented in the existing Axum service.
- Per-user ownership filtering on list, detail, update, activity and lifecycle-placeholder routes.
- Creation and update validation for bot names, runtime/version allowlists, relative entrypoint paths and resource limits.
- Additive SQLite schema for bot configurations and bot activity records. Existing deployments auto-create these tables at startup using `CREATE TABLE IF NOT EXISTS`, matching the current Phase 2 schema-initialization pattern; `migrations/0003_bots.sql` documents the equivalent migration.
- Runtime capability endpoint that explicitly reports execution disabled.
- Start, stop, restart, install, logs, files and environment placeholder routes respond with HTTP 503 and make no claims that operations succeeded.
- Bot deletion is blocked until a runtime supervisor can confirm cleanup, preventing orphaned containers/workspaces.

## API

All endpoints below are behind the existing `require_auth` middleware. State-changing routes retain the existing CSRF requirement. They also resolve the authenticated user again and scope queries by that account's database user ID.

- `GET /api/bots/runtime-capabilities`
- `GET /api/bots`
- `POST /api/bots`
- `GET /api/bots/{id}`
- `PATCH /api/bots/{id}`
- `DELETE /api/bots/{id}` (currently returns 503 until runtime-aware cleanup exists)
- `GET /api/bots/{id}/activity`
- `POST /api/bots/{id}/start`
- `POST /api/bots/{id}/stop`
- `POST /api/bots/{id}/restart`
- `POST /api/bots/{id}/dependencies/install`
- `GET /api/bots/{id}/logs`
- `GET /api/bots/{id}/files`
- `GET /api/bots/{id}/environment`

Create request example:

```json
{
  "name": "megabot",
  "description": "Discord utility bot",
  "runtime": "python",
  "runtime_version": "3.12",
  "entrypoint": "main.py",
  "cpu_limit_millicores": 1000,
  "memory_limit_mb": 512
}
```

Allowlisted metadata versions: Python 3.11 / 3.12 and Node.js 20 / 22. These values are not a substitute for immutable image digests; the separate supervisor must map them to reviewed, pinned image digests.

## Critical limitations

This patch is **not the full production bot runtime**. It deliberately does not execute submitted code, access Docker, install dependencies, manage files, persist secrets or return sample runtime metrics. A dedicated, narrow supervisor with immutable image digests, container confinement, resource enforcement, network isolation, filesystem boundaries, secret encryption, and cleanup/reconciliation must be implemented and independently verified before enabling bot execution. The `nexus-web` account must not be granted Docker socket access or membership in the Docker group.

Do not deploy until `cargo fmt --check`, `cargo test`, `cargo build --release`, endpoint tests and a review against the current production `src/main.rs` have passed on a compatible Rust toolchain. This authoring environment did not have Rust/Cargo installed, so compilation has not been verified here.
