CREATE TABLE IF NOT EXISTS loser_counts (
    user_id    TEXT    PRIMARY KEY,
    count      INTEGER NOT NULL DEFAULT 0,
    updated_at TEXT    NOT NULL DEFAULT (datetime('now'))
);
