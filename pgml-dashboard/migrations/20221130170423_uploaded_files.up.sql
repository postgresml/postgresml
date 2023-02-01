-- Add up migration script here
-- Add up migration script here
CREATE TABLE pgml.uploaded_files (
    id BIGSERIAL PRIMARY KEY,
    created_at TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT NOW()
);
