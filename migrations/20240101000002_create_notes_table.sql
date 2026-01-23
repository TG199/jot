CREATE TABLE notes(
    note_id UUID NOT NULL,
    PRIMARY KEY (note_id),
    user_id UUID NOT NULL,
    FOREIGN KEY (user_id) REFERENCES users(user_id) ON DELETE CASCADE,
    title TEXT NOT NULL,
    content TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_notes_user_id ON notes(user_id);

CREATE INDEX idx_notes_created_at ON notes(created_at DESC);

CREATE INDEX idx_notes_search ON notes USING GIN(to_tsvector('english', title || ' ' || content));