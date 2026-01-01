-- Add privacy setting to users table
ALTER TABLE users ADD COLUMN is_private BOOLEAN NOT NULL DEFAULT false;
