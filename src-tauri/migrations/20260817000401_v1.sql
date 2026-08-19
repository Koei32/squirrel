PRAGMA user_version = 1;

-- https://sqlite.org/wal.html
PRAGMA journal_mode=WAL;

-- Arbitrary amount of cache (bytes)
PRAGMA cache_size=-8000;

CREATE TABLE clipboard (
    id INTEGER PRIMARY KEY, -- unix timestamp
    event_type TEXT NOT NULL CHECK(event_type IN ('text', 'image', 'file')),
    is_pinned INTEGER NOT NULL CHECK (is_pinned IN (0, 1)) DEFAULT 0,
    content_text TEXT, -- for text and files
    content_blob BLOB, -- for images
    expires_at INTEGER NOT NULL
) STRICT;
