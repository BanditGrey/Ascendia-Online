-- Fase 3: Skills, Awakening, Torre, Arena, Dungeon, Amigos, Quests
-- Skills: 1 ponto por level, 3 branches, reset com item

CREATE TABLE IF NOT EXISTS skill_trees (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    class character_class NOT NULL,
    subclass varchar(32) NOT NULL,
    branch varchar(16) NOT NULL CHECK (branch IN ('offensive','defensive','utility')),
    skill_code varchar(40) NOT NULL,
    max_level smallint NOT NULL DEFAULT 5 CHECK (max_level BETWEEN 1 AND 10),
    UNIQUE (class, subclass, skill_code)
);

-- Semente de skills para cada subclasse (exemplo por classe)
INSERT INTO skill_trees (class, subclass, branch, skill_code, max_level) VALUES
-- Guerreiro
('warrior','guardian','defensive','shield_wall',5),
('warrior','guardian','defensive','taunt',5),
('warrior','berserker','offensive','rage_mode',5),
('warrior','berserker','offensive','aoe_melee',5),
('warrior','paladin','utility','holy_heal',5),
-- Arqueiro
('archer','marksman','offensive','headshot',5),
('archer','crossbowman','offensive','armor_break',5),
('archer','ranger','utility','poison_trap',5),
-- Mago
('mage','elementalista','offensive','elemental_burst',5),
('mage','necromante','utility','summon_undead',5),
('mage','arcano','offensive','mana_burst',5),
-- Assassino
('assassin','sombra','utility','shadow_step',5),
('assassin','ninja','utility','smoke_bomb',5),
('assassin','lamina_dupla','offensive','bleed_stack',5),
-- Suporte
('support','curandeiro','utility','mass_heal',5),
('support','buffador','utility','haste_buff',5),
('support','xama','utility','totem_heal_buff',5),
-- Comandante
('commander','emperor','utility','imperial_buff',5),
('commander','war_lord','offensive','war_cry',5),
('commander','strategist','utility','tactical_cdr',5)
ON CONFLICT DO NOTHING;

CREATE TABLE IF NOT EXISTS character_skills (
    character_id uuid NOT NULL REFERENCES characters(id) ON DELETE CASCADE,
    skill_code varchar(40) NOT NULL,
    level smallint NOT NULL DEFAULT 0 CHECK (level BETWEEN 0 AND 10),
    PRIMARY KEY (character_id, skill_code)
);

CREATE TABLE IF NOT EXISTS character_skill_points (
    character_id uuid PRIMARY KEY REFERENCES characters(id) ON DELETE CASCADE,
    available smallint NOT NULL DEFAULT 0 CHECK (available >= 0),
    total_earned smallint NOT NULL DEFAULT 0 CHECK (total_earned >= 0),
    updated_at timestamptz NOT NULL DEFAULT now()
);

-- Awakening já existe (characters.awakening 0-5), apenas garantir log
CREATE TABLE IF NOT EXISTS awakening_logs (
    id bigserial PRIMARY KEY,
    user_id uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    character_id uuid NOT NULL REFERENCES characters(id) ON DELETE CASCADE,
    from_level smallint NOT NULL,
    to_awakening smallint NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now()
);

-- Torre Infinita
CREATE TABLE IF NOT EXISTS tower_progress (
    user_id uuid PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    current_floor integer NOT NULL DEFAULT 0 CHECK (current_floor >= 0),
    best_floor integer NOT NULL DEFAULT 0 CHECK (best_floor >= 0),
    updated_at timestamptz NOT NULL DEFAULT now()
);

-- Arena PvP (5/dia VIP 20, tiers Bronze→Primordial)
CREATE TYPE arena_tier AS ENUM ('bronze','prata','ouro','platina','diamante','mestre','lenda','divino','primordial');
CREATE TABLE IF NOT EXISTS arena_status (
    user_id uuid PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    tier arena_tier NOT NULL DEFAULT 'bronze',
    rating integer NOT NULL DEFAULT 1000 CHECK (rating >= 0),
    wins integer NOT NULL DEFAULT 0 CHECK (wins >= 0),
    losses integer NOT NULL DEFAULT 0 CHECK (losses >= 0),
    daily_fights integer NOT NULL DEFAULT 0 CHECK (daily_fights >= 0),
    last_reset date NOT NULL DEFAULT CURRENT_DATE,
    updated_at timestamptz NOT NULL DEFAULT now()
);
CREATE TABLE IF NOT EXISTS arena_matches (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    attacker_user_id uuid NOT NULL REFERENCES users(id),
    defender_user_id uuid NOT NULL REFERENCES users(id),
    winner_user_id uuid REFERENCES users(id),
    attacker_power bigint NOT NULL,
    defender_power bigint NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now()
);

