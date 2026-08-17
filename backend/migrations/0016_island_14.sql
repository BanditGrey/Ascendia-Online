-- Ilha 14: Eclipse Eterno (651-700) — Ilha final pré-launch
-- Requer Vazio 650 + 15000 Gold + VIP 10 + Despertar 1

INSERT INTO item_templates (code,name,slot,rarity,base_stats,min_stage,tier) VALUES
('eclipse_blade_primordial','Lâmina do Eclipse','main_hand','primordial','{"attack":350,"crit_rate":0.07}',651,8),
('eclipse_armor_primordial','Armadura do Eclipse','chest','primordial','{"defense":300,"hp":1800}',665,8),
('eclipse_crown_primordial','Coroa do Eclipse Eterno','head','primordial','{"attack":380,"defense":320,"hp":2000}',700,8)
ON CONFLICT (code) DO NOTHING;

INSERT INTO cosmetic_skins (cosmetic_type, tier, skin_code, name, tradable) VALUES
('wings',8,'wings_t8_eclipse','Asas do Eclipse — Ilha 14', true),
('aura',8,'aura_t8_eclipse','Aura do Eclipse — Ilha 14', true)
ON CONFLICT (skin_code) DO NOTHING;

CREATE OR REPLACE FUNCTION can_unlock_island(p_user_id uuid, p_island varchar) RETURNS boolean AS $$
DECLARE v_max smallint; v_gold bigint; v_vip smallint; v_prev boolean; v_awak smallint;
BEGIN
  SELECT max_stage INTO v_max FROM stage_progress WHERE user_id=p_user_id;
  SELECT gold INTO v_gold FROM users WHERE id=p_user_id;
  SELECT vip_level INTO v_vip FROM users WHERE id=p_user_id;
  SELECT COALESCE((SELECT MAX(awakening) FROM characters WHERE user_id=p_user_id),0) INTO v_awak;
  IF p_island='abyss_island' THEN RETURN COALESCE(v_max,0) >= 500 AND COALESCE(v_gold,0) >= 5000; END IF;
  IF p_island='golden_kingdom' THEN
    SELECT COALESCE((SELECT unlocked FROM island_progress WHERE user_id=p_user_id AND island_code='abyss_island'), false) INTO v_prev;
    RETURN v_prev AND COALESCE(v_max,0) >= 550 AND COALESCE(v_gold,0) >= 8000 AND COALESCE(v_vip,0) >= 5;
  END IF;
  IF p_island='void_star' THEN
    SELECT COALESCE((SELECT unlocked FROM island_progress WHERE user_id=p_user_id AND island_code='golden_kingdom'), false) INTO v_prev;
    RETURN v_prev AND COALESCE(v_max,0) >= 600 AND COALESCE(v_gold,0) >= 12000 AND COALESCE(v_vip,0) >= 8;
  END IF;
  IF p_island='eclipse' THEN
    SELECT COALESCE((SELECT unlocked FROM island_progress WHERE user_id=p_user_id AND island_code='void_star'), false) INTO v_prev;
    RETURN v_prev AND COALESCE(v_max,0) >= 650 AND COALESCE(v_gold,0) >= 15000 AND COALESCE(v_vip,0) >= 10 AND COALESCE(v_awak,0) >= 1;
  END IF;
  RETURN false;
END; $$ LANGUAGE plpgsql;
