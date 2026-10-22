# NEXUS Server — Phase 2 Auth Foundation

NEXUS Phase 2 builds on the existing Rust/Axum dashboard and adds account registration, a first-account administrator rule, Argon2 password hashes, server-side sessions, CSRF validation for authenticated state-changing requests, login/registration throttling, security headers, and a Telangana-only public-IP geolocation gate.

**Phase 2 is a separate development copy. It does not change or replace Phase 1.** Default bind is `127.0.0.1:3001`; the server refuses non-loopback binding. Do not expose it directly to the public internet.

## Features

- The first successful registration is assigned `admin` atomically in SQLite. All later registrations get the `user` role.
- Passwords are hashed with Argon2. Plain-text passwords are never written to the database or logs.
- Random, opaque session cookies are `HttpOnly` and `SameSite=Strict`; only a SHA-256 hash of the session token is stored in SQLite.
- Sessions expire after 12 hours. Logout revokes the server-side session.
- CSRF token validation is included for authenticated non-safe HTTP methods.
- Login and registration attempts are throttled in memory per forwarded client IP.
- Every non-health request is restricted to public IPs whose `ipapi.co` response says country `IN` and region `Telangana`. Successful checks are cached for 15 minutes.
- Health endpoint: `GET /api/health`; auth status: `GET /api/auth/status`; session: `GET /api/auth/me`.
- CPU, memory, disk, uptime, and SQLite events require a valid login.

## Important setup order

1. Deploy on loopback only.
2. Open a **private SSH tunnel** to port 3001 and register the first account. That first account becomes admin.
3. Confirm the account and sign-in work.
4. Only then consider an HTTPS tunnel or reverse proxy. Set `NEXUS_COOKIE_SECURE=true` before using an HTTPS URL.

Do not expose the service before creating your admin account: anyone who registers first can become admin. This project deliberately does not bind to `0.0.0.0`.

## Build on Ubuntu / the VPS

Required tools: Rust/Cargo, Git, a C build toolchain, and a supported system environment. With the source in `/opt/nexus-server-phase2`:

```bash
cd /opt/nexus-server-phase2
cargo fmt --check
cargo check
cargo test
cargo build --release
```

The maintainer's authoring workspace did not have Rust installed, so Rust compilation has **not** been verified here. Run these commands on your VPS and fix any compiler output before enabling the service.

## Example dedicated systemd service

Review `deploy/nexus-server-phase2.service`. It is a template: check paths and the `nexus-web` account before installing it. Keep Phase 1 on port 3000 and give Phase 2 a separate data directory and port 3001. Never point Phase 2 at the Phase 1 SQLite file.

## Telangana IP gate and privacy

The gate uses HTTPS requests to `ipapi.co` to estimate the region for the public IP. That third-party service receives the connecting public IP. IP geolocation is approximate and can be incorrect for mobile carriers, VPNs, proxies, and roaming connections. If the lookup fails, remote requests are denied (fail closed). Loopback requests are allowed for private local setup/recovery. Do not trust forwarded headers if you expose the app without keeping its backend bound to loopback and routing requests through a known local proxy.

The region gate checks state, not individual districts. It accepts a response identifying `Telangana` (or region code `TG` / `TS`) in India. A state-level IP database cannot guarantee a person's physical location.

## What is not implemented yet

- Application hosting, arbitrary process execution, Docker controls, file manager, Hermes controls, backups, public tunnel setup, or an app installation interface.
- Multi-factor authentication, email verification, password reset, persistent rate-limit storage, and administrator invite management.
- A standalone GeoIP database; region checks require the external HTTPS lookup service.

Application hosting should be added later using an **allowlist of named apps and fixed service definitions**, dedicated unprivileged users, restricted working directories, and audit logs. Do not add arbitrary shell commands or expose the Docker socket to the web application.
