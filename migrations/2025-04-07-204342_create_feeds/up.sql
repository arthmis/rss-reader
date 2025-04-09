-- Your SQL goes here
CREATE TABLE IF NOT EXISTS feeds (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    url TEXT NOT NULL,
    feed_url TEXT UNIQUE NOT NULL,
    name TEXT NOT NULL,
    create_date TEXT NOT NULL,
    update_date TEXT NOT NULL
)
