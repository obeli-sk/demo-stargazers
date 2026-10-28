-- Speeds up `max(updated_at)` used when ordering stargazers.
CREATE INDEX IF NOT EXISTS users_updated_at ON users (updated_at);
