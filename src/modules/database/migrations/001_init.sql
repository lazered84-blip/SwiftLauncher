-- Migration 1 : schéma initial.
-- RÈGLE : aucun token, mot de passe ni secret dans cette base.

CREATE TABLE IF NOT EXISTS instances (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    name        TEXT    NOT NULL UNIQUE,
    mc_version  TEXT    NOT NULL,
    loader      TEXT    NOT NULL DEFAULT 'vanilla',
    ram_mb      INTEGER NOT NULL DEFAULT 4096,
    created_at  TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);

CREATE TABLE IF NOT EXISTS accounts (
    uuid        TEXT    PRIMARY KEY,
    name        TEXT    NOT NULL,
    is_active   INTEGER NOT NULL DEFAULT 0,
    created_at  TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);
