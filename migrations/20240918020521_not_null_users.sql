-- Add migration script here

ALTER TABLE users ALTER COLUMN user_name SET NOT NULL;
