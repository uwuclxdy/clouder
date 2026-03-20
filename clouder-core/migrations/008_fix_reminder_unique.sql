-- 008: Fix reminder_configs

-- Remove duplicate reminder_configs, keeping the row with the highest id per guild+type
DELETE FROM reminder_configs
WHERE id NOT IN (
    SELECT MAX(id) FROM reminder_configs GROUP BY guild_id, reminder_type
);

-- Add unique constraint to fix INSERT ... ON CONFLICT
CREATE UNIQUE INDEX IF NOT EXISTS reminder_configs_guild_type_unique
ON reminder_configs (guild_id, reminder_type);
