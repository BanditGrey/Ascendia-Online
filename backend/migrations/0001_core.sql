-- Fundação transacional da Fase 1. Extensões e enums são idempotentes.
CREATE EXTENSION IF NOT EXISTS pgcrypto;

CREATE TYPE user_status AS ENUM ('active','suspended','banned','deleted');
CREATE TYPE character_gender AS ENUM ('male','female');
CREATE TYPE character_class AS ENUM ('commander','warrior','archer');
CREATE TYPE item_rarity AS ENUM ('common','uncommon','rare','epic','legendary','mythic','divine','primordial');
CREATE TYPE item_slot AS ENUM ('head','main_hand','chest','off_hand','legs','ring','feet','necklace','hands','relic');
CREATE TYPE combat_difficulty AS ENUM ('normal','hard','inferno','chaos');

CREATE TABLE users (
    id uuid PRIMARY KEY,
    email varchar(320) NOT NULL,
    password_hash text NOT NULL,
    display_name varchar(24) NOT NULL,
    status user_status NOT NULL DEFAULT 'active',
    gold bigint NOT NULL DEFAULT 0 CHECK (gold >= 0),
    diamonds bigint NOT NULL DEFAULT 0 CHECK (diamonds >= 0),
    vip_level smallint NOT NULL DEFAULT 0 CHECK (vip_level BETWEEN 0 AND 15),
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    last_login_at timestamptz
);
CREATE UNIQUE INDEX users_email_unique ON users(lower(email));
CREATE UNIQUE INDEX users_display_name_unique ON users(lower(display_name));

CREATE TABLE refresh_sessions (
    id uuid PRIMARY KEY,
    user_id uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token_hash char(64) NOT NULL UNIQUE,
    expires_at timestamptz NOT NULL,
    revoked_at timestamptz,
    user_agent varchar(512) NOT NULL DEFAULT '',
    ip_address varchar(64) NOT NULL DEFAULT '',
    created_at timestamptz NOT NULL DEFAULT now()
);
CREATE INDEX refresh_sessions_user_active ON refresh_sessions(user_id) WHERE revoked_at IS NULL;

CREATE TABLE characters (
    id uuid PRIMARY KEY,
    user_id uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name varchar(24) NOT NULL,
    gender character_gender NOT NULL,
    class character_class NOT NULL,
    subclass varchar(32) NOT NULL,
    level smallint NOT NULL DEFAULT 1 CHECK (level BETWEEN 1 AND 200),
    experience bigint NOT NULL DEFAULT 0 CHECK (experience >= 0),
    awakening smallint NOT NULL DEFAULT 0 CHECK (awakening BETWEEN 0 AND 5),
    star_rating smallint NOT NULL DEFAULT 1 CHECK (star_rating BETWEEN 1 AND 6),
    is_leader boolean NOT NULL DEFAULT false,
    created_at timestamptz NOT NULL DEFAULT now()
);
CREATE UNIQUE INDEX one_leader_per_user ON characters(user_id) WHERE is_leader;

