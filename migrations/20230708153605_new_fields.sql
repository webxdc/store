-- Add date and size fields to app_infos

ALTER TABLE app_infos ADD COLUMN date INTEGER NOT NULL DEFAULT 0;
ALTER TABLE app_infos ADD COLUMN size INTEGER NOT NULL DEFAULT 0;