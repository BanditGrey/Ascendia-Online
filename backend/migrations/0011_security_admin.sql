-- Fase 5: OAuth2, 2FA, Admin, Rate Limit log, Observabilidade

CREATE TABLE IF NOT EXISTS oauth_accounts (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    provider varchar(16) NOT NULL CHECK (provider IN ('google','discord')),
    provider_user_id varchar(128) NOT NULL,
    email varchar(320),
    created_at timestamptz NOT NULL DEFAULT now(),
    UNIQUE (provider, provider_user_id),
    UNIQUE (user_id, provider)
);

CREATE TABLE IF NOT EXISTS user_totp (
    user_id uuid PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    secret_encrypted text NOT NULL,
    enabled boolean NOT NULL DEFAULT false,
    verified_at timestamptz,
    created_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS rate_limit_log (
    id bigserial PRIMARY KEY,
    key varchar(128) NOT NULL,
    count integer NOT NULL DEFAULT 1,
    window_start timestamptz NOT NULL DEFAULT now()
);
CREATE INDEX IF NOT EXISTS rate_limit_key_time ON rate_limit_log(key, window_start DESC);

-- Admin: marca usuário como admin (flag simples)
ALTER TABLE users ADD COLUMN IF NOT EXISTS is_admin boolean NOT NULL DEFAULT false;
ALTER TABLE users ADD COLUMN IF NOT EXISTS is_gm boolean NOT NULL DEFAULT false;

-- Métricas básicas (contadores)
CREATE TABLE IF NOT EXISTS metrics_counters (
    name varchar(64) PRIMARY KEY,
    value bigint NOT NULL DEFAULT 0,
    updated_at timestamptz NOT NULL DEFAULT now()
);
INSERT INTO metrics_counters (name, value) VALUES ('combat_started',0),('items_crafted',0),('trades_completed',0) ON CONFLICT (name) DO NOTHING;

-- Assets: tabela de skins alternativas (4-6/tier tradáveis)
CREATE TABLE IF NOT EXISTS cosmetic_skins (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    cosmetic_type varchar(24) NOT NULL CHECK (cosmetic_type IN ('wings','mount','pet','aura','mask','trail','hit_effect','frame')),
    tier smallint NOT NULL CHECK (tier BETWEEN 1 AND 8),
    skin_code varchar(40) NOT NULL UNIQUE,
    name varchar(80) NOT NULL,
    tradable boolean NOT NULL DEFAULT true,
    created_at timestamptz NOT NULL DEFAULT now()
);
INSERT INTO cosmetic_skins (cosmetic_type, tier, skin_code, name) VALUES
('wings',1,'wings_t1_skin_a','Asas Aprendiz — Cinza'),
('wings',1,'wings_t1_skin_b','Asas Aprendiz — Carmim'),
('wings',2,'wings_t2_skin_a','Angelicais — Prata'),
('mount',1,'mount_t1_skin_a','Cavalo — Negro'),
('pet',1,'pet_t1_skin_a','Gato — Sombrio')
ON CONFLICT (skin_code) DO NOTHING;

CREATE TABLE IF NOT EXISTS user_cosmetic_skins (
    user_id uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    skin_id uuid NOT NULL REFERENCES cosmetic_skins(id),
    acquired_at timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (user_id, skin_id)
);
