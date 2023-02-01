-- Add up migration script here
CREATE TABLE pgml.notebooks (
    id BIGSERIAL PRIMARY KEY,
    name VARCHAR NOT NULL,
    created_at TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT NOW()
);

CREATE TABLE pgml.notebook_cells (
    id BIGSERIAL PRIMARY KEY,
    notebook_id BIGINT NOT NULL REFERENCES pgml.notebooks(id),
    cell_type INT NOT NULL,
    cell_number INT NOT NULL,
    version INT NOT NULL,
    contents TEXT NOT NULL,
    rendering TEXT,
    execution_time INTERVAL,
    deleted_at TIMESTAMP WITHOUT TIME ZONE
);
