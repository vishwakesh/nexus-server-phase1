-- NEXUS Phase B backend foundation: user-owned bot configuration and audit events.
-- Runtime execution is intentionally not enabled by this migration alone.
CREATE TABLE IF NOT EXISTS bots (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    owner_user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name TEXT NOT NULL CHECK(length(name) BETWEEN 2 AND 48),
    description TEXT,
    runtime TEXT NOT NULL CHECK(runtime IN ('python','node')),
    runtime_version TEXT NOT NULL,
    entrypoint TEXT NOT NULL,
    cpu_limit_millicores INTEGER NOT NULL CHECK(cpu_limit_millicores BETWEEN 100 AND 2000),
    memory_limit_mb INTEGER NOT NULL CHECK(memory_limit_mb BETWEEN 128 AND 2048),
    desired_state TEXT NOT NULL DEFAULT 'stopped' CHECK(desired_state IN ('stopped','running')),
    observed_state TEXT NOT NULL DEFAULT 'stopped' CHECK(observed_state IN ('stopped','starting','running','stopping','failed','deploying','unavailable')),
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
    last_activity_at TEXT
);
CREATE INDEX IF NOT EXISTS bots_owner_idx ON bots(owner_user_id, id DESC);

CREATE TABLE IF NOT EXISTS bot_events (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    bot_id INTEGER NOT NULL REFERENCES bots(id) ON DELETE CASCADE,
    owner_user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    event_type TEXT NOT NULL,
    message TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))
);
CREATE INDEX IF NOT EXISTS bot_events_owner_bot_idx ON bot_events(owner_user_id, bot_id, id DESC);
