# NEXUS Phase B Frontend — UI Prototype

This is a responsive static HTML/CSS/JavaScript prototype based on the supplied NEXUS Phase B reference image.

## Included screens

- Bot Hosting inventory and summary cards
- Create Bot wizard (runtime selection, configuration, review, preview-create)
- Bot details and overview
- File manager / text editor preview
- Environment variable manager UI
- Dependency management UI
- Logs viewer and filtering
- Activity timeline
- Responsive sidebar/mobile layouts

## Important status

**This is a frontend prototype, not a production bot-hosting implementation.** It uses in-memory sample data and simulates safe UI states locally. It does not connect to the NEXUS Rust API, create containers, execute code, install dependencies, write server files, or change production configuration. The interface clearly labels itself as a design preview.

The production backend must provide authenticated and ownership-checked API endpoints before operational features are enabled. Do not make this static preview public with real bot data or secrets.

## Preview locally

Open `index.html` in a modern browser, or serve the folder with any static file server, for example:

```bash
python -m http.server 8080
```

Then open `http://127.0.0.1:8080`.

## Integration notes

- Keep the existing NEXUS application shell and authentication flow when integrating into the production repository.
- Merge the UI into the existing `frontend/index.html`, `frontend/app.css`, and `frontend/app.js` rather than replacing unrelated Phase A functionality wholesale.
- Replace the in-memory handlers with the existing authenticated API helper and real backend responses.
- Ensure all state-changing requests use the existing CSRF mechanism.
- Since the production Rust app embeds frontend assets, rebuild the release binary after the final integration.
- Verify desktop/mobile layout, API error states, cross-user ownership denial, and existing Phase A regression behavior before deploying.
