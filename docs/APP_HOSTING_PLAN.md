# Future application hosting module (not implemented in Phase 2)

NEXUS is intended to host independent tools such as a separate video-clipping service, small APIs, or bot services. That capability is intentionally not enabled in this authentication foundation.

The planned safe design:
1. Store an allowlisted catalog of named applications; each entry has a fixed working directory, runtime executable, fixed arguments, user/group, port, and health check.
2. Run each application under its own unprivileged Linux user or isolated container.
3. Admin actions call a narrow backend service adapter; no user-supplied shell command, interpolated command line, or arbitrary path.
4. Use systemd templates or a tightly scoped supervisor, with resource limits where practical.
5. Require admin role and CSRF validation for start/stop/restart, and write audit events.
6. Proxy app web traffic through an authenticated, allowlisted routing layer; do not expose arbitrary app ports to the internet.
7. Avoid mounting `/var/run/docker.sock` into NEXUS. If Docker control is added, use a narrow privileged helper or API policy rather than raw socket access.

First target later: a standalone app with a known health endpoint and a fixed port. Do not treat the catalog as arbitrary code execution.
