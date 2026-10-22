# NEXUS Phase 2 — Build status at handoff

Implemented in source:
- Existing localhost-only Rust/Axum dashboard endpoints
- SQLite events, users, and sessions tables
- First registered account assigned admin in an SQLite `BEGIN IMMEDIATE` transaction; later registrations assigned user
- Argon2 password hashing and secure random opaque sessions
- Server-side session lookup/revocation, 12-hour expiry, HttpOnly/SameSite cookie, optional Secure cookie flag
- CSRF-token check for authenticated non-safe methods
- In-memory auth attempt throttle
- Telangana public IP geolocation gate via HTTPS `ipapi.co`, with 15-minute success-result cache and fail-closed remote behavior
- Security response headers and login/register frontend
- Local-only bind enforced by Rust startup validation

Checks performed in the authoring workspace:
- JavaScript syntax check: passed (`node --check frontend/app.js`)
- HTML parser smoke check: passed

Not verified:
- Rust compilation and Cargo tests: Rust/Cargo are not installed in the authoring workspace; must be checked on the VPS.
- End-to-end browser authentication, IP-geolocation provider behavior, and HTTPS proxy headers must be tested on the VPS.

Application hosting remains a planned phase and is not implemented in this build. Keep Phase 2 loopback-only until the first admin account is created and the deployment is reviewed.
