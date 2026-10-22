# NEXUS Product Feature Tracker

**Project goal:** a functional self-hosted server control and bot hosting platform with a controlled Hermes Agent integration.

**Current deliverable:** Phase A frontend patch. This bundle has not been installed on the VPS. Feature states below describe this patch's intended behavior, not deployment status.

State meanings:
- **Planned** — not implemented.
- **In progress** — implementation has started but acceptance checks are incomplete.
- **Implemented (not deployed)** — code exists in a local deliverable but has not been deployed to the target environment.
- **Tested** — acceptance checks passed in the target environment.
- **Blocked** — a dependency, missing API, or unresolved security constraint prevents implementation.

| Phase | Feature | Status | Acceptance evidence required |
|---|---|---|---|
| A | Responsive overview layout and sidebar | Implemented (not deployed) | Check 320–1920px layouts and keyboard focus on target browser |
| A | Sign-in, registration, session restoration and sign-out UI | Implemented (not deployed) | Verify real cookie/session/CSRF flow against Phase 2 service |
| A | API health and database indicator | Implemented (not deployed) | `/api/health` response from running service |
| A | CPU, memory, disk, uptime cards | In progress | Only display fields actually returned by `/api/metrics` and `/api/info` |
| A | Historical resource chart | In progress | Requires a recognized real time-series shape; otherwise show empty state |
| A | Recent activity | In progress | Requires `/api/events` returning event records |
| A | Verified service health | In progress | Phase 2 API from health endpoint; other services only from real server-reported data |
| A | Product roadmap in UI | Implemented (not deployed) | Check roadmap screen and planned-status language |
| B | Python/Node bot inventory and creation | Planned | Persistent bot records, validated input, create/read tests |
| B | Isolated workloads and resource limits | Planned | Threat-model review and container isolation tests |
| B | Start/stop/restart/delete | Planned | Server-side authorization and lifecycle integration tests |
| B | Logs, dependencies, environment variables | Planned | Secret-redaction and per-workload isolation tests |
| C | Hermes inspection and guided installation | Planned | Documented supported install path and safe integration permissions |
| C | Model configuration, health, logs, updates | Planned | Secrets remain server-side; all operations tested against actual service |
| D | Application inventory and operations | Planned | Backend-backed inventory, validated lifecycle routes |
| E | Restricted container management | Planned | No host Docker socket exposed to workloads; operation permissions tested |
| F | Historical monitoring and alerts | Planned | Collection persists real samples; alert and failure tests |
| F | Backup and restore | Planned | Backup integrity and restoration test to a separate target |
| G | Settings, access policies, audit logs | Planned | Server-side authorization and audit test coverage |

## Global invariants

- Keep Phase 1 (`nexus-server.service`, port 3000) untouched unless explicitly approved.
- Keep Phase 2 on `127.0.0.1:3001`.
- Preserve the SQLite database, users, sessions, authentication, CSRF, and current service configuration.
- Do not expose extra ports or execute bot workload commands as root.
- Never mark a service as running without a real server-side status check.
- Never fabricate resource values or historical samples.
- Review forwarded-IP trust and remote access boundaries before relying on IP-based restrictions.
