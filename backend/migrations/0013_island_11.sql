-- Ilha 11: Abismo Profundo (501-550) — Conteúdo pós-500
-- Nova ilha acessível via portal, com mobs/bosses exclusivos e loot lendário

-- Expande stages para 600
DO $$ DECLARE r RECORD; BEGIN
  FOR r IN SELECT conname FROM pg_constraint WHERE conrelid='combat_sessions'::regclass AND contype='c' LOOP
    EXECUTE 'ALTER TABLE combat_sessions DROP CONSTRAINT '||quote_ident(r.conname);
  END LOOP;
END $$;
ALTER TABLE combat_sessions ADD CONSTRAINT combat_sessions_stage_island CHECK (stage BETWEEN 1 AND 600);

DO $$ DECLARE r RECORD; BEGIN
  FOR r IN SELECT conname FROM pg_constraint WHERE conrelid='stage_stars'::regclass AND contype='c' LOOP
    EXECUTE 'ALTER TABLE stage_stars DROP CONSTRAINT '||quote_ident(r.conname);
  END LOOP;
END $$;
ALTER TABLE stage_stars ADD CONSTRAINT stage_stars_stage_island CHECK (stage BETWEEN 1 AND 600);

-- Novos templates para Ilha 11
INSERT INTO item_templates (code,name,slot,rarity,base_stats,min_stage,tier) VALUES
('abyss_island_blade_legendary','Lâmina do Abismo Profundo','main_hand','legendary','{"attack":185,"crit_rate":0.05}',501,8),
('abyss_island_armor_mythic','Armadura Abissal','chest','mythic','{"defense":180,"hp":900}',515,8),
('abyss_island_relic_divine','Coração do Abismo','relic','divine','{"attack":220,"defense":160,"hp":1100}',530,8),
('abyss_island_crown_primordial','Coroa do Leviatã','head','primordial','{"attack":280,"defense":220,"hp":1400}',550,8)
ON CONFLICT (code) DO NOTHING;

-- Cosmético exclusivo Ilha 11: Asas Abissais
INSERT INTO cosmetic_skins (cosmetic_type, tier, skin_code, name, tradable) VALUES
('wings',8,'wings_t8_island_abyss','Asas Abissais — Ilha 11', true),
('mount',8,'mount_t8_island_leviathan','Leviatã Abissal — Ilha 11', true),
('pet',8,'pet_t8_island_kraken','Kraken Bebê — Ilha 11', true)
ON CONFLICT (skin_code) DO NOTHING;

-- Ilha 11: tabela de progresso
CREATE TABLE IF NOT EXISTS island_progress (
    user_id uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    island_code varchar(32) NOT NULL DEFAULT 'abyss_island',
    max_stage smallint NOT NULL DEFAULT 500 CHECK (max_stage BETWEEN 500 AND 600),
    unlocked boolean NOT NULL DEFAULT false,
    unlocked_at timestamptz,
    PRIMARY KEY (user_id, island_code)
);

-- Portal para ilha: requer Cap.10 completo (fase 500) + 5000 Gold
CREATE OR REPLACE FUNCTION can_unlock_island(p_user_id uuid) RETURNS boolean AS $$
DECLARE v_max smallint; v_gold bigint;
BEGIN
  SELECT max_stage INTO v_max FROM stage_progress WHERE user_id=p_user_id;
  SELECT gold INTO v_gold FROM users WHERE id=p_user_id;
  RETURN COALESCE(v_max,0) >= 500 AND COALESCE(v_gold,0) >= 5000;
END; $$ LANGUAGE plpgsql;
