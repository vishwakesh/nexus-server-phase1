# Security notes for Phase 1

- The service intentionally binds to loopback only.
- Authentication and authorization are not implemented in this milestone.
- Do not start an ngrok tunnel to this application yet.
- Never expose the service by changing the bind to `0.0.0.0`.
- This server currently offers only read-only host metrics and local event reads; it has no shell-execution, file-management, container-control, or service-control endpoint.
- Keep the SQLite directory private to the `nexus` user.
- If any credential is accidentally committed or logged, revoke/rotate it.
