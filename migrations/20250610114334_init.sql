CREATE TABLE IF NOT EXISTS services (
    id UUID PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    kind VARCHAR(50) NOT NULL,
    interval SMALLINT NOT NULL,
    config JSONB,
    checker_id UUID
);

CREATE TABLE IF NOT EXISTS healthcheck_results (
    id UUID PRIMARY KEY,
    service_id UUID NOT NULL,
    success BOOLEAN NOT NULL,
    response_time BIGINT,
    error VARCHAR(255),
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (service_id) REFERENCES services(id)
);
