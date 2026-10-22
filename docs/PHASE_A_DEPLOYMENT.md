# NEXUS Phase A Frontend Patch

This patch only replaces the Phase 2 frontend assets:

- `frontend/index.html`
- `frontend/app.css`
- `frontend/app.js`

It does **not** modify Rust routes, authentication logic, environment files, database files, systemd definitions, Phase 1, Docker, or Hermes Agent.

## Important compatibility note

The existing Rust application appears to serve these assets from `/assets/app.css` and `/assets/app.js` and may embed them at compile time. After copying the files, rebuild the release binary and restart only `nexus-server-phase2` so the compiled binary serves the updated assets. Do not assume the dashboard is deployed until it is verified through the actual tunnel/browser.

## Recommended deployment

1. Upload and extract the patch ZIP on the VPS to a temporary directory.
2. Read `scripts/apply-phase-a.sh` before executing it.
3. Run the script from its extracted patch directory as root. It backs up the three frontend files, installs the patch, builds the Rust release binary, restarts only Phase 2 after a successful build, and checks local health. If the build or health check fails, it attempts to restore the previous frontend files and rebuild/restart the previous version.
4. Verify login, registration status, session restoration, logout, `/api/health`, `/api/auth/status`, `/api/metrics`, and `/api/events`.
5. Test the UI at phone and desktop widths and inspect browser console and service logs.

No install or test on the user's VPS has been performed by this deliverable's creator. The patch is not a substitute for testing the exact current API payload shape.

## Security notes

- Do not paste ngrok authentication tokens into chat or commit them to Git.
- First-account administrator registration should not be left openly available to random visitors. Keep a private access gate in front of registration until an administrator account is claimed.
- The frontend does not implement bot execution, Docker operations, or Hermes installation. Those features remain roadmap items and require separately reviewed backend work.
- Frontend status labels use the existing `/api/health` endpoint and show unavailable/empty states when relevant API data is absent.
