# Security notes — Phase 2

- The server rejects non-loopback `NEXUS_BIND` values. Do not remove this guard.
- Create the first administrator account over SSH port forwarding before enabling a public tunnel. The first registered account becomes admin; a public race could let another Telangana IP claim it first.
- Keep the Phase 2 SQLite database private and separate from Phase 1.
- `NEXUS_COOKIE_SECURE=true` is required when users access the app through HTTPS. Keep the public edge HTTPS-only.
- The application interprets the final valid IP in `X-Forwarded-For` when the TCP peer is loopback. Keep the backend bound to loopback and ensure only a trusted local reverse proxy/tunnel can connect to it.
- Region enforcement depends on a third-party HTTPS IP geolocation provider. The provider receives the public IP; remote traffic is denied if lookup fails. Location is approximate, not proof of physical presence.
- IP-region restrictions are not a replacement for passwords/session security.
- Do not add arbitrary shell execution, arbitrary process launch, raw Docker socket access, or unrestricted file access to HTTP endpoints.
- Do not log credentials, session cookies, CSRF values, or password hashes.
