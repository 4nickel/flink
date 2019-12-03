CREATE TABLE users (
    id              INTEGER NOT NULL PRIMARY KEY,
    name            TEXT NOT NULL
);

CREATE TABLE sessions (
    id              INTEGER NOT NULL PRIMARY KEY,
    user_id         INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token           TEXT NOT NULL
);

CREATE TABLE passwords (
    user_id         INTEGER NOT NULL PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    hash            BLOB NOT NULL,
    salt            TEXT NOT NULL
);

CREATE TABLE files (
    id              INTEGER NOT NULL PRIMARY KEY,
    user_id         INTEGER NOT NULL REFERENCES users(id),
    key             TEXT NOT NULL,
    val             TEXT NOT NULL,
    upload_date     TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    delete_date     TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    downloads       INTEGER NOT NULL,
    bytes           INTEGER NOT NULL
);
