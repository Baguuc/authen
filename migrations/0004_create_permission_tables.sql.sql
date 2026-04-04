CREATE TABLE IF NOT EXISTS permissions (name TEXT PRIMARY KEY);

CREATE TABLE IF NOT EXISTS user_permissions (
    user_id UUID REFERENCES users(id),
    permission_name TEXT REFERENCES permissions(name)
);