-- Your SQL goes here
CREATE TABLE IF NOT EXISTS feeds (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    url TEXT NOT NULL,
    feed_url TEXT UNIQUE NOT NULL,
    name TEXT NOT NULL,
    create_date TEXT NOT NULL,
    update_date TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS feed_items (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    channel_id INTEGER NOT NULL,
    title TEXT,
    url TEXT UNIQUE,
    description TEXT,
    author TEXT,
    pub_date TEXT,
    create_date TEXT NOT NULL,
    update_date TEXT NOT NULL,
    FOREIGN KEY(channel_id) REFERENCES feeds(id)
)