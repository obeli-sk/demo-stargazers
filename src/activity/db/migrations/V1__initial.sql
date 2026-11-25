CREATE TABLE IF NOT EXISTS repos (name TEXT PRIMARY KEY);

CREATE TABLE IF NOT EXISTS users (
    name TEXT PRIMARY KEY,
    description TEXT,
    updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS stars (
    repo_name TEXT NOT NULL,
    user_name TEXT NOT NULL,
    FOREIGN KEY (repo_name) REFERENCES repos (name) ON DELETE CASCADE,
    FOREIGN KEY (user_name) REFERENCES users (name) ON DELETE CASCADE,
    UNIQUE (repo_name, user_name)
);

CREATE TABLE IF NOT EXISTS llm (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    settings TEXT NOT NULL
);
