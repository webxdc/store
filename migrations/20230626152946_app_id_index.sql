-- Add migration script here
CREATE INDEX IF NOT EXISTS app_id ON app_infos (app_id);
