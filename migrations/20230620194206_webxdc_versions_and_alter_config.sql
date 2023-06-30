-- Add migration script here
CREATE TABLE IF NOT EXISTS webxdc_versions (
    msg_id INTEGER PRIMARY KEY NOT NULL,
    webxdc INTEGER NOT NULL,
    version INTEGER NOT NULL
);

ALTER TABLE config ADD COLUMN shop_xdc_version TEXT NOT NULL DEFAULT '0.0.0';
ALTER TABLE config ADD COLUMN submit_xdc_version TEXT NOT NULL DEFAULT '0.0.0';
ALTER TABLE config ADD COLUMN review_xdc_version TEXT NOT NULL DEFAULT '0.0.0';