CREATE TABLE character_stats (
    character_id uuid PRIMARY KEY REFERENCES characters(id) ON DELETE CASCADE,
    hp bigint NOT NULL DEFAULT 1000 CHECK (hp > 0),
    attack bigint NOT NULL DEFAULT 150 CHECK (attack > 0),
    defense bigint NOT NULL DEFAULT 100 CHECK (defense >= 0),
    attack_speed double precision NOT NULL DEFAULT 1.2 CHECK (attack_speed > 0),
    crit_rate double precision NOT NULL DEFAULT .05 CHECK (crit_rate BETWEEN 0 AND 1),
    crit_damage double precision NOT NULL DEFAULT 1.5 CHECK (crit_damage >= 1),
    luck double precision NOT NULL DEFAULT 0 CHECK (luck BETWEEN 0 AND 1),
    accuracy double precision NOT NULL DEFAULT 0 CHECK (accuracy BETWEEN 0 AND 1),
    dodge double precision NOT NULL DEFAULT 0 CHECK (dodge BETWEEN 0 AND 1),
    penetration double precision NOT NULL DEFAULT 0 CHECK (penetration BETWEEN 0 AND 1),
    power_rating bigint NOT NULL DEFAULT 0 CHECK (power_rating >= 0),
    calculated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE squads (
    id uuid PRIMARY KEY,
    user_id uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name varchar(40) NOT NULL,
    is_active boolean NOT NULL DEFAULT true,
    created_at timestamptz NOT NULL DEFAULT now()
);
CREATE UNIQUE INDEX one_active_squad_per_user ON squads(user_id) WHERE is_active;
CREATE TABLE squad_slots (
    squad_id uuid NOT NULL REFERENCES squads(id) ON DELETE CASCADE,
    slot smallint NOT NULL CHECK (slot BETWEEN 1 AND 6),
    character_id uuid NOT NULL REFERENCES characters(id) ON DELETE CASCADE,
    PRIMARY KEY (squad_id,slot),
    UNIQUE (squad_id,character_id)
);

CREATE TABLE item_templates (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    code varchar(80) NOT NULL UNIQUE,
    name varchar(120) NOT NULL,
    slot item_slot,
    rarity item_rarity NOT NULL,
    base_stats jsonb NOT NULL DEFAULT '{}',
    min_stage smallint NOT NULL DEFAULT 1,
    tradeable boolean NOT NULL DEFAULT true
);
CREATE TABLE inventory_items (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    template_id uuid NOT NULL REFERENCES item_templates(id),
    quantity integer NOT NULL DEFAULT 1 CHECK (quantity > 0),
    enhancement smallint NOT NULL DEFAULT 0 CHECK (enhancement BETWEEN 0 AND 20),
    rolled_stats jsonb NOT NULL DEFAULT '{}',
    bound boolean NOT NULL DEFAULT false,
    trade_locked_until timestamptz,
    acquired_at timestamptz NOT NULL DEFAULT now()
);
CREATE INDEX inventory_owner ON inventory_items(user_id);
CREATE TABLE equipment_slots (
    character_id uuid NOT NULL REFERENCES characters(id) ON DELETE CASCADE,
    slot item_slot NOT NULL,
    slot_index smallint NOT NULL DEFAULT 1,
    inventory_item_id uuid NOT NULL UNIQUE REFERENCES inventory_items(id),
    PRIMARY KEY (character_id,slot,slot_index)
);

CREATE TABLE stage_progress (
    user_id uuid PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    max_stage smallint NOT NULL DEFAULT 0 CHECK (max_stage BETWEEN 0 AND 500),
    total_stars integer NOT NULL DEFAULT 0 CHECK (total_stars >= 0),
    updated_at timestamptz NOT NULL DEFAULT now()
);
CREATE TABLE combat_runs (
    id uuid PRIMARY KEY,
    user_id uuid NOT NULL REFERENCES users(id),
    stage smallint NOT NULL CHECK (stage BETWEEN 1 AND 500),
    difficulty combat_difficulty NOT NULL,
    seed bigint NOT NULL,
    victory boolean NOT NULL,
    duration_ms bigint NOT NULL CHECK (duration_ms >= 0),
    damage_dealt bigint NOT NULL CHECK (damage_dealt >= 0),
    damage_taken bigint NOT NULL CHECK (damage_taken >= 0),
    gold_reward bigint NOT NULL DEFAULT 0 CHECK (gold_reward >= 0),
    experience_reward bigint NOT NULL DEFAULT 0 CHECK (experience_reward >= 0),
    started_at timestamptz NOT NULL DEFAULT now()
);
CREATE INDEX combat_runs_user_time ON combat_runs(user_id,started_at DESC);

CREATE TABLE cosmetic_progress (
    user_id uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    cosmetic_type varchar(24) NOT NULL CHECK (cosmetic_type IN ('wings','mount')),
    tier smallint NOT NULL DEFAULT 1 CHECK (tier BETWEEN 1 AND 8),
    stars smallint NOT NULL DEFAULT 0 CHECK (stars BETWEEN 0 AND 10),
    fragments integer NOT NULL DEFAULT 0 CHECK (fragments >= 0),
    essences integer NOT NULL DEFAULT 0 CHECK (essences >= 0),
    PRIMARY KEY (user_id,cosmetic_type)
);

CREATE TABLE audit_logs (
    id bigserial PRIMARY KEY,
    actor_user_id uuid REFERENCES users(id),
    action varchar(80) NOT NULL,
    metadata jsonb NOT NULL DEFAULT '{}',
    created_at timestamptz NOT NULL DEFAULT now()
);
CREATE INDEX audit_actor_time ON audit_logs(actor_user_id,created_at DESC);

-- Catálogo inicial do capítulo Floresta Encantada.
INSERT INTO item_templates (code,name,slot,rarity,base_stats,min_stage) VALUES
('forest_sword_common','Espada da Floresta','main_hand','common','{"attack":15}',1),
('forest_helm_common','Elmo da Floresta','head','common','{"defense":8,"hp":30}',1),
('forest_bow_uncommon','Arco do Batedor','main_hand','uncommon','{"attack":22,"crit_rate":0.01}',5),
('forest_staff_rare','Cajado Ancestral','main_hand','rare','{"attack":35,"crit_rate":0.02}',10),
('forest_armor_epic','Armadura do Troll','chest','epic','{"defense":55,"hp":180}',20),
('forest_relic_legendary','Coração da Floresta','relic','legendary','{"attack":80,"hp":350}',40),
('forest_crown_mythic','Coroa do Troll Ancestral','head','mythic','{"attack":120,"defense":100,"hp":500}',50);
