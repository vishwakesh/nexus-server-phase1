# NEXUS Server — Phase 1

A local-first Rust/Axum server dashboard foundation for Ubuntu. This milestone provides a responsive dashboard, SQLite event storage, health/info/metrics/events APIs, structured logs, and graceful shutdown.

## Important security limitation

**Phase 1 does not implement authentication. Do not expose it through ngrok, a public IP, or a public reverse proxy.** The application rejects non-loopback `NEXUS_BIND` values and defaults to `127.0.0.1:3000`. Finish and test authentication/authorization before enabling any public access.

## Requirements

- Rust/Cargo toolchain
- Ubuntu/Linux
- Build dependencies required by SQLx SQLite (usually available in standard Rust build environment)

Docker and Node.js are not needed for Phase 1.

## Install without overwriting an existing project

On the VPS, first inspect the current directory:

```bash
cd ~/nexus-server
pwd
find . -maxdepth 3 -type f -print | sort | head -200
```

Upload/extract this archive to a separate directory such as `~/nexus-server-phase1` or another unused path. Do not overwrite existing files until you've inspected and backed them up. Then enter the extracted folder.

```bash
cargo fmt
cargo fmt --check
cargo check
cargo test
cargo run --release
```

The app logs its bound address and data directory. Open the dashboard from the VPS itself or use an SSH port forward from your phone/computer. It listens on `127.0.0.1:3000` by default.

## API endpoints

- `GET /` — dashboard
- `GET /assets/app.css` — local CSS
- `GET /assets/app.js` — dashboard JavaScript
- `GET /api/health` — health and SQLite connectivity
- `GET /api/info` — app version, bind address, process uptime
- `GET /api/metrics` — host CPU, memory, disk, system uptime
- `GET /api/events` — most recent 12 NEXUS events

Example local checks in a second terminal:

```bash
curl -fsS http://127.0.0.1:3000/api/health
curl -fsS http://127.0.0.1:3000/api/metrics
curl -fsS http://127.0.0.1:3000/api/events
```

## Data and configuration

- Default data directory: `$HOME/.local/share/nexus`
- SQLite database: `$HOME/.local/share/nexus/nexus.sqlite3`
- Override data path with `NEXUS_DATA_DIR`
- Override port with `NEXUS_PORT`
- `NEXUS_BIND` may be set only to a loopback address in this release; non-loopback values are rejected intentionally.

The server creates its SQLite events table at startup. `migrations/0001_initial.sql` documents the same schema.

## Testing and limitations

The dashboard reads real values via `sysinfo`, and loads events from SQLite. A two-test unit suite covers simple API contract/percentage logic. Integration tests against a running VPS should still be performed after transfer. Rust compilation has not been verified in the authoring environment because this workspace does not include `rustc`/`cargo` and has no network access to install them. Run `cargo fmt`, `cargo check`, and `cargo test` on the VPS before treating it as runnable; share any errors so they can be corrected.

Not implemented in Phase 1: authentication, service management, Hermes controls, Docker controls, file manager, backups, ngrok deployment, and authorization/audit policy for privileged operations. UI labels for future features explicitly show these are not integrated.

## Roadmap

1. Phase 1: dashboard foundation, actual metrics, SQLite, health API.
2. Phase 2: first-run administrator setup, secure sessions, CSRF, rate limiting, audit events.
3. Phase 3: allowlisted app/systemd service adapters and bot hosting.
4. Phase 4: read-only/controlled Hermes integration.
5. Phase 5: private file manager with path confinement.
6. Phase 6: allowlisted Docker integration.
7. Phase 7: ngrok only after auth/authorization and security review.
8. Phase 8–9: backup/recovery, security audit, operations documentation.
