-- 006: Selfrole label cache
CREATE TABLE IF NOT EXISTS selfrole_labels (
	guild_id TEXT NOT NULL,
	role_id TEXT NOT NULL,
	name TEXT NOT NULL,
	updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
	PRIMARY KEY (guild_id, role_id)
);

CREATE INDEX IF NOT EXISTS idx_selfrole_labels_guild ON selfrole_labels (guild_id);