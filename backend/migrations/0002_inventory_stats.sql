-- Separa stats base dos stats calculados para impedir acumulação a cada recálculo.
CREATE TABLE character_base_stats (
    character_id uuid PRIMARY KEY REFERENCES characters(id) ON DELETE CASCADE,
    hp bigint NOT NULL DEFAULT 1000 CHECK (hp > 0),
    attack bigint NOT NULL DEFAULT 150 CHECK (attack > 0),
    defense bigint NOT NULL DEFAULT 100 CHECK (defense >= 0),
    attack_speed double precision NOT NULL DEFAULT 1.2 CHECK (attack_speed > 0),
    crit_rate double precision NOT NULL DEFAULT .05 CHECK (crit_rate BETWEEN 0 AND 1),
    crit_damage double precision NOT NULL DEFAULT 1.5 CHECK (crit_damage >= 1),
    luck double precision NOT NULL DEFAULT 0 CHECK (luck BETWEEN 0 AND 1),
    accuracy double precision NOT NULL DEFAULT 0 CHECK (accuracy BETWEEN 0 AND 1),
    dodge double precision NOT NULL DEFAULT 0 CHECK (dodge BETWEEN 0 AND 1),
    penetration double precision NOT NULL DEFAULT 0 CHECK (penetration BETWEEN 0 AND 1)
);
INSERT INTO character_base_stats
SELECT character_id,hp,attack,defense,attack_speed,crit_rate,crit_damage,luck,accuracy,dodge,penetration
FROM character_stats;

ALTER TABLE item_templates ADD COLUMN tier smallint NOT NULL DEFAULT 1 CHECK (tier BETWEEN 1 AND 8);

CREATE TABLE player_materials (
    user_id uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    material_code varchar(80) NOT NULL,
    quantity bigint NOT NULL DEFAULT 0 CHECK (quantity >= 0),
    updated_at timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (user_id,material_code)
);

CREATE INDEX equipment_character ON equipment_slots(character_id);
