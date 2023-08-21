-- Drop the constraint on snapshot_id for models
ALTER TABLE pgml.models ALTER COLUMN snapshot_id DROP NOT NULL;

-- Add openai option to pgml.runtime
ALTER TYPE pgml.runtime ADD VALUE IF NOT EXISTS 'openai';

-- Add embedding option to pgml.task
ALTER TYPE pgml.task ADD VALUE IF NOT EXISTS 'embedding';
