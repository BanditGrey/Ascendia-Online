-- Ilha 12: Reino Dourado (551-600) — Ilha premium pós-550
-- Requer Ilha 11 completa (550) + 8000 Gold + VIP 5

-- Expande stages para 600 já feito, apenas adiciona templates
INSERT INTO item_templates (code,name,slot,rarity,base_stats,min_stage,tier) VALUES
('golden_blade_mythic','Lâmina Dourada','main_hand','mythic','{"attack":210,"crit_rate":0.06}',551,8),
('golden_armor_divine','Armadura do Rei Dourado','chest','divine','{"defense":200,"hp":1100}',565,8),
('golden_crown_primordial','Coroa Dourada Suprema','head','primordial','{"attack":300,"defense":240,"hp":1600}',600,8)
ON CONFLICT (code) DO NOTHING;

INSERT INTO cosmetic_skins (cosmetic_type, tier, skin_code, name, tradable) VALUES
('wings',8,'wings_t8_golden_kingdom','Asas Douradas — Reino Dourado', true),
('mount',8,'mount_t8_golden_phoenix','Fênix Dourada — Reino Dourado', true),
('frame',8,'frame_t8_golden','Frame Dourado — Reino Dourado', true)
ON CONFLICT (skin_code) DO NOTHING;

-- Ilha 12 usa mesma island_progress com code golden_kingdom
-- Função atualizada para 600
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
  RETURN false;
END; $$ LANGUAGE plpgsql;
