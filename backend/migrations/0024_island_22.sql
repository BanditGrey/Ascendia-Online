-- Ilha 22: Pesadelo Lúcido (1051-1100) — Mecânica Pesadelo (200% dano recebido, 300% loot)
-- Requer Sonho 1050 + 50000 Gold + VIP 15 + Despertar 5 + Power 150000

INSERT INTO item_templates (code,name,slot,rarity,base_stats,min_stage,tier) VALUES
('nightmare_blade_primordial','Lâmina do Pesadelo','main_hand','primordial','{"attack":700,"crit_rate":0.18}',1051,8),
('nightmare_armor_primordial','Armadura do Pesadelo','chest','primordial','{"defense":700,"hp":7000}',1065,8),
('nightmare_crown_primordial','Coroa do Pesadelo','head','primordial','{"attack":800,"defense":700,"hp":8000}',1100,8)
ON CONFLICT (code) DO NOTHING;

INSERT INTO cosmetic_skins (cosmetic_type, tier, skin_code, name, tradable) VALUES
('wings',8,'wings_t8_nightmare','Asas do Pesadelo — Ilha 22', true)
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
  RETURN false;
END; $$ LANGUAGE plpgsql;
