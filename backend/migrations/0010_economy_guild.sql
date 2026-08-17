-- Fase 4: Runas/Sockets/Sets, Crafting/Fusão, Coleção, Trade P2P, Leilão, Guild War, Torneio

-- Runas e Sockets (Épico+ 1-4 sockets por slot/raridade)
CREATE TABLE IF NOT EXISTS runes (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    code varchar(40) NOT NULL UNIQUE,
    rune_type varchar(16) NOT NULL CHECK (rune_type IN ('ATK','DEF','HP','CRIT','SPD','elemental','luck')),
    bonus jsonb NOT NULL DEFAULT '{}',
    created_at timestamptz NOT NULL DEFAULT now()
);
INSERT INTO runes (code, rune_type, bonus) VALUES
('rune_atk_t1','ATK','{"attack":10}'),
('rune_def_t1','DEF','{"defense":12}'),
('rune_hp_t1','HP','{"hp":80}'),
('rune_crit_t1','CRIT','{"crit_rate":0.02}'),
('rune_spd_t1','SPD','{"attack_speed":0.05}'),
('rune_elemental_t1','elemental','{"elemental":5}'),
('rune_luck_t1','luck','{"luck":0.02}')
ON CONFLICT (code) DO NOTHING;

CREATE TABLE IF NOT EXISTS item_sockets (
    inventory_item_id uuid NOT NULL REFERENCES inventory_items(id) ON DELETE CASCADE,
    socket_index smallint NOT NULL CHECK (socket_index BETWEEN 1 AND 4),
    rune_id uuid REFERENCES runes(id) ON DELETE SET NULL,
    PRIMARY KEY (inventory_item_id, socket_index)
);

-- Set Bonus (2/4/6 peças)
CREATE TABLE IF NOT EXISTS item_sets (
    code varchar(40) PRIMARY KEY,
    name varchar(80) NOT NULL,
    pieces integer[] NOT NULL
);
CREATE TABLE IF NOT EXISTS set_bonuses (
    set_code varchar(40) NOT NULL REFERENCES item_sets(code),
    required_pieces smallint NOT NULL CHECK (required_pieces IN (2,4,6)),
    bonus jsonb NOT NULL,
    PRIMARY KEY (set_code, required_pieces)
);
INSERT INTO item_sets (code, name, pieces) VALUES
('forest','Set Floresta', ARRAY[2,4,6]),
('desert','Set Deserto', ARRAY[2,4,6]),
('abyss','Set Abismo', ARRAY[2,4,6])
ON CONFLICT (code) DO NOTHING;
INSERT INTO set_bonuses (set_code, required_pieces, bonus) VALUES
('forest',2,'{"defense":15}'),('forest',4,'{"attack":25,"hp":100}'),('forest',6,'{"crit_rate":0.05,"glow":true}'),
('desert',2,'{"attack":20}'),('desert',4,'{"defense":30}'),('desert',6,'{"penetration":0.08}'),
('abyss',2,'{"crit_damage":0.1}'),('abyss',4,'{"attack":40}'),('abyss',6,'{"lifesteal":0.1}')
ON CONFLICT DO NOTHING;

-- Crafting/Fusão: 3× Comum→Incomum etc, 5× Mítico→Divino
CREATE TABLE IF NOT EXISTS craft_recipes (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    result_code varchar(80) NOT NULL REFERENCES item_templates(code),
    materials jsonb NOT NULL,
    gold_cost bigint NOT NULL DEFAULT 100 CHECK (gold_cost >= 0)
);
INSERT INTO craft_recipes (result_code, materials, gold_cost) VALUES
('forest_bow_uncommon','{"forest_sword_common":3}',50),
('forest_staff_rare','{"forest_bow_uncommon":3}',150),
('forest_armor_epic','{"forest_staff_rare":3}',500),
('forest_relic_legendary','{"forest_armor_epic":3}',1500)
ON CONFLICT DO NOTHING;

CREATE TABLE IF NOT EXISTS craft_history (
    id bigserial PRIMARY KEY,
    user_id uuid NOT NULL REFERENCES users(id),
    recipe_id uuid NOT NULL REFERENCES craft_recipes(id),
    created_at timestamptz NOT NULL DEFAULT now()
);

-- Coleção Bônus (ter X itens de um set = bônus global)
CREATE TABLE IF NOT EXISTS collections (
    user_id uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    collection_type varchar(32) NOT NULL,
    progress integer NOT NULL DEFAULT 0,
    claimed boolean NOT NULL DEFAULT false,
    PRIMARY KEY (user_id, collection_type)
);

