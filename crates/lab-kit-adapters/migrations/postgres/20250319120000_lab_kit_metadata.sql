-- Lab Kit operational metadata (not Ferrum platform schema).

CREATE TABLE IF NOT EXISTS service_registry (
    id BIGSERIAL PRIMARY KEY,
    lab_name TEXT NOT NULL DEFAULT '',
    service_name TEXT NOT NULL,
    endpoint_url TEXT,
    health_ok BOOLEAN,
    last_health_check TIMESTAMPTZ,
    UNIQUE (lab_name, service_name)
);

CREATE TABLE IF NOT EXISTS conformance_runs (
    id BIGSERIAL PRIMARY KEY,
    run_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    helix_output JSONB NOT NULL,
    overall_pass BOOLEAN NOT NULL,
    per_service JSONB NOT NULL
);

CREATE TABLE IF NOT EXISTS license_activations (
    id BIGSERIAL PRIMARY KEY,
    key_hash TEXT NOT NULL UNIQUE,
    activated_at TIMESTAMPTZ NOT NULL,
    expires_at TIMESTAMPTZ,
    features JSONB NOT NULL
);
