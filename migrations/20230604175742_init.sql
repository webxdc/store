-- Add migration script here
CREATE TABLE IF NOT EXISTS config (
    id INTEGER PRIMARY KEY NOT NULL,
    invite_qr TEXT NOT NULL,
    genesis_qr TEXT NOT NULL,
    genesis_group INTEGER NOT NULL,
    serial INTEGER NOT NULL,
    shop_xdc_version TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS chat_to_chat_type (
    chat_id INTEGER PRIMARY KEY NOT NULL,
    chat_type INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS users (
    contact_id INTEGER PRIMARY KEY NOT NULL,
    tester BOOLEAN NOT NULL,
    publisher BOOLEAN NOT NULL,
    genesis BOOLEAN NOT NULL
);

CREATE TABLE IF NOT EXISTS app_infos (
    id INTEGER PRIMARY KEY NOT NULL,
    app_id TEXT NOT NULL,
    name TEXT NOT NULL,
    submitter_uri TEXT,
    source_code_url TEXT,
    image TEXT NOT NULL,
    description TEXT NOT NULL,
    xdc_blob_path TEXT NOT NULL,
    version NUMBER NOT NULL,
    originator INTEGER NOT NULL,
    serial INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS app_id ON app_infos (app_id);

CREATE TABLE IF NOT EXISTS webxdc_versions (
    msg_id INTEGER PRIMARY KEY NOT NULL,
    webxdc INTEGER NOT NULL,
    version INTEGER NOT NULL
);