-- Trade P2P direto (60s confirmação, taxa Diamantes)
CREATE TABLE IF NOT EXISTS trades (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    from_user_id uuid NOT NULL REFERENCES users(id),
    to_user_id uuid NOT NULL REFERENCES users(id),
    status varchar(16) NOT NULL DEFAULT 'pending' CHECK (status IN ('pending','accepted','cancelled','completed')),
    from_diamonds bigint NOT NULL DEFAULT 0 CHECK (from_diamonds >= 0),
    to_diamonds bigint NOT NULL DEFAULT 0 CHECK (to_diamonds >= 0),
    created_at timestamptz NOT NULL DEFAULT now(),
    expires_at timestamptz NOT NULL DEFAULT now() + interval '60 seconds',
    CHECK (from_user_id <> to_user_id)
);
CREATE TABLE IF NOT EXISTS trade_items (
    trade_id uuid NOT NULL REFERENCES trades(id) ON DELETE CASCADE,
    user_id uuid NOT NULL REFERENCES users(id),
    inventory_item_id uuid NOT NULL REFERENCES inventory_items(id),
    PRIMARY KEY (trade_id, inventory_item_id)
);

-- Leilão (Lendário+ 6/12/24h, Primordial 48h, anti-snipe 30min)
CREATE TABLE IF NOT EXISTS auctions (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    seller_user_id uuid NOT NULL REFERENCES users(id),
    inventory_item_id uuid NOT NULL REFERENCES inventory_items(id),
    start_price bigint NOT NULL CHECK (start_price > 0),
    current_price bigint NOT NULL CHECK (current_price >= start_price),
    current_winner uuid REFERENCES users(id),
    ends_at timestamptz NOT NULL,
    status varchar(16) NOT NULL DEFAULT 'active' CHECK (status IN ('active','sold','cancelled')),
    created_at timestamptz NOT NULL DEFAULT now()
);
CREATE TABLE IF NOT EXISTS auction_bids (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    auction_id uuid NOT NULL REFERENCES auctions(id) ON DELETE CASCADE,
    bidder_user_id uuid NOT NULL REFERENCES users(id),
    amount bigint NOT NULL CHECK (amount > 0),
    created_at timestamptz NOT NULL DEFAULT now()
);
CREATE INDEX IF NOT EXISTS auction_active_ends ON auctions(status, ends_at) WHERE status='active';

-- Guild War (GvG semanal) e Territory
CREATE TABLE IF NOT EXISTS guild_wars (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    guild_a uuid NOT NULL REFERENCES guilds(id),
    guild_b uuid NOT NULL REFERENCES guilds(id),
    score_a integer NOT NULL DEFAULT 0,
    score_b integer NOT NULL DEFAULT 0,
    winner uuid REFERENCES guilds(id),
    week_start date NOT NULL DEFAULT date_trunc('week', now())::date,
    created_at timestamptz NOT NULL DEFAULT now(),
    CHECK (guild_a <> guild_b)
);
CREATE TABLE IF NOT EXISTS guild_territories (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    name varchar(40) NOT NULL UNIQUE,
    owner_guild uuid REFERENCES guilds(id) ON DELETE SET NULL,
    buff jsonb NOT NULL DEFAULT '{"attack":5}'
);

INSERT INTO guild_territories (name, buff) VALUES
('Floresta Central','{"attack":5}'),
('Deserto Dourado','{"defense":8}'),
('Pico Gelado','{"crit_rate":0.02}')
ON CONFLICT (name) DO NOTHING;

-- Torneio semanal bracket 32 (Quinta)
CREATE TABLE IF NOT EXISTS tournaments (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    name varchar(80) NOT NULL DEFAULT 'Torneio Semanal',
    week_start date NOT NULL DEFAULT date_trunc('week', now())::date,
    status varchar(16) NOT NULL DEFAULT 'registration' CHECK (status IN ('registration','running','finished')),
    created_at timestamptz NOT NULL DEFAULT now()
);
CREATE TABLE IF NOT EXISTS tournament_participants (
    tournament_id uuid NOT NULL REFERENCES tournaments(id) ON DELETE CASCADE,
    user_id uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    seed integer NOT NULL,
    PRIMARY KEY (tournament_id, user_id)
);
CREATE TABLE IF NOT EXISTS tournament_matches (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    tournament_id uuid NOT NULL REFERENCES tournaments(id) ON DELETE CASCADE,
    round smallint NOT NULL CHECK (round BETWEEN 1 AND 5),
    player_a uuid REFERENCES users(id),
    player_b uuid REFERENCES users(id),
    winner uuid REFERENCES users(id),
    created_at timestamptz NOT NULL DEFAULT now()
);

-- Admin/GM logs e métricas
CREATE TABLE IF NOT EXISTS admin_logs (
    id bigserial PRIMARY KEY,
    admin_user_id uuid REFERENCES users(id),
    action varchar(80) NOT NULL,
    target_user_id uuid REFERENCES users(id),
    metadata jsonb NOT NULL DEFAULT '{}',
    created_at timestamptz NOT NULL DEFAULT now()
);
