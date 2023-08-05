-- Add migration script here
CREATE TABLE IF NOT EXISTS config (
    id INTEGER PRIMARY KEY NOT NULL,
    invite_qr TEXT NOT NULL,
    serial INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS app_infos (
    id INTEGER PRIMARY KEY autoincrement,
    app_id TEXT NOT NULL,
    date NUMBER NOT NULL,
    size NUMBER NOT NULL,
    name TEXT NOT NULL,
    source_code_url TEXT NOT NULL,
    image TEXT NOT NULL,
    description TEXT NOT NULL,
    xdc_blob_path TEXT NOT NULL,
    tag_name TEXT NOT NULL,
    serial INTEGER NOT NULL,
    removed BOOLEAN NOT NULL DEFAULT 0
);

CREATE INDEX IF NOT EXISTS app_id ON app_infos (app_id);

CREATE TABLE IF NOT EXISTS webxdc_tag_names (
    msg_id INTEGER PRIMARY KEY NOT NULL,
    tag_name TEXT NOT NULL
);
