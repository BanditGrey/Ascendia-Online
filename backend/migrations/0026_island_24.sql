-- Ilha 24: Luz Sombria (1151-1200) — Pós-Pesadelo Profundo
-- Requer Pesadelo Profundo 1150 + 60000 Gold + VIP 15 + Despertar 5 + Power 200000

INSERT INTO item_templates (code,name,slot,rarity,base_stats,min_stage,tier) VALUES
('shadow_light_blade_primordial','Lâmina da Luz Sombria','main_hand','primordial','{"attack":800,"crit_rate":0.20}',1151,8),
('shadow_light_armor_primordial','Armadura da Luz Sombria','chest','primordial','{"defense":800,"hp":9000}',1165,8),
('shadow_light_crown_primordial','Coroa da Luz Sombria','head','primordial','{"attack":900,"defense":800,"hp":10000}',1200,8)
ON CONFLICT (code) DO NOTHING;

INSERT INTO cosmetic_skins (cosmetic_type, tier, skin_code, name, tradable) VALUES
('wings',8,'wings_t8_shadow_light','Asas da Luz Sombria — Ilha 24', true)
ON CONFLICT (skin_code) DO NOTHING;

CREATE OR REPLACE FUNCTION can_unlock_island(p_user_id uuid, p_island varchar) RETURNS boolean AS $$
DECLARE v_max smallint; v_gold bigint; v_vip smallint; v_prev boolean; v_awak smallint; v_power bigint;
BEGIN
  SELECT max_stage INTO v_max FROM stage_progress WHERE user_id=p_user_id;
  SELECT gold INTO v_gold FROM users WHERE id=p_user_id;
  SELECT vip_level INTO v_vip FROM users WHERE id=p_user_id;
  SELECT COALESCE((SELECT MAX(awakening) FROM characters WHERE user_id=p_user_id),0) INTO v_awak;
  SELECT COALESCE((SELECT MAX(power_rating) FROM character_stats cs JOIN characters c ON c.id=cs.character_id WHERE c.user_id=p_user_id),0) INTO v_power;
  IF p_island='abyss_island' THEN RETURN COALESCE(v_max,0) >= 500 AND COALESCE(v_gold,0) >= 5000; END IF;
  IF p_island='golden_kingdom' THEN SELECT COALESCE((SELECT unlocked FROM island_progress WHERE user_id=p_user_id AND island_code='abyss_island'), false) INTO v_prev; RETURN v_prev AND COALESCE(v_max,0) >= 550 AND COALESCE(v_gold,0) >= 8000 AND COALESCE(v_vip,0) >= 5; END IF;
  IF p_island='void_star' THEN SELECT COALESCE((SELECT unlocked FROM island_progress WHERE user_id=p_user_id AND island_code='golden_kingdom'), false) INTO v_prev; RETURN v_prev AND COALESCE(v_max,0) >= 600 AND COALESCE(v_gold,0) >= 12000 AND COALESCE(v_vip,0) >= 8; END IF;
  IF p_island='eclipse' THEN SELECT COALESCE((SELECT unlocked FROM island_progress WHERE user_id=p_user_id AND island_code='void_star'), false) INTO v_prev; RETURN v_prev AND COALESCE(v_max,0) >= 650 AND COALESCE(v_gold,0) >= 15000 AND COALESCE(v_vip,0) >= 10 AND COALESCE(v_awak,0) >= 1; END IF;
  IF p_island='storm' THEN SELECT COALESCE((SELECT unlocked FROM island_progress WHERE user_id=p_user_id AND island_code='eclipse'), false) INTO v_prev; RETURN v_prev AND COALESCE(v_max,0) >= 700 AND COALESCE(v_gold,0) >= 18000 AND COALESCE(v_vip,0) >= 12 AND COALESCE(v_awak,0) >= 2; END IF;
  IF p_island='time_labyrinth' THEN SELECT COALESCE((SELECT unlocked FROM island_progress WHERE user_id=p_user_id AND island_code='storm'), false) INTO v_prev; RETURN v_prev AND COALESCE(v_max,0) >= 750 AND COALESCE(v_gold,0) >= 22000 AND COALESCE(v_vip,0) >= 13 AND COALESCE(v_awak,0) >= 3; END IF;
  IF p_island='eternity' THEN SELECT COALESCE((SELECT unlocked FROM island_progress WHERE user_id=p_user_id AND island_code='time_labyrinth'), false) INTO v_prev; RETURN v_prev AND COALESCE(v_max,0) >= 800 AND COALESCE(v_gold,0) >= 26000 AND COALESCE(v_vip,0) >= 14 AND COALESCE(v_awak,0) >= 4; END IF;
  IF p_island='origin' THEN SELECT COALESCE((SELECT unlocked FROM island_progress WHERE user_id=p_user_id AND island_code='eternity'), false) INTO v_prev; RETURN v_prev AND COALESCE(v_max,0) >= 850 AND COALESCE(v_gold,0) >= 30000 AND COALESCE(v_vip,0) >= 15 AND COALESCE(v_awak,0) >= 5 AND COALESCE(v_power,0) >= 50000; END IF;
  IF p_island='final_abyss' THEN SELECT COALESCE((SELECT unlocked FROM island_progress WHERE user_id=p_user_id AND island_code='origin'), false) INTO v_prev; RETURN v_prev AND COALESCE(v_max,0) >= 900 AND COALESCE(v_gold,0) >= 35000 AND COALESCE(v_vip,0) >= 15 AND COALESCE(v_awak,0) >= 5 AND COALESCE(v_power,0) >= 75000; END IF;
  IF p_island='supreme_void' THEN SELECT COALESCE((SELECT unlocked FROM island_progress WHERE user_id=p_user_id AND island_code='final_abyss'), false) INTO v_prev; RETURN v_prev AND COALESCE(v_max,0) >= 950 AND COALESCE(v_gold,0) >= 40000 AND COALESCE(v_vip,0) >= 15 AND COALESCE(v_awak,0) >= 5 AND COALESCE(v_power,0) >= 100000; END IF;
  IF p_island='dream' THEN SELECT COALESCE((SELECT unlocked FROM island_progress WHERE user_id=p_user_id AND island_code='supreme_void'), false) INTO v_prev; RETURN v_prev AND COALESCE(v_max,0) >= 1000 AND COALESCE(v_gold,0) >= 45000 AND COALESCE(v_vip,0) >= 15 AND COALESCE(v_awak,0) >= 5 AND COALESCE(v_power,0) >= 120000; END IF;
  IF p_island='nightmare' THEN SELECT COALESCE((SELECT unlocked FROM island_progress WHERE user_id=p_user_id AND island_code='dream'), false) INTO v_prev; RETURN v_prev AND COALESCE(v_max,0) >= 1050 AND COALESCE(v_gold,0) >= 50000 AND COALESCE(v_vip,0) >= 15 AND COALESCE(v_awak,0) >= 5 AND COALESCE(v_power,0) >= 150000; END IF;
  IF p_island='deep_nightmare' THEN SELECT COALESCE((SELECT unlocked FROM island_progress WHERE user_id=p_user_id AND island_code='nightmare'), false) INTO v_prev; RETURN v_prev AND COALESCE(v_max,0) >= 1100 AND COALESCE(v_gold,0) >= 55000 AND COALESCE(v_vip,0) >= 15 AND COALESCE(v_awak,0) >= 5 AND COALESCE(v_power,0) >= 180000; END IF;
  IF p_island='shadow_light' THEN SELECT COALESCE((SELECT unlocked FROM island_progress WHERE user_id=p_user_id AND island_code='deep_nightmare'), false) INTO v_prev; RETURN v_prev AND COALESCE(v_max,0) >= 1150 AND COALESCE(v_gold,0) >= 60000 AND COALESCE(v_vip,0) >= 15 AND COALESCE(v_awak,0) >= 5 AND COALESCE(v_power,0) >= 200000; END IF;
  RETURN false;
END; $$ LANGUAGE plpgsql;
