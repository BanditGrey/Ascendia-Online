-- Ilha 17: Eternidade Dourada (801-850) — Conteúdo pós-800
-- Requer Labirinto 800 + 26000 Gold + VIP 14 + Despertar 4

INSERT INTO item_templates (code,name,slot,rarity,base_stats,min_stage,tier) VALUES
('eternity_blade_primordial','Lâmina da Eternidade','main_hand','primordial','{"attack":450,"crit_rate":0.10}',801,8),
('eternity_armor_primordial','Armadura da Eternidade','chest','primordial','{"defense":420,"hp":2700}',815,8),
('eternity_crown_primordial','Coroa da Eternidade','head','primordial','{"attack":500,"defense":440,"hp":3000}',850,8)
ON CONFLICT (code) DO NOTHING;

INSERT INTO cosmetic_skins (cosmetic_type, tier, skin_code, name, tradable) VALUES
('wings',8,'wings_t8_eternity','Asas da Eternidade — Ilha 17', true),
('frame',8,'frame_t8_eternity','Frame da Eternidade — Ilha 17', true)
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
  IF p_island='eternity' THEN SELECT COALESCE((SELECT unlocked FROM island_progress WHERE user_id=p_user_id AND island_code='time_labyrinth'), false) INTO v_prev; RETURN v_prev AND COALESCE(v_max,0) >= 800 AND COALESCE(v_gold,0) >= 26000 AND COALESCE(v_vip,0) >= 14 AND COALESCE(v_awak,0) >= 4; END IF;
  RETURN false;
END; $$ LANGUAGE plpgsql;
