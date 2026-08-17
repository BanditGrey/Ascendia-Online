-- Ilha 13: Vazio Estelar (601-650) + Evento Sazonal Dourado (14 dias)
-- Requer Reino Dourado completo (600) + 12000 Gold + VIP 8

INSERT INTO item_templates (code,name,slot,rarity,base_stats,min_stage,tier) VALUES
('void_blade_primordial','Lâmina do Vazio','main_hand','primordial','{"attack":320,"penetration":0.18}',601,8),
('void_armor_primordial','Armadura Estelar','chest','primordial','{"defense":260,"hp":1500}',615,8),
('void_crown_primordial','Coroa do Vazio','head','primordial','{"attack":340,"defense":280,"hp":1800}',650,8)
ON CONFLICT (code) DO NOTHING;

INSERT INTO cosmetic_skins (cosmetic_type, tier, skin_code, name, tradable) VALUES
('wings',8,'wings_t8_void_star','Asas do Vazio Estelar — Ilha 13', true),
('aura',8,'aura_t8_void','Aura do Vazio — Ilha 13', true)
ON CONFLICT (skin_code) DO NOTHING;

-- Evento Sazonal Dourado: 14 dias, moeda golden_feather, shop com skins ilha 12
INSERT INTO seasonal_events (code, name, currency, starts_at, ends_at, is_active) VALUES
('golden_festival_2026','Festival Dourado — Reino Dourado 14 dias','golden_feather', now(), now() + interval '14 days', true)
ON CONFLICT (code) DO NOTHING;

INSERT INTO event_shop_items (event_id, item_code, cost) VALUES
((SELECT id FROM seasonal_events WHERE code='golden_festival_2026'), 'wings_t8_golden_kingdom', 800),
((SELECT id FROM seasonal_events WHERE code='golden_festival_2026'), 'mount_t8_golden_phoenix', 600)
ON CONFLICT DO NOTHING;

-- Atualiza função para ilha 13
CREATE OR REPLACE FUNCTION can_unlock_island(p_user_id uuid, p_island varchar) RETURNS boolean AS $$
DECLARE v_max smallint; v_gold bigint; v_vip smallint; v_prev boolean;
BEGIN
  SELECT max_stage INTO v_max FROM stage_progress WHERE user_id=p_user_id;
  SELECT gold INTO v_gold FROM users WHERE id=p_user_id;
  SELECT vip_level INTO v_vip FROM users WHERE id=p_user_id;
  IF p_island='abyss_island' THEN RETURN COALESCE(v_max,0) >= 500 AND COALESCE(v_gold,0) >= 5000; END IF;
  IF p_island='golden_kingdom' THEN
    SELECT COALESCE((SELECT unlocked FROM island_progress WHERE user_id=p_user_id AND island_code='abyss_island'), false) INTO v_prev;
    RETURN v_prev AND COALESCE(v_max,0) >= 550 AND COALESCE(v_gold,0) >= 8000 AND COALESCE(v_vip,0) >= 5;
  END IF;
  IF p_island='void_star' THEN
    SELECT COALESCE((SELECT unlocked FROM island_progress WHERE user_id=p_user_id AND island_code='golden_kingdom'), false) INTO v_prev;
    RETURN v_prev AND COALESCE(v_max,0) >= 600 AND COALESCE(v_gold,0) >= 12000 AND COALESCE(v_vip,0) >= 8;
  END IF;
  RETURN false;
END; $$ LANGUAGE plpgsql;
