-- Fase Final: Enchant, Raid Boss, Eventos Sazonais, Observabilidade

-- Enchant: reroll stats secundários com Scroll, pode travar stats
CREATE TABLE IF NOT EXISTS enchant_scrolls (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    quantity integer NOT NULL DEFAULT 1 CHECK (quantity > 0)
);
CREATE TABLE IF NOT EXISTS item_enchant_locks (
    inventory_item_id uuid NOT NULL REFERENCES inventory_items(id) ON DELETE CASCADE,
    stat_key varchar(32) NOT NULL,
    PRIMARY KEY (inventory_item_id, stat_key)
);

-- Raid Boss cooperativo (2×/semana Seg/Qui, HP enorme, ranking DPS guilda)
CREATE TABLE IF NOT EXISTS raid_bosses (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    name varchar(80) NOT NULL DEFAULT 'Raid Boss Semanal',
    hp bigint NOT NULL DEFAULT 50000000,
    max_hp bigint NOT NULL DEFAULT 50000000,
    week_start date NOT NULL DEFAULT date_trunc('week', now())::date,
    status varchar(16) NOT NULL DEFAULT 'active' CHECK (status IN ('active','defeated')),
    created_at timestamptz NOT NULL DEFAULT now()
);
INSERT INTO raid_bosses (name, hp, max_hp) VALUES ('Dragão Ancião — Raid Semanal', 50000000, 50000000) ON CONFLICT DO NOTHING;

CREATE TABLE IF NOT EXISTS raid_hits (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    raid_id uuid NOT NULL REFERENCES raid_bosses(id) ON DELETE CASCADE,
    user_id uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    guild_id uuid REFERENCES guilds(id),
    damage bigint NOT NULL CHECK (damage > 0),
    created_at timestamptz NOT NULL DEFAULT now()
);
CREATE INDEX IF NOT EXISTS raid_hits_raid_damage ON raid_hits(raid_id, damage DESC);

-- Evento Sazonal (ex: Inferno, fases especiais, moeda própria, shop)
CREATE TABLE IF NOT EXISTS seasonal_events (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    code varchar(40) NOT NULL UNIQUE,
    name varchar(80) NOT NULL,
    currency varchar(32) NOT NULL DEFAULT 'inferno_coin',
    starts_at timestamptz NOT NULL DEFAULT now(),
    ends_at timestamptz NOT NULL DEFAULT now() + interval '14 days',
    is_active boolean NOT NULL DEFAULT true
);
INSERT INTO seasonal_events (code, name, currency) VALUES
('inferno_2026','Evento Inferno — 14 dias','inferno_coin'),
('celestial_2026','Evento Celestial','celestial_feather')
ON CONFLICT (code) DO NOTHING;

CREATE TABLE IF NOT EXISTS event_progress (
    user_id uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    event_id uuid NOT NULL REFERENCES seasonal_events(id) ON DELETE CASCADE,
    currency_amount bigint NOT NULL DEFAULT 0 CHECK (currency_amount >= 0),
    updated_at timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (user_id, event_id)
);
CREATE TABLE IF NOT EXISTS event_shop_items (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    event_id uuid NOT NULL REFERENCES seasonal_events(id) ON DELETE CASCADE,
    item_code varchar(80) NOT NULL,
    cost bigint NOT NULL CHECK (cost > 0),
    stock integer NOT NULL DEFAULT -1
);
INSERT INTO event_shop_items (event_id, item_code, cost) VALUES
((SELECT id FROM seasonal_events WHERE code='inferno_2026'), 'wings_t2_skin_a', 500),
((SELECT id FROM seasonal_events WHERE code='inferno_2026'), 'mount_t1_skin_a', 300)
ON CONFLICT DO NOTHING;

-- Observabilidade: tracing/metrics já tem metrics_counters, adiciona logs estruturados
CREATE TABLE IF NOT EXISTS trace_logs (
    id bigserial PRIMARY KEY,
    trace_id uuid NOT NULL DEFAULT gen_random_uuid(),
    span varchar(80) NOT NULL,
    duration_ms integer NOT NULL,
    metadata jsonb NOT NULL DEFAULT '{}',
    created_at timestamptz NOT NULL DEFAULT now()
);
