-- Add migration script here

-- Add next_run field to services table
ALTER TABLE services ADD COLUMN next_run TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP;
