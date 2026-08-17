ALTER TABLE squads ADD COLUMN formation varchar(16) NOT NULL DEFAULT 'balanced'
    CHECK (formation IN ('balanced', 'vanguard', 'assault'));
