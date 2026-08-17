-- Ilha 16: Labirinto do Tempo (751-800) — Conteúdo pós-750
-- Requer Tempestade 750 + 22000 Gold + VIP 13 + Despertar 3

INSERT INTO item_templates (code,name,slot,rarity,base_stats,min_stage,tier) VALUES
('time_blade_primordial','Lâmina do Tempo','main_hand','primordial','{"attack":410,"crit_rate":0.09}',751,8),
('time_armor_primordial','Armadura Temporal','chest','primordial','{"defense":380,"hp":2400}',765,8),
('time_crown_primordial','Coroa do Tempo Eterno','head','primordial','{"attack":460,"defense":400,"hp":2600}',800,8)
ON CONFLICT (code) DO NOTHING;

INSERT INTO cosmetic_skins (cosmetic_type, tier, skin_code, name, tradable) VALUES
('wings',8,'wings_t8_time','Asas do Tempo — Ilha 16', true),
('trail',8,'trail_t8_time','Rastro Temporal — Ilha 16', true)
ON CONFLICT (skin_code) DO NOTHING;

CREATE OR REPLACE FUNCTION can_unlock_island(p_user_id uuid, p_island varchar) RETURNS boolean AS $$
DECLARE v_max smallint; v_gold bigint; v_vip smallint; v_prev boolean; v_awak smallint;
BEGIN
  SELECT max_stage INTO v_max FROM stage_progress WHERE user_id=p_user_id;
  SELECT gold INTO v_gold FROM users WHERE id=p_user_id;
  SELECT vip_level INTO v_vip FROM users WHERE id=p_user_id;
  SELECT COALESCE((SELECT MAX(awakening) FROM characters WHERE user_id=p_user_id),0) INTO v_awak;
  IF p_island='abyss_island' THEN RETURN COALESCE(v_max,0) >= 500 AND COALESCE(v_gold,0) >= 5000; END IF;
  IF p_island='golden_kingdom' THEN SELECT COALESCE((SELECT unlocked FROM island_progress WHERE user_id=p_user_id AND island_code='abyss_island'), false) INTO v_prev; RETURN v_prev AND COALESCE(v_max,0) >= 550 AND COALESCE(v_gold,0) >= 8000 AND COALESCE(v_vip,0) >= 5; END IF;
  IF p_island='void_star' THEN SELECT COALESCE((SELECT unlocked FROM island_progress WHERE user_id=p_user_id AND island_code='golden_kingdom'), false) INTO v_prev; RETURN v_prev AND COALESCE(v_max,0) >= 600 AND COALESCE(v_gold,0) >= 12000 AND COALESCE(v_vip,0) >= 8; END IF;
  IF p_island='eclipse' THEN SELECT COALESCE((SELECT unlocked FROM island_progress WHERE user_id=p_user_id AND island_code='void_star'), false) INTO v_prev; RETURN v_prev AND COALESCE(v_max,0) >= 650 AND COALESCE(v_gold,0) >= 15000 AND COALESCE(v_vip,0) >= 10 AND COALESCE(v_awak,0) >= 1; END IF;
  IF p_island='storm' THEN SELECT COALESCE((SELECT unlocked FROM island_progress WHERE user_id=p_user_id AND island_code='eclipse'), false) INTO v_prev; RETURN v_prev AND COALESCE(v_max,0) >= 700 AND COALESCE(v_gold,0) >= 18000 AND COALESCE(v_vip,0) >= 12 AND COALESCE(v_awak,0) >= 2; END IF;
  IF p_island='time_labyrinth' THEN SELECT COALESCE((SELECT unlocked FROM island_progress WHERE user_id=p_user_id AND island_code='storm'), false) INTO v_prev; RETURN v_prev AND COALESCE(v_max,0) >= 750 AND COALESCE(v_gold,0) >= 22000 AND COALESCE(v_vip,0) >= 13 AND COALESCE(v_awak,0) >= 3; END IF;
  RETURN false;
END; $$ LANGUAGE plpgsql;
