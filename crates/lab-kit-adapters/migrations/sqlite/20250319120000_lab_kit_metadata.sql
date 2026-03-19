-- Same logical schema as postgres/ (SQLite: JSON as TEXT).

CREATE TABLE IF NOT EXISTS service_registry (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    lab_name TEXT NOT NULL DEFAULT '',
    service_name TEXT NOT NULL,
    endpoint_url TEXT,
    health_ok INTEGER,
    last_health_check TEXT,
    UNIQUE (lab_name, service_name)
);

CREATE TABLE IF NOT EXISTS conformance_runs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    -- RFC3339 so adapters can parse with DateTime::parse_from_rfc3339 (unlike datetime('now')).
    run_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    helix_output TEXT NOT NULL,
    overall_pass INTEGER NOT NULL,
    per_service TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS license_activations (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    key_hash TEXT NOT NULL UNIQUE,
    activated_at TEXT NOT NULL,
    expires_at TEXT,
    features TEXT NOT NULL
);
