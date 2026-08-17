-- Estado e recibos idempotentes para recompensas offline. O horário é do servidor.
CREATE TABLE offline_reward_state (
    user_id uuid PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    last_claim_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE offline_reward_claims (
    id uuid PRIMARY KEY,
    user_id uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    idempotency_key uuid NOT NULL,
    elapsed_seconds integer NOT NULL CHECK (elapsed_seconds BETWEEN 0 AND 86400),
    gold_reward bigint NOT NULL CHECK (gold_reward >= 0),
    experience_reward bigint NOT NULL CHECK (experience_reward >= 0),
    claimed_at timestamptz NOT NULL DEFAULT now(),
    UNIQUE (user_id, idempotency_key)
);
CREATE INDEX offline_reward_claims_user_time ON offline_reward_claims(user_id, claimed_at DESC);
