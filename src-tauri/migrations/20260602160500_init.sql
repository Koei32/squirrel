CREATE TABLE clipboard (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    event_type TEXT CHECK(event_type IN ('text', 'image', 'file')),
    content TEXT,
    timestamp TEXT
) STRICT;


