-- Limites específicos do MVP: asas T1–T3 e montaria T1–T2.
ALTER TABLE cosmetic_progress ADD CONSTRAINT cosmetic_tier_mvp CHECK (
    (cosmetic_type = 'wings' AND tier BETWEEN 1 AND 3) OR
    (cosmetic_type = 'mount' AND tier BETWEEN 1 AND 2)
);
