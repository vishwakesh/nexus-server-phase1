# SQLite schema

`0001_initial.sql` holds the existing events table. `0002_auth.sql` defines Phase 2 accounts and server-side sessions. The Rust startup currently uses equivalent `CREATE TABLE IF NOT EXISTS` statements so a fresh separate Phase 2 database is initialized automatically. Keep the Phase 2 database directory separate from Phase 1.
