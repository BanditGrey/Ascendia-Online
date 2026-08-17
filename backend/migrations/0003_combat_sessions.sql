-- Sessões preservam o snapshot e o resultado de waves para replay/auditoria.
CREATE TYPE combat_session_status AS ENUM ('resolved', 'abandoned');

CREATE TABLE combat_sessions (
    id uuid PRIMARY KEY,
    user_id uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    stage smallint NOT NULL CHECK (stage BETWEEN 1 AND 50),
    difficulty combat_difficulty NOT NULL,
    seed bigint NOT NULL,
    balance_version varchar(32) NOT NULL DEFAULT 'mvp-wave-v1',
    squad_snapshot jsonb NOT NULL,
    events jsonb NOT NULL,
    status combat_session_status NOT NULL DEFAULT 'resolved',
    created_at timestamptz NOT NULL DEFAULT now(),
    resolved_at timestamptz NOT NULL DEFAULT now()
);
CREATE INDEX combat_sessions_user_time ON combat_sessions(user_id, created_at DESC);

ALTER TABLE combat_runs ADD COLUMN combat_session_id uuid UNIQUE REFERENCES combat_sessions(id);

-- Uma fase possui a melhor avaliação por dificuldade. A projeção total_stars é atualizada
-- transacionalmente pelo servidor, nunca pelo cliente.
CREATE TABLE stage_stars (
    user_id uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    stage smallint NOT NULL CHECK (stage BETWEEN 1 AND 50),
    difficulty combat_difficulty NOT NULL,
    stars smallint NOT NULL CHECK (stars BETWEEN 0 AND 3),
    updated_at timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (user_id, stage, difficulty)
);
