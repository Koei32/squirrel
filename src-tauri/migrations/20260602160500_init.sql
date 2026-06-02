CREATE TABLE clipboard (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    type TEXT CHECK(type IN ('text', 'image', 'file')),
    content TEXT
) STRICT;


