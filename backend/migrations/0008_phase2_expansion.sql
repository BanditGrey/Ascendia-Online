-- Expansão Fase 2: roster completo, cosméticos 8 tipos e capítulos 1-10
-- Adiciona classes restantes ao enum (seguro para re-execução)

DO $$ BEGIN
  ALTER TYPE character_class ADD VALUE IF NOT EXISTS 'mage';
EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN
  ALTER TYPE character_class ADD VALUE IF NOT EXISTS 'assassin';
EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN
  ALTER TYPE character_class ADD VALUE IF NOT EXISTS 'support';
EXCEPTION WHEN duplicate_object THEN NULL; END $$;

-- Cosméticos: libera 8 sistemas completos (8 tiers × 10 estrelas)
ALTER TABLE cosmetic_progress DROP CONSTRAINT IF EXISTS cosmetic_tier_mvp;
-- Remove check antigo de tipo (wings/mount apenas)
DO $$ DECLARE r RECORD; BEGIN
  FOR r IN SELECT conname FROM pg_constraint WHERE conrelid='cosmetic_progress'::regclass AND contype='c' LOOP
    IF r.conname LIKE '%cosmetic_type%' OR r.conname LIKE '%cosmetic_progress_cosmetic_type%' THEN
      EXECUTE 'ALTER TABLE cosmetic_progress DROP CONSTRAINT '||quote_ident(r.conname);
    END IF;
  END LOOP;
END $$;
ALTER TABLE cosmetic_progress ADD CONSTRAINT cosmetic_type_full CHECK (cosmetic_type IN ('wings','mount','pet','aura','mask','trail','hit_effect','frame'));
ALTER TABLE cosmetic_progress ADD CONSTRAINT cosmetic_tier_full CHECK (tier BETWEEN 1 AND 8);

-- Estágios: libera capítulos 1-10 (1-500) nas tabelas que ainda limitavam a 50
DO $$ DECLARE r RECORD; BEGIN
  FOR r IN SELECT conname FROM pg_constraint WHERE conrelid='combat_sessions'::regclass AND contype='c' LOOP
    EXECUTE 'ALTER TABLE combat_sessions DROP CONSTRAINT '||quote_ident(r.conname);
  END LOOP;
END $$;
ALTER TABLE combat_sessions ADD CONSTRAINT combat_sessions_stage_full CHECK (stage BETWEEN 1 AND 500);

DO $$ DECLARE r RECORD; BEGIN
  FOR r IN SELECT conname FROM pg_constraint WHERE conrelid='stage_stars'::regclass AND contype='c' LOOP
    EXECUTE 'ALTER TABLE stage_stars DROP CONSTRAINT '||quote_ident(r.conname);
  END LOOP;
END $$;
ALTER TABLE stage_stars ADD CONSTRAINT stage_stars_stage_full CHECK (stage BETWEEN 1 AND 500);

-- VIP e Battle Pass (estrutura mínima autoritativa)
CREATE TABLE IF NOT EXISTS vip_progress (
    user_id uuid PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    vip_points bigint NOT NULL DEFAULT 0 CHECK (vip_points >= 0),
    updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS battle_pass_seasons (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    name varchar(80) NOT NULL,
    starts_at timestamptz NOT NULL DEFAULT now(),
    ends_at timestamptz NOT NULL DEFAULT now() + interval '30 days',
    premium_cost integer NOT NULL DEFAULT 500 CHECK (premium_cost >= 0),
    is_active boolean NOT NULL DEFAULT true
);

CREATE TABLE IF NOT EXISTS battle_pass_progress (
    user_id uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    season_id uuid NOT NULL REFERENCES battle_pass_seasons(id) ON DELETE CASCADE,
    level smallint NOT NULL DEFAULT 0 CHECK (level BETWEEN 0 AND 50),
    xp bigint NOT NULL DEFAULT 0 CHECK (xp >= 0),
    premium boolean NOT NULL DEFAULT false,
    updated_at timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (user_id, season_id)
);

-- Semente de season ativa (idempotente)
INSERT INTO battle_pass_seasons (id, name, premium_cost, is_active)
VALUES ('00000000-0000-0000-0000-000000000001'::uuid, 'Season 1 — Inferno', 500, true)
ON CONFLICT (id) DO NOTHING;

-- Guilda (estrutura base GvG/Raid)
CREATE TABLE IF NOT EXISTS guilds (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    name varchar(40) NOT NULL UNIQUE,
    level smallint NOT NULL DEFAULT 1 CHECK (level BETWEEN 1 AND 50),
    leader_user_id uuid NOT NULL REFERENCES users(id),
    member_count smallint NOT NULL DEFAULT 1 CHECK (member_count BETWEEN 1 AND 50),
    created_at timestamptz NOT NULL DEFAULT now()
);
CREATE TABLE IF NOT EXISTS guild_members (
    guild_id uuid NOT NULL REFERENCES guilds(id) ON DELETE CASCADE,
    user_id uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    role varchar(16) NOT NULL DEFAULT 'member' CHECK (role IN ('leader','vice','officer','member','recruit')),
    contributed bigint NOT NULL DEFAULT 0,
    joined_at timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (guild_id, user_id),
    UNIQUE (user_id)
);

-- Marketplace (Diamantes entre players, taxa 10%)
CREATE TABLE IF NOT EXISTS marketplace_listings (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    seller_user_id uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    inventory_item_id uuid NOT NULL REFERENCES inventory_items(id) ON DELETE CASCADE,
    price_diamonds bigint NOT NULL CHECK (price_diamonds > 0),
    status varchar(16) NOT NULL DEFAULT 'active' CHECK (status IN ('active','sold','cancelled')),
    listed_at timestamptz NOT NULL DEFAULT now(),
    sold_at timestamptz
);
CREATE INDEX IF NOT EXISTS marketplace_active_price ON marketplace_listings(status, price_diamonds) WHERE status='active';

-- Catálogo expandido: itens para capítulos 2-10 e raridades altas
INSERT INTO item_templates (code,name,slot,rarity,base_stats,min_stage,tier) VALUES
('desert_blade_rare','Lâmina do Deserto','main_hand','rare','{"attack":42}',51,2),
('desert_armor_epic','Armadura de Areia','chest','epic','{"defense":70,"hp":220}',60,2),
('ice_staff_rare','Cajado de Gelo','main_hand','rare','{"attack":55,"crit_rate":0.03}',101,3),
('volcano_hammer_legendary','Martelo Vulcânico','main_hand','legendary','{"attack":95,"hp":400}',151,4),
('swamp_bow_epic','Arco do Pântano','main_hand','epic','{"attack":78,"crit_rate":0.04}',201,5),
('ruins_relic_mythic','Relíquia Ancestral','relic','mythic','{"attack":140,"defense":120,"hp":600}',251,6),
('abyss_dagger_legendary','Adaga do Abismo','main_hand','legendary','{"attack":110,"crit_damage":0.3}',301,7),
('celestial_wings_divine','Asas Celestiais','head','divine','{"attack":180,"defense":150,"hp":800}',351,7),
('chaos_blade_mythic','Lâmina do Caos','main_hand','mythic','{"attack":160,"penetration":0.15}',401,8),
('primordial_crown_primordial','Coroa Primordial','head','primordial','{"attack":250,"defense":200,"hp":1200}',451,8)
ON CONFLICT (code) DO NOTHING;
