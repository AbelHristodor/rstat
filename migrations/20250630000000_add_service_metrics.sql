-- Create service_metrics table to store daily uptime and latency data
CREATE TABLE IF NOT EXISTS service_metrics (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    service_id UUID NOT NULL,
    date DATE NOT NULL,
    uptime_percentage DECIMAL(5,2) NOT NULL CHECK (uptime_percentage >= 0 AND uptime_percentage <= 100),
    average_latency_ms INTEGER NOT NULL CHECK (average_latency_ms >= 0),
    total_checks INTEGER NOT NULL DEFAULT 0 CHECK (total_checks >= 0),
    successful_checks INTEGER NOT NULL DEFAULT 0 CHECK (successful_checks >= 0),
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (service_id) REFERENCES services(id) ON DELETE CASCADE,
    UNIQUE(service_id, date)
);

-- Create index for efficient queries by service_id and date
CREATE INDEX IF NOT EXISTS idx_service_metrics_service_date ON service_metrics(service_id, date);

-- Create index for date range queries
CREATE INDEX IF NOT EXISTS idx_service_metrics_date ON service_metrics(date);

-- Create a function to update the updated_at timestamp
CREATE OR REPLACE FUNCTION update_service_metrics_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Create trigger to automatically update updated_at
CREATE TRIGGER trigger_update_service_metrics_updated_at
    BEFORE UPDATE ON service_metrics
    FOR EACH ROW
    EXECUTE FUNCTION update_service_metrics_updated_at(); 