-- Dungeon diária (3 tipos, 3/dia VIP10, reset 00:00 UTC)
CREATE TYPE dungeon_type AS ENUM ('exp','material','equipment');
CREATE TABLE IF NOT EXISTS dungeon_runs (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    dungeon_type dungeon_type NOT NULL,
    cleared boolean NOT NULL DEFAULT true,
    run_date date NOT NULL DEFAULT CURRENT_DATE,
    created_at timestamptz NOT NULL DEFAULT now()
);
CREATE INDEX IF NOT EXISTS dungeon_user_date ON dungeon_runs(user_id, run_date);

-- Raid Boss cooperativo (2×/semana Seg/Qui) e World Boss 6h
CREATE TABLE IF NOT EXISTS raid_contributions (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    guild_id uuid REFERENCES guilds(id) ON DELETE SET NULL,
    damage bigint NOT NULL DEFAULT 0 CHECK (damage >= 0),
    week_start date NOT NULL DEFAULT date_trunc('week', now())::date,
    created_at timestamptz NOT NULL DEFAULT now()
);
CREATE TABLE IF NOT EXISTS world_boss_state (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    boss_name varchar(80) NOT NULL DEFAULT 'World Boss',
    hp bigint NOT NULL DEFAULT 10000000,
    max_hp bigint NOT NULL DEFAULT 10000000,
    spawns_at timestamptz NOT NULL DEFAULT now(),
    expires_at timestamptz NOT NULL DEFAULT now() + interval '6 hours'
);
INSERT INTO world_boss_state (boss_name, hp, max_hp) VALUES ('Colosso Primordial', 15000000, 15000000) ON CONFLICT DO NOTHING;

-- Expedição (2h,4h,8h,12h,24h, 3 slots VIP8)
CREATE TYPE expedition_duration AS ENUM ('2h','4h','8h','12h','24h');
CREATE TABLE IF NOT EXISTS expeditions (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    character_id uuid NOT NULL REFERENCES characters(id) ON DELETE CASCADE,
    duration expedition_duration NOT NULL,
    started_at timestamptz NOT NULL DEFAULT now(),
    ends_at timestamptz NOT NULL,
    claimed boolean NOT NULL DEFAULT false
);

-- Amigos (max 100)
CREATE TABLE IF NOT EXISTS friendships (
    user_id uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    friend_user_id uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    created_at timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (user_id, friend_user_id),
    CHECK (user_id <> friend_user_id)
);
CREATE TABLE IF NOT EXISTS friend_requests (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    from_user_id uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    to_user_id uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    status varchar(16) NOT NULL DEFAULT 'pending' CHECK (status IN ('pending','accepted','rejected')),
    created_at timestamptz NOT NULL DEFAULT now(),
    UNIQUE (from_user_id, to_user_id)
);

-- Quests diárias/semanais e Achievements
CREATE TABLE IF NOT EXISTS daily_quests (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    quest_code varchar(40) NOT NULL,
    progress integer NOT NULL DEFAULT 0,
    target integer NOT NULL DEFAULT 1,
    claimed boolean NOT NULL DEFAULT false,
    quest_date date NOT NULL DEFAULT CURRENT_DATE,
    UNIQUE (user_id, quest_code, quest_date)
);
CREATE TABLE IF NOT EXISTS weekly_quests (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    quest_code varchar(40) NOT NULL,
    progress integer NOT NULL DEFAULT 0,
    target integer NOT NULL DEFAULT 5,
    claimed boolean NOT NULL DEFAULT false,
    week_start date NOT NULL DEFAULT date_trunc('week', now())::date,
    UNIQUE (user_id, quest_code, week_start)
);
CREATE TABLE IF NOT EXISTS achievements (
    code varchar(40) PRIMARY KEY,
    category varchar(32) NOT NULL,
    name varchar(80) NOT NULL,
    target integer NOT NULL
);
INSERT INTO achievements (code, category, name, target) VALUES
('combat_100','Combate','100 Fases Completas',100),
('tower_100','Torre','Torre Andar 100',100),
('arena_50','PvP','50 Vitórias Arena',50),
('collector_wings','Coleção','Asas T8',1),
('guild_raid','Raid','10 Raids',10)
ON CONFLICT DO NOTHING;
CREATE TABLE IF NOT EXISTS player_achievements (
    user_id uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    code varchar(40) NOT NULL REFERENCES achievements(code),
    progress integer NOT NULL DEFAULT 0,
    claimed boolean NOT NULL DEFAULT false,
    PRIMARY KEY (user_id, code)
);

-- Login diário 28 dias
CREATE TABLE IF NOT EXISTS login_streak (
    user_id uuid PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    streak integer NOT NULL DEFAULT 0 CHECK (streak >= 0),
    last_login date NOT NULL DEFAULT CURRENT_DATE,
    updated_at timestamptz NOT NULL DEFAULT now()
);
