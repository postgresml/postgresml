-- Add conversation, text-pair-classification task type
ALTER TYPE pgml.task ADD VALUE IF NOT EXISTS 'conversation';
ALTER TYPE pgml.task ADD VALUE IF NOT EXISTS 'text-pair-classification';

-- Crate pgml.logs table
CREATE TABLE IF NOT EXISTS pgml.logs (
    id SERIAL PRIMARY KEY,
    model_id BIGINT,
    project_id BIGINT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    logs JSONB
);
