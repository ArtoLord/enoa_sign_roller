CREATE TABLE IF NOT EXISTS users (
    id text NOT NULL,
    guild_id text NOT NULL,
    shaman_power int NOT NULL,
    PRIMARY KEY (id, guild_id)
);

CREATE TABLE IF NOT EXISTS guilds (
    id text PRIMARY KEY,
    sign_id text NOT NULL,
    sign_created_at timestamp NOT NULL,
    sign_created_by_id text NOT NULL,
    sign_state text NOT NULL,
    sign_state_made_by_id text